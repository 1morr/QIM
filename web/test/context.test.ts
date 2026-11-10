import assert from "node:assert/strict";
import { test } from "node:test";
import { textBefore } from "../src/context.ts";

test("取游標前最多 limit 個字", () => {
	assert.equal(textBefore("今天天氣好", 4, 2), "天氣");
	assert.equal(textBefore("今天天氣好", 2, 5), "今天");
	assert.equal(textBefore("今天", 0, 5), "");
});

test("以字計、不切開代理對", () => {
	const astral = "𠻺𡃉𨳊";
	// 每個字兩個 UTF-16 碼元；游標在最後。
	assert.equal(textBefore(astral, astral.length, 2), "𡃉𨳊");
	assert.equal(textBefore(`a${astral}`, astral.length + 1, 3), astral);
	assert.equal(textBefore(astral.repeat(3), astral.length * 3, 3), astral);
});
