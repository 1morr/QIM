/** 網頁轉給核心的按鍵；對應 `qim_core::Key`，本票只用到字母與空白。 */
export type Key = { kind: "letter"; char: string } | { kind: "space" };

/** 鍵盤事件轉成 `Key`；不是輸入法要的鍵回 `null`，留給瀏覽器。 */
export function toKey(event: KeyboardEvent): Key | null {
	if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey) {
		return null;
	}
	if (/^[a-z]$/.test(event.key)) {
		return { kind: "letter", char: event.key };
	}
	if (event.key === " ") {
		return { kind: "space" };
	}
	return null;
}
