/**
 * 網頁轉給核心的按鍵：種類與值照 `qim_core::Key::parse`，原樣交給綁定的 `Session.press`（網頁的按鍵都經它）。
 * 加一種鍵只改這裡與核心。
 */
export type Key =
	| { kind: "letter"; value: string }
	| { kind: "punct"; value: string }
	| { kind: "digit"; value: string }
	| { kind: "toggleAscii"; value: ShiftKey }
	| {
			kind:
				| "space"
				| "enter"
				| "backspace"
				| "escape"
				| "pageUp"
				| "pageDown"
				| "left"
				| "right"
				| "other"
				| "modifier";
			value?: undefined;
	  };

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
 * 單按時沒有字元的修飾鍵（`KeyboardEvent.key`），照 MDN「Modifier keys」一表；Rime 不記這類鍵，
 * 送核心的 `modifier` 不結束數字（brief §4.7）。
 */
const MODIFIERS = new Set([
	"Alt",
	"AltGraph",
	"CapsLock",
	"Control",
	"Fn",
	"FnLock",
	"Hyper",
	"Meta",
	"NumLock",
	"ScrollLock",
	"Shift",
	"Super",
	"Symbol",
	"SymbolLock",
]);

/**
 * 鍵盤事件轉成 `Key`；屬於別的輸入法的事件回 `null`，直接留給瀏覽器。帶 Ctrl、Meta、Alt 的
 * 組合鍵沒有組字時也回 `null`；`composing`（組字中）時是 `other`，由核心吃下：組字中每個鍵都
 * 吃下，否則 Cmd+V 貼進輸入框而組字還在（brief §4.3）。
 * 單按修飾鍵（Ctrl、Cmd、Alt、CapsLock…，事件本身也帶 `ctrlKey` 這類旗標）是 `modifier`。
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
	if (MODIFIERS.has(event.key)) {
		return { kind: "modifier" };
	}
	if (event.ctrlKey || event.metaKey || event.altKey) {
		return composing ? { kind: "other" } : null;
	}
	if (inlineAscii && (event.key === "-" || event.key === "=")) {
		return { kind: "punct", value: event.key };
	}
	if (Object.hasOwn(NAMED, event.key)) {
		return NAMED[event.key];
	}
	if (/^[a-zA-Z]$/.test(event.key)) {
		return { kind: "letter", value: event.key };
	}
	if (/^[0-9]$/.test(event.key)) {
		return { kind: "digit", value: event.key };
	}
	// 其餘的可見 ASCII 字元：標點與符號（Shift 加數字鍵的 `!`、`@` 也是）。
	if (/^[!-/:-@[-`{-~]$/.test(event.key)) {
		return { kind: "punct", value: event.key };
	}
	return { kind: "other" };
}

/** `physicalLetter` 讀的 `KeyboardEvent` 欄位。 */
export type PhysicalKeyEvent = KeyEvent &
	Pick<KeyboardEvent, "code" | "shiftKey">;

/**
 * 實體鍵盤的字母鍵照實體鍵位送機率（`Key::Weighted`，M4 票 07）時回 `KeyboardEvent.code`，交給綁定的
 * `Session.pressCode`；其他照 `toKey`，回 `null`。速成碼照實體鍵位：`code` 是 `KeyA`–`KeyZ` 就送，
 * 不看 `key`，換 Dvorak 等配列時打的碼不變（macOS 的輸入法也把字母鍵固定成美式鍵盤）。
 * 英文（`ascii`：英文直通與臨時英文）打的是英文，照配列（`key`）；大寫字母（Shift、CapsLock）、
 * 組合鍵與別的輸入法的事件也照舊。
 */
export function physicalLetter(
	event: PhysicalKeyEvent,
	ascii: boolean,
): string | null {
	if (
		ascii ||
		event.isComposing ||
		event.ctrlKey ||
		event.metaKey ||
		event.altKey ||
		event.shiftKey ||
		/^[A-Z]$/.test(event.key) ||
		!/^Key[A-Z]$/.test(event.code)
	) {
		return null;
	}
	return event.code;
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
 * 別的鍵（打大寫字母就不算）。照 librime `ascii_composer`：按下時武裝，任何別的鍵按下或放開就解除。
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

	/**
	 * `keyup`：放開的是 Shift、武裝中、沒超過時限，就是單按，回傳哪一邊；否則 `undefined`。
	 * 放開別的鍵也解除：按住 a 時按下 Shift、先放開 a 不算單按（librime 的 other keys）。
	 */
	up(code: string, now: number): ShiftKey | undefined {
		const side = SHIFT_SIDE[code];
		if (side === undefined) {
			this.#since = undefined;
			return undefined;
		}
		if (this.#since === undefined) {
			return undefined;
		}
		const lone = now - this.#since < LONE_SHIFT_MS;
		this.#since = undefined;
		return lone ? side : undefined;
	}
}
