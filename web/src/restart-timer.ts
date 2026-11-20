// 可重新計時的計時器：核心不讀時間，有時限的狀態由殼計時、到了叫核心（architecture〈核心對殼〉）。網頁用在
// 退格撤銷與連按兩下空白輸出句點，時限由呼叫端給。

/** 計時器：`restart` 從頭算 `ms`，到了呼叫一次 `fire`；再 `restart` 取消上一次的。 */
export function restartTimer(
	fire: () => void,
	ms: number,
): { restart: () => void } {
	let timer: ReturnType<typeof setTimeout> | undefined;
	return {
		restart: () => {
			clearTimeout(timer);
			timer = setTimeout(fire, ms);
		},
	};
}
