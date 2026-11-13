// 設定區（plan〈網頁 prototype〉）：改候選與組字行為的開關（核心選項與重排）。新開關在 `ROWS` 加一列，畫面與套用都由這裡做；
// 之後的設定（常用語、詞庫匯入、資料包）在 `index.html` 的設定區各加一段。

import { shiftSwitch } from "./keys.ts";
import { type RerankMode, rerankMode } from "./rerank.ts";
import type { Session, Snapshot } from "./wasm/qim_wasm.js";

/**
 * 套用一個開關到核心。會重列候選的回傳 `Snapshot`（組字中切換也要立刻重畫）；不動目前組字的
 * 回傳 `undefined`。
 */
type Apply<T> = (session: Session, value: T) => Snapshot | undefined;

interface Checkbox {
	kind: "checkbox";
	id: string;
	label: string;
	checked: boolean;
	apply: Apply<boolean>;
}

interface Select {
	kind: "select";
	id: string;
	label: string;
	/** `[值, 顯示的字]`。 */
	options: readonly (readonly [string, string])[];
	selected: string;
	apply: Apply<string>;
}

type Row = Checkbox | Select;

/** 重排不在核心選項裡：每次要求重排時才讀（`Settings.rerank`）。 */
const RERANK: Select = {
	kind: "select",
	id: "rerank",
	label: "重排",
	options: [
		["neural", "神經模型"],
		["off", "關"],
		["identity", "原樣（假實作，測試）"],
		["raise-last", "最後一條路徑提前（假實作，測試）"],
	],
	selected: "neural",
	apply: () => undefined,
};

/** 開關照這個順序排；預設值照 brief〈設定與同步〉。 */
const ROWS: readonly Row[] = [
	{
		kind: "checkbox",
		id: "incognito",
		label: "無痕",
		checked: false,
		apply: (session, on) => {
			session.setIncognito(on);
			return undefined;
		},
	},
	{
		// 關掉時不寫學習資料，已學的照用；和無痕不同，前文照用（brief〈學習與排位〉）。
		kind: "checkbox",
		id: "learning",
		label: "學習",
		checked: true,
		apply: (session, on) => {
			session.setLearning(on);
			return undefined;
		},
	},
	{
		kind: "checkbox",
		id: "simplified",
		label: "簡體",
		checked: false,
		apply: (session, on) => session.setSimplified(on),
	},
	{
		// 繁體模式的簡體字（门、农）排在同碼的繁體字之後，同 macOS 速成。
		kind: "checkbox",
		id: "simplified-after-traditional",
		label: "繁體字優先",
		checked: true,
		apply: (session, on) => session.setSimplifiedAfterTraditional(on),
	},
	{
		kind: "checkbox",
		id: "english-candidates",
		label: "英文候選",
		checked: true,
		apply: (session, on) => session.setEnglishCandidates(on),
	},
	{
		// 整串的字詞後面接它的 emoji（M3 票 16）。
		kind: "checkbox",
		id: "emoji-candidates",
		label: "emoji 候選",
		checked: true,
		apply: (session, on) => session.setEmojiCandidates(on),
	},
	{
		// 學打過的英文串：打過 2 次的中文打完整串時排第 2，打過 3 次的打開頭 3 碼也給（M4 票 14）。
		kind: "checkbox",
		id: "english-user-words",
		label: "英文使用者詞",
		checked: true,
		apply: (session, on) => session.setEnglishUserWords(on),
	},
	{
		// 容錯：觸控點一下的鄰鍵字母換進輸入碼，換出來的讀法從候選列第 2 格起插、不搶首選；實體鍵盤不換（M4 票 09c）。
		kind: "checkbox",
		id: "neighbor-readings",
		label: "容錯（觸控鄰鍵）",
		checked: true,
		apply: (session, on) => session.setNeighborReadings(on),
	},
	{
		// 上屏之後候選列給接下來可能要打的字詞（M3 票 13），數字鍵或點選選。
		kind: "checkbox",
		id: "association",
		label: "聯想",
		checked: true,
		apply: (session, on) => session.setAssociation(on),
	},
	{
		// 數字後的句點、冒號用半形；逗號照舊全形。
		kind: "checkbox",
		id: "digit-separators",
		label: "數字後半形",
		checked: true,
		apply: (session, on) => {
			session.setDigitSeparators(on);
			return undefined;
		},
	},
	RERANK,
	{
		kind: "select",
		id: "shift-switch",
		label: "組字中單按 Shift",
		options: [
			["rime", "照 Rime：左留碼接英文、右先上屏"],
			["commit-text", "左右都先上屏中文"],
			["inline-ascii", "左右都留碼接英文"],
		],
		selected: "rime",
		apply: (session, value) => {
			session.setShiftSwitch(shiftSwitch(value));
			return undefined;
		},
	},
];

/** 一列畫好的樣子：`<label>` 包著控制項，標籤就是無障礙名稱；`applyTo` 照控制項目前的值套用。 */
interface Rendered {
	label: HTMLLabelElement;
	control: HTMLInputElement | HTMLSelectElement;
	applyTo(session: Session): Snapshot | undefined;
}

function renderRow(row: Row): Rendered {
	const label = document.createElement("label");
	if (row.kind === "checkbox") {
		const input = document.createElement("input");
		input.type = "checkbox";
		input.id = row.id;
		input.checked = row.checked;
		label.append(input, ` ${row.label}`);
		return {
			label,
			control: input,
			applyTo: (session) => row.apply(session, input.checked),
		};
	}
	const select = document.createElement("select");
	select.id = row.id;
	select.append(
		...row.options.map(([value, text]) => {
			const option = document.createElement("option");
			option.value = value;
			option.textContent = text;
			option.selected = value === row.selected;
			return option;
		}),
	);
	label.append(`${row.label} `, select);
	return {
		label,
		control: select,
		applyTo: (session) => row.apply(session, select.value),
	};
}

export interface Settings {
	/**
	 * 接上核心：照各開關目前的值套用一次，之後改了就再套用。`show` 收到會重列候選的
	 * `Snapshot`，用完由這裡釋放；`afterChange` 在使用者改了開關之後呼叫（網頁把焦點還給輸入框）。
	 */
	connect(
		session: Session,
		show: (snapshot: Snapshot) => void,
		afterChange: () => void,
	): void;
	/** 目前的重排模式。 */
	rerank(): RerankMode;
}

/** 在 `container` 畫出全部開關；接上核心前就畫，載入中也看得到。 */
export function mountSettings(container: HTMLElement): Settings {
	const rendered = ROWS.map((row) => ({ row, ...renderRow(row) }));
	container.replaceChildren(...rendered.map(({ label }) => label));
	const rerank = rendered.find(({ row }) => row === RERANK)?.control;
	if (!rerank) {
		throw new Error("設定區少了重排選單");
	}
	return {
		connect(session, show, afterChange) {
			for (const { control, applyTo } of rendered) {
				const applyNow = (): void => {
					const snapshot = applyTo(session);
					if (snapshot) {
						try {
							show(snapshot);
						} finally {
							snapshot.free();
						}
					}
				};
				applyNow();
				control.addEventListener("change", () => {
					applyNow();
					afterChange();
				});
			}
		},
		rerank: () => rerankMode(rerank.value),
	};
}
