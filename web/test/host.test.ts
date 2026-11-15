import assert from "node:assert/strict";
import { test } from "node:test";
import { deleteStart, type Editable, insert } from "../src/host.ts";

test("游標前刪 N 字：以字計（Unicode scalar），不切開代理對", () => {
	assert.equal(deleteStart("teh ", 4, 4), 0);
	assert.equal(deleteStart("I teh ", 6, 4), 2);
	const astral = "𠻺𡃉";
	// 每個字兩個 UTF-16 碼元。
	assert.equal(deleteStart(`a${astral}`, 5, 1), 3);
	assert.equal(deleteStart(`a${astral}`, 5, 2), 1);
});

test("刪 0 字就是游標處", () => {
	assert.equal(deleteStart("abc", 2, 0), 2);
});

test("有選取時從選取的開頭往前算（同 Android deleteSurroundingText）", () => {
	// 游標（選取開頭）在 3，選取的 "de" 不算在要刪的 N 字裡。
	assert.equal(deleteStart("abcde", 3, 2), 1);
});

test("字數超過游標前的字時刪到開頭", () => {
	assert.equal(deleteStart("ab", 2, 5), 0);
});

/** 照 HTML 的 `setRangeText`（"end" 與 "preserve"）做的假輸入框，記下收到的 input 事件數。 */
function fakeText(value: string, start: number, end = start) {
	const text = {
		value,
		selectionStart: start,
		selectionEnd: end,
		inputs: 0,
		setRangeText(
			replacement: string,
			from: number,
			to: number,
			mode: "end" | "preserve",
		) {
			text.value =
				text.value.slice(0, from) + replacement + text.value.slice(to);
			const newEnd = from + replacement.length;
			const delta = replacement.length - (to - from);
			if (mode === "end") {
				text.selectionStart = newEnd;
				text.selectionEnd = newEnd;
				return;
			}
			// 選取在刪掉的範圍之後的往前挪，在範圍裡的收到範圍開頭。
			const keep = (i: number) => (i > to ? i + delta : Math.min(i, from));
			text.selectionStart = keep(text.selectionStart);
			text.selectionEnd = keep(text.selectionEnd);
		},
		dispatchEvent() {
			text.inputs += 1;
			return true;
		},
	};
	return text;
}

test("insert：先刪游標前 N 字再上屏，游標在上屏的字後面", () => {
	const text = fakeText("I teh ", 6);
	insert(text as unknown as Editable, "the ", 4);
	assert.equal(text.value, "I the ");
	assert.equal(text.selectionStart, 6);
	assert.equal(text.inputs, 1);
});

test("insert：有選取時刪選取前的 N 字，上屏取代選取", () => {
	const text = fakeText("ab「」xyz", 4, 7);
	insert(text as unknown as Editable, "」", 1);
	assert.equal(text.value, "ab「」");
	assert.equal(text.selectionStart, 4);
});

test("insert：只刪不上屏時選取留著、跟著往前挪", () => {
	const text = fakeText("abcdef", 3, 5);
	insert(text as unknown as Editable, "", 2);
	assert.equal(text.value, "adef");
	assert.equal(text.value.slice(text.selectionStart, text.selectionEnd), "de");
});

test("insert：不刪時照舊插入游標處", () => {
	const text = fakeText("ab", 1);
	insert(text as unknown as Editable, "X");
	assert.equal(text.value, "aXb");
	assert.equal(text.selectionStart, 2);
});
