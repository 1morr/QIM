// 使用者詞與學習資料（plan §2.4、§3.1）：手動加詞、存 `localStorage`、匯出匯入 `# qim-userdict v3`（也讀得進 v1、v2）。

import { element, message } from "./dom.ts";
import type { Session } from "./wasm/qim_wasm.js";

/** 學習資料在 `localStorage` 的鍵；值是 `Session.exportUserDict()` 的 TSV。 */
const USER_DICT_KEY = "qim.userdict";
/** 載入時匯入失敗的資料搬到這裡，免得下次上屏被空白資料蓋掉。 */
const BROKEN_KEY = "qim.userdict.broken";
/** 這台裝置的 id 在 `localStorage` 的鍵：第一次載入時產生，匯出檔頭帶著它（plan §2.4）。 */
const DEVICE_KEY = "qim.device";
/** 匯出檔名；副檔名照內容是 TSV。 */
const EXPORT_NAME = "qim-userdict.tsv";

/** 學習資料檔的資料列數：`#` 開頭的行與空行以外的行。 */
function rows(tsv: string): number {
	return tsv
		.split("\n")
		.filter((line) => line.trim() !== "" && !line.startsWith("#")).length;
}

/**
 * 這台裝置的 id：存過就用存的，沒有就產生一個 UUID 存起來（照 librime 每次安裝一個 `installation_id`）。
 * 清掉網站資料等於換一台裝置。
 */
function ensureDeviceId(): string {
	const stored = localStorage.getItem(DEVICE_KEY);
	if (stored !== null) {
		return stored;
	}
	const id = crypto.randomUUID();
	localStorage.setItem(DEVICE_KEY, id);
	return id;
}

/** 匯入略過的列（不認得的種類）寫成一句；沒有時是空字串。 */
function skipped(warnings: string[]): string {
	return warnings.length === 0
		? ""
		: `，略過 ${warnings.length} 列：${warnings.join("；")}`;
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

	// 存的 id 壞了（手動改過）就換一個新的。
	try {
		session.setDevice(ensureDeviceId());
	} catch {
		localStorage.removeItem(DEVICE_KEY);
		session.setDevice(ensureDeviceId());
	}
	const stored = localStorage.getItem(USER_DICT_KEY);
	if (stored !== null) {
		try {
			// 新版存的檔有這版不認得的種類時略過那些列，說一聲（下次存檔就沒有了）。
			const warnings = session.importUserDict(stored);
			if (warnings.length > 0) {
				status.textContent = `學習資料讀進來了${skipped(warnings)}`;
			}
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

	// 匯入取代整份（plan §2.4）：先檢查格式，再在頁面上說明會覆蓋多少筆、請使用者確認，
	// 不用瀏覽器原生的對話框。
	const file = element<HTMLInputElement>("#import-user-dict");
	const confirm = element<HTMLElement>("#confirm-import");
	const confirmText = element<HTMLElement>("#confirm-import-text");
	let pending: { name: string; text: string } | undefined;
	const closeConfirm = () => {
		pending = undefined;
		confirm.hidden = true;
	};
	file.addEventListener("change", async () => {
		const chosen = file.files?.[0];
		file.value = "";
		closeConfirm();
		if (!chosen) {
			return;
		}
		const text = await chosen.text();
		let warnings: string[];
		try {
			warnings = session.checkUserDict(text);
		} catch (error) {
			status.textContent = `匯入失敗，學習資料沒動：${message(error)}`;
			return;
		}
		pending = { name: chosen.name, text };
		const current = rows(session.exportUserDict());
		confirmText.textContent = `匯入 ${chosen.name}（${rows(text)} 筆${skipped(warnings)}）會覆蓋目前的 ${current} 筆學習資料，原本的不會保留。確定匯入？`;
		status.textContent = "";
		confirm.hidden = false;
	});
	element<HTMLButtonElement>("#confirm-import-ok").addEventListener(
		"click",
		() => {
			if (!pending) {
				return;
			}
			const { name, text } = pending;
			closeConfirm();
			try {
				const warnings = session.importUserDict(text);
				save();
				status.textContent = `已匯入 ${name}${skipped(warnings)}`;
			} catch (error) {
				status.textContent = `匯入失敗，學習資料沒動：${message(error)}`;
			}
			afterChange();
		},
	);
	element<HTMLButtonElement>("#confirm-import-cancel").addEventListener(
		"click",
		() => {
			closeConfirm();
			status.textContent = "沒有匯入，學習資料沒動";
			afterChange();
		},
	);

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
