import assert from "node:assert/strict";
import { test } from "node:test";
import { fakeOrder, rerankMode } from "../src/rerank.ts";

test("原樣：名次照原順序", () => {
	assert.deepEqual(fakeOrder("identity", 4), [0, 1, 2, 3]);
});

// 核心只讓它到第 2 位（ADR 0005）；這裡要的是「排最前」，上限由核心擋。
test("最後一條提前：最後一條排最前，其餘照原順序", () => {
	assert.deepEqual(fakeOrder("raise-last", 4), [3, 0, 1, 2]);
});

test("只有一條路徑時照原順序", () => {
	assert.deepEqual(fakeOrder("raise-last", 1), [0]);
});

test("選單的值轉成模式，不認得的當關", () => {
	assert.equal(rerankMode("identity"), "identity");
	assert.equal(rerankMode("raise-last"), "raise-last");
	assert.equal(rerankMode("raise-fifth"), "off");
	assert.equal(rerankMode("off"), "off");
	assert.equal(rerankMode("cloud"), "off");
});
