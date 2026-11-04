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
	| { kind: "toggleAscii" }
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
 * 鍵盤事件轉成 `Key`；屬於別的輸入法的事件回 `null`，直接留給瀏覽器。帶 Ctrl、Meta、Alt 的
 * 組合鍵沒有組字時也回 `null`；`composing`（組字中）時是 `other`，由核心吃下：組字中每個鍵都
 * 吃下，否則 Cmd+V 貼進輸入框而組字還在（brief §4.3）。
 * 其他沒有對應的鍵是 `other`：組字中核心吃下，免得 Tab、Delete、方向鍵改到輸入框。
 */
export function toKey(event: KeyEvent, composing = false): Key | null {
	if (event.isComposing) {
		return null;
	}
	if (event.ctrlKey || event.metaKey || event.altKey) {
		return composing ? { kind: "other" } : null;
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

/** Shift 按下到放開多久以內算單按：librime `ascii_composer` 的 500 ms。 */
export const LONE_SHIFT_MS = 500;

/**
 * 判定單按 Shift（切中英，brief §4.3）：Shift 按下到放開不超過 `LONE_SHIFT_MS`、中間沒有按
 * 別的鍵（打大寫字母就不算）。照 librime `ascii_composer`：按下時武裝，任何別的鍵按下就解除。
 */
export class LoneShift {
	/** Shift 按下的時間；沒有武裝時是 `undefined`。 */
	#since: number | undefined;

	/** `keydown`：Shift 開始計時（自動重複不重新計時），其他鍵解除。 */
	down(key: string, repeat: boolean, now: number): void {
		if (key !== "Shift") {
			this.#since = undefined;
		} else if (!repeat) {
			this.#since = now;
		}
	}

	/** `keyup`：放開的是武裝中的 Shift、沒超過時限，就是單按。 */
	up(key: string, now: number): boolean {
		if (key !== "Shift" || this.#since === undefined) {
			return false;
		}
		const lone = now - this.#since < LONE_SHIFT_MS;
		this.#since = undefined;
		return lone;
	}
}
