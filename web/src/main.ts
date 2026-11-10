// 網頁的組裝：載入資料包（`packs.ts`）、建核心，接上各區的模組與輸入框的鍵盤事件。畫面與互動各在自己的模組，
// 核心的排序與組字一律經綁定（plan §1.3）。

import { mountCandidateBar } from "./candidate-bar.ts";
import { mountComposition } from "./composition.ts";
import { mountTheme, mountTouch } from "./display.ts";
import { element, message } from "./dom.ts";
import { hostDefault, insert } from "./host.ts";
import { LoneShift, toKey } from "./keys.ts";
import { mountPackSettings } from "./pack-settings.ts";
import { PackLibrary } from "./packs.ts";
import { mountPanel } from "./panel.ts";
import { mountPhrases, type Phrases } from "./phrases.ts";
import { mountReportPanel } from "./report-panel.ts";
import { fakeReranker } from "./rerank.ts";
import { mountSettings } from "./settings.ts";
import "./style.css";
import type { Keyboard } from "./touch.ts";
import { mountUserDict } from "./user-dict.ts";
import init, {
	pageSize,
	RadicalHints,
	Session,
	type Snapshot,
} from "./wasm/qim_wasm.js";

/** 由 `cargo xtask data` 算出的預設 Gboard 速成佈局（plan §2.6）。 */
const LAYOUT_URL = "/layouts/gboard-quick.json";
/** 同上，字母頁上方多一排數字；設定的「數字列」打開時用（brief §4.4）。 */
const NUMBER_ROW_LAYOUT_URL = "/layouts/gboard-quick-number-row.json";

const text = element<HTMLTextAreaElement>("#text");
const status = element<HTMLElement>("#status");
const focusText = (): void => text.focus();
// 開關在載入前就畫出來；接上核心後才套用。
const settings = mountSettings(element("#settings-options"));

/** `cargo xtask web` 放到 `web/public/` 的檔；沒有就叫人先跑它。 */
async function load(url: string): Promise<Response> {
	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`${url}：${response.status}，先跑 cargo xtask web`);
	}
	return response;
}

/** 畫面上的組字狀態；鍵盤事件靠它決定哪些鍵由網頁自己攔下。 */
const shown = {
	composing: false,
	/** 臨時英文（組字中單按留碼的 Shift）：組字是英文，沒有候選、字根提示與展開面板。 */
	inlineAscii: false,
	/** 候選列目前頁，點候選時換算成全部候選的位置。 */
	page: 0,
};
/** 中文組字中：有候選、字根提示與展開面板可用；臨時英文不算。 */
function composingChinese(): boolean {
	return shown.composing && !shown.inlineAscii;
}

async function start(): Promise<void> {
	const [, library, layoutJson, numberRowLayoutJson] = await Promise.all([
		init(),
		PackLibrary.open(),
		load(LAYOUT_URL).then((response) => response.text()),
		load(NUMBER_ROW_LAYOUT_URL).then((response) => response.text()),
	]);
	let session: Session;
	/** 載入時就有要說的事（改用內建的資料包），載入完不清掉狀態列。 */
	let notice = false;
	try {
		session = new Session(library.sessionPacks());
	} catch (error) {
		// 選的或匯入的資料包載入不了（例如 IndexedDB 裡的檔壞了）：這次只用內建的，說明原因。
		library.fallBackToBuiltin();
		session = new Session(library.sessionPacks());
		status.textContent = `匯入的資料包載入不了，這次只用內建的：${message(error)}`;
		notice = true;
	}
	/** 正在用的字根提示資料包，查完提示就釋放。 */
	const radicalHints = (): RadicalHints => {
		const pack = library.radicalHintPack();
		try {
			return new RadicalHints(pack, session);
		} finally {
			pack.free();
		}
	};
	/** 常用語；要用 `userDict.save`，所以比它晚掛上。 */
	let phrases: Phrases | undefined;
	// 匯入、清空換掉整份使用者詞，常用語的清單跟著重畫。
	const userDict = mountUserDict(session, () => {
		phrases?.refresh();
		focusText();
	});
	const reportPanel = mountReportPanel(status, focusText);
	/** 觸控鍵盤；掛上前是 `undefined`（開關的初始套用比它早）。 */
	let touchKeyboard: Keyboard | undefined;

	/** 畫組字、候選列與面板。 */
	const render = (snapshot: Snapshot): void => {
		shown.composing = composition.show(snapshot);
		shown.inlineAscii = shown.composing && snapshot.ascii;
		shown.page = snapshot.page;
		candidateBar.setMode(snapshot.ascii);
		candidateBar.show(
			snapshot.candidates,
			snapshot.candidatesOriginal,
			composingChinese(),
		);
		panel.show(composingChinese());
		phrases?.setComposing(shown.composing);
	};
	const requestRerank = fakeReranker(session, settings.rerank, (snapshot) => {
		render(snapshot);
		touchKeyboard?.refresh();
	});
	/** 核心吃下的鍵：有上屏就插入，再畫組字與候選。`chosen` 是這個鍵是不是選字（空白、數字鍵）。 */
	const apply = (snapshot: Snapshot, chosen: boolean): void => {
		reportPanel.observe(snapshot, chosen);
		if (snapshot.commit !== undefined) {
			insert(text, snapshot.commit);
			// 上屏時核心可能學到東西；每次都存，重新整理網頁後還在。
			userDict.save();
		}
		render(snapshot);
		touchKeyboard?.refresh();
		requestRerank(snapshot.generation);
	};
	/** 點分段、點候選的結果：核心吃下才畫，用完釋放。`chosen` 同 `apply`。 */
	const applyHandled = (snapshot: Snapshot, chosen: boolean): void => {
		try {
			if (snapshot.handled) {
				apply(snapshot, chosen);
			}
		} finally {
			snapshot.free();
		}
	};

	const composition = mountComposition((index) =>
		applyHandled(session.focusSegment(index), false),
	);
	const candidateBar = mountCandidateBar(radicalHints(), (index) =>
		applyHandled(session.select(shown.page * pageSize() + index), true),
	);
	const panel = mountPanel(
		() => session.allCandidates(),
		(index) => applyHandled(session.select(index), true),
	);
	phrases = mountPhrases(
		session,
		(snapshot) => applyHandled(snapshot, false),
		userDict.save,
		focusText,
	);
	// 開關改了不是選字。
	settings.connect(session, (snapshot) => apply(snapshot, false), focusText);
	touchKeyboard = mountTouch(
		session,
		{ plain: layoutJson, numberRow: numberRowLayoutJson },
		text,
		// 回報：點空白選候選才算「選了」；長按或組字中的動作照綁定回報的實際動作。
		(snapshot) => apply(snapshot, snapshot.action === "space"),
		(snapshot) => hostDefault(text, snapshot),
	);
	// 換資料包：核心換上新的那組（學習資料與開關留著），組字重列，字根提示跟著換。
	mountPackSettings(
		library,
		() => {
			const snapshot = session.setPacks(library.sessionPacks());
			candidateBar.setRadicalHints(radicalHints());
			try {
				render(snapshot);
			} finally {
				snapshot.free();
			}
		},
		focusText,
	);
	mountTheme();

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
			panel.setOpen(event.key === "ArrowDown");
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
			candidateBar.toggleHints();
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
	if (!notice) {
		status.textContent = "";
	}
	text.focus();
}

start().catch((error: unknown) => {
	status.textContent = `載入失敗：${String(error)}`;
});
