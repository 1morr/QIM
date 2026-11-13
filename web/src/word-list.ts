// 匯入詞庫（plan〈學習與排位〉〈網頁 prototype〉，M3 票 17）：選一份詞表，先在頁面上列出會加幾個詞、略過哪些列，
// 使用者確認後加進使用者詞。同匯入學習資料的流程，不用瀏覽器原生的對話框；格式與規則在核心。

import { element, message } from "./dom.ts";
import type { Session, WordListReport } from "./wasm/qim_wasm.js";

/** 每張清單最多列幾列，其餘只寫列數。 */
const SHOWN_LINES = 20;

/**
 * 把 `lines` 畫進 `list`，超過 [`SHOWN_LINES`] 的只寫列數；`marked` 裡的列加 `className`。
 */
function showLines(
	list: HTMLUListElement,
	lines: string[],
	marked: string[] = [],
	className = "",
): void {
	const items = lines.slice(0, SHOWN_LINES).map((line) => {
		const item = document.createElement("li");
		item.textContent = line;
		if (marked.includes(line)) {
			item.className = className;
		}
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
	const ok = element<HTMLButtonElement>("#confirm-word-list-ok");
	const shortcuts = element<HTMLElement>("#word-list-shortcuts");
	// Gboard 的快捷鍵是在別的鍵盤設的，在速成多半撞到別的碼：預設不設成觸發詞（plan〈學習與排位〉）。
	const asTriggers = element<HTMLInputElement>("#word-list-as-triggers");
	const shortcutsText = element<HTMLElement>("#word-list-shortcuts-text");
	const triggerList = element<HTMLUListElement>("#word-list-triggers");
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
	/** 列出略過的列與不能當觸發詞的快捷鍵，用完釋放 `report`。 */
	const showReport = (report: WordListReport): void => {
		showLines(skippedList, report.skipped);
		showLines(unusedList, report.unusedShortcuts);
		report.free();
	};
	const clearLines = (): void => {
		skippedList.replaceChildren();
		unusedList.replaceChildren();
	};
	/**
	 * 照勾選框檢查待確認的檔、更新確認框；回傳要不要問（有可加的詞，或有能設成觸發詞的快捷鍵
	 * 勾了才加得進去）。整份讀不了時丟例外。
	 */
	const preview = (name: string, text: string): boolean => {
		const report = session.checkWordList(text, asTriggers.checked);
		const { added, format } = report;
		const skipped = report.skipped.length;
		const triggers = report.triggerShortcuts;
		const displacing = report.displacingShortcuts;
		showReport(report);
		shortcuts.hidden = triggers.length === 0;
		shortcutsText.textContent = `${triggers.length} 列的快捷鍵能設成觸發詞，其中 ${displacing.length} 列撞到速成既有的碼、會搶走首選（標 ⚠）。快捷鍵是在別的鍵盤設的，預設不設；不設時詞照加，由字的碼打得出。`;
		showLines(triggerList, triggers, displacing, "displacing");
		confirmText.textContent = `${name}（${format}）會加 ${added} 個詞、略過 ${skipped} 列。加進使用者詞，原本的學習資料不動。確定匯入？`;
		ok.disabled = added === 0;
		return added > 0 || triggers.length > 0;
	};

	file.addEventListener("change", async () => {
		const chosen = file.files?.[0];
		file.value = "";
		closeConfirm();
		clearLines();
		asTriggers.checked = false;
		const current = ++choice;
		if (!chosen) {
			return;
		}
		const text = await chosen.text();
		if (current !== choice) {
			return;
		}
		let ask: boolean;
		try {
			ask = preview(chosen.name, text);
		} catch (error) {
			status.textContent = `匯入失敗，使用者詞沒動：${message(error)}`;
			return;
		}
		if (!ask) {
			status.textContent = `${chosen.name} 沒有可加的詞`;
			return;
		}
		pending = { name: chosen.name, text };
		status.textContent = "";
		confirm.hidden = false;
	});
	// 勾了或取消勾選要重算：有字沒有碼的詞只有用快捷鍵當碼才加得進去，會加幾個跟著變。
	asTriggers.addEventListener("change", () => {
		if (pending) {
			preview(pending.name, pending.text);
		}
	});

	ok.addEventListener("click", () => {
		if (!pending) {
			return;
		}
		const { name, text } = pending;
		closeConfirm();
		try {
			const report = session.importWordList(text, asTriggers.checked);
			status.textContent = `已從 ${name} 加入 ${report.added} 個詞`;
			showReport(report);
			save();
		} catch (error) {
			status.textContent = `匯入失敗，使用者詞沒動：${message(error)}`;
		}
		afterChange();
	});
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
