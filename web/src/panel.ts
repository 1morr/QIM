// 候選展開面板（plan §3.1）：候選列右側的展開鍵，一次列全部候選（brief §4.4）。

import { element } from "./dom.ts";

export interface Panel {
	/** 打開或收起；不是中文組字中時不開。 */
	setOpen(open: boolean): void;
	/**
	 * 組字或候選變了之後重畫。`chinese` 是中文組字中（臨時英文不算）：不是時藏起展開鍵、收起面板。
	 */
	show(chinese: boolean): void;
}

/**
 * 接上展開鍵與面板。`allCandidates` 問核心全部候選；`select` 收到點的候選在全部候選的位置。
 */
export function mountPanel(
	allCandidates: () => string[],
	select: (index: number) => void,
): Panel {
	const expand = element<HTMLButtonElement>("#expand");
	const panel = element<HTMLOListElement>("#panel");
	let chinese = false;
	/** 面板開著；組字結束就關掉。 */
	let open = false;

	/** 每項記著它在全部候選的位置，點了照它選字。 */
	const render = (): void => {
		panel.hidden = !open;
		expand.setAttribute("aria-expanded", String(open));
		expand.textContent = open ? "⌃" : "⌄";
		expand.setAttribute("aria-label", open ? "收起候選" : "展開候選");
		if (!open) {
			panel.replaceChildren();
			return;
		}
		panel.replaceChildren(
			...allCandidates().map((candidate, i) => {
				const item = document.createElement("li");
				item.dataset.index = String(i);
				item.textContent = candidate;
				return item;
			}),
		);
	};
	const setOpen = (next: boolean): void => {
		open = next && chinese;
		render();
	};

	// 按下時不讓輸入框失去焦點，點完還能接著打字。
	expand.addEventListener("pointerdown", (event) => event.preventDefault());
	expand.addEventListener("click", () => setOpen(!open));
	panel.addEventListener("pointerdown", (event) => event.preventDefault());
	panel.addEventListener("click", (event) => {
		const item = (event.target as Element).closest<HTMLElement>("li");
		if (item) {
			// 選了字就收起，同 Gboard 的展開面板。
			open = false;
			select(Number(item.dataset.index));
		}
	});

	return {
		setOpen,
		show(next) {
			chinese = next;
			if (!chinese) {
				open = false;
			}
			expand.hidden = !chinese;
			render();
		},
	};
}
