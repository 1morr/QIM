import { type Key, toKey } from "./keys.ts";
import "./style.css";
import init, { Session, type Snapshot } from "./wasm/qim_wasm.js";

const PACK_URL = "/packs/quick/quick.pack";
/** 速成方案檔 `[packs]` 指名的詞庫資料包。 */
const LEXICON_URL = "/packs/quick-lexicon/quick-lexicon.pack";
/** 速成方案檔 `[packs]` 指名的語言模型資料包（字級 bigram）。 */
const LANGUAGE_MODEL_URL = "/packs/zh-hant-bigram/zh-hant-bigram.pack";

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

async function loadPack(url: string): Promise<Uint8Array> {
	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`${url}：${response.status}，先跑 cargo xtask web`);
	}
	return new Uint8Array(await response.arrayBuffer());
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

async function start(): Promise<void> {
	const [, pack, lexicon, languageModel] = await Promise.all([
		init(),
		loadPack(PACK_URL),
		loadPack(LEXICON_URL),
		loadPack(LANGUAGE_MODEL_URL),
	]);
	const session = new Session(pack, lexicon, languageModel);

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
			if (snapshot.commit !== undefined) {
				insert(snapshot.commit);
			}
			render(snapshot);
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
