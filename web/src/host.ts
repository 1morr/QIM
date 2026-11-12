// 網頁當宿主程式：上屏到輸入框，觸控鍵盤上核心沒吃下的鍵照輸入框的預設動作。

import type { Snapshot } from "./wasm/qim_wasm.js";

/** 上屏：插入輸入框游標處，取代選取的文字。 */
export function insert(text: HTMLTextAreaElement, commit: string): void {
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
