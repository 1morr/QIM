import assert from "node:assert/strict";
import { test } from "node:test";
import {
	type KeyEvent,
	LoneShift,
	type PhysicalKeyEvent,
	physicalCode,
	shiftSwitch,
	toKey,
	usCharacter,
} from "../src/keys.ts";

function press(key: string, modifiers: Partial<KeyEvent> = {}): KeyEvent {
	return {
		key,
		isComposing: false,
		ctrlKey: false,
		metaKey: false,
		altKey: false,
		...modifiers,
	};
}

test("小寫與大寫字母都是 letter，大小寫照原樣", () => {
	assert.deepEqual(toKey(press("a")), { kind: "letter", value: "a" });
	assert.deepEqual(toKey(press("z")), { kind: "letter", value: "z" });
	assert.deepEqual(toKey(press("A")), { kind: "letter", value: "A" });
});

test("數字鍵是 digit", () => {
	assert.deepEqual(toKey(press("0")), { kind: "digit", value: "0" });
	assert.deepEqual(toKey(press("9")), { kind: "digit", value: "9" });
});

test("具名的鍵", () => {
	const named: [string, string][] = [
		[" ", "space"],
		["Enter", "enter"],
		["Backspace", "backspace"],
		["Escape", "escape"],
		["PageUp", "pageUp"],
		["PageDown", "pageDown"],
		["ArrowLeft", "left"],
		["ArrowRight", "right"],
	];
	for (const [key, kind] of named) {
		assert.deepEqual(toKey(press(key)), { kind }, key);
	}
});

test("- 與 = 是翻頁，不是標點", () => {
	assert.deepEqual(toKey(press("-")), { kind: "pageUp" });
	assert.deepEqual(toKey(press("=")), { kind: "pageDown" });
});

test("其餘可見 ASCII 字元是 punct", () => {
	for (const char of [
		",",
		".",
		"?",
		"!",
		"@",
		"/",
		"\\",
		"[",
		"]",
		"`",
		"~",
		"'",
		'"',
		"_",
		"{",
		"}",
		":",
		";",
	]) {
		assert.deepEqual(toKey(press(char)), { kind: "punct", value: char }, char);
	}
});

test("沒有對應的鍵是 other", () => {
	for (const key of [
		"Tab",
		"ArrowUp",
		"ArrowDown",
		"Home",
		"Delete",
		"F5",
		"é",
		"，",
		"ab",
	]) {
		assert.deepEqual(toKey(press(key)), { kind: "other" }, key);
	}
});

// MDN 的 Modifier keys 一表：單按沒有字元，Rime 不記，送核心不結束數字（M1 票 26b）。
// 單按 Ctrl 的事件本身也帶 ctrlKey，組字中與否都是 modifier。
test("單按修飾鍵是 modifier，組字中也是", () => {
	const modifiers = [
		["Control", { ctrlKey: true }],
		["Meta", { metaKey: true }],
		["Alt", { altKey: true }],
		["AltGraph", {}],
		["CapsLock", {}],
		["Shift", {}],
		["Fn", {}],
		["NumLock", {}],
	] as const;
	for (const [key, flags] of modifiers) {
		for (const composing of [false, true]) {
			assert.deepEqual(
				toKey(press(key, flags), composing),
				{ kind: "modifier" },
				`${key} ${composing}`,
			);
		}
	}
});

test("沒有組字時，帶 Ctrl、Meta、Alt 的事件留給瀏覽器", () => {
	for (const modifier of ["ctrlKey", "metaKey", "altKey"] as const) {
		assert.equal(toKey(press("a", { [modifier]: true })), null, modifier);
		assert.equal(toKey(press(",", { [modifier]: true })), null, modifier);
	}
});

// brief §4.3：組字中每個鍵都吃下，Cmd+V 貼上的話組字還留著。
test("組字中帶 Ctrl、Meta、Alt 的事件是 other，由核心吃下", () => {
	for (const modifier of ["ctrlKey", "metaKey", "altKey"] as const) {
		assert.deepEqual(
			toKey(press("v", { [modifier]: true }), true),
			{ kind: "other" },
			modifier,
		);
	}
	assert.deepEqual(toKey(press("a"), true), { kind: "letter", value: "a" });
});

test("在別的輸入法組字中的事件一律留給瀏覽器", () => {
	assert.equal(toKey(press("a", { isComposing: true })), null);
	assert.equal(toKey(press("a", { isComposing: true }), true), null);
});

// librime ascii_composer：Shift 按下到放開 500 ms 內、中間沒有別的鍵，才算單按。
// `down` / `up` 吃 `KeyboardEvent.code`，單按時回傳哪一邊（Rime 左右 Shift 的預設不同）。
test("單按 Shift：按下後 500 ms 內放開，回傳哪一邊", () => {
	const shift = new LoneShift();
	shift.down("ShiftLeft", false, 0);
	assert.equal(shift.up("ShiftLeft", 499), "left");
	shift.down("ShiftRight", false, 1000);
	assert.equal(shift.up("ShiftRight", 1010), "right");
});

test("Shift 按太久不算單按", () => {
	const shift = new LoneShift();
	shift.down("ShiftLeft", false, 0);
	assert.equal(shift.up("ShiftLeft", 500), undefined);
});

test("Shift 按著時按了別的鍵（打大寫）不算單按", () => {
	const shift = new LoneShift();
	shift.down("ShiftLeft", false, 0);
	shift.down("KeyA", false, 10);
	assert.equal(shift.up("ShiftLeft", 20), undefined);
});

// librime：第二個 Shift 按下時不重新計時，先放開的那邊切換，另一邊放開時已經解除。
test("左右 Shift 一起按：照第一個按下的計時，先放開的那邊算單按", () => {
	const shift = new LoneShift();
	shift.down("ShiftLeft", false, 0);
	shift.down("ShiftRight", false, 400);
	assert.equal(shift.up("ShiftRight", 450), "right");
	assert.equal(shift.up("ShiftLeft", 460), undefined);
	shift.down("ShiftLeft", false, 1000);
	shift.down("ShiftRight", false, 1400);
	assert.equal(shift.up("ShiftRight", 1500), undefined);
});

test("按住 Shift 的自動重複不重新計時", () => {
	const shift = new LoneShift();
	shift.down("ShiftRight", false, 0);
	shift.down("ShiftRight", true, 400);
	assert.equal(shift.up("ShiftRight", 600), undefined);
});

test("沒有先按下 Shift 不算，一次單按只算一次", () => {
	const shift = new LoneShift();
	assert.equal(shift.up("ShiftLeft", 0), undefined);
	shift.down("ShiftLeft", false, 0);
	assert.equal(shift.up("ShiftLeft", 20), "left");
	assert.equal(shift.up("ShiftLeft", 30), undefined);
});

// librime ascii_composer 的 other keys 分支：按下或放開別的鍵都解除（M1 票 26b）。
test("按住 a 時按下 Shift、先放開 a 再放開 Shift，不算單按", () => {
	const shift = new LoneShift();
	shift.down("KeyA", false, 0);
	shift.down("ShiftLeft", false, 10);
	assert.equal(shift.up("KeyA", 20), undefined);
	assert.equal(shift.up("ShiftLeft", 30), undefined);
});

// Ctrl+Shift 是 Windows 切換輸入法，Cmd+Shift 是各種快捷鍵，都不是單按。
test("帶 Ctrl、Meta、Alt 按下的 Shift 不算單按", () => {
	const shift = new LoneShift();
	shift.down("ShiftLeft", false, 0, true);
	assert.equal(shift.up("ShiftLeft", 10), undefined);
});

test("臨時英文中 - 與 = 是字元，不是翻頁", () => {
	assert.deepEqual(toKey(press("-"), true, true), {
		kind: "punct",
		value: "-",
	});
	assert.deepEqual(toKey(press("="), true, true), {
		kind: "punct",
		value: "=",
	});
	assert.deepEqual(toKey(press("-"), true), { kind: "pageUp" });
	assert.deepEqual(toKey(press("PageDown"), true, true), { kind: "pageDown" });
});

test("組字中單按 Shift 的設定值：認得的照原樣，其他當預設 rime", () => {
	assert.equal(shiftSwitch("commit-text"), "commit-text");
	assert.equal(shiftSwitch("inline-ascii"), "inline-ascii");
	assert.equal(shiftSwitch("rime"), "rime");
	assert.equal(shiftSwitch("left"), "rime");
});

/** 實體鍵盤上的一個鍵：`code` 是實體鍵位，`key` 照目前的配列。 */
function physical(
	code: string,
	key: string,
	modifiers: Partial<PhysicalKeyEvent> = {},
	locked: string[] = [],
): PhysicalKeyEvent {
	return {
		...press(key),
		code,
		shiftKey: false,
		getModifierState: (name: string) => locked.includes(name),
		...modifiers,
	};
}

test("中文模式的字母鍵照實體鍵位送 code（M4 票 07）", () => {
	assert.equal(physicalCode(physical("KeyF", "f"), false), "KeyF");
	assert.equal(physicalCode(physical("KeyQ", "q"), false), "KeyQ");
});

test("key 不同但 code 相同時結果相同：換 Dvorak 照實體鍵位", () => {
	// Dvorak 的實體 F 鍵打 u、實體 Q 鍵打 '。
	assert.equal(physicalCode(physical("KeyF", "u"), false), "KeyF");
	assert.equal(physicalCode(physical("KeyQ", "'"), false), "KeyQ");
});

test("英文（直通與臨時英文）的字母照配列打，不送 code", () => {
	assert.equal(physicalCode(physical("KeyF", "f"), true), null);
	assert.equal(physicalCode(physical("KeyF", "u"), true), null);
});

test("大寫字母、組合鍵與別的輸入法的事件不送 code", () => {
	assert.equal(
		physicalCode(physical("KeyF", "F", { shiftKey: true }), false),
		null,
	);
	// CapsLock 開著：沒按 Shift 也是大寫；Dvorak 的實體 Q 鍵打的 ' 看不出大小寫，照修飾鍵的狀態判斷。
	assert.equal(
		physicalCode(physical("KeyF", "F", {}, ["CapsLock"]), false),
		null,
	);
	assert.equal(
		physicalCode(physical("KeyQ", "'", {}, ["CapsLock"]), false),
		null,
	);
	// Linux 的 AltGr 加字母（打 € 這類字元）不帶 altKey。
	assert.equal(
		physicalCode(physical("KeyE", "€", {}, ["AltGraph"]), false),
		null,
	);
	for (const modifier of ["ctrlKey", "metaKey", "altKey"] as const) {
		assert.equal(
			physicalCode(physical("KeyF", "f", { [modifier]: true }), false),
			null,
			modifier,
		);
	}
	assert.equal(
		physicalCode(physical("KeyF", "f", { isComposing: true }), false),
		null,
	);
});

test("非字母鍵不送 code，即使配列讓它打出字母", () => {
	// Dvorak 的實體分號鍵打 s。
	assert.equal(physicalCode(physical("Semicolon", "s"), false), null);
	assert.equal(physicalCode(physical("Digit1", "1"), false), null);
	assert.equal(physicalCode(physical("Space", " "), false), null);
	assert.equal(physicalCode(physical("", "f"), false), null);
});

// 中文模式整個當美式鍵盤（使用者 2026-11-12 決定，M4 票 07）：字母與標點都照實體鍵位。

test("Dvorak：實體 W、E、Z、Q 鍵打美式的字母，標點鍵打美式的標點", () => {
	const dvorak: [string, string, string][] = [
		["KeyW", ",", "w"],
		["KeyE", ".", "e"],
		["KeyZ", ";", "z"],
		["KeyQ", "'", "q"],
		["Comma", "w", ","],
		["Period", "v", "."],
		["Semicolon", "s", ";"],
		["Quote", "-", "'"],
		["Slash", "z", "/"],
		["BracketLeft", "/", "["],
		["Minus", "[", "-"],
		["Equal", "]", "="],
	];
	for (const [code, key, us] of dvorak) {
		assert.equal(usCharacter(physical(code, key)), us, code);
	}
});

test("AZERTY：字母、標點與數字列照美式鍵盤", () => {
	const azerty: [string, string, string][] = [
		["KeyQ", "a", "q"],
		["KeyW", "z", "w"],
		["KeyZ", "w", "z"],
		["KeyM", ",", "m"],
		["Semicolon", "m", ";"],
		["Comma", ";", ","],
		["Period", ":", "."],
		["Slash", "!", "/"],
		["Quote", "ù", "'"],
		["Digit1", "&", "1"],
		["Digit2", "é", "2"],
		["Backquote", "²", "`"],
	];
	for (const [code, key, us] of azerty) {
		assert.equal(usCharacter(physical(code, key)), us, code);
	}
});

test("Shift 與 CapsLock 照美式鍵盤：Shift 換上排的符號，CapsLock 只管字母", () => {
	// AZERTY 的 Shift+實體 1 鍵打 1，美式是 !；Dvorak 的 Shift+實體 W 鍵打 <。
	assert.equal(usCharacter(physical("Digit1", "1", { shiftKey: true })), "!");
	assert.equal(usCharacter(physical("KeyW", "<", { shiftKey: true })), "W");
	assert.equal(usCharacter(physical("Comma", "W", { shiftKey: true })), "<");
	assert.equal(usCharacter(physical("KeyQ", "'", {}, ["CapsLock"])), "Q");
	assert.equal(
		usCharacter(physical("KeyQ", '"', { shiftKey: true }, ["CapsLock"])),
		"q",
	);
	assert.equal(usCharacter(physical("Digit1", "&", {}, ["CapsLock"])), "1");
});

test("美式鍵盤沒有的鍵、具名的鍵、組合鍵與 AltGr 不換", () => {
	for (const [code, key] of [
		["IntlBackslash", "<"],
		["Enter", "Enter"],
		["Backspace", "Backspace"],
		["ArrowLeft", "ArrowLeft"],
		["Numpad1", "1"],
		["ShiftLeft", "Shift"],
		["", "a"],
	]) {
		assert.equal(usCharacter(physical(code, key)), undefined, code);
	}
	for (const modifier of ["ctrlKey", "metaKey", "altKey"] as const) {
		assert.equal(
			usCharacter(physical("KeyQ", "a", { [modifier]: true })),
			undefined,
			modifier,
		);
	}
	assert.equal(
		usCharacter(physical("Digit2", "~", {}, ["AltGraph"])),
		undefined,
	);
	assert.equal(
		usCharacter(physical("KeyQ", "a", { isComposing: true })),
		undefined,
	);
});

test("key 已是美式鍵盤上那個字元時不換", () => {
	// 美式鍵盤、AZERTY 的實體 E 鍵、美式的 Shift+2。
	assert.equal(usCharacter(physical("KeyE", "e")), undefined);
	assert.equal(
		usCharacter(physical("KeyF", "F", { shiftKey: true })),
		undefined,
	);
	assert.equal(
		usCharacter(physical("Digit2", "@", { shiftKey: true })),
		undefined,
	);
	assert.equal(
		usCharacter(physical("Quote", '"', { shiftKey: true })),
		undefined,
	);
	// 沒按 Shift 卻是上排的字元：不是美式鍵盤這個狀態打的，照實體鍵位換。
	assert.equal(usCharacter(physical("Digit2", "@")), "2");
	// 同一個字元在美式鍵盤的別的鍵上（AZERTY 實體 1 鍵的 & 是美式 7 鍵的）照樣換。
	assert.equal(usCharacter(physical("Digit1", "&")), "1");
});

test("字母的大小寫照 key：macOS 的 Shift 加 CapsLock 是大寫，照系統", () => {
	assert.equal(
		usCharacter(physical("KeyA", "A", { shiftKey: true }, ["CapsLock"])),
		undefined,
	);
	// AZERTY 的實體 Q 鍵在 macOS 按 Shift 加 CapsLock 打 A：照美式是大寫 Q。
	assert.equal(
		usCharacter(physical("KeyQ", "A", { shiftKey: true }, ["CapsLock"])),
		"Q",
	);
	assert.equal(usCharacter(physical("KeyQ", "a", { shiftKey: true })), "q");
});

test("dead key 不換，交給系統組字", () => {
	// AZERTY 的實體 [ 鍵是 dead key ^。
	assert.equal(usCharacter(physical("BracketLeft", "Dead")), undefined);
	assert.equal(usCharacter(physical("Quote", "Dead")), undefined);
});
