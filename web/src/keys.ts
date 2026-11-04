/** 網頁轉給核心的按鍵；對應 `qim_core::Key` 裡網頁用到的部分。 */
export type Key =
	| { kind: "letter"; char: string }
	| { kind: "punct"; char: string }
	| { kind: "digit"; digit: number }
	| { kind: "space" }
	| { kind: "enter" }
	| { kind: "backspace" }
	| { kind: "escape" }
	| { kind: "pageUp" }
	| { kind: "pageDown" }
	| { kind: "left" }
	| { kind: "right" }
	| { kind: "other" };

/** `toKey` 讀的 `KeyboardEvent` 欄位。 */
export type KeyEvent = Pick<
	KeyboardEvent,
	"key" | "isComposing" | "ctrlKey" | "metaKey" | "altKey"
>;

/** 不帶資料的鍵：`KeyboardEvent.key` → `Key`；`-` / `=` 也是翻頁（brief §4.3）。 */
const NAMED: Record<string, Key> = {
	" ": { kind: "space" },
	Enter: { kind: "enter" },
	Backspace: { kind: "backspace" },
	Escape: { kind: "escape" },
	PageUp: { kind: "pageUp" },
	"-": { kind: "pageUp" },
	PageDown: { kind: "pageDown" },
	"=": { kind: "pageDown" },
	ArrowLeft: { kind: "left" },
	ArrowRight: { kind: "right" },
};

/**
 * 鍵盤事件轉成 `Key`；帶 Ctrl、Meta、Alt 或屬於別的輸入法的事件回 `null`，直接留給瀏覽器。
 * 其他沒有對應的鍵是 `other`：組字中核心吃下，免得 Tab、Delete、方向鍵改到輸入框。
 */
export function toKey(event: KeyEvent): Key | null {
	if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey) {
		return null;
	}
	if (Object.hasOwn(NAMED, event.key)) {
		return NAMED[event.key];
	}
	if (/^[a-zA-Z]$/.test(event.key)) {
		return { kind: "letter", char: event.key };
	}
	if (/^[0-9]$/.test(event.key)) {
		return { kind: "digit", digit: Number(event.key) };
	}
	// 其餘的可見 ASCII 字元：標點與符號（Shift 加數字鍵的 `!`、`@` 也是）。
	if (/^[!-/:-@[-`{-~]$/.test(event.key)) {
		return { kind: "punct", char: event.key };
	}
	return { kind: "other" };
}
