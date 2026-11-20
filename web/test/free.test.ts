import assert from "node:assert/strict";
import { test } from "node:test";
import { freeAfter } from "../src/free.ts";

/** 像綁定的物件：記下被釋放了幾次。 */
function wasmObject() {
	const object = { freed: 0, free: () => object.freed++ };
	return object;
}

test("交給 use 之後釋放一次，回傳 use 的結果", () => {
	const object = wasmObject();
	const seen = freeAfter(object, (o) => o.freed);
	assert.equal(seen, 0, "use 讀的時候還沒釋放");
	assert.equal(object.freed, 1);
});

test("use 丟錯也釋放，錯照樣丟出來", () => {
	const object = wasmObject();
	assert.throws(
		() =>
			freeAfter(object, () => {
				throw new Error("壞了");
			}),
		/壞了/,
	);
	assert.equal(object.freed, 1);
});

test("給陣列時每一個都釋放", () => {
	const objects = [wasmObject(), wasmObject(), wasmObject()];
	const count = freeAfter(objects, (list) => list.length);
	assert.equal(count, 3);
	assert.deepEqual(
		objects.map((o) => o.freed),
		[1, 1, 1],
	);
});
