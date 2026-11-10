// 組字區：一段一個 `<span>`，點一段就移到那段（brief §4.4）。

import { element } from "./dom.ts";
import type { Snapshot } from "./wasm/qim_wasm.js";

export interface Composition {
	/** 畫出 `snapshot` 的組字；回傳是否在組字中。 */
	show(snapshot: Snapshot): boolean;
}

/** 接上組字區。`focus` 收到點的分段位置。 */
export function mountComposition(focus: (index: number) => void): Composition {
	const composition = element<HTMLElement>("#composition");
	// 取消預設動作，輸入框不失去焦點（同觸控鍵盤）。
	composition.addEventListener("pointerdown", (event) => {
		const segment = (event.target as Element).closest<HTMLElement>(".segment");
		if (!segment) {
			return;
		}
		event.preventDefault();
		focus(Number(segment.dataset.index));
	});
	return {
		/**
		 * 選定的顯示字詞，沒選定的顯示字根（brief §4.3），臨時英文顯示英文；目前分段標
		 * `aria-current`。
		 */
		show(snapshot) {
			const segments = snapshot.segments;
			composition.replaceChildren(
				...segments.map((segment, i) => {
					const span = document.createElement("span");
					span.className = "segment";
					span.dataset.index = String(i);
					span.textContent =
						segment.selected || segment.raw ? segment.text : segment.radicals;
					if (i === snapshot.focus) {
						span.setAttribute("aria-current", "true");
					}
					return span;
				}),
			);
			for (const segment of segments) {
				segment.free();
			}
			return segments.length > 0;
		},
	};
}
