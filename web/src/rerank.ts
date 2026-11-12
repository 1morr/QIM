/**
 * 停鍵層的重排（plan §2.7、§3.1、ADR 0005）：停鍵 `STOP_MS` 後把核心的 `rerankRequest` 交給模型，
 * 回來的路徑名次經 `applyRerank` 套用；過期的由核心丟掉，打字不等它。模型是 Web Worker 裡的神經模型
 * （`neural-rerank.ts`，M3 票 11e），或測試用的假實作：延遲一段時間後回傳固定的名次。
 *
 * 重排會改候選位置，所以有開關（brief §7 的停鍵重排，預設開）。
 */

import type { Session, Snapshot } from "./wasm/qim_wasm.js";

/**
 * `neural` 是停鍵層的神經模型；`off` 不要求重排。其餘是假實作：`identity` 原樣回傳（首選就是第一條路徑，
 * 名次不變）；`raise-last` 把最後一條路徑排到最前，測插入與上限用：它多半不在候選列裡。
 */
export type RerankMode = "off" | "neural" | FakeMode;
/** 測試用的假實作。 */
export type FakeMode = "identity" | "raise-last";
const MODES: readonly string[] = [
	"off",
	"neural",
	"identity",
	"raise-last",
] satisfies RerankMode[];

/** 停鍵多久才要求重排：照青簡在 macOS 的 80 ms（brief §12.4）；這段時間內再按鍵就重新計時。 */
export const STOP_MS = 80;

/** 假的模型多久回來；這段時間內再按鍵，結果就過期、被核心丟掉。 */
export const FAKE_DELAY_MS = 300;

/** 設定選單的值轉成模式；不認得的當關。 */
export function rerankMode(value: string): RerankMode {
	return MODES.includes(value) ? (value as RerankMode) : "off";
}

/** 交給模型的重排要求：核心 `RerankRequest` 裡模型要的欄位，可以傳進 Web Worker。 */
export interface RerankJob {
	context: string;
	paths: string[];
	scores: Float64Array;
}

/** 模型給的名次：路徑的序號，好的在前，和模型算了多久。 */
export interface Ranked {
	order: Uint32Array;
	ms: number;
	/** 神經模型給各路徑的 `路徑分 + λ·神經分`，和要求的路徑同順序（debug 面板）；假實作沒有。 */
	totals?: Float64Array;
}

/** 一個模型：收要求、回名次；沒有模型時回 `undefined`。 */
export type Rank = (job: RerankJob) => Promise<Ranked | undefined>;

/** 假的模型給 `count` 條路徑排的名次：要求的路徑的序號，好的在前。 */
export function fakeOrder(mode: FakeMode, count: number): number[] {
	const order = Array.from({ length: count }, (_, i) => i);
	if (mode === "raise-last") {
		const last = order.pop();
		if (last !== undefined) {
			order.unshift(last);
		}
	}
	return order;
}

/** 假實作：`FAKE_DELAY_MS` 後回 `fakeOrder`。 */
export function fakeRank(mode: FakeMode): Rank {
	return (job) =>
		new Promise((resolve) =>
			setTimeout(
				() =>
					resolve({
						order: Uint32Array.from(fakeOrder(mode, job.paths.length)),
						ms: 0,
					}),
				FAKE_DELAY_MS,
			),
		);
}

/**
 * 一次重排的結果：`applied` 名次變了、已重畫；`unchanged` 名次沒變；`stale` 要求之後候選變了，核心丟掉；
 * `none` 沒有模型或等待中關掉了重排。
 */
export type Outcome = "applied" | "unchanged" | "stale" | "none";

export interface RerankSchedulerOptions {
	/** 目前的模式，每次要求與套用前讀。 */
	mode: () => RerankMode;
	/** 各模式的模型；`off` 以外每種都要有。 */
	models: Readonly<Record<Exclude<RerankMode, "off">, Rank>>;
	/** 名次變了：畫核心回的 `Snapshot`（之後由這裡釋放）。 */
	show: (snapshot: Snapshot) => void;
	/** 要求交給模型時（debug 面板顯示要求）。 */
	onRequest?: (generation: bigint, job: RerankJob) => void;
	/** 每次結果回來之後（給畫面記錄與 e2e 讀）。 */
	onResult?: (outcome: Outcome, ranked: Ranked | undefined) => void;
	/** 結果回來後再等多久才套用：e2e 模擬慢的模型（網址的 `rerank-delay`），平常是 0。 */
	extraDelayMs?: number;
}

/**
 * 接上重排：回傳的函式在每次畫完核心的 `Snapshot` 後以它的 `generation` 呼叫。停鍵 `STOP_MS` 後取
 * 核心當時的要求交給模型；同時只有一個在算，算的時候又停鍵了就記著，回來後再取一次最新的。
 * 名次變了才交給 `show` 重畫，也不再要求重排；回報只看首選，重排不動首選，所以不經過回報。
 */
export function rerankScheduler(
	session: Session,
	options: RerankSchedulerOptions,
): (generation: bigint) => void {
	/** 上次要求重排或套用後的 `generation`；候選沒變就不再要求。 */
	let rerankedAt: bigint | undefined;
	let timer: ReturnType<typeof setTimeout> | undefined;
	let busy = false;
	/** 模型在算的時候又停鍵了：回來後再要求一次。 */
	let again = false;

	const finish = (asked: bigint, ranked: Ranked | undefined): Outcome => {
		if (!ranked || options.mode() === "off") {
			return "none";
		}
		const result = session.applyRerank(asked, ranked.order);
		if (!result) {
			return "stale";
		}
		try {
			// `generation` 沒變是名次沒變，不必重畫。
			if (result.generation === asked) {
				return "unchanged";
			}
			rerankedAt = result.generation;
			options.show(result);
			return "applied";
		} finally {
			result.free();
		}
	};

	const ask = (): void => {
		const mode = options.mode();
		if (mode === "off") {
			return;
		}
		if (busy) {
			again = true;
			return;
		}
		const request = session.rerankRequest();
		if (!request) {
			return;
		}
		const asked = request.generation;
		const job: RerankJob = {
			context: request.context,
			paths: request.paths,
			scores: Float64Array.from(request.scores),
		};
		request.free();
		if (asked === rerankedAt) {
			return;
		}
		rerankedAt = asked;
		busy = true;
		options.onRequest?.(asked, job);
		let reported = false;
		const delay = options.extraDelayMs ?? 0;
		options.models[mode](job)
			.then((ranked) =>
				delay > 0
					? new Promise<Ranked | undefined>((resolve) =>
							setTimeout(() => resolve(ranked), delay),
						)
					: ranked,
			)
			.then((ranked) => {
				const outcome = finish(asked, ranked);
				// 回報過了（`onResult` 自己丟錯）就不再回報第二次。
				reported = true;
				options.onResult?.(outcome, ranked);
			})
			// 模型、`finish`（含 `show`）、`onResult` 丟的錯都在這裡接：只記 console.error，沒回報過就記成沒有結果。
			.catch((error: unknown) => {
				console.error("重排失敗", error);
				if (!reported) {
					options.onResult?.("none", undefined);
				}
			})
			.finally(() => {
				busy = false;
				if (again) {
					again = false;
					ask();
				}
			});
	};

	return (generation) => {
		clearTimeout(timer);
		if (options.mode() === "off" || generation === rerankedAt) {
			return;
		}
		timer = setTimeout(ask, STOP_MS);
	};
}
