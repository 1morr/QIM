import assert from "node:assert/strict";
import { test } from "node:test";
import { type KeyEvent, LoneShift, shiftSwitch, toKey } from "../src/keys.ts";

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
