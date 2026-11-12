// 網頁的組裝：載入資料包（`packs.ts`）、建核心，接上各區的模組與輸入框的鍵盤事件。畫面與互動各在自己的模組，
// 核心的排序與組字一律經綁定（plan §1.3）。

import { mountCandidateBar } from "./candidate-bar.ts";
import { mountComposition } from "./composition.ts";
import { mountContext } from "./context.ts";
import { mountDebugPanel } from "./debug-panel.ts";
import { mountTheme, mountTouch } from "./display.ts";
import { element, message } from "./dom.ts";
import { hostDefault, insert } from "./host.ts";
import { LoneShift, toKey } from "./keys.ts";
import {
	checkModelPack,
	NeuralReranker,
	type PackSource,
} from "./neural-rerank.ts";
import { mountPackSettings } from "./pack-settings.ts";
import { PackLibrary } from "./packs.ts";
import { mountPanel } from "./panel.ts";
import { mountPhrases, type Phrases } from "./phrases.ts";
import { mountReportPanel } from "./report-panel.ts";
import { fakeRank, rerankScheduler } from "./rerank.ts";
import { mountSettings } from "./settings.ts";
import "./style.css";
import {
	type EnglishUserWords,
	mountEnglishUserWords,
} from "./english-user-words.ts";
import { localToday } from "./today.ts";
import type { Keyboard } from "./touch.ts";
import { mountUserDict } from "./user-dict.ts";
import init, {
	contextLimit,
	pageSize,
	RadicalHints,
	Session,
	type Snapshot,
} from "./wasm/qim_wasm.js";
import { mountWordList } from "./word-list.ts";

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

/**
 * 結果回來後再等幾毫秒才套用（網址的 `?rerank-delay=<ms>`）：e2e 模擬慢的模型，驗重排回來前繼續打字時
 * 舊結果不套用。平常沒有。
 */
function rerankDelay(): number {
	const value = Number(
		new URLSearchParams(location.search).get("rerank-delay"),
	);
	return Number.isFinite(value) && value > 0 ? value : 0;
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
	// 停鍵層模型在 Web Worker 裡載入，打字不等它；載入不了就不重排，打字與候選照常。
	const neural = new NeuralReranker();
	/** Worker 正在用的模型的 manifest；沒變就不重載，載入失敗時清掉，下次換資料包再試。 */
	let loadedModel: string | undefined;
	const loadModel = (model: PackSource | undefined): void => {
		loadedModel = model?.manifest;
		neural.load(model).catch((error: unknown) => {
			loadedModel = undefined;
			status.textContent = `停鍵層模型載入不了，不重排：${message(error)}`;
		});
	};
	loadModel(library.modelPack());
	/**
	 * 換資料包時的模型：先在這裡核對（不收就丟錯，資料包的改動退回），核心也收下其他資料包之後才呼叫
	 * 回傳的函式交給 Worker；沒換時是 `undefined`。
	 */
	const checkModel = (): (() => void) | undefined => {
		const model = library.modelPack();
		if (model?.manifest === loadedModel) {
			return undefined;
		}
		if (model) {
			checkModelPack(model);
		}
		return () => loadModel(model);
	};
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
	/** 英文使用者詞的清單；同上比 `userDict` 晚掛上。 */
	let englishUserWords: EnglishUserWords | undefined;
	// 殼的日期契約（plan §2.4）：核心不讀系統時間，第一次學習（含匯入存檔）前、之後每次使用者操作前給
	// 今天，過了午夜才不會記成前一天。按鍵、點擊、觸控都從 keydown 或 pointerdown 開始；失焦時核心
	// 會把正在打的串學成英文使用者詞，失焦可能沒有前導的按鍵或點擊（Alt+Tab、切分頁）。都在 window 的捕獲階段
	// 給（blur 不冒泡，但捕獲階段照樣經過 window），比任何處理器早。
	const giveToday = (): void => session.setToday(localToday());
	giveToday();
	for (const type of ["keydown", "pointerdown", "blur"]) {
		window.addEventListener(type, giveToday, { capture: true });
	}
	// 匯入、清空換掉整份使用者詞，常用語與英文使用者詞的清單跟著重畫。
	const userDict = mountUserDict(session, () => {
		phrases?.refresh();
		englishUserWords?.refresh();
		focusText();
	});
	/** 核心可能學到東西之後存檔：上屏、英文直通打完一串字（核心學英文使用者詞）、失焦。 */
	const saveLearning = (): void => {
		userDict.save();
		englishUserWords?.refresh();
	};
	/**
	 * 核心沒吃下、交還瀏覽器的鍵之後：空白、標點、Enter、Tab 可能讓核心學到剛打完的英文串，要存檔；
	 * 字母、數字只是接在那串後面，不必存。`kind` 是實體鍵的種類或觸控的動作（觸控字母鍵是 `weighted`）。
	 */
	const afterHostKey = (kind: string | undefined): void => {
		if (kind !== "letter" && kind !== "weighted" && kind !== "digit") {
			saveLearning();
		}
	};
	const reportPanel = mountReportPanel(status, focusText);
	// debug 面板（plan §3.1）：開著時每次重畫後問核心為什麼這樣排，關著時不問。
	const debugPanel = mountDebugPanel(
		element("#debug"),
		element("#debug-toggle"),
		session,
	);
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
		debugPanel.refresh();
	};
	const candidateList = element<HTMLOListElement>("#candidates");
	const requestRerank = rerankScheduler(session, {
		mode: settings.rerank,
		models: {
			neural: neural.rank,
			identity: fakeRank("identity"),
			"raise-last": fakeRank("raise-last"),
		},
		show: (snapshot) => {
			render(snapshot);
			touchKeyboard?.refresh();
		},
		onRequest: (generation, job) => debugPanel.rerankRequested(generation, job),
		// 給 e2e 與量延遲讀：各種結果的次數、模型上次算了幾毫秒。
		onResult: (outcome, ranked) => {
			debugPanel.rerankResult(outcome, ranked);
			const key = `rerank${outcome[0].toUpperCase()}${outcome.slice(1)}`;
			candidateList.dataset[key] = String(
				Number(candidateList.dataset[key] ?? 0) + 1,
			);
			if (ranked) {
				candidateList.dataset.rerankMs = ranked.ms.toFixed(2);
			}
		},
		extraDelayMs: rerankDelay(),
	});
	/** 核心吃下的鍵：有上屏就插入，再畫組字與候選。`chosen` 是這個鍵是不是選字（空白、數字鍵）。 */
	const apply = (snapshot: Snapshot, chosen: boolean): void => {
		reportPanel.observe(snapshot, chosen);
		if (snapshot.commit !== undefined) {
			insert(text, snapshot.commit);
			// 上屏時核心可能學到東西；每次都存，重新整理網頁後還在。
			saveLearning();
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
	englishUserWords = mountEnglishUserWords(session, userDict.save, focusText);
	mountWordList(session, userDict.save, focusText);
	// 給前文或失焦後核心可能收起了聯想（游標移走、換欄位）：沒有組字、核心也沒有候選了，就清掉候選列。
	// 失焦時核心把正在打的串當打完學起來，要存檔。
	const syncContext = mountContext(
		text,
		session,
		contextLimit(),
		() => {
			if (!shown.composing && session.allCandidates().length === 0) {
				candidateBar.show([], [], false);
			}
			debugPanel.refresh();
		},
		saveLearning,
	);
	// 開關改了不是選字；改完重給前文（關掉無痕之後要立刻給）。
	settings.connect(
		session,
		(snapshot) => apply(snapshot, false),
		() => {
			syncContext();
			// 無痕時不給前文、上面不會重畫面板；開關都可能改了排序的原因。
			debugPanel.refresh();
			focusText();
		},
	);
	touchKeyboard = mountTouch(
		session,
		{ plain: layoutJson, numberRow: numberRowLayoutJson },
		text,
		// 回報：點空白選候選才算「選了」；長按或組字中的動作照綁定回報的實際動作。
		(snapshot) => apply(snapshot, snapshot.action === "space"),
		// 交還宿主的鍵也可能收起了聯想（空白、Enter），照樣重畫候選列；也可能讓核心學到剛打完的英文串。
		(snapshot) => {
			hostDefault(text, snapshot);
			render(snapshot);
			afterHostKey(snapshot.action);
		},
	);
	// 換資料包：核心換上新的那組（學習資料與開關留著），組字重列，字根提示跟著換。
	mountPackSettings(
		library,
		() => {
			const changeModel = checkModel();
			const snapshot = session.setPacks(library.sessionPacks());
			changeModel?.();
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
			// 留給瀏覽器的組合鍵（Cmd+V、Ctrl+B…）可能改字或移游標：照樣告訴核心游標前不知道是什麼了，
			// 無痕時（不給前文）數字後半形才不會看錯（brief §4.7）。沒有組字時核心不吃、不動作。
			if (!shown.composing) {
				const snapshot = session.press("other");
				try {
					// 也收起了聯想。
					render(snapshot);
				} finally {
					snapshot.free();
				}
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
		// 交還瀏覽器的鍵（方向鍵、退格…）讓核心忘掉前文，游標沒動時不會有 selectionchange 補給；
		// 每個鍵之前重給一次，這時前一個鍵的預設動作已經做完。組字中給的核心要到下次組字才用，不必給。
		if (!shown.composing) {
			syncContext();
		}
		const snapshot = session.press(key.kind, key.value);
		try {
			if (!snapshot.handled) {
				// 交還瀏覽器的鍵（空白、Enter、方向鍵…）收起了聯想：候選列跟著清掉。
				render(snapshot);
				afterHostKey(key.kind);
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
	// 載入中已經點進輸入框時 `focus()` 不會再觸發 focus 事件。
	syncContext();
	text.focus();
}

start().catch((error: unknown) => {
	status.textContent = `載入失敗：${String(error)}`;
});
