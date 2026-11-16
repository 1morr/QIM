// 切換鍵盤的選單（長按地球鍵，M4 票 23，設計稿 docs/design/23-touch-english-keyboard.html〈長按地球鍵〉）：
// 照 Gboard 長按地球鍵的「變更鍵盤」選單，一列一個鍵盤、目前的打勾。畫在鍵盤範圍內蓋住按鍵（iOS 鍵盤
// 不能畫到鍵盤外，事實 M29）。「系統鍵盤」在網頁只占位：iOS、Android 殼叫系統的清單（事實 P29、P30）。

/** 選單掛好後給觸控鍵盤用的操作。 */
export interface KeyboardMenu {
	/** 蓋在按鍵上的整層；觸控鍵盤重畫按鍵時把它留在最後（畫在最上面）。 */
	element: HTMLElement;
	/** 打開；`ascii` 是現在的中英狀態，打勾在對應的那列。 */
	open(ascii: boolean): void;
	close(): void;
}

interface Item {
	name: string;
	note: string;
	/** 選了要切到的中英狀態；`undefined` 是占位、點了沒有作用。 */
	ascii: boolean | undefined;
	/** 上面畫一條分隔線（QIM 自己的鍵盤和平台的分開）。 */
	separated?: boolean;
}

const ITEMS: Item[] = [
	{ name: "速成", note: "中文", ascii: false },
	{ name: "English", note: "英文 QWERTY", ascii: true },
	{
		name: "系統鍵盤…",
		note: "網頁原型沒有；平台殼叫出系統的清單",
		ascii: undefined,
		separated: true,
	},
];

/**
 * 做給觸控鍵盤放在按鍵上層的選單，選了速成或 English 時以要切到的中英狀態叫 `choose`。
 * 點選單外面收起、什麼都不做。手指要在選單裡按下再放開才算選：長按打開選單的那根手指放開時不選。
 */
export function createKeyboardMenu(
	choose: (ascii: boolean) => void,
): KeyboardMenu {
	const scrim = document.createElement("div");
	scrim.className = "keyboard-menu-scrim";
	scrim.dataset.testid = "keyboard-menu-scrim";
	scrim.hidden = true;
	const menu = document.createElement("div");
	menu.className = "keyboard-menu";
	menu.setAttribute("role", "menu");
	menu.setAttribute("aria-label", "切換鍵盤");
	const title = document.createElement("p");
	title.className = "title";
	title.textContent = "切換鍵盤";
	const checks: HTMLElement[] = [];
	const rows = ITEMS.map((item) => {
		const row = document.createElement("div");
		row.className = "item";
		row.setAttribute("role", "menuitemradio");
		if (item.ascii === undefined) {
			row.setAttribute("aria-disabled", "true");
		}
		const check = document.createElement("span");
		check.className = "check";
		check.setAttribute("aria-hidden", "true");
		checks.push(check);
		const name = document.createElement("span");
		name.className = "name";
		name.textContent = item.name;
		const note = document.createElement("span");
		note.className = "detail";
		note.textContent = item.note;
		row.append(check, name, note);
		return row;
	});
	menu.append(title);
	ITEMS.forEach((item, i) => {
		if (item.separated) {
			const rule = document.createElement("div");
			rule.className = "rule";
			rule.setAttribute("role", "separator");
			menu.append(rule);
		}
		menu.append(rows[i]);
	});
	scrim.append(menu);

	/** 這次放開之前，手指是不是在打開的選單上按下的。 */
	let armed = false;

	const close = (): void => {
		scrim.hidden = true;
		armed = false;
	};

	// 觸控鍵盤在 pointerdown 判定按鍵；選單開著時事件停在這裡，不按到底下的鍵，也不讓輸入框失去焦點。
	scrim.addEventListener("pointerdown", (event) => {
		event.preventDefault();
		event.stopPropagation();
		armed = true;
	});
	scrim.addEventListener("pointerup", (event) => {
		event.stopPropagation();
		if (!armed) {
			return;
		}
		armed = false;
		const index = rows.findIndex((row) =>
			row.contains(event.target as Node | null),
		);
		if (index === -1) {
			if (!menu.contains(event.target as Node | null)) {
				close();
			}
			return;
		}
		const { ascii } = ITEMS[index];
		if (ascii === undefined) {
			return;
		}
		close();
		choose(ascii);
	});

	return {
		element: scrim,
		open(ascii) {
			ITEMS.forEach((item, i) => {
				const current = item.ascii === ascii;
				rows[i].classList.toggle("current", current);
				rows[i].setAttribute("aria-checked", String(current));
				checks[i].textContent = current ? "✓" : "";
			});
			armed = false;
			scrim.hidden = false;
		},
		close,
	};
}
