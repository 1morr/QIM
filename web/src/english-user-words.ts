// 英文使用者詞（plan §2.4，M4 票 14）：設定區列出核心學到的英文串（沒到出現門檻的也列），可以逐條刪
// （同 Gboard、iOS 學到的字詞能刪）。學習與給候選都在核心；這裡只畫清單。

import { element } from "./dom.ts";
import type { Session } from "./wasm/qim_wasm.js";

export interface EnglishUserWords {
	/** 學習資料可能變了（上屏、打完一串英文、匯入、清空）之後重畫清單。 */
	refresh(): void;
}

/** 接上清單。`save` 存學習資料；`afterChange` 在刪了之後呼叫（網頁把焦點還給輸入框）。 */
export function mountEnglishUserWords(
	session: Session,
	save: () => void,
	afterChange: () => void,
): EnglishUserWords {
	const list = element<HTMLUListElement>("#english-user-words-list");
	const status = element<HTMLElement>("#english-user-words-status");
	/** 畫上去的清單；沒變時不重畫，打字中每次存檔都會呼叫 `refresh`。 */
	let drawn: string[] | undefined;

	const refresh = (): void => {
		const strings = session.englishUserWords();
		if (drawn && strings.join("\n") === drawn.join("\n")) {
			return;
		}
		drawn = strings;
		if (strings.length === 0) {
			const empty = document.createElement("li");
			empty.className = "empty";
			empty.textContent = "還沒有英文使用者詞";
			list.replaceChildren(empty);
			return;
		}
		list.replaceChildren(
			...strings.map((text) => {
				const item = document.createElement("li");
				const shown = document.createElement("span");
				shown.className = "english-user-word-text";
				shown.textContent = text;
				const remove = document.createElement("button");
				remove.type = "button";
				remove.textContent = "刪除";
				remove.setAttribute("aria-label", `刪除 ${text}`);
				remove.addEventListener("click", () => {
					session.forgetEnglishUserWord(text);
					save();
					status.textContent = `已刪除「${text}」`;
					refresh();
					afterChange();
				});
				item.append(shown, remove);
				return item;
			}),
		);
	};
	refresh();
	return { refresh };
}
