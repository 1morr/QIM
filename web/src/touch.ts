import type { Layout, LayoutKey, Session, Snapshot } from "./wasm/qim_wasm.js";

/** 觸控鍵盤掛好後，核心狀態改變時叫 `refresh` 重畫標籤（組字中的標籤可能不同）。 */
export interface Keyboard {
	refresh(): void;
}

/**
 * 觸控鍵盤：照 `qim-layout` 給的按鍵畫在 `container` 裡，點擊位置換算成設計尺寸的座標交給
 * `Layout.hit` 判定，網頁自己不判定按到哪個鍵（plan §3.1）。
 *
 * `onSnapshot` 收到核心吃下的結果與按的鍵；沒吃下時 `onUnhandled` 收到按鍵，照宿主的預設動作處理。
 */
export function mountKeyboard(
	container: HTMLElement,
	layout: Layout,
	session: Session,
	onSnapshot: (snapshot: Snapshot, key: LayoutKey) => void,
	onUnhandled: (key: LayoutKey) => void,
): Keyboard {
	let keys = layout.keys(session);
	const width = layout.canvasWidth;
	const height = layout.canvasHeight;
	container.style.aspectRatio = `${width} / ${height}`;
	const percent = (value: number, of: number) => `${(value / of) * 100}%`;
	// 長度依鍵盤寬度縮放：`#keyboard` 是 inline-size 容器，1cqw 是它寬度的 1%。
	const scaled = (value: number) => `${(value / width) * 100}cqw`;
	const labels: HTMLElement[] = [];
	container.replaceChildren(
		...keys.map((key) => {
			const element = document.createElement("div");
			element.className = "key";
			element.dataset.testid = `key-${key.id}`;
			element.setAttribute("role", "button");
			element.setAttribute("aria-label", key.a11yLabel);
			element.style.left = percent(key.x, width);
			element.style.top = percent(key.y, height);
			element.style.width = percent(key.w, width);
			element.style.height = percent(key.h, height);
			element.style.zIndex = String(key.z);
			element.style.borderRadius = scaled(key.radius);
			if (key.rotate !== 0) {
				element.style.transform = `rotate(${key.rotate}deg)`;
			}
			if (key.fill !== undefined) {
				element.style.background = key.fill;
			}
			if (key.opacity !== undefined) {
				element.style.opacity = String(key.opacity);
			}
			const label = document.createElement("span");
			label.className = "label";
			label.textContent = key.label;
			if (key.labelColor !== undefined) {
				label.style.color = key.labelColor;
			}
			if (key.labelSize !== undefined) {
				label.style.fontSize = scaled(key.labelSize);
			}
			if (key.labelOffsetX !== 0 || key.labelOffsetY !== 0) {
				label.style.transform = `translate(${scaled(key.labelOffsetX)}, ${scaled(key.labelOffsetY)})`;
			}
			labels.push(label);
			element.append(label);
			if (key.hint !== undefined) {
				const hint = document.createElement("span");
				hint.className = "hint";
				hint.textContent = key.hint;
				element.append(hint);
			}
			return element;
		}),
	);

	container.addEventListener("pointerdown", (event) => {
		// 不讓輸入框失去焦點，也不觸發相容的滑鼠事件。
		event.preventDefault();
		const rect = container.getBoundingClientRect();
		const x = ((event.clientX - rect.left) / rect.width) * width;
		const y = ((event.clientY - rect.top) / rect.height) * height;
		const index = layout.hit(x, y);
		if (index === undefined) {
			return;
		}
		// 按下前的動作：組字中與否決定 `composing` 的動作，核心處理後狀態就變了。
		const key = keys[index];
		const snapshot = session.tap(layout, index);
		try {
			if (snapshot.handled) {
				onSnapshot(snapshot, key);
			} else {
				onUnhandled(key);
			}
		} finally {
			snapshot.free();
		}
	});

	return {
		refresh() {
			const old = keys;
			keys = layout.keys(session);
			for (const key of old) {
				key.free();
			}
			keys.forEach((key, i) => {
				labels[i].textContent = key.label;
				labels[i].parentElement?.setAttribute("aria-label", key.a11yLabel);
			});
		},
	};
}
