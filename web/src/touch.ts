import type { Layout, LayoutKey, Session, Snapshot } from "./wasm/qim_wasm.js";

/**
 * 觸控鍵盤：照 `qim-layout` 給的按鍵畫在 `container` 裡，點擊位置換算成設計尺寸的座標交給
 * `Layout.hit` 判定，網頁自己不判定按到哪個鍵（plan §3.1）。
 *
 * `onSnapshot` 收到核心吃下的結果；沒吃下時 `onUnhandled` 收到按鍵，照宿主的預設動作處理。
 */
export function mountKeyboard(
	container: HTMLElement,
	layout: Layout,
	session: Session,
	onSnapshot: (snapshot: Snapshot) => void,
	onUnhandled: (key: LayoutKey) => void,
): void {
	const keys = layout.keys(session);
	const width = layout.canvasWidth;
	const height = layout.canvasHeight;
	container.style.aspectRatio = `${width} / ${height}`;
	const percent = (value: number, of: number) => `${(value / of) * 100}%`;
	container.replaceChildren(
		...keys.map((key) => {
			const element = document.createElement("div");
			element.className = "key";
			element.dataset.testid = `key-${key.id}`;
			element.style.left = percent(key.x, width);
			element.style.top = percent(key.y, height);
			element.style.width = percent(key.w, width);
			element.style.height = percent(key.h, height);
			element.style.zIndex = String(key.z);
			const label = document.createElement("span");
			label.className = "label";
			label.textContent = key.label;
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
		const snapshot = session.tap(layout, index);
		try {
			if (snapshot.handled) {
				onSnapshot(snapshot);
			} else {
				onUnhandled(keys[index]);
			}
		} finally {
			snapshot.free();
		}
	});
}
