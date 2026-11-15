// 網頁當宿主程式：上屏到輸入框，觸控鍵盤上核心沒吃下的鍵照輸入框的預設動作。

import type { Snapshot } from "./wasm/qim_wasm.js";

/**
 * 游標（有選取時是選取的開頭）`cursor` 前刪 `count` 個字時要刪到哪裡：UTF-16 的位置。以 Unicode
 * scalar 計、不切開代理對，同核心 `Snapshot.delete_before` 的單位（ADR 0008）；不夠刪時刪到開頭。
 */
export function deleteStart(
	value: string,
	cursor: number,
	count: number,
): number {
	const before = Array.from(value.slice(0, cursor));
	const removed = before.slice(Math.max(0, before.length - count));
	return cursor - removed.join("").length;
}

/**
 * 上屏：先刪游標前 `deleteBefore` 個字（核心的 `delete_before`，ADR 0008），再把 `commit` 插入游標處、
 * 取代選取的文字。
 */
export function insert(
	text: HTMLTextAreaElement,
	commit: string,
	deleteBefore = 0,
): void {
	const start = deleteStart(text.value, text.selectionStart, deleteBefore);
	if (start < text.selectionStart) {
		// 選取跟著往前挪、不收起（"preserve"），接著的上屏照舊取代它。
		text.setRangeText("", start, text.selectionStart, "preserve");
	}
	text.setRangeText(commit, text.selectionStart, text.selectionEnd, "end");
	text.dispatchEvent(new Event("input", { bubbles: true }));
}

/** 觸控鍵盤上核心沒吃下的動作：照輸入框收到實體按鍵時的預設動作。 */
export function hostDefault(
	text: HTMLTextAreaElement,
	snapshot: Snapshot,
): void {
	switch (snapshot.action) {
		case "letter":
		case "weighted":
		case "punct":
			insert(text, snapshot.character ?? "");
			return;
		case "space":
			insert(text, " ");
			return;
		case "enter":
			insert(text, "\n");
			return;
		case "backspace": {
			const { selectionStart: start, selectionEnd: end } = text;
			if (start === end && start > 0) {
				// 刪游標前一個字元，不切開代理對。
				text.setRangeText("", deleteStart(text.value, start, 1), start, "end");
			} else {
				text.setRangeText("", start, end, "end");
			}
			text.dispatchEvent(new Event("input", { bubbles: true }));
			return;
		}
	}
}
