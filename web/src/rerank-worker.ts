// 停鍵層的 Web Worker（ADR 0005，M3 票 11e）：載入模型資料包建 `Reranker`，收重排要求、回路徑的
// 名次與算了多久。和主執行緒用同一份 WebAssembly 綁定，各自一個實例；主執行緒的 `Session` 不在這裡。

import { freeAfter } from "./free.ts";
import type { WorkerReply, WorkerRequest } from "./neural-rerank.ts";
import init, { Pack, Reranker } from "./wasm/qim_wasm.js";

// tsconfig 的 lib 是 DOM（給網頁），沒有 Worker 的 `DedicatedWorkerGlobalScope`；這裡只用到這兩個成員。
const scope = self as unknown as {
	onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
	postMessage(message: WorkerReply): void;
};

const ready = init();
let reranker: Reranker | undefined;

function message(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}

function handle(request: WorkerRequest): WorkerReply {
	switch (request.type) {
		case "load": {
			if (reranker) {
				freeAfter(reranker, () => {
					reranker = undefined;
				});
			}
			const source = request.pack;
			if (!source) {
				return { id: request.id, type: "loaded" };
			}
			const pack = new Pack(source.manifest, source.bytes);
			try {
				reranker = freeAfter(pack, (pack) => new Reranker(pack));
				return { id: request.id, type: "loaded" };
			} catch (error) {
				return { id: request.id, type: "failed", message: message(error) };
			}
		}
		case "rank": {
			if (!reranker) {
				return { id: request.id, type: "ranked" };
			}
			const start = performance.now();
			const ranking = reranker.rank(
				request.job.context,
				request.job.paths,
				request.job.scores,
			);
			const ms = performance.now() - start;
			return freeAfter(ranking, ({ order, totals }) => ({
				id: request.id,
				type: "ranked",
				ranked: { order, ms, totals },
			}));
		}
	}
}

// 綁定載入完才處理；`await` 之後的接續照收到的順序跑。綁定載入失敗也回 `failed`，主執行緒不會一直等。
scope.onmessage = async (event) => {
	try {
		await ready;
		scope.postMessage(handle(event.data));
	} catch (error) {
		scope.postMessage({
			id: event.data.id,
			type: "failed",
			message: message(error),
		});
	}
};
