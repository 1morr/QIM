// 匯入詞庫（plan §2.4、§3.1，M3 票 17）：選一份詞表，先在頁面上列出會加幾個詞、略過哪些列，
// 使用者確認後加進使用者詞。同匯入學習資料的流程，不用瀏覽器原生的對話框；格式與規則在核心。

import { element, message } from "./dom.ts";
import type { Session, WordListReport } from "./wasm/qim_wasm.js";

/** 每張清單最多列幾列，其餘只寫列數。 */
const SHOWN_LINES = 20;

/** 把 `lines` 畫進 `list`，超過 [`SHOWN_LINES`] 的只寫列數。 */
function showLines(list: HTMLUListElement, lines: string[]): void {
	const items = lines.slice(0, SHOWN_LINES).map((line) => {
		const item = document.createElement("li");
		item.textContent = line;
		return item;
	});
	if (lines.length > SHOWN_LINES) {
		const more = document.createElement("li");
		more.textContent = `還有 ${lines.length - SHOWN_LINES} 列`;
		items.push(more);
	}
	list.replaceChildren(...items);
}

/**
 * 接上匯入詞庫。`save` 存學習資料；`afterChange` 在匯入或取消之後呼叫（網頁把焦點還給輸入框）。
 */
export function mountWordList(
	session: Session,
	save: () => void,
	afterChange: () => void,
): void {
	const file = element<HTMLInputElement>("#import-word-list");
	const confirm = element<HTMLElement>("#confirm-word-list");
	const confirmText = element<HTMLElement>("#confirm-word-list-text");
	const status = element<HTMLElement>("#word-list-status");
	const skippedList = element<HTMLUListElement>("#word-list-skipped");
	const unusedList = element<HTMLUListElement>("#word-list-unused");
	let pending: { name: string; text: string } | undefined;
	/** 每選一次檔加一；讀完檔時已換過檔就不理（後選的檔先讀完時不被先選的蓋掉）。 */
	let choice = 0;

	const closeConfirm = (): void => {
		pending = undefined;
		confirm.hidden = true;
	};
	/** 列出略過的列與沒當觸發詞的快捷鍵，用完釋放 `report`。 */
	const showReport = (report: WordListReport): void => {
		showLines(skippedList, report.skipped);
		showLines(unusedList, report.unusedShortcuts);
		report.free();
	};
	const clearLines = (): void => {
		skippedList.replaceChildren();
		unusedList.replaceChildren();
	};

	file.addEventListener("change", async () => {
		const chosen = file.files?.[0];
		file.value = "";
		closeConfirm();
		clearLines();
		const current = ++choice;
		if (!chosen) {
			return;
		}
		const text = await chosen.text();
		if (current !== choice) {
			return;
		}
		let report: WordListReport;
		try {
			report = session.checkWordList(text);
		} catch (error) {
			status.textContent = `匯入失敗，使用者詞沒動：${message(error)}`;
			return;
		}
		const { added, format } = report;
		const skipped = report.skipped.length;
		showReport(report);
		if (added === 0) {
			status.textContent = `${chosen.name} 沒有可加的詞`;
			return;
		}
		pending = { name: chosen.name, text };
		confirmText.textContent = `${chosen.name}（${format}）會加 ${added} 個詞、略過 ${skipped} 列。加進使用者詞，原本的學習資料不動。確定匯入？`;
		status.textContent = "";
		confirm.hidden = false;
	});

	element<HTMLButtonElement>("#confirm-word-list-ok").addEventListener(
		"click",
		() => {
			if (!pending) {
				return;
			}
			const { name, text } = pending;
			closeConfirm();
			try {
				const report = session.importWordList(text);
				status.textContent = `已從 ${name} 加入 ${report.added} 個詞`;
				showReport(report);
				save();
			} catch (error) {
				status.textContent = `匯入失敗，使用者詞沒動：${message(error)}`;
			}
			afterChange();
		},
	);
	element<HTMLButtonElement>("#confirm-word-list-cancel").addEventListener(
		"click",
		() => {
			closeConfirm();
			clearLines();
			status.textContent = "沒有匯入，使用者詞沒動";
			afterChange();
		},
	);
}
