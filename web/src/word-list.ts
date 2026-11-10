// 匯入詞庫（plan §2.4、§3.1，M3 票 17）：選一份詞表，先在頁面上列出會加幾個詞、略過哪些列，
// 使用者確認後加進使用者詞。同匯入學習資料的流程，不用瀏覽器原生的對話框；格式與規則在核心。

import { element, message } from "./dom.ts";
import type { Session, WordListReport } from "./wasm/qim_wasm.js";

/** 略過的列最多列幾列，其餘只寫列數。 */
const SHOWN_SKIPPED = 20;

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
	let pending: { name: string; text: string } | undefined;

	const closeConfirm = (): void => {
		pending = undefined;
		confirm.hidden = true;
	};
	/** 列出略過與沒設觸發詞的列，用完釋放 `report`。 */
	const showLines = (report: WordListReport): void => {
		const lines = [...report.skipped, ...report.withoutTrigger];
		report.free();
		const items = lines.slice(0, SHOWN_SKIPPED).map((line) => {
			const item = document.createElement("li");
			item.textContent = line;
			return item;
		});
		if (lines.length > SHOWN_SKIPPED) {
			const more = document.createElement("li");
			more.textContent = `還有 ${lines.length - SHOWN_SKIPPED} 列`;
			items.push(more);
		}
		skippedList.replaceChildren(...items);
	};

	file.addEventListener("change", async () => {
		const chosen = file.files?.[0];
		file.value = "";
		closeConfirm();
		skippedList.replaceChildren();
		if (!chosen) {
			return;
		}
		const text = await chosen.text();
		let report: WordListReport;
		try {
			report = session.checkWordList(text);
		} catch (error) {
			status.textContent = `匯入失敗，使用者詞沒動：${message(error)}`;
			return;
		}
		const { added, format } = report;
		const skipped = report.skipped.length;
		showLines(report);
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
				showLines(report);
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
			skippedList.replaceChildren();
			status.textContent = "沒有匯入，使用者詞沒動";
			afterChange();
		},
	);
}
