import { type Key, toKey } from "./keys.ts";
import "./style.css";
import { type Keyboard, mountKeyboard } from "./touch.ts";
import init, { Layout, Session, type Snapshot } from "./wasm/qim_wasm.js";

const PACK_URL = "/packs/quick/quick.pack";
/** 速成方案檔 `[packs]` 指名的詞庫資料包。 */
const LEXICON_URL = "/packs/quick-lexicon/quick-lexicon.pack";
/** 速成方案檔 `[packs]` 指名的語言模型資料包（字級 bigram）。 */
const LANGUAGE_MODEL_URL = "/packs/zh-hant-bigram/zh-hant-bigram.pack";
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
const darkToggle = element<HTMLButtonElement>("#dark-toggle");
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

function render(snapshot: Snapshot): void {
	composition.textContent = snapshot.selected + snapshot.unselected;
	candidates.replaceChildren(
		...snapshot.candidates.map((candidate, i) => {
			const item = document.createElement("li");
			const index = document.createElement("span");
			index.className = "index";
			index.textContent = String(i + 1);
			item.append(index, candidate);
			return item;
		}),
	);
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

/** 觸控鍵盤上核心沒吃下的動作：照輸入框收到實體按鍵時的預設動作。 */
function hostDefault(snapshot: Snapshot): void {
	switch (snapshot.action) {
		case "letter":
			insert(snapshot.letter ?? "");
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
	const [, pack, lexicon, languageModel, layoutJson] = await Promise.all([
		init(),
		loadPack(PACK_URL),
		loadPack(LEXICON_URL),
		loadPack(LANGUAGE_MODEL_URL),
		load(LAYOUT_URL).then((response) => response.text()),
	]);
	const session = new Session(pack, lexicon, languageModel);
	session.setIncognito(incognito.checked);
	incognito.addEventListener("change", () => {
		session.setIncognito(incognito.checked);
		text.focus();
	});
	const layout = new Layout(layoutJson, device(), orientation());
	for (const warning of layout.warnings) {
		console.warn(`佈局：${warning}`);
	}
	touchKeyboard = mountKeyboard(keyboard, layout, session, apply, hostDefault);
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

	text.addEventListener("keydown", (event) => {
		const key = toKey(event);
		if (!key) {
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
