import assert from "node:assert/strict";
import { test } from "node:test";
import { type KeyEvent, LoneShift, toKey } from "../src/keys.ts";

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
	assert.deepEqual(toKey(press("a")), { kind: "letter", char: "a" });
	assert.deepEqual(toKey(press("z")), { kind: "letter", char: "z" });
	assert.deepEqual(toKey(press("A")), { kind: "letter", char: "A" });
});

test("數字鍵是 digit", () => {
	assert.deepEqual(toKey(press("0")), { kind: "digit", digit: 0 });
	assert.deepEqual(toKey(press("9")), { kind: "digit", digit: 9 });
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
		assert.deepEqual(toKey(press(char)), { kind: "punct", char }, char);
	}
});

test("沒有對應的鍵是 other", () => {
	for (const key of [
		"Tab",
		"ArrowUp",
		"ArrowDown",
		"Home",
		"Delete",
		"Shift",
		"F5",
		"é",
		"，",
		"ab",
	]) {
		assert.deepEqual(toKey(press(key)), { kind: "other" }, key);
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
	assert.deepEqual(toKey(press("a"), true), { kind: "letter", char: "a" });
});

test("在別的輸入法組字中的事件一律留給瀏覽器", () => {
	assert.equal(toKey(press("a", { isComposing: true })), null);
	assert.equal(toKey(press("a", { isComposing: true }), true), null);
});

// librime ascii_composer：Shift 按下到放開 500 ms 內、中間沒有別的鍵，才算單按。
test("單按 Shift：按下後 500 ms 內放開", () => {
	const shift = new LoneShift();
	shift.down("Shift", false, 0);
	assert.equal(shift.up("Shift", 499), true);
});

test("Shift 按太久不算單按", () => {
	const shift = new LoneShift();
	shift.down("Shift", false, 0);
	assert.equal(shift.up("Shift", 500), false);
});

test("Shift 按著時按了別的鍵（打大寫）不算單按", () => {
	const shift = new LoneShift();
	shift.down("Shift", false, 0);
	shift.down("A", false, 10);
	assert.equal(shift.up("Shift", 20), false);
});

test("按住 Shift 的自動重複不重新計時", () => {
	const shift = new LoneShift();
	shift.down("Shift", false, 0);
	shift.down("Shift", true, 400);
	assert.equal(shift.up("Shift", 600), false);
});

test("放開的不是 Shift、或沒有先按下 Shift 都不算", () => {
	const shift = new LoneShift();
	assert.equal(shift.up("Shift", 0), false);
	shift.down("Shift", false, 0);
	assert.equal(shift.up("a", 10), false);
	// 一次單按只算一次。
	assert.equal(shift.up("Shift", 20), true);
	assert.equal(shift.up("Shift", 30), false);
});
