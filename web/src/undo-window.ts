// 退格撤銷的時限（plan §2.4〈退格撤銷〉）：核心不讀時間，殼計時。照 librime 的 3 秒
// （`UserDictionary::RevertRecentTransaction`，brief §12.2）：核心可能學到東西之後（上屏、交還瀏覽器的
// 空白與標點讓英文串打完）重新計時，3 秒到了叫核心讓撤銷紀錄失效（`Session.expireUndo`），之後的退格只刪字。

/** 學到東西之後多久還能用退格撤銷（毫秒）。 */
export const UNDO_WINDOW_MS = 3000;

/** 計時器：`restart` 從頭算 `ms`，到了呼叫一次 `expire`；再 `restart` 取消上一次的。 */
export function undoWindow(
	expire: () => void,
	ms: number = UNDO_WINDOW_MS,
): { restart: () => void } {
	let timer: ReturnType<typeof setTimeout> | undefined;
	return {
		restart: () => {
			clearTimeout(timer);
			timer = setTimeout(expire, ms);
		},
	};
}
