// 網頁的 `localStorage`（plan §3.1）：學習資料、設定、每種資料包用哪份都經這裡存。瀏覽器封鎖網站資料
// （讀 `window.localStorage` 就丟 `SecurityError`）或存不下（`QuotaExceededError`）時不丟錯：第一次出錯
// 之後寫入只留在記憶體、讀先看記憶體，這次不會存，網頁照樣能打字；`storage.watch` 讓網頁在狀態列說明。

/** 讀、寫、刪照 `Storage` 的三個方法（值是 `null` 表示沒有），加上出錯時的通知。 */
export interface SafeStorage {
	get(key: string): string | null;
	set(key: string, value: string): void;
	remove(key: string): void;
	/** 出錯時呼叫 `listener` 一次；已經出過錯就立刻呼叫。 */
	watch(listener: (error: unknown) => void): void;
}

/** 包住 `backing()`（每次都重取：讀 `window.localStorage` 本身就可能丟錯）。 */
export function safeStorage(backing: () => Storage): SafeStorage {
	/** 出錯之後的寫入；`null` 是刪掉。 */
	const memory = new Map<string, string | null>();
	let failure: { error: unknown } | undefined;
	const listeners: ((error: unknown) => void)[] = [];
	const fail = (error: unknown) => {
		if (failure) {
			return;
		}
		failure = { error };
		for (const listener of listeners) {
			listener(error);
		}
	};
	/** 還沒出錯時寫進 `backing()`；寫成了回 `true`。 */
	const write = (action: (storage: Storage) => void): boolean => {
		if (failure) {
			return false;
		}
		try {
			action(backing());
			return true;
		} catch (error) {
			fail(error);
			return false;
		}
	};
	return {
		get(key) {
			const kept = memory.get(key);
			if (kept !== undefined) {
				return kept;
			}
			try {
				return backing().getItem(key);
			} catch (error) {
				fail(error);
				return null;
			}
		},
		set(key, value) {
			if (!write((storage) => storage.setItem(key, value))) {
				memory.set(key, value);
			}
		},
		remove(key) {
			if (!write((storage) => storage.removeItem(key))) {
				memory.set(key, null);
			}
		},
		watch(listener) {
			listeners.push(listener);
			if (failure) {
				listener(failure.error);
			}
		},
	};
}

/** 網頁用的那一個。 */
export const storage = safeStorage(() => localStorage);
