import { type Key, toKey } from "./keys.ts";
import { fixedLine, initialReport, observe } from "./report.ts";
import "./style.css";
import { type Keyboard, mountKeyboard } from "./touch.ts";
import init, {
	Layout,
	type LayoutKey,
	pageSize,
	RadicalHints,
	Session,
	type Snapshot,
} from "./wasm/qim_wasm.js";

const PACK_URL = "/packs/quick/quick.pack";
/** 速成方案檔 `[packs]` 指名的詞庫資料包。 */
const LEXICON_URL = "/packs/quick-lexicon/quick-lexicon.pack";
/** 速成方案檔 `[packs]` 指名的語言模型資料包（字級 bigram）。 */
const LANGUAGE_MODEL_URL = "/packs/zh-hant-bigram/zh-hant-bigram.pack";
/** 速成方案檔 `[packs]` 指名的字根提示資料包（倉頡三代 / 五代碼）。 */
const RADICAL_HINT_URL = "/packs/cangjie-hint/cangjie-hint.pack";
/** 由 `cargo xtask data` 算出的預設 Gboard 速成佈局（plan §2.6）。 */
const LAYOUT_URL = "/layouts/gboard-quick.json";

function element<T extends HTMLElement>(selector: string): T {
	const found = document.querySelector<T>(selector);
	if (!found) {
		throw new Error(`index.html 少了 ${selector}`);
	}
	return found;
}

const text = element<HTMLTextAreaElement>("#text");
const composition = element<HTMLElement>("#composition");
const candidates = element<HTMLOListElement>("#candidates");
const status = element<HTMLElement>("#status");
const incognito = element<HTMLInputElement>("#incognito");
const keyboard = element<HTMLElement>("#keyboard");
const touchToggle = element<HTMLButtonElement>("#touch-toggle");
const reportButton = element<HTMLButtonElement>("#report");
const reports = element<HTMLTextAreaElement>("#reports");
const copyReports = element<HTMLButtonElement>("#copy-reports");

/** 回報清單存在 `localStorage`，重新整理不會不見。 */
const REPORTS_KEY = "qim.reports";
let report = initialReport;
const hintVersion = element<HTMLSelectElement>("#hint-version");
/** 長按候選多久叫出字根提示（毫秒）；和 Android 的長按門檻同級。 */
const LONG_PRESS_MS = 500;
/** 觸控鍵盤；載入完成前是 `undefined`。 */
let touchKeyboard: Keyboard | undefined;

/** `cargo xtask web` 放到 `web/public/` 的檔；沒有就叫人先跑它。 */
async function load(url: string): Promise<Response> {
	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`${url}：${response.status}，先跑 cargo xtask web`);
	}
	return response;
}

async function loadPack(url: string): Promise<Uint8Array> {
	return new Uint8Array(await (await load(url)).arrayBuffer());
}

function press(session: Session, key: Key): Snapshot {
	switch (key.kind) {
		case "letter":
			return session.letter(key.char);
		case "punct":
			return session.punct(key.char);
		case "digit":
			return session.digit(key.digit);
		case "space":
			return session.space();
		case "enter":
			return session.enter();
		case "backspace":
			return session.backspace();
		case "escape":
			return session.escape();
		case "pageUp":
			return session.pageUp();
		case "pageDown":
			return session.pageDown();
		case "left":
			return session.left();
		case "right":
			return session.right();
		case "other":
			return session.other();
	}
}

/** 候選列目前頁，點候選時換算成全部候選的位置。 */
let page = 0;

/** 畫面上的組字與候選；切換字根提示時不必再問核心就能重畫。 */
const shown = {
	composing: false,
	candidates: [] as string[],
	/** 候選旁顯示字根提示；組字結束就關掉（brief §4.3）。 */
	hints: false,
};
/** 字根提示；載入完成前是 `undefined`。 */
let radicalHints: RadicalHints | undefined;

/**
 * 一個候選的字根提示：每個字的字根接著倉頡碼，如「日弓(an) 口(r)」；詞裡沒有碼的字是「—」。
 * 整個候選都沒有碼（英文候選）時是空字串，不顯示。
 */
function hintText(candidate: string): string {
	if (!radicalHints) {
		return "";
	}
	const hints = radicalHints.hint(candidate, hintVersion.value);
	const parts = hints.map((hint) =>
		hint.codes.map((code, i) => `${hint.radicals[i]}(${code})`).join("／"),
	);
	for (const hint of hints) {
		hint.free();
	}
	return parts.every((part) => part === "")
		? ""
		: parts.map((part) => part || "—").join(" ");
}

/**
 * 組字一段一個 `<span>`：選定的顯示字詞，沒選定的顯示字根（brief §4.3）；目前分段標
 * `aria-current`。點一段就移到那段（brief §4.4），由 `start` 掛的事件處理。
 */
function render(snapshot: Snapshot): void {
	const segments = snapshot.segments;
	composition.replaceChildren(
		...segments.map((segment, i) => {
			const span = document.createElement("span");
			span.className = "segment";
			span.dataset.index = String(i);
			span.textContent = segment.selected ? segment.text : segment.radicals;
			if (i === snapshot.focus) {
				span.setAttribute("aria-current", "true");
			}
			return span;
		}),
	);
	shown.composing = segments.length > 0;
	for (const segment of segments) {
		segment.free();
	}
	page = snapshot.page;
	shown.candidates = snapshot.candidates;
	if (!shown.composing) {
		shown.hints = false;
	}
	renderCandidates();
}

function renderCandidates(): void {
	candidates.replaceChildren(
		...shown.candidates.map((candidate, i) => {
			const item = document.createElement("li");
			item.dataset.index = String(i);
			const index = document.createElement("span");
			index.className = "index";
			index.textContent = String(i + 1);
			item.append(index, candidate);
			const text = shown.hints ? hintText(candidate) : "";
			if (text !== "") {
				const hint = document.createElement("span");
				hint.className = "radical-hint";
				hint.dataset.testid = `hint-${i + 1}`;
				hint.textContent = text;
				item.append(hint);
			}
			return item;
		}),
	);
}

/** 組字中才有候選可查；切換候選旁的字根提示。 */
function toggleHints(): void {
	if (shown.composing) {
		shown.hints = !shown.hints;
		renderCandidates();
	}
}

/** 上屏：插入輸入框游標處，取代選取的文字。 */
function insert(commit: string): void {
	text.setRangeText(commit, text.selectionStart, text.selectionEnd, "end");
	text.dispatchEvent(new Event("input", { bubbles: true }));
}

/** 核心吃下的鍵：有上屏就插入，再畫組字與候選。`chosen` 是這個鍵是不是選字（空白、數字鍵）。 */
function apply(snapshot: Snapshot, chosen: boolean): void {
	const before = report.last;
	report = observe(report, { ...readReported(snapshot), chosen });
	if (report.last !== before) {
		reportButton.disabled = report.last === undefined;
	}
	if (snapshot.commit !== undefined) {
		insert(snapshot.commit);
	}
	render(snapshot);
	touchKeyboard?.refresh();
}

/** 點分段、點候選的結果：核心吃下才畫，用完釋放。`chosen` 同 `apply`。 */
function applyHandled(snapshot: Snapshot, chosen: boolean): void {
	try {
		if (snapshot.handled) {
			apply(snapshot, chosen);
		}
	} finally {
		snapshot.free();
	}
}

/** `Snapshot` 是 WebAssembly 的物件、用完就釋放，回報只留要的欄位。 */
function readReported(snapshot: Snapshot) {
	return {
		input: snapshot.input,
		selected: snapshot.selected,
		candidates: snapshot.candidates,
		commit: snapshot.commit,
	};
}

/** 把剛才那次組字加到回報清單最後一行；同一次只加一次。 */
function addReport(): void {
	if (!report.last) {
		return;
	}
	const lines = reports.value === "" ? [] : reports.value.split("\n");
	lines.push(fixedLine(report.last));
	reports.value = lines.join("\n");
	localStorage.setItem(REPORTS_KEY, reports.value);
	reportButton.disabled = true;
	text.focus();
}

/** 觸控鍵盤上核心沒吃下的鍵：照輸入框收到實體按鍵時的預設動作。 */
function hostDefault(key: LayoutKey): void {
	switch (key.tap) {
		case "letter":
			insert(key.letter ?? "");
			return;
		case "text":
			insert(key.text ?? "");
			return;
		case "space":
			insert(" ");
			return;
		case "enter":
			insert("\n");
			return;
		case "backspace": {
			const { selectionStart: start, selectionEnd: end } = text;
			if (start === end && start > 0) {
				// 刪游標前一個字元，不切開代理對。
				const before = Array.from(text.value.slice(0, start));
				const removed = before.pop()?.length ?? 0;
				text.setRangeText("", start - removed, start, "end");
			} else {
				text.setRangeText("", start, end, "end");
			}
			text.dispatchEvent(new Event("input", { bubbles: true }));
			return;
		}
	}
}

/** 選佈局版本用的裝置：短邊 600 px 以上當平板（Android 的 sw600dp 慣例）。 */
function device(): "phone" | "tablet" {
	return Math.min(window.screen.width, window.screen.height) >= 600
		? "tablet"
		: "phone";
}

function orientation(): "portrait" | "landscape" {
	return window.matchMedia("(orientation: landscape)").matches
		? "landscape"
		: "portrait";
}

/** 觸控鍵盤開或關；開著時輸入框不叫出系統鍵盤。 */
function setTouch(on: boolean): void {
	keyboard.hidden = !on;
	touchToggle.setAttribute("aria-pressed", String(on));
	text.inputMode = on ? "none" : "";
}

async function start(): Promise<void> {
	const [, pack, lexicon, languageModel, radicalHint, layoutJson] =
		await Promise.all([
			init(),
			loadPack(PACK_URL),
			loadPack(LEXICON_URL),
			loadPack(LANGUAGE_MODEL_URL),
			loadPack(RADICAL_HINT_URL),
			load(LAYOUT_URL).then((response) => response.text()),
		]);
	const session = new Session(pack, lexicon, languageModel);
	radicalHints = new RadicalHints(radicalHint, session);
	hintVersion.addEventListener("change", renderCandidates);
	// 點候選選字，長按叫出字根提示（brief §4.4）：放開時還沒到長按的門檻才選字。
	let pressTimer: number | undefined;
	/** 按下的候選在目前頁的位置；長按觸發或取消後是 `undefined`。 */
	let pressed: number | undefined;
	const cancelPress = () => {
		window.clearTimeout(pressTimer);
		pressTimer = undefined;
		pressed = undefined;
	};
	candidates.addEventListener("pointerdown", (event) => {
		// 不讓輸入框失去焦點，長按完還能接著打字。
		event.preventDefault();
		cancelPress();
		const item = (event.target as Element).closest<HTMLElement>("li");
		pressed = item ? Number(item.dataset.index) : undefined;
		pressTimer = window.setTimeout(() => {
			cancelPress();
			toggleHints();
		}, LONG_PRESS_MS);
	});
	candidates.addEventListener("pointerup", () => {
		const index = pressed;
		cancelPress();
		if (index !== undefined) {
			applyHandled(session.select(page * pageSize() + index), true);
		}
	});
	// 手指左右滑候選列時瀏覽器接手捲動、送 pointercancel，長按與選字都取消。
	for (const type of ["pointerleave", "pointercancel"]) {
		candidates.addEventListener(type, cancelPress);
	}
	candidates.addEventListener("contextmenu", (event) => event.preventDefault());
	session.setIncognito(incognito.checked);
	incognito.addEventListener("change", () => {
		session.setIncognito(incognito.checked);
		text.focus();
	});
	const layout = new Layout(layoutJson, device(), orientation());
	for (const warning of layout.warnings) {
		console.warn(`佈局：${warning}`);
	}
	touchKeyboard = mountKeyboard(
		keyboard,
		layout,
		session,
		(snapshot, key) => apply(snapshot, key.tap === "space"),
		hostDefault,
	);
	// 行動裝置（主要指標是手指）預設開觸控鍵盤。
	setTouch(window.matchMedia("(pointer: coarse)").matches);
	touchToggle.addEventListener("click", () => {
		setTouch(touchToggle.getAttribute("aria-pressed") !== "true");
	});
	reports.value = localStorage.getItem(REPORTS_KEY) ?? "";
	reports.addEventListener("input", () => {
		localStorage.setItem(REPORTS_KEY, reports.value);
	});
	reportButton.addEventListener("click", addReport);
	copyReports.addEventListener("click", () => {
		navigator.clipboard.writeText(reports.value).catch((error: unknown) => {
			status.textContent = `複製失敗：${String(error)}`;
		});
	});

	// 點組字的一段就移到那段；取消預設動作，輸入框不失去焦點（同觸控鍵盤）。
	composition.addEventListener("pointerdown", (event) => {
		const segment = (event.target as Element).closest<HTMLElement>(".segment");
		if (!segment) {
			return;
		}
		event.preventDefault();
		applyHandled(session.focusSegment(Number(segment.dataset.index)), false);
	});

	text.addEventListener("keydown", (event) => {
		const key = toKey(event);
		if (!key) {
			return;
		}
		// 組字中按 Tab 切換字根提示（brief §4.3）；核心本來就吃下不動作，不必轉給它。
		if (event.key === "Tab" && shown.composing) {
			event.preventDefault();
			toggleHints();
			return;
		}
		const snapshot = press(session, key);
		try {
			if (!snapshot.handled) {
				return;
			}
			event.preventDefault();
			apply(snapshot, key.kind === "space" || key.kind === "digit");
		} finally {
			snapshot.free();
		}
	});

	text.readOnly = false;
	text.dataset.ready = "true";
	status.textContent = "";
	text.focus();
}

start().catch((error: unknown) => {
	status.textContent = `載入失敗：${String(error)}`;
});
