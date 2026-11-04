import type { Layout, LayoutKey, Session, Snapshot } from "./wasm/qim_wasm.js";

/** 觸控鍵盤掛好後給外面用的操作。 */
export interface Keyboard {
	/** 核心狀態改變時重畫標籤（組字中的標籤可能不同）。 */
	refresh(): void;
	/** 佈局版本換了（轉向）時整個重畫。 */
	relayout(): void;
	/** 換一份佈局（例如打開數字列），整個重畫；舊的由呼叫端釋放。 */
	setLayout(layout: Layout): void;
}

/** 按住多久算長按。 */
const KEY_LONG_PRESS_MS = 400;
/** 同一個鍵兩次點擊隔多久以內算快速點兩下（Shift 鎖定大寫用）；Android 的雙擊門檻也是 300 ms。 */
const QUICK_REPEAT_MS = 300;
/** 長按清單一排最多幾個選項，多的換到下一排（Gboard 句號鍵的清單是兩排）。 */
const LONG_PRESS_COLUMNS = 8;
/** Shift 鍵開著時畫的圖示：開一次、鎖定（同 Gboard 的實心、實心加底線）；關著時用佈局的標籤（空心 ⇧）。 */
const SHIFT_GLYPHS: Record<string, string> = {
	once: "⬆",
	locked: "⇪",
};

/** 一根手指按下到放開之間的狀態。 */
interface Press {
	pointerId: number;
	/** 按到的鍵在 `keys` 的位置。 */
	index: number;
	timer: number | undefined;
	/** 長按清單出現後，目前選的選項；`undefined` 是還沒出現。 */
	choice: number | undefined;
}

/** 有寫淺色與深色時交給 CSS `light-dark()`，由 `color-scheme` 選（網頁的深色開關改 `color-scheme`）。 */
function themed(light: string | undefined, dark: string | undefined) {
	if (light === undefined || dark === undefined) {
		return light ?? dark;
	}
	return `light-dark(${light}, ${dark})`;
}

/**
 * 觸控鍵盤：照 `qim-layout` 給的按鍵畫在 `container` 裡，點擊位置換算成設計尺寸的座標交給
 * `Layout.hit` 判定，網頁自己不判定按到哪個鍵（plan §3.1）。長按與預覽的狀態在這裡；
 * 目前頁記在綁定的 `Layout`，動作由綁定照佈局做。
 *
 * `onSnapshot` 收到核心吃下的結果；沒吃下時 `onUnhandled` 收到同一份結果，照它的 `action`
 * 做宿主的預設動作。
 */
export function mountKeyboard(
	container: HTMLElement,
	initialLayout: Layout,
	session: Session,
	onSnapshot: (snapshot: Snapshot) => void,
	onUnhandled: (snapshot: Snapshot) => void,
): Keyboard {
	let layout = initialLayout;
	let keys: LayoutKey[] = [];
	let elements: HTMLElement[] = [];
	let press: Press | undefined;
	/** 畫出來的是哪一頁；切頁鍵換了 `layout` 的目前頁後，和它不同就要整個重畫。 */
	let shownPage = "";
	/** 畫出來時的 Shift 狀態；和 `layout.shift` 不同就要重畫標籤。 */
	let shownShift = "";
	/** 上一次點一下的鍵與時間，判斷快速點兩下。 */
	let lastTap: { page: string; index: number; at: number } | undefined;
	const preview = document.createElement("div");
	preview.className = "key-preview";
	preview.dataset.testid = "key-preview";
	preview.hidden = true;
	const popup = document.createElement("div");
	popup.className = "long-press";
	popup.dataset.testid = "long-press";
	popup.setAttribute("role", "listbox");
	popup.hidden = true;

	// 長度依鍵盤寬度縮放：`#keyboard` 是 inline-size 容器，1cqw 是它寬度的 1%。
	const scaled = (value: number) => `${(value / layout.canvasWidth) * 100}cqw`;
	// 設計尺寸換成鍵盤寬、高的百分比。
	const ofWidth = (value: number) => `${(value / layout.canvasWidth) * 100}%`;
	const ofHeight = (value: number) => `${(value / layout.canvasHeight) * 100}%`;

	/** 依目前的佈局版本與頁面畫全部按鍵。 */
	function render(): void {
		cancel();
		for (const key of keys) {
			key.free();
		}
		keys = layout.keys(session);
		shownPage = layout.pageId;
		shownShift = layout.shift;
		const width = layout.canvasWidth;
		const height = layout.canvasHeight;
		container.style.aspectRatio = `${width} / ${height}`;
		container.style.setProperty("--aspect", String(width / height));
		elements = keys.map((key) => {
			const element = document.createElement("div");
			element.className = "key";
			element.dataset.testid = `key-${key.id}`;
			element.setAttribute("role", "button");
			element.setAttribute("aria-label", key.a11yLabel);
			element.style.left = ofWidth(key.x);
			element.style.top = ofHeight(key.y);
			element.style.width = ofWidth(key.w);
			element.style.height = ofHeight(key.h);
			element.style.zIndex = String(key.z);
			element.style.borderRadius = scaled(key.radius);
			if (key.rotate !== 0) {
				element.style.transform = `rotate(${key.rotate}deg)`;
			}
			const fill = themed(key.fillLight, key.fillDark);
			if (fill !== undefined) {
				element.style.background = fill;
			}
			if (key.opacity !== undefined) {
				element.style.opacity = String(key.opacity);
			}
			const label = document.createElement("span");
			label.className = "label";
			const color = themed(key.labelColorLight, key.labelColorDark);
			if (color !== undefined) {
				label.style.color = color;
			}
			if (key.labelSize !== undefined) {
				label.style.fontSize = scaled(key.labelSize);
			}
			if (key.labelOffsetX !== 0 || key.labelOffsetY !== 0) {
				label.style.transform = `translate(${scaled(key.labelOffsetX)}, ${scaled(key.labelOffsetY)})`;
			}
			const hint = document.createElement("span");
			hint.className = "hint";
			element.append(label, hint);
			paint(element, key);
			return element;
		});
		container.replaceChildren(...elements, preview, popup);
	}

	/** 畫會隨狀態變的部分：標籤、角標、無障礙標籤、Shift 鍵的狀態。 */
	function paint(element: HTMLElement, key: LayoutKey): void {
		const label = element.querySelector<HTMLElement>(".label");
		const hint = element.querySelector<HTMLElement>(".hint");
		if (label) {
			label.textContent = key.isShift
				? (SHIFT_GLYPHS[layout.shift] ?? key.label)
				: key.label;
		}
		if (hint) {
			hint.textContent = key.hint ?? "";
			hint.hidden = key.hint === undefined;
		}
		element.setAttribute("aria-label", key.a11yLabel);
		if (key.isShift) {
			element.dataset.shift = layout.shift;
			element.setAttribute("aria-pressed", String(layout.shift !== "off"));
		}
	}

	/**
	 * 把浮在鍵上方的元素（預覽、長按清單）放在 `key` 上方，高度是鍵的 `rise` 倍。
	 * 預覽置中；長按清單往鍵盤中間展開（`spread`），免得靠邊的鍵的清單超出畫面。
	 */
	function placeAbove(
		element: HTMLElement,
		key: LayoutKey,
		rise: number,
		spread = false,
	) {
		const centre = key.x + key.w / 2;
		element.style.left = "";
		element.style.right = "";
		element.style.transform = "";
		if (!spread) {
			element.style.left = ofWidth(centre);
			element.style.transform = "translateX(-50%)";
		} else if (centre < layout.canvasWidth / 2) {
			element.style.left = ofWidth(key.x);
		} else {
			element.style.right = ofWidth(layout.canvasWidth - key.x - key.w);
		}
		element.style.top = ofHeight(key.y - key.h * rise);
		element.style.height = ofHeight(key.h * rise);
		element.style.minWidth = ofWidth(key.w * 1.2);
		element.style.borderRadius = scaled(key.radius);
		element.style.fontSize = scaled(key.h * 0.6);
	}

	/** 放大預覽照 Gboard：鍵上方的圓形氣泡，直徑約 1.2 個鍵高。 */
	function showPreview(key: LayoutKey): void {
		preview.textContent = key.label;
		placeAbove(preview, key, 1.2);
		preview.style.minWidth = "";
		preview.style.aspectRatio = "1";
		preview.style.borderRadius = "50%";
		preview.hidden = false;
	}

	function showPopup(key: LayoutKey): void {
		popup.replaceChildren(
			...key.longPress.map((label) => {
				const option = document.createElement("div");
				option.className = "option";
				option.dataset.testid = `long-press-${label}`;
				option.setAttribute("role", "option");
				option.textContent = label;
				return option;
			}),
		);
		const columns = Math.min(key.longPress.length, LONG_PRESS_COLUMNS);
		const rows = Math.ceil(key.longPress.length / LONG_PRESS_COLUMNS);
		popup.style.gridTemplateColumns = `repeat(${columns}, auto)`;
		placeAbove(popup, key, 1.1 * rows, true);
		if (rows > 1) {
			// 多排時字小一點，一排 8 個才放得進手機寬度。
			popup.style.fontSize = scaled(key.h * 0.4);
		}
		preview.hidden = true;
		popup.hidden = false;
		choose(0);
	}

	function choose(choice: number): void {
		if (press) {
			press.choice = choice;
		}
		popup.querySelectorAll("[role=option]").forEach((option, i) => {
			option.setAttribute("aria-selected", String(i === choice));
		});
	}

	/** 長按清單出現後手指移到哪個選項：取離手指最近的那個，超出清單也取最近的。 */
	function choiceAt(clientX: number, clientY: number): number {
		const distances = [...popup.querySelectorAll("[role=option]")].map(
			(option) => {
				const rect = option.getBoundingClientRect();
				const dx = Math.max(rect.left - clientX, 0, clientX - rect.right);
				const dy = Math.max(rect.top - clientY, 0, clientY - rect.bottom);
				return dx * dx + dy * dy;
			},
		);
		return distances.indexOf(Math.min(...distances));
	}

	/** 收起預覽與長按清單，忘掉這次按下。 */
	function cancel(): void {
		if (press) {
			clearTimeout(press.timer);
		}
		press = undefined;
		preview.hidden = true;
		popup.hidden = true;
	}

	/** 切頁鍵換了 `layout` 的目前頁時整個重畫，Shift 狀態變了時重畫標籤；回傳有沒有整個重畫。 */
	function followPage(): boolean {
		if (layout.pageId !== shownPage) {
			render();
			return true;
		}
		if (layout.shift !== shownShift) {
			repaint();
		}
		return false;
	}

	/** 重取按鍵、重畫標籤（同一頁，組字或 Shift 狀態變了）。 */
	function repaint(): void {
		const old = keys;
		keys = layout.keys(session);
		for (const key of old) {
			key.free();
		}
		shownShift = layout.shift;
		keys.forEach((key, i) => {
			paint(elements[i], key);
		});
	}

	/** 照綁定的結果更新畫面。重畫會釋放舊頁的鍵，所以先交給外面再看要不要重畫。 */
	function finish(snapshot: Snapshot) {
		try {
			if (snapshot.handled) {
				onSnapshot(snapshot);
			} else {
				onUnhandled(snapshot);
			}
		} finally {
			snapshot.free();
			followPage();
		}
	}

	container.addEventListener("pointerdown", (event) => {
		// 不讓輸入框失去焦點，也不觸發相容的滑鼠事件。
		event.preventDefault();
		if (press) {
			// 快打時前一指還沒放開就按下一鍵：前一鍵先當點一下打出；長按清單開著時不理新的手指。
			if (press.choice !== undefined) {
				return;
			}
			release();
		}
		const rect = container.getBoundingClientRect();
		const x = ((event.clientX - rect.left) / rect.width) * layout.canvasWidth;
		const y = ((event.clientY - rect.top) / rect.height) * layout.canvasHeight;
		const index = layout.hit(x, y);
		if (index === undefined) {
			return;
		}
		const key = keys[index];
		press = {
			pointerId: event.pointerId,
			index,
			choice: undefined,
			timer:
				key.longPress.length > 0
					? window.setTimeout(() => showPopup(key), KEY_LONG_PRESS_MS)
					: undefined,
		};
		container.setPointerCapture(event.pointerId);
		if (key.preview) {
			showPreview(key);
		}
	});

	container.addEventListener("pointermove", (event) => {
		if (press?.pointerId === event.pointerId && press.choice !== undefined) {
			choose(choiceAt(event.clientX, event.clientY));
		}
	});

	/** 放開目前按著的鍵：長按清單開著就做選到的選項，否則點一下。 */
	function release(): void {
		if (!press) {
			return;
		}
		const { index, choice } = press;
		cancel();
		if (choice !== undefined) {
			lastTap = undefined;
			finish(session.longPress(layout, index, choice));
			return;
		}
		const now = performance.now();
		const quickRepeat =
			lastTap !== undefined &&
			lastTap.page === layout.pageId &&
			lastTap.index === index &&
			now - lastTap.at < QUICK_REPEAT_MS;
		// 快速點兩下用掉了就重算，第三下不再算「快速再點」。
		lastTap = quickRepeat ? undefined : { page: layout.pageId, index, at: now };
		finish(session.tap(layout, index, quickRepeat));
	}

	container.addEventListener("pointerup", (event) => {
		if (press?.pointerId === event.pointerId) {
			release();
		}
	});

	container.addEventListener("pointercancel", (event) => {
		if (press?.pointerId === event.pointerId) {
			cancel();
		}
	});

	render();

	return {
		refresh() {
			// 組字中按切頁鍵：核心吃下後外面叫 refresh，這時頁已換，舊頁的元素對不上。
			if (followPage()) {
				return;
			}
			repaint();
		},
		relayout: render,
		setLayout(next) {
			layout = next;
			render();
		},
	};
}
