// 使用者詞與學習資料（plan §2.4、§3.1）：手動加詞、存 `localStorage`、匯出匯入 `# qim-userdict v1`。

import type { Session } from "./wasm/qim_wasm.js";

/** 學習資料在 `localStorage` 的鍵；值是 `Session.exportUserDict()` 的 TSV。 */
export const USER_DICT_KEY = "qim.userdict";
/** 載入時匯入失敗的資料搬到這裡，免得下次上屏被空白資料蓋掉。 */
const BROKEN_KEY = "qim.userdict.broken";
/** 匯出檔名；副檔名照內容是 TSV。 */
const EXPORT_NAME = "qim-userdict.tsv";

function element<T extends HTMLElement>(selector: string): T {
	const found = document.querySelector<T>(selector);
	if (!found) {
		throw new Error(`index.html 少了 ${selector}`);
	}
	return found;
}

/** `JsError` 與其他例外的訊息。 */
function message(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}

export interface UserDict {
	/** 把目前的學習資料存進 `localStorage`；上屏後呼叫。 */
	save(): void;
}

/**
 * 接上使用者詞的介面，並把 `localStorage` 裡的學習資料匯入 `session`。
 * `afterChange` 在加詞、匯入、清空之後呼叫（網頁把焦點還給輸入框）。
 */
export function mountUserDict(
	session: Session,
	afterChange: () => void,
): UserDict {
	const word = element<HTMLInputElement>("#word");
	const trigger = element<HTMLInputElement>("#trigger");
	const status = element<HTMLElement>("#user-dict-status");
	const save = () => {
		localStorage.setItem(USER_DICT_KEY, session.exportUserDict());
	};

	const stored = localStorage.getItem(USER_DICT_KEY);
	if (stored !== null) {
		try {
			session.importUserDict(stored);
		} catch (error) {
			localStorage.setItem(BROKEN_KEY, stored);
			localStorage.removeItem(USER_DICT_KEY);
			status.textContent = `學習資料讀不進來，已另存到 ${BROKEN_KEY}：${message(error)}`;
		}
	}

	element<HTMLFormElement>("#add-word").addEventListener("submit", (event) => {
		event.preventDefault();
		try {
			session.addWord(word.value, trigger.value);
			save();
			status.textContent = `已加入「${word.value}」`;
			word.value = "";
			trigger.value = "";
			afterChange();
		} catch (error) {
			status.textContent = `加不進去：${message(error)}`;
		}
	});

	element<HTMLButtonElement>("#export-user-dict").addEventListener(
		"click",
		() => {
			const blob = new Blob([session.exportUserDict()], {
				type: "text/tab-separated-values",
			});
			const link = document.createElement("a");
			link.href = URL.createObjectURL(blob);
			link.download = EXPORT_NAME;
			link.click();
			URL.revokeObjectURL(link.href);
			afterChange();
		},
	);

	const file = element<HTMLInputElement>("#import-user-dict");
	file.addEventListener("change", async () => {
		const chosen = file.files?.[0];
		file.value = "";
		if (!chosen) {
			return;
		}
		try {
			session.importUserDict(await chosen.text());
			save();
			status.textContent = `已匯入 ${chosen.name}`;
		} catch (error) {
			status.textContent = `匯入失敗，學習資料沒動：${message(error)}`;
		}
		afterChange();
	});

	element<HTMLButtonElement>("#clear-user-dict").addEventListener(
		"click",
		() => {
			session.clearUserDict();
			save();
			status.textContent = "已清空學習資料";
			afterChange();
		},
	);

	return { save };
}
