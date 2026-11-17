import assert from "node:assert/strict";
import { webcrypto } from "node:crypto";
import { test } from "node:test";
import { newDeviceId } from "../src/device-id.ts";

const UUID_V4 =
	/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

test("非安全環境沒有 randomUUID 時，用 getRandomValues 組出 UUID v4", () => {
	const insecure = {
		getRandomValues: webcrypto.getRandomValues.bind(webcrypto),
	};
	for (let i = 0; i < 100; i++) {
		assert.match(newDeviceId(insecure), UUID_V4);
	}
});

test("有 randomUUID 時照舊用它", () => {
	const secure = {
		getRandomValues: () => {
			throw new Error("不該用到");
		},
		randomUUID: () => "11111111-2222-4333-8444-555555555555",
	};
	assert.equal(newDeviceId(secure), "11111111-2222-4333-8444-555555555555");
});

test("版本與變體位元蓋過隨機值：全 0xff 變成 4fff、bfff", () => {
	const allOnes = {
		getRandomValues: (array: Uint8Array<ArrayBuffer>) => array.fill(0xff),
	};
	assert.equal(newDeviceId(allOnes), "ffffffff-ffff-4fff-bfff-ffffffffffff");
});
