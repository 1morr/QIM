// 釋放 WebAssembly 綁定的物件：wasm-bindgen 的物件不會被 GC 回收，用完要呼叫 `free()`。網頁只用這一種寫法，
// 不用 `using` 宣告：綁定只在瀏覽器有 `Symbol.dispose` 時才掛上它，較舊的手機瀏覽器會在 `using` 丟錯。

/** 綁定的物件：`Snapshot`、`Pack`、`Segment` 這類。 */
export interface Freeable {
	free(): void;
}

/**
 * 把 `item` 交給 `use`，之後釋放它（陣列則每一個）；`use` 丟錯也釋放。`use` 回傳時就釋放了，所以不能是
 * async：要跨 `await` 用的，先在 `use` 裡讀成一般的值。換掉手上留著的物件時，`use` 換上新的、舊的跟著釋放。
 */
export function freeAfter<T extends Freeable | readonly Freeable[], R>(
	item: T,
	use: (item: T) => R,
): R {
	try {
		return use(item);
	} finally {
		const items: readonly Freeable[] = Array.isArray(item) ? item : [item];
		for (const each of items) {
			each.free();
		}
	}
}
