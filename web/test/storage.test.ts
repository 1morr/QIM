import assert from "node:assert/strict";
import { test } from "node:test";
import { safeStorage } from "../src/storage.ts";

/** 記憶體裡的 `Storage`；`fail` 裡列的方法丟錯。 */
function fakeStorage(
	fail: ReadonlySet<string> = new Set(),
	entries: [string, string][] = [],
): Storage {
	const data = new Map(entries);
	const guard = (name: string) => {
		if (fail.has(name)) {
			throw new Error(`${name} 丟錯`);
		}
	};
	return {
		getItem: (key: string) => {
			guard("getItem");
			return data.get(key) ?? null;
		},
		setItem: (key: string, value: string) => {
			guard("setItem");
			data.set(key, value);
		},
		removeItem: (key: string) => {
			guard("removeItem");
			data.delete(key);
		},
		clear: () => data.clear(),
		key: () => null,
		get length() {
			return data.size;
		},
	};
}

test("沒出錯時讀寫都經 localStorage，不報錯", () => {
	const backing = fakeStorage();
	const storage = safeStorage(() => backing);
	const errors: unknown[] = [];
	storage.watch((error) => errors.push(error));
	storage.set("a", "1");
	assert.equal(backing.getItem("a"), "1");
	assert.equal(storage.get("a"), "1");
	storage.remove("a");
	assert.equal(backing.getItem("a"), null);
	assert.deepEqual(errors, []);
});

test("讀 localStorage 就丟錯：讀到沒有、寫的留在記憶體讀得回來，只報一次", () => {
	const storage = safeStorage(() => {
		throw new Error("封鎖了");
	});
	const errors: string[] = [];
	storage.watch((error) => errors.push(String(error)));
	assert.equal(storage.get("a"), null);
	storage.set("a", "1");
	assert.equal(storage.get("a"), "1");
	storage.remove("a");
	assert.equal(storage.get("a"), null);
	assert.deepEqual(errors, ["Error: 封鎖了"]);
});

test("寫入丟錯（存不下）：之後的寫入都只在記憶體，沒寫過的照樣讀 localStorage", () => {
	const backing = fakeStorage(new Set(["setItem"]), [
		["old", "舊的"],
		["gone", "要刪的"],
	]);
	const storage = safeStorage(() => backing);
	storage.set("a", "1");
	assert.equal(storage.get("a"), "1");
	assert.equal(storage.get("old"), "舊的");
	// 出過錯之後刪除也不碰 localStorage，記憶體記著刪了。
	storage.remove("gone");
	assert.equal(storage.get("gone"), null);
	assert.equal(backing.getItem("gone"), "要刪的");
});

test("出錯之後才看的照樣收到", () => {
	const storage = safeStorage(() => fakeStorage(new Set(["getItem"])));
	storage.get("a");
	const errors: unknown[] = [];
	storage.watch((error) => errors.push(error));
	assert.equal(errors.length, 1);
});
