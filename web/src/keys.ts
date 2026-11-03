/** 網頁轉給核心的按鍵；對應 `qim_core::Key` 裡網頁用到的部分。 */
export type Key =
	| { kind: "letter"; char: string }
	| { kind: "digit"; digit: number }
	| { kind: "space" }
	| { kind: "backspace" }
	| { kind: "escape" }
	| { kind: "pageUp" }
	| { kind: "pageDown" };

/** 不帶資料的鍵：`KeyboardEvent.key` → `Key`；`-` / `=` 也是翻頁（brief §4.3）。 */
const NAMED: Record<string, Key> = {
	" ": { kind: "space" },
	Backspace: { kind: "backspace" },
	Escape: { kind: "escape" },
	PageUp: { kind: "pageUp" },
	"-": { kind: "pageUp" },
	PageDown: { kind: "pageDown" },
	"=": { kind: "pageDown" },
};

/** 鍵盤事件轉成 `Key`；不是輸入法要的鍵回 `null`，留給瀏覽器。 */
export function toKey(event: KeyboardEvent): Key | null {
	if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey) {
		return null;
	}
	if (/^[a-z]$/.test(event.key)) {
		return { kind: "letter", char: event.key };
	}
	if (/^[0-9]$/.test(event.key)) {
		return { kind: "digit", digit: Number(event.key) };
	}
	return Object.hasOwn(NAMED, event.key) ? NAMED[event.key] : null;
}
