import { LoneShift, shiftSwitch, toKey } from "./keys.ts";
import { fixedLine, initialReport, observe } from "./report.ts";
import { fakeOrder, RERANK_DELAY_MS, rerankMode } from "./rerank.ts";
import "./style.css";
import { element } from "./dom.ts";
import { type Keyboard, mountKeyboard } from "./touch.ts";
import { mountUserDict, type UserDict } from "./user-dict.ts";
import init, {
	Layout,
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
/** 簡繁資料包（OpenCC `hk2s`），簡體模式用。 */
const SIMPLIFIER_URL = "/packs/opencc-hk2s/opencc-hk2s.pack";
/** 英文資料包（ESDB），中文狀態的英文候選用。 */
const ENGLISH_URL = "/packs/scowl-en-us/scowl-en-us.pack";
/** 由 `cargo xtask data` 算出的預設 Gboard 速成佈局（plan §2.6）。 */
const LAYOUT_URL = "/layouts/gboard-quick.json";
/** 同上，字母頁上方多一排數字；設定的「數字列」打開時用（brief §4.4）。 */
const NUMBER_ROW_LAYOUT_URL = "/layouts/gboard-quick-number-row.json";

const text = element<HTMLTextAreaElement>("#text");
const composition = element<HTMLElement>("#composition");
const candidates = element<HTMLOListElement>("#candidates");
const mode = element<HTMLElement>("#mode");
const expand = element<HTMLButtonElement>("#expand");
const panel = element<HTMLOListElement>("#panel");
const rerankSelect = element<HTMLSelectElement>("#rerank");
const status = element<HTMLElement>("#status");
const incognito = element<HTMLInputElement>("#incognito");
const simplified = element<HTMLInputElement>("#simplified");
const englishCandidates = element<HTMLInputElement>("#english-candidates");
const digitSeparators = element<HTMLInputElement>("#digit-separators");
const shiftSwitchSelect = element<HTMLSelectElement>("#shift-switch");
const keyboard = element<HTMLElement>("#keyboard");
const touchToggle = element<HTMLButtonElement>("#touch-toggle");
const darkToggle = element<HTMLButtonElement>("#dark-toggle");
const numberRow = element<HTMLInputElement>("#number-row");
const reportButton = element<HTMLButtonElement>("#report");
const reports = element<HTMLTextAreaElement>("#reports");
const copyReports = element<HTMLButtonElement>("#copy-reports");

/** 回報清單存在 `localStorage`，重新整理不會不見。 */
const REPORTS_KEY = "qim.reports";
let report = initialReport;
const hintVersion = element<HTMLSelectElement>("#hint-version");
/** 長按候選多久叫出字根提示（毫秒）；和 Android 的長按門檻同級。 */
const CANDIDATE_HINT_PRESS_MS = 500;
/** 觸控鍵盤；載入完成前是 `undefined`。 */
let touchKeyboard: Keyboard | undefined;
/** 學習資料；載入完成前是 `undefined`。 */
let userDict: UserDict | undefined;

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

/** 候選列目前頁，點候選時換算成全部候選的位置。 */
let page = 0;

/** 畫面上的組字與候選；切換字根提示時不必再問核心就能重畫。 */
const shown = {
	composing: false,
	/** 臨時英文（組字中單按留碼的 Shift）：組字是英文，沒有候選、字根提示與展開面板。 */
	inlineAscii: false,
	candidates: [] as string[],
	/** 候選的繁體原文，字根提示用：簡體模式下要提示的是打得出的繁體字。 */
	originals: [] as string[],
	/** 候選旁顯示字根提示；組字結束就關掉（brief §4.3）。 */
	hints: false,
	/** 候選展開面板開著；組字結束就關掉。 */
	panel: false,
};
/** 中文組字中：有候選、字根提示與展開面板可用；臨時英文不算。 */
function composingChinese(): boolean {
	return shown.composing && !shown.inlineAscii;
}

/** 核心；載入完成前是 `undefined`。展開面板與重排要直接問它。 */
let core: Session | undefined;
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
 * 組字一段一個 `<span>`：選定的顯示字詞，沒選定的顯示字根（brief §4.3），臨時英文顯示英文；目前分段標
 * `aria-current`。點一段就移到那段（brief §4.4），由 `start` 掛的事件處理。
 */
function render(snapshot: Snapshot): void {
	const segments = snapshot.segments;
	composition.replaceChildren(
		...segments.map((segment, i) => {
			const span = document.createElement("span");
			span.className = "segment";
			span.dataset.index = String(i);
			span.textContent =
				segment.selected || segment.raw ? segment.text : segment.radicals;
			if (i === snapshot.focus) {
				span.setAttribute("aria-current", "true");
			}
			return span;
		}),
	);
	shown.composing = segments.length > 0;
	shown.inlineAscii = shown.composing && snapshot.ascii;
	for (const segment of segments) {
		segment.free();
	}
	page = snapshot.page;
	shown.candidates = snapshot.candidates;
	shown.originals = snapshot.candidatesOriginal;
	if (!composingChinese()) {
		shown.hints = false;
		shown.panel = false;
	}
	mode.textContent = snapshot.ascii ? "英" : "中";
	mode.setAttribute("aria-label", snapshot.ascii ? "英文" : "中文");
	expand.hidden = !composingChinese();
	renderCandidates();
	renderPanel();
}

/** 候選展開面板：一次列全部候選（brief §4.4），每項記著它在全部候選的位置，點了照它選字。 */
function renderPanel(): void {
	panel.hidden = !shown.panel;
	expand.setAttribute("aria-expanded", String(shown.panel));
	expand.textContent = shown.panel ? "⌃" : "⌄";
	expand.setAttribute("aria-label", shown.panel ? "收起候選" : "展開候選");
	if (!shown.panel || !core) {
		panel.replaceChildren();
		return;
	}
	panel.replaceChildren(
		...core.allCandidates().map((candidate, i) => {
			const item = document.createElement("li");
			item.dataset.index = String(i);
			item.textContent = candidate;
			return item;
		}),
	);
}

/** 打開或收起展開面板；沒有組字或臨時英文時不開。 */
function setPanel(open: boolean): void {
	shown.panel = open && composingChinese();
	renderPanel();
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
			const text = shown.hints ? hintText(shown.originals[i] ?? candidate) : "";
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
	if (composingChinese()) {
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
		// 上屏時核心可能學到東西；每次都存，重新整理網頁後還在。
		userDict?.save();
	}
	render(snapshot);
	touchKeyboard?.refresh();
	requestRerank(snapshot.generation);
}

/** 上次要求重排時的 `generation`；候選沒變就不再要求。 */
let rerankedAt: bigint | undefined;

/**
 * 候選變了就把重排要求交給假的模型，`RERANK_DELAY_MS` 後套用（plan §3.1）。這段時間內又按了鍵，
 * 結果帶的 `generation` 過期，核心丟掉。名次變了才重畫，也不再要求重排；回報只看首選，
 * 重排不動首選，所以不經過回報。
 */
function requestRerank(generation: bigint): void {
	const chosen = rerankMode(rerankSelect.value);
	if (chosen === "off" || !core || generation === rerankedAt) {
		return;
	}
	rerankedAt = generation;
	const request = core.rerankRequest();
	if (!request) {
		return;
	}
	const asked = request.generation;
	const order = fakeOrder(chosen, request.candidates.length);
	request.free();
	const session = core;
	window.setTimeout(() => {
		// 等待中關掉了重排，就不套用。
		if (rerankMode(rerankSelect.value) === "off") {
			return;
		}
		const result = session.applyRerank(asked, Uint32Array.from(order));
		if (!result) {
			return;
		}
		try {
			// `generation` 沒變是名次沒變，不必重畫。
			if (result.generation === asked) {
				return;
			}
			rerankedAt = result.generation;
			render(result);
			touchKeyboard?.refresh();
		} finally {
			result.free();
		}
	}, RERANK_DELAY_MS);
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

/**
 * `Snapshot` 是 WebAssembly 的物件、用完就釋放，回報只留要的欄位。
 * 候選與上屏取繁體原文：`fixed.tsv` 的期望首選是繁體，簡體模式下也一樣。
 */
function readReported(snapshot: Snapshot) {
	return {
		input: snapshot.input,
		selected: snapshot.selected,
		candidates: snapshot.candidatesOriginal,
		commit: snapshot.commitOriginal,
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

/** 觸控鍵盤上核心沒吃下的動作：照輸入框收到實體按鍵時的預設動作。 */
function hostDefault(snapshot: Snapshot): void {
	switch (snapshot.action) {
		case "letter":
		case "punct":
			insert(snapshot.character ?? "");
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

const systemDark = window.matchMedia("(prefers-color-scheme: dark)");
/** 網頁的深色開關按過就照它，否則跟系統（`prefers-color-scheme`）。 */
let chosenDark: boolean | undefined;

/** 套用深色或淺色：按鍵顏色由 CSS `light-dark()` 依 `color-scheme` 選（touch.ts）。 */
function applyTheme(): void {
	const dark = chosenDark ?? systemDark.matches;
	if (chosenDark === undefined) {
		delete document.documentElement.dataset.theme;
	} else {
		document.documentElement.dataset.theme = dark ? "dark" : "light";
	}
	darkToggle.setAttribute("aria-pressed", String(dark));
}

/** 觸控鍵盤開或關；開著時輸入框不叫出系統鍵盤。 */
function setTouch(on: boolean): void {
	keyboard.hidden = !on;
	touchToggle.setAttribute("aria-pressed", String(on));
	text.inputMode = on ? "none" : "";
}

async function start(): Promise<void> {
	const [
		,
		pack,
		lexicon,
		languageModel,
		radicalHint,
		simplifier,
		english,
		layoutJson,
		numberRowLayoutJson,
	] = await Promise.all([
		init(),
		loadPack(PACK_URL),
		loadPack(LEXICON_URL),
		loadPack(LANGUAGE_MODEL_URL),
		loadPack(RADICAL_HINT_URL),
		loadPack(SIMPLIFIER_URL),
		loadPack(ENGLISH_URL),
		load(LAYOUT_URL).then((response) => response.text()),
		load(NUMBER_ROW_LAYOUT_URL).then((response) => response.text()),
	]);
	const session = new Session(
		pack,
		lexicon,
		languageModel,
		simplifier,
		english,
	);
	core = session;
	userDict = mountUserDict(session, () => text.focus());
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
		}, CANDIDATE_HINT_PRESS_MS);
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
	// 展開鍵與面板：按下時不讓輸入框失去焦點，點完還能接著打字。
	expand.addEventListener("pointerdown", (event) => event.preventDefault());
	expand.addEventListener("click", () => setPanel(!shown.panel));
	panel.addEventListener("pointerdown", (event) => event.preventDefault());
	panel.addEventListener("click", (event) => {
		const item = (event.target as Element).closest<HTMLElement>("li");
		if (item) {
			// 選了字就收起，同 Gboard 的展開面板。
			shown.panel = false;
			applyHandled(session.select(Number(item.dataset.index)), true);
		}
	});
	session.setIncognito(incognito.checked);
	incognito.addEventListener("change", () => {
		session.setIncognito(incognito.checked);
		text.focus();
	});
	// 組字中切換也立刻重列候選。
	const applySimplified = (): void => {
		const snapshot = session.setSimplified(simplified.checked);
		try {
			// 切換不是選字。
			apply(snapshot, false);
		} finally {
			snapshot.free();
		}
	};
	applySimplified();
	simplified.addEventListener("change", () => {
		applySimplified();
		text.focus();
	});
	// 英文候選（預設開）：組字中切換也立刻重列候選。
	const applyEnglishCandidates = (): void => {
		const snapshot = session.setEnglishCandidates(englishCandidates.checked);
		try {
			apply(snapshot, false);
		} finally {
			snapshot.free();
		}
	};
	applyEnglishCandidates();
	englishCandidates.addEventListener("change", () => {
		applyEnglishCandidates();
		text.focus();
	});
	// 組字中單按 Shift 的處理（預設照 Rime）；不動目前的組字，不必重畫。
	session.setShiftSwitch(shiftSwitch(shiftSwitchSelect.value));
	shiftSwitchSelect.addEventListener("change", () => {
		session.setShiftSwitch(shiftSwitch(shiftSwitchSelect.value));
		text.focus();
	});
	// 數字後的句點、冒號用半形（預設開）；逗號照舊全形。
	session.setDigitSeparators(digitSeparators.checked);
	digitSeparators.addEventListener("change", () => {
		session.setDigitSeparators(digitSeparators.checked);
		text.focus();
	});
	/** 依「數字列」開關讀佈局，選目前裝置與方向的版本。 */
	const readLayout = (): Layout => {
		const read = new Layout(
			numberRow.checked ? numberRowLayoutJson : layoutJson,
			device(),
			orientation(),
		);
		for (const warning of read.warnings) {
			console.warn(`佈局：${warning}`);
		}
		return read;
	};
	let layout = readLayout();
	touchKeyboard = mountKeyboard(
		keyboard,
		layout,
		session,
		// 回報：點空白選候選才算「選了」；長按或組字中的動作照綁定回報的實際動作。
		(snapshot) => apply(snapshot, snapshot.action === "space"),
		hostDefault,
	);
	const mounted = touchKeyboard;
	// 轉向時重選佈局版本（plan §2.6），留在同一頁。
	window
		.matchMedia("(orientation: landscape)")
		.addEventListener("change", () => {
			const warning = layout.select(device(), orientation());
			if (warning !== undefined) {
				console.warn(`佈局：${warning}`);
			}
			mounted.relayout();
		});
	numberRow.addEventListener("change", () => {
		const old = layout;
		layout = readLayout();
		layout.keepStateOf(old);
		mounted.setLayout(layout);
		old.free();
		text.focus();
	});
	// 頁面底部留出鍵盤實際的高度，鍵盤才不會蓋住輸入框；高度隨佈局版本的畫布比例與寬度變。
	new ResizeObserver(([entry]) => {
		document.body.style.setProperty(
			"--keyboard-height",
			`${entry.borderBoxSize[0].blockSize}px`,
		);
	}).observe(keyboard);
	// 行動裝置（主要指標是手指）預設開觸控鍵盤。
	setTouch(window.matchMedia("(pointer: coarse)").matches);
	touchToggle.addEventListener("click", () => {
		setTouch(touchToggle.getAttribute("aria-pressed") !== "true");
	});
	applyTheme();
	systemDark.addEventListener("change", applyTheme);
	darkToggle.addEventListener("click", () => {
		chosenDark = darkToggle.getAttribute("aria-pressed") !== "true";
		applyTheme();
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

	// 單按 Shift 切中英（brief §4.3）：Shift 自己不交給核心，放開時才判定；左右由
	// `KeyboardEvent.code` 分辨。
	const loneShift = new LoneShift();
	text.addEventListener("keyup", (event) => {
		const side = loneShift.up(event.code, event.timeStamp);
		if (side && !event.isComposing) {
			const snapshot = session.press("toggleAscii", side);
			try {
				apply(snapshot, false);
			} finally {
				snapshot.free();
			}
		}
	});
	text.addEventListener("keydown", (event) => {
		const plain = !(event.ctrlKey || event.metaKey || event.altKey);
		loneShift.down(event.code, event.repeat, event.timeStamp, !plain);
		if (event.key === "Shift") {
			return;
		}
		// 組字中 ↓ 打開展開面板、↑ 收起（plan §3.1）；核心本來就吃下不動作，不必轉給它。
		if (
			plain &&
			composingChinese() &&
			(event.key === "ArrowDown" || event.key === "ArrowUp")
		) {
			event.preventDefault();
			setPanel(event.key === "ArrowDown");
			return;
		}
		const key = toKey(event, shown.composing, shown.inlineAscii);
		if (!key) {
			// 留給瀏覽器的組合鍵（Cmd+V、Ctrl+B…）可能改字或移游標：照樣告訴核心，數字後半形才
			// 知道前一個字不一定是數字了（brief §4.7）。沒有組字時核心不吃、不動作。
			if (!shown.composing) {
				session.press("other").free();
			}
			return;
		}
		// 組字中按 Tab 切換字根提示（brief §4.3）；核心本來就吃下不動作，不必轉給它。
		if (event.key === "Tab" && composingChinese()) {
			event.preventDefault();
			toggleHints();
			return;
		}
		// 臨時英文中空白與數字是字元，不會上屏，所以不必另外排除。
		const chosen = key.kind === "space" || key.kind === "digit";
		const snapshot = session.press(key.kind, key.value);
		try {
			if (!snapshot.handled) {
				return;
			}
			event.preventDefault();
			apply(snapshot, chosen);
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
