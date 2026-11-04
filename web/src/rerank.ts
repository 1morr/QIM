/**
 * 重排的假實作（plan §3.1）：M1 沒有背景模型，網頁延遲一段時間後回傳名次，驗證核心的
 * `rerankRequest` / `applyRerank`：過期的結果丟掉、最高到第 2 位（ADR 0005）。M3 換成雲端 LLM。
 *
 * 重排會改候選位置，所以有開關（brief §7 的雲端候選，預設開）。
 */

/** `off` 不要求重排；`identity` 原樣回傳；`raise-fifth` 把第 5 個提到最前，測上限用。 */
export type RerankMode = "off" | "identity" | "raise-fifth";

/** 假的模型多久回來；這段時間內再按鍵，結果就過期、被核心丟掉。 */
export const RERANK_DELAY_MS = 300;

/** 設定選單的值轉成模式；不認得的當關。 */
export function rerankMode(value: string): RerankMode {
	return value === "identity" || value === "raise-fifth" ? value : "off";
}

/** 假的模型給 `count` 個候選排的名次：要求的候選的位置，好的在前。 */
export function fakeOrder(
	mode: Exclude<RerankMode, "off">,
	count: number,
): number[] {
	const order = Array.from({ length: count }, (_, i) => i);
	if (mode === "raise-fifth" && count >= 5) {
		order.splice(4, 1);
		order.unshift(4);
	}
	return order;
}
