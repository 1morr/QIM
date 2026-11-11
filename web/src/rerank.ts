/**
 * 重排的假實作（plan §3.1）：網頁延遲一段時間後回傳路徑的名次，驗證核心的
 * `rerankRequest` / `applyRerank`：過期的結果丟掉、排出的路徑最高到第 2 位，候選列沒有就插入
 * （ADR 0005，M3 票 11c）。神經模型由票 11e 接上。
 *
 * 重排會改候選位置，所以有開關（brief §7 的雲端候選，預設開）。
 */

import type { Session, Snapshot } from "./wasm/qim_wasm.js";

/**
 * `off` 不要求重排；`identity` 原樣回傳（首選就是第一條路徑，名次不變）；`raise-last` 把最後一條
 * 路徑排到最前，測插入與上限用：它多半不在候選列裡。
 */
export type RerankMode = "off" | "identity" | "raise-last";

/** 假的模型多久回來；這段時間內再按鍵，結果就過期、被核心丟掉。 */
export const RERANK_DELAY_MS = 300;

/** 設定選單的值轉成模式；不認得的當關。 */
export function rerankMode(value: string): RerankMode {
	return value === "identity" || value === "raise-last" ? value : "off";
}

/** 假的模型給 `count` 條路徑排的名次：要求的路徑的序號，好的在前。 */
export function fakeOrder(
	mode: Exclude<RerankMode, "off">,
	count: number,
): number[] {
	const order = Array.from({ length: count }, (_, i) => i);
	if (mode === "raise-last") {
		const last = order.pop();
		if (last !== undefined) {
			order.unshift(last);
		}
	}
	return order;
}

/**
 * 候選變了就把重排要求交給假的模型，`RERANK_DELAY_MS` 後套用（plan §3.1）。這段時間內又按了鍵，
 * 結果帶的 `generation` 過期，核心丟掉。名次變了才交給 `show` 重畫，也不再要求重排；回報只看
 * 首選，重排不動首選，所以不經過回報。回傳的函式在每次畫完核心的 `Snapshot` 後呼叫。
 */
export function fakeReranker(
	session: Session,
	mode: () => RerankMode,
	show: (snapshot: Snapshot) => void,
): (generation: bigint) => void {
	/** 上次要求重排時的 `generation`；候選沒變就不再要求。 */
	let rerankedAt: bigint | undefined;
	return (generation) => {
		const chosen = mode();
		if (chosen === "off" || generation === rerankedAt) {
			return;
		}
		rerankedAt = generation;
		const request = session.rerankRequest();
		if (!request) {
			return;
		}
		const asked = request.generation;
		const order = fakeOrder(chosen, request.paths.length);
		request.free();
		setTimeout(() => {
			// 等待中關掉了重排，就不套用。
			if (mode() === "off") {
				return;
			}
			const result = session.applyRerank(asked, Uint32Array.from(order));
			if (!result) {
				return;
			}
			try {
				// `generation` 沒變是名次沒變，不必重畫。
				if (result.generation === asked) {
					return;
				}
				rerankedAt = result.generation;
				show(result);
			} finally {
				result.free();
			}
		}, RERANK_DELAY_MS);
	};
}
