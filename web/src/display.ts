// 畫面開關：觸控鍵盤（開關、數字列、隨裝置與方向選佈局版本）與深色。

import { element } from "./dom.ts";
import { type Keyboard, mountKeyboard } from "./touch.ts";
import { Layout, type Session, type Snapshot } from "./wasm/qim_wasm.js";

/** 選佈局版本用的裝置：短邊 600 px 以上當平板（Android 的 sw600dp 慣例）。 */
function device(): "phone" | "tablet" {
	return Math.min(window.screen.width, window.screen.height) >= 600
		? "tablet"
		: "phone";
}

function orientation(): "portrait" | "landscape" {
	return window.matchMedia("(orientation: landscape)").matches
		? "landscape"
		: "portrait";
}

/** 深色開關：按過就照它，否則跟系統（`prefers-color-scheme`）。 */
export function mountTheme(): void {
	const darkToggle = element<HTMLButtonElement>("#dark-toggle");
	const systemDark = window.matchMedia("(prefers-color-scheme: dark)");
	let chosenDark: boolean | undefined;
	/** 套用深色或淺色：按鍵顏色由 CSS `light-dark()` 依 `color-scheme` 選（touch.ts）。 */
	const applyTheme = (): void => {
		const dark = chosenDark ?? systemDark.matches;
		if (chosenDark === undefined) {
			delete document.documentElement.dataset.theme;
		} else {
			document.documentElement.dataset.theme = dark ? "dark" : "light";
		}
		darkToggle.setAttribute("aria-pressed", String(dark));
	};
	applyTheme();
	systemDark.addEventListener("change", applyTheme);
	darkToggle.addEventListener("click", () => {
		chosenDark = darkToggle.getAttribute("aria-pressed") !== "true";
		applyTheme();
	});
}

/** 兩份預設佈局的 JSON（plan §2.6）。 */
export interface Layouts {
	plain: string;
	/** 字母頁上方多一排數字；「數字列」打開時用（brief §4.4）。 */
	numberRow: string;
}

/**
 * 掛上觸控鍵盤與它的開關。`apply` 收到核心吃下的動作，`hostDefault` 收到沒吃下的；
 * `text` 是輸入框，觸控鍵盤開著時不叫出系統鍵盤，改了開關後焦點還給它。
 */
export function mountTouch(
	session: Session,
	layouts: Layouts,
	text: HTMLTextAreaElement,
	apply: (snapshot: Snapshot) => void,
	hostDefault: (snapshot: Snapshot) => void,
): Keyboard {
	const keyboard = element<HTMLElement>("#keyboard");
	const touchToggle = element<HTMLButtonElement>("#touch-toggle");
	const numberRow = element<HTMLInputElement>("#number-row");

	/** 依「數字列」開關讀佈局，選目前裝置與方向的版本。 */
	const readLayout = (): Layout => {
		const read = new Layout(
			numberRow.checked ? layouts.numberRow : layouts.plain,
			device(),
			orientation(),
		);
		for (const warning of read.warnings) {
			console.warn(`佈局：${warning}`);
		}
		return read;
	};
	let layout = readLayout();
	const mounted = mountKeyboard(keyboard, layout, session, apply, hostDefault);
	// 轉向時重選佈局版本（plan §2.6），留在同一頁。
	window
		.matchMedia("(orientation: landscape)")
		.addEventListener("change", () => {
			const warning = layout.select(device(), orientation());
			if (warning !== undefined) {
				console.warn(`佈局：${warning}`);
			}
			mounted.relayout();
		});
	numberRow.addEventListener("change", () => {
		const old = layout;
		layout = readLayout();
		layout.keepStateOf(old);
		mounted.setLayout(layout);
		old.free();
		text.focus();
	});
	// 頁面底部留出鍵盤實際的高度，鍵盤才不會蓋住輸入框；高度隨佈局版本的畫布比例與寬度變。
	new ResizeObserver(([entry]) => {
		document.body.style.setProperty(
			"--keyboard-height",
			`${entry.borderBoxSize[0].blockSize}px`,
		);
	}).observe(keyboard);

	/** 觸控鍵盤開或關；開著時輸入框不叫出系統鍵盤。 */
	const setTouch = (on: boolean): void => {
		keyboard.hidden = !on;
		touchToggle.setAttribute("aria-pressed", String(on));
		text.inputMode = on ? "none" : "";
	};
	// 行動裝置（主要指標是手指）預設開觸控鍵盤。
	setTouch(window.matchMedia("(pointer: coarse)").matches);
	touchToggle.addEventListener("click", () => {
		setTouch(touchToggle.getAttribute("aria-pressed") !== "true");
	});
	return mounted;
}
