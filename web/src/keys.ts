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
	| { kind: "toggleAscii"; shift: ShiftKey }
	| { kind: "other" };

/** 單按的是哪一邊的 Shift；對應 `qim_core::ShiftKey`。 */
export type ShiftKey = "left" | "right";

/**
 * 組字中單按 Shift 的處理（`qim_core::ShiftSwitch`，brief §4.3）：`rime` 左 Shift 留碼接英文、
 * 右 Shift 先上屏組字；`commit-text` 左右都先上屏；`inline-ascii` 左右都留碼。
 */
export type ShiftSwitch = "rime" | "commit-text" | "inline-ascii";

/** 設定選單的值轉成 `ShiftSwitch`；不認得的當預設 `rime`。 */
export function shiftSwitch(value: string): ShiftSwitch {
	return value === "commit-text" || value === "inline-ascii" ? value : "rime";
}

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
 * `inlineAscii`（臨時英文，組字中單按留碼的 Shift）時 `-` / `=` 是字元，接在英文後面。
 */
export function toKey(
	event: KeyEvent,
	composing = false,
	inlineAscii = false,
): Key | null {
	if (event.isComposing) {
		return null;
	}
	if (event.ctrlKey || event.metaKey || event.altKey) {
		return composing ? { kind: "other" } : null;
	}
	if (inlineAscii && (event.key === "-" || event.key === "=")) {
		return { kind: "punct", char: event.key };
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

/** `KeyboardEvent.code` → 哪一邊的 Shift；不是 Shift 時是 `undefined`。 */
const SHIFT_SIDE: Record<string, ShiftKey> = {
	ShiftLeft: "left",
	ShiftRight: "right",
};

/**
 * 判定單按 Shift（切中英，brief §4.3）：Shift 按下到放開不超過 `LONE_SHIFT_MS`、中間沒有按
 * 別的鍵（打大寫字母就不算）。照 librime `ascii_composer`：按下時武裝，任何別的鍵按下就解除。
 * 用 `KeyboardEvent.code` 分辨左右：組字中左右 Shift 的處理可以不同（Rime 預設）。左右一起按時
 * 也照 librime：第二個不重新計時，先放開的那邊算。
 */
export class LoneShift {
	/** Shift 按下的時間；沒有武裝時是 `undefined`。 */
	#since: number | undefined;

	/**
	 * `keydown`：Shift 開始計時（自動重複、已在計時中的另一邊不重新計時），其他鍵解除。`modified`
	 * 是按著 Ctrl、Meta 或 Alt：Ctrl+Shift（Windows 切換輸入法）這類組合不算單按。
	 */
	down(code: string, repeat: boolean, now: number, modified = false): void {
		if (SHIFT_SIDE[code] === undefined || modified) {
			this.#since = undefined;
		} else if (!repeat && this.#since === undefined) {
			this.#since = now;
		}
	}

	/** `keyup`：放開的是 Shift、武裝中、沒超過時限，就是單按，回傳哪一邊；否則 `undefined`。 */
	up(code: string, now: number): ShiftKey | undefined {
		const side = SHIFT_SIDE[code];
		if (side === undefined || this.#since === undefined) {
			return undefined;
		}
		const lone = now - this.#since < LONE_SHIFT_MS;
		this.#since = undefined;
		return lone ? side : undefined;
	}
}
