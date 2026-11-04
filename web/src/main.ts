import { type Key, toKey } from "./keys.ts";
import "./style.css";
import { type Keyboard, mountKeyboard } from "./touch.ts";
import init, {
	Layout,
	type LayoutKey,
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
const keyboard = element<HTMLElement>("#keyboard");
const touchToggle = element<HTMLButtonElement>("#touch-toggle");
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

function render(snapshot: Snapshot): void {
	// 組字顯示選定的字詞，接著其餘輸入碼的字根（brief §4.3）。
	composition.textContent = snapshot.selected + snapshot.radicals;
	shown.composing = snapshot.selected !== "" || snapshot.unselected !== "";
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

/** 核心吃下的鍵：有上屏就插入，再畫組字與候選。 */
function apply(snapshot: Snapshot): void {
	if (snapshot.commit !== undefined) {
		insert(snapshot.commit);
	}
	render(snapshot);
	touchKeyboard?.refresh();
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
	// 觸控：長按候選叫出字根提示（brief §4.4）。
	let pressTimer: number | undefined;
	const cancelPress = () => {
		window.clearTimeout(pressTimer);
		pressTimer = undefined;
	};
	candidates.addEventListener("pointerdown", (event) => {
		// 不讓輸入框失去焦點，長按完還能接著打字。
		event.preventDefault();
		cancelPress();
		pressTimer = window.setTimeout(() => {
			pressTimer = undefined;
			toggleHints();
		}, LONG_PRESS_MS);
	});
	// 手指左右滑候選列時瀏覽器接手捲動、送 pointercancel，長按就取消。
	for (const type of ["pointerup", "pointerleave", "pointercancel"]) {
		candidates.addEventListener(type, cancelPress);
	}
	candidates.addEventListener("contextmenu", (event) => event.preventDefault());
	const layout = new Layout(layoutJson, device(), orientation());
	for (const warning of layout.warnings) {
		console.warn(`佈局：${warning}`);
	}
	touchKeyboard = mountKeyboard(keyboard, layout, session, apply, hostDefault);
	// 行動裝置（主要指標是手指）預設開觸控鍵盤。
	setTouch(window.matchMedia("(pointer: coarse)").matches);
	touchToggle.addEventListener("click", () => {
		setTouch(touchToggle.getAttribute("aria-pressed") !== "true");
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
			apply(snapshot);
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
