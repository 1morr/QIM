import assert from "node:assert/strict";
import { test } from "node:test";
import { fakeOrder, rerankMode } from "../src/rerank.ts";

test("原樣：名次照原順序", () => {
	assert.deepEqual(fakeOrder("identity", 4), [0, 1, 2, 3]);
});

// 核心只讓它到第 2 位（ADR 0005）；這裡要的是「提到最前」，上限由核心擋。
test("第 5 個提前：第 5 個排最前，其餘照原順序", () => {
	assert.deepEqual(fakeOrder("raise-fifth", 6), [4, 0, 1, 2, 3, 5]);
});

test("候選不到 5 個時照原順序", () => {
	assert.deepEqual(fakeOrder("raise-fifth", 3), [0, 1, 2]);
});

test("選單的值轉成模式，不認得的當關", () => {
	assert.equal(rerankMode("identity"), "identity");
	assert.equal(rerankMode("raise-fifth"), "raise-fifth");
	assert.equal(rerankMode("off"), "off");
	assert.equal(rerankMode("cloud"), "off");
});
