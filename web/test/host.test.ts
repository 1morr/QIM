import assert from "node:assert/strict";
import { test } from "node:test";
import { deleteStart } from "../src/host.ts";

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
