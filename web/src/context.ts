// 上下文（plan §2.1）：把輸入框游標前的文字交給核心（`Session.setContext`），候選照前文排、數字後半形
// 看前文。聚焦、移動游標（點擊、方向鍵、選取）、輸入框內容改了（上屏、宿主打的字、刪字）之後都給，
// 失焦時叫核心忘掉；無痕時不給（brief §8）。

import type { Session } from "./wasm/qim_wasm.js";

/**
 * 游標（選取的開頭）前最多 `limit` 個字。先切一段 UTF-16 再分字，不必每次分整個輸入框；多切一個碼元，
 * 切在代理對中間時落單的半個一定被 `slice(-limit)` 丟掉。
 */
export function textBefore(
	value: string,
	cursor: number,
	limit: number,
): string {
	const start = Math.max(0, cursor - limit * 2 - 1);
	return Array.from(value.slice(start, cursor)).slice(-limit).join("");
}

/**
 * 接上輸入框，每次給游標前最多 `limit` 個字（綁定的 `contextLimit()`）；回傳的函式立刻給一次
 * （例如關掉無痕之後）。`changed` 在每次給前文或失焦忘掉之後呼叫：游標移走、失焦時核心收起聯想
 * （M3 票 13），網頁要跟著重畫候選列。`afterLeave` 在失焦、叫核心忘掉前文之後呼叫：核心這時可能記住
 * 剛打完的 email（記住的字串，M3 票 15），網頁要存檔。
 */
export function mountContext(
	text: HTMLTextAreaElement,
	session: Session,
	limit: number,
	changed: () => void = () => {},
	afterLeave: () => void = () => {},
): () => void {
	// 不比對上次給的：核心在交還宿主的退格、方向鍵後會忘掉前文，重給同一段也要給。
	const sync = (): void => {
		if (!session.incognito) {
			session.setContext(textBefore(text.value, text.selectionStart, limit));
			changed();
		}
	};
	text.addEventListener("focus", sync);
	// 失焦等於換欄位：核心忘掉前文，回來時 focus 重給（plan §2.1）。
	text.addEventListener("blur", () => {
		session.clearContext();
		afterLeave();
		changed();
	});
	text.addEventListener("input", sync);
	// 輸入框裡的游標移動在 Chromium 也會在 document 觸發 `selectionchange`。
	document.addEventListener("selectionchange", () => {
		if (document.activeElement === text) {
			sync();
		}
	});
	return sync;
}
