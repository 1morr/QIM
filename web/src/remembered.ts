// 記住的字串（plan §2.4，M3 票 15）：設定區列出核心記住的 email，可以逐條刪（同 Gboard、iOS 學到的字詞
// 能刪）。記住與給候選都在核心；這裡只畫清單。

import { element } from "./dom.ts";
import type { Session } from "./wasm/qim_wasm.js";

export interface Remembered {
	/** 學習資料可能變了（上屏、打完一串英文、匯入、清空）之後重畫清單。 */
	refresh(): void;
}

/** 接上清單。`save` 存學習資料；`afterChange` 在刪了之後呼叫（網頁把焦點還給輸入框）。 */
export function mountRemembered(
	session: Session,
	save: () => void,
	afterChange: () => void,
): Remembered {
	const list = element<HTMLUListElement>("#remembered-list");
	const status = element<HTMLElement>("#remembered-status");
	/** 畫上去的清單；沒變時不重畫，打字中每次存檔都會呼叫 `refresh`。 */
	let drawn: string[] | undefined;

	const refresh = (): void => {
		const strings = session.rememberedStrings();
		if (drawn && strings.join("\n") === drawn.join("\n")) {
			return;
		}
		drawn = strings;
		if (strings.length === 0) {
			const empty = document.createElement("li");
			empty.className = "empty";
			empty.textContent = "還沒有記住的 email";
			list.replaceChildren(empty);
			return;
		}
		list.replaceChildren(
			...strings.map((text) => {
				const item = document.createElement("li");
				const shown = document.createElement("span");
				shown.className = "remembered-text";
				shown.textContent = text;
				const remove = document.createElement("button");
				remove.type = "button";
				remove.textContent = "刪除";
				remove.setAttribute("aria-label", `刪除 ${text}`);
				remove.addEventListener("click", () => {
					session.forgetRememberedString(text);
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
