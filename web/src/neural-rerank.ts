// 停鍵層的神經模型（plan §2.7、§3.1，M3 票 11e）：主執行緒這邊的介面，推論在 Web Worker
// （`rerank-worker.ts`）。換資料包時先在主執行緒核對模型資料包（核心不收就丟錯，同其他資料包），再把
// manifest 與位元組交給 Worker；Worker 正在用哪份、載入失敗要不要報都在這裡管。
// 不在執行期 import WebAssembly 綁定：Worker 與核對由殼給，單元測試換成假的（不必先跑 `cargo xtask web`）。

import type { Rank, Ranked, RerankJob } from "./rerank.ts";

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

/** `NeuralReranker` 用到的 `Worker` 成員；單元測試給假的。 */
export type RerankWorker = Pick<
	Worker,
	"postMessage" | "onmessage" | "onerror" | "onmessageerror"
>;

export interface NeuralRerankerOptions {
	/** 跑 `rerank-worker.ts` 的 Worker。 */
	worker: RerankWorker;
	/** 核心與 `qim-neural` 收不收這份模型資料包：不收時丟錯（訊息說明原因）。 */
	check: (pack: PackSource) => void;
	/** Worker 載入不了模型（之後不重排，打字與候選照常）。 */
	onLoadFailed: (error: Error) => void;
}

export class NeuralReranker {
	private readonly worker: RerankWorker;
	private readonly check: (pack: PackSource) => void;
	private readonly onLoadFailed: (error: Error) => void;
	private next = 0;
	private readonly waiting = new Map<
		number,
		{ resolve: (reply: WorkerReply) => void; reject: (error: Error) => void }
	>();
	/** Worker 起不來或丟了沒接住的錯：之後的要求都直接失敗，不會一直等。 */
	private broken: Error | undefined;
	/** Worker 正在用（或正在載入）的模型的 manifest；沒變就不重載，載入失敗時清掉，下次換資料包再試。 */
	private loaded: string | undefined;
	/** 交給 Worker 載入過幾次；只有最後一次的失敗算數。 */
	private loads = 0;

	constructor(options: NeuralRerankerOptions) {
		this.worker = options.worker;
		this.check = options.check;
		this.onLoadFailed = options.onLoadFailed;
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

	/**
	 * 交給 Worker 換成 `model`；`undefined` 是拿掉（之後不重排）。不在主執行緒核對：載入不了時由
	 * `onLoadFailed` 報，打字不等它。
	 */
	load(model: PackSource | undefined): void {
		this.loaded = model?.manifest;
		const load = ++this.loads;
		this.send({ type: "load", pack: model }).catch((error: unknown) => {
			// 之後又換了別份（換模型 A→B 時 A 晚失敗）：Worker 用的是後來那份，這次的失敗不管、不報。
			if (load !== this.loads) {
				return;
			}
			this.loaded = undefined;
			this.onLoadFailed(
				error instanceof Error ? error : new Error(String(error)),
			);
		});
	}

	/**
	 * 換資料包時的模型：先核對（不收就丟錯，資料包的改動退回），核心也收下其他資料包之後才呼叫回傳的
	 * 函式交給 Worker；和正在用的同一份時是 `undefined`。
	 */
	prepare(model: PackSource | undefined): (() => void) | undefined {
		if (model?.manifest === this.loaded) {
			return undefined;
		}
		if (model) {
			this.check(model);
		}
		return () => this.load(model);
	}

	/** Worker 正在用（或正在載入）的模型的 manifest；沒有模型或載入失敗時是 `undefined`。 */
	get manifest(): string | undefined {
		return this.loaded;
	}

	/** 交給 `rerankScheduler` 的模型。 */
	readonly rank: Rank = async (job) => {
		const reply = await this.send({ type: "rank", job });
		return reply.type === "ranked" ? reply.ranked : undefined;
	};
}
