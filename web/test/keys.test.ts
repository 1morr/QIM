import assert from "node:assert/strict";
import { test } from "node:test";
import { type KeyEvent, toKey } from "../src/keys.ts";

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

test("帶 Ctrl、Meta、Alt 或在別的輸入法組字中的事件留給瀏覽器", () => {
	for (const modifier of [
		"ctrlKey",
		"metaKey",
		"altKey",
		"isComposing",
	] as const) {
		assert.equal(toKey(press("a", { [modifier]: true })), null, modifier);
		assert.equal(toKey(press(",", { [modifier]: true })), null, modifier);
	}
});
