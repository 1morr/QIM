// 候選列（plan §3.1）：左邊中英狀態、目前頁的候選與字根提示；點候選選字，長按叫出字根提示。

import { element } from "./dom.ts";
import type { RadicalHints } from "./wasm/qim_wasm.js";

/** 長按候選多久叫出字根提示（毫秒）；和 Android 的長按門檻同級。 */
const CANDIDATE_HINT_PRESS_MS = 500;

/**
 * 一個候選的字根提示：每個字的字根接著倉頡碼，如「日弓(an) 口(r)」；詞裡沒有碼的字是「—」。
 * 整個候選都沒有碼（英文候選）時是空字串，不顯示。
 */
function hintText(
	radicalHints: RadicalHints,
	version: string,
	candidate: string,
): string {
	const hints = radicalHints.hint(candidate, version);
	const parts = hints.map((hint) =>
		hint.codes.map((code, i) => `${hint.radicals[i]}(${code})`).join("／"),
	);
	for (const hint of hints) {
		hint.free();
	}
	return parts.every((part) => part === "")
		? ""
		: parts.map((part) => part || "—").join(" ");
}

export interface CandidateBar {
	/**
	 * 換成目前頁的候選；`originals` 是繁體原文，字根提示用：簡體模式下要提示的是打得出的繁體字。
	 * `chinese` 是中文組字中（臨時英文不算）：不是時收起字根提示（brief §4.3）。
	 */
	show(candidates: string[], originals: string[], chinese: boolean): void;
	/** 中英狀態。 */
	setMode(ascii: boolean): void;
	/** 切換候選旁的字根提示；中文組字中才有候選可查。 */
	toggleHints(): void;
}

/**
 * 接上候選列。`select` 收到點的候選在目前頁的位置。
 */
export function mountCandidateBar(
	radicalHints: RadicalHints,
	select: (index: number) => void,
): CandidateBar {
	const list = element<HTMLOListElement>("#candidates");
	const mode = element<HTMLElement>("#mode");
	const hintVersion = element<HTMLSelectElement>("#hint-version");
	let candidates: string[] = [];
	let originals: string[] = [];
	let chinese = false;
	/** 候選旁顯示字根提示；組字結束就關掉。 */
	let hints = false;

	const render = (): void => {
		list.replaceChildren(
			...candidates.map((candidate, i) => {
				const item = document.createElement("li");
				item.dataset.index = String(i);
				const index = document.createElement("span");
				index.className = "index";
				index.textContent = String(i + 1);
				item.append(index, candidate);
				const text = hints
					? hintText(radicalHints, hintVersion.value, originals[i] ?? candidate)
					: "";
				if (text !== "") {
					const hint = document.createElement("span");
					hint.className = "radical-hint";
					hint.dataset.testid = `hint-${i + 1}`;
					hint.textContent = text;
					item.append(hint);
				}
				return item;
			}),
		);
	};
	const toggleHints = (): void => {
		if (chinese) {
			hints = !hints;
			render();
		}
	};
	hintVersion.addEventListener("change", render);

	// 點候選選字，長按叫出字根提示（brief §4.4）：放開時還沒到長按的門檻才選字。
	let pressTimer: number | undefined;
	/** 按下的候選在目前頁的位置；長按觸發或取消後是 `undefined`。 */
	let pressed: number | undefined;
	const cancelPress = (): void => {
		window.clearTimeout(pressTimer);
		pressTimer = undefined;
		pressed = undefined;
	};
	list.addEventListener("pointerdown", (event) => {
		// 不讓輸入框失去焦點，長按完還能接著打字。
		event.preventDefault();
		cancelPress();
		const item = (event.target as Element).closest<HTMLElement>("li");
		pressed = item ? Number(item.dataset.index) : undefined;
		pressTimer = window.setTimeout(() => {
			cancelPress();
			toggleHints();
		}, CANDIDATE_HINT_PRESS_MS);
	});
	list.addEventListener("pointerup", () => {
		const index = pressed;
		cancelPress();
		if (index !== undefined) {
			select(index);
		}
	});
	// 手指左右滑候選列時瀏覽器接手捲動、送 pointercancel，長按與選字都取消。
	for (const type of ["pointerleave", "pointercancel"]) {
		list.addEventListener(type, cancelPress);
	}
	list.addEventListener("contextmenu", (event) => event.preventDefault());

	return {
		show(nextCandidates, nextOriginals, nextChinese) {
			candidates = nextCandidates;
			originals = nextOriginals;
			chinese = nextChinese;
			if (!chinese) {
				hints = false;
			}
			render();
		},
		setMode(ascii) {
			mode.textContent = ascii ? "英" : "中";
			mode.setAttribute("aria-label", ascii ? "英文" : "中文");
		},
		toggleHints,
	};
}
