// 停鍵層的神經模型（plan §2.7、§3.1，M3 票 11e）：主執行緒這邊的介面，推論在 Web Worker
// （`rerank-worker.ts`）。主執行緒先用同一份綁定建一次 `Reranker` 核對模型資料包（核心不收就丟錯，同
// 其他資料包），再把 manifest 與位元組交給 Worker。

import type { Rank, Ranked, RerankJob } from "./rerank.ts";
import { Pack, Reranker } from "./wasm/qim_wasm.js";

/** 一份資料包的原料：manifest 的 JSON 與資料包檔。 */
export interface PackSource {
	manifest: string;
	bytes: Uint8Array;
}

/** 交給 Worker 的訊息；送出時加上 `id`（`WorkerRequest`），回覆帶同一個 `id`。 */
type WorkerMessage =
	| { type: "load"; pack: PackSource | undefined }
	| { type: "rank"; job: RerankJob };

export type WorkerRequest = WorkerMessage & { id: number };

export type WorkerReply =
	| { id: number; type: "loaded" }
	| { id: number; type: "failed"; message: string }
	/** 沒載入模型時沒有 `ranked`。 */
	| { id: number; type: "ranked"; ranked?: Ranked };

/** 核心與 `qim-neural` 收不收這份模型資料包：不收時丟錯（訊息說明原因）。 */
export function checkModelPack(source: PackSource): void {
	const pack = new Pack(source.manifest, source.bytes);
	try {
		new Reranker(pack).free();
	} finally {
		pack.free();
	}
}

export class NeuralReranker {
	private readonly worker = new Worker(
		new URL("./rerank-worker.ts", import.meta.url),
		{ type: "module" },
	);
	private next = 0;
	private readonly waiting = new Map<
		number,
		{ resolve: (reply: WorkerReply) => void; reject: (error: Error) => void }
	>();
	/** Worker 起不來或丟了沒接住的錯：之後的要求都直接失敗，不會一直等。 */
	private broken: Error | undefined;

	constructor() {
		this.worker.onmessage = (event: MessageEvent<WorkerReply>) => {
			const waiting = this.waiting.get(event.data.id);
			this.waiting.delete(event.data.id);
			if (event.data.type === "failed") {
				waiting?.reject(new Error(event.data.message));
			} else {
				waiting?.resolve(event.data);
			}
		};
		const fail = (error: Error) => {
			this.broken = error;
			for (const { reject } of this.waiting.values()) {
				reject(error);
			}
			this.waiting.clear();
		};
		this.worker.onerror = (event) =>
			fail(new Error(`停鍵層的 Worker 出錯：${event.message}`));
		this.worker.onmessageerror = () =>
			fail(new Error("停鍵層的 Worker 收到讀不了的訊息"));
	}

	/** 送一則訊息，等同 `id` 的回覆；Worker 回 `failed` 或出錯時丟錯。 */
	private send(message: WorkerMessage): Promise<WorkerReply> {
		if (this.broken) {
			return Promise.reject(this.broken);
		}
		const id = this.next++;
		return new Promise((resolve, reject) => {
			this.waiting.set(id, { resolve, reject });
			this.worker.postMessage({ ...message, id } satisfies WorkerRequest);
		});
	}

	/** 換模型；`undefined` 是拿掉（之後不重排）。Worker 建不起來時丟錯。 */
	async load(pack: PackSource | undefined): Promise<void> {
		await this.send({ type: "load", pack });
	}

	/** 交給 `rerankScheduler` 的模型。 */
	readonly rank: Rank = async (job) => {
		const reply = await this.send({ type: "rank", job });
		return reply.type === "ranked" ? reply.ranked : undefined;
	};
}
