/**
 * 「回報」（M1 票 12）：把剛才那次組字存成 `bench/fixed.tsv` 的一行
 * （`輸入碼<TAB>期望首選<TAB>來源`，plan〈首選字基準〉）。
 *
 * 使用者要的字就是那次組字選字上屏的字：首選不對時，使用者照常選字（數字鍵、點候選、
 * 選開頭一段、移到中間分段改選）打出要的字，用空白、數字鍵或點候選上屏，然後按「回報」。選不出來的，在回報清單裡直接改那一行。
 * 組字中按標點、Enter、大寫字母也會上屏，但上屏的是首選加標點或輸入碼，不是選出來的字，
 * 不算數。
 */

/** 一次上屏的組字。 */
export interface Composed {
	/** 整串輸入碼。 */
	input: string;
	/** 上屏的字，當作期望首選。 */
	wanted: string;
	/** 打完整串、還沒選字時的首選；選開頭一段後又打了字就不知道，是空字串。 */
	top: string;
}

/** 回報要看的 `Snapshot` 欄位，與這個鍵是不是選字。 */
export interface Observed {
	input: string;
	selected: string;
	candidates: readonly string[];
	commit?: string;
	/** 空白、數字鍵、點候選（觸控的空白鍵）：上屏的是使用者選的字。 */
	chosen: boolean;
}

export interface ReportState {
	/** 上一個 snapshot 的整串輸入碼。 */
	input: string;
	/** 最近一次輸入碼變動、沒有選定分段時的輸入碼與首選。 */
	topAtInput?: { input: string; top: string };
	/** 最近一次上屏的組字。 */
	last?: Composed;
}

export const initialReport: ReportState = { input: "" };

/** 核心每回一個 snapshot 就交給它。 */
export function observe(state: ReportState, snapshot: Observed): ReportState {
	let { topAtInput, last } = state;
	if (snapshot.commit !== undefined && state.input !== "") {
		last = snapshot.chosen
			? {
					input: state.input,
					wanted: snapshot.commit,
					top: topAtInput?.input === state.input ? topAtInput.top : "",
				}
			: undefined;
	}
	// 輸入碼一變，候選回到第一頁；翻頁與選字不改輸入碼，不會蓋掉打完時的首選。
	if (
		snapshot.input !== state.input &&
		snapshot.input !== "" &&
		snapshot.selected === ""
	) {
		topAtInput = { input: snapshot.input, top: snapshot.candidates[0] ?? "" };
	}
	return { input: snapshot.input, topAtInput, last };
}

/** `fixed.tsv` 的一行，不含換行；來源欄註明當時的首選，之後看得出原本錯成什麼。 */
export function fixedLine(composed: Composed): string {
	const wanted = composed.wanted.replace(/[\t\r\n]/g, "");
	const source =
		composed.top === "" ? "網頁回報" : `網頁回報（首選 ${composed.top}）`;
	return `${composed.input}\t${wanted}\t${source}`;
}
