// 常用語（plan〈學習與排位〉〈網頁 prototype〉）：設定區的清單可以新增、編輯、刪除；候選列在沒有組字時有「常用語」鍵，
// 叫出清單點了就上屏（Gboard 工具列的剪貼簿、微信輸入法的常用語面板）。觸發詞的部分在核心。

import { element, message } from "./dom.ts";
import type { Session, Snapshot } from "./wasm/qim_wasm.js";

/** 清單裡的一條常用語：換行畫成 ⏎，整條才看得出是一則。 */
function oneLine(text: string): string {
	return text.replaceAll("\n", "⏎");
}

export interface Phrases {
	/** 組字中藏起候選列的「常用語」鍵並收起清單；沒有組字時再顯示。 */
	setComposing(composing: boolean): void;
	/** 使用者詞換了一整份（匯入、清空）之後重畫清單。 */
	refresh(): void;
}

/**
 * 接上常用語。`commit` 收到從清單選了之後的快照（照觸控的文字鍵上屏）；`save` 存學習資料；
 * `afterChange` 在改了清單之後呼叫（網頁把焦點還給輸入框）。
 */
export function mountPhrases(
	session: Session,
	commit: (snapshot: Snapshot) => void,
	save: () => void,
	afterChange: () => void,
): Phrases {
	const open = element<HTMLButtonElement>("#phrases-open");
	const picker = element<HTMLUListElement>("#phrase-picker");
	const list = element<HTMLUListElement>("#phrase-list");
	const form = element<HTMLFormElement>("#phrase-form");
	const text = element<HTMLTextAreaElement>("#phrase-text");
	const trigger = element<HTMLInputElement>("#phrase-trigger");
	const submit = element<HTMLButtonElement>("#phrase-save");
	const cancel = element<HTMLButtonElement>("#phrase-cancel");
	const status = element<HTMLElement>("#phrase-status");
	/** 編輯中的那條的原文；新增時是 `undefined`。 */
	let editing: string | undefined;
	/** 候選列叫出的清單開著。 */
	let pickerOpen = false;

	/** 核心的清單，依文字排；用完釋放。 */
	const phrases = (): { text: string; trigger: string }[] =>
		session.phrases().map((phrase) => {
			const plain = { text: phrase.text, trigger: phrase.trigger };
			phrase.free();
			return plain;
		});

	const setPickerOpen = (next: boolean): void => {
		pickerOpen = next;
		picker.hidden = !next;
		open.setAttribute("aria-expanded", String(next));
		if (!next) {
			picker.replaceChildren();
			return;
		}
		const items = phrases().map((phrase) => {
			const item = document.createElement("li");
			item.dataset.text = phrase.text;
			item.textContent = oneLine(phrase.text);
			return item;
		});
		if (items.length === 0) {
			const empty = document.createElement("li");
			empty.className = "empty";
			empty.textContent = "還沒有常用語，在下方設定區新增";
			items.push(empty);
		}
		picker.replaceChildren(...items);
	};
	// 按下時不讓輸入框失去焦點，點完還能接著打字。
	open.addEventListener("pointerdown", (event) => event.preventDefault());
	open.addEventListener("click", () => setPickerOpen(!pickerOpen));
	picker.addEventListener("pointerdown", (event) => event.preventDefault());
	picker.addEventListener("click", (event) => {
		const item = (event.target as Element).closest<HTMLElement>("li");
		const chosen = item?.dataset.text;
		if (chosen !== undefined) {
			setPickerOpen(false);
			commit(session.commitText(chosen));
		}
	});

	const stopEditing = (): void => {
		editing = undefined;
		text.value = "";
		trigger.value = "";
		submit.textContent = "新增";
		cancel.hidden = true;
	};
	const renderList = (): void => {
		list.replaceChildren(
			...phrases().map((phrase) => {
				const item = document.createElement("li");
				const shown = document.createElement("span");
				shown.className = "phrase-text";
				shown.textContent = oneLine(phrase.text);
				const code = document.createElement("code");
				code.textContent = phrase.trigger || "（沒有觸發詞）";
				const edit = document.createElement("button");
				edit.type = "button";
				edit.textContent = "編輯";
				edit.addEventListener("click", () => {
					editing = phrase.text;
					text.value = phrase.text;
					trigger.value = phrase.trigger;
					submit.textContent = "儲存修改";
					cancel.hidden = false;
					status.textContent = "";
					text.focus();
				});
				const remove = document.createElement("button");
				remove.type = "button";
				remove.textContent = "刪除";
				remove.addEventListener("click", () => {
					session.removePhrase(phrase.text);
					save();
					if (editing === phrase.text) {
						stopEditing();
					}
					status.textContent = `已刪除「${oneLine(phrase.text)}」`;
					renderList();
					afterChange();
				});
				item.append(shown, code, edit, remove);
				return item;
			}),
		);
	};

	form.addEventListener("submit", (event) => {
		event.preventDefault();
		const next = text.value;
		// 同文字只存一條（核心再存一次是換觸發詞）：擋下，免得不聲不響改了另一條的觸發詞。
		if (next !== editing && phrases().some((phrase) => phrase.text === next)) {
			status.textContent = `已有「${oneLine(next)}」，要改它請按「編輯」`;
			return;
		}
		try {
			// 先存新的再刪舊的：存不進去時原本那條還在。
			session.addPhrase(next, trigger.value);
			if (editing !== undefined && editing !== next) {
				session.removePhrase(editing);
			}
		} catch (error) {
			status.textContent = `存不進去：${message(error)}`;
			return;
		}
		save();
		status.textContent = `${editing === undefined ? "已新增" : "已修改"}「${oneLine(next)}」`;
		stopEditing();
		renderList();
		afterChange();
	});
	cancel.addEventListener("click", () => {
		stopEditing();
		status.textContent = "";
	});

	renderList();
	return {
		setComposing(composing) {
			open.hidden = composing;
			if (composing && pickerOpen) {
				setPickerOpen(false);
			}
		},
		refresh() {
			stopEditing();
			renderList();
		},
	};
}
