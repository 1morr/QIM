import assert from "node:assert/strict";
import { test } from "node:test";
import {
	NeuralReranker,
	type PackSource,
	type RerankWorker,
	type WorkerRequest,
} from "../src/neural-rerank.ts";

/** 假的 Worker：記下收到的要求，回覆由測試決定何時送。 */
class FakeWorker implements RerankWorker {
	requests: WorkerRequest[] = [];
	onmessage: RerankWorker["onmessage"] = null;
	onerror: RerankWorker["onerror"] = null;
	onmessageerror: RerankWorker["onmessageerror"] = null;

	postMessage(message: WorkerRequest): void {
		this.requests.push(message);
	}

	/** 回覆第 `index` 個要求。 */
	reply(
		index: number,
		reply: { type: "loaded" } | { type: "failed"; message: string },
	): void {
		const id = this.requests[index]?.id ?? -1;
		this.onmessage?.call(
			this as unknown as Worker,
			new MessageEvent("message", { data: { ...reply, id } }),
		);
	}
}

const modelA: PackSource = { manifest: "A", bytes: new Uint8Array() };
const modelB: PackSource = { manifest: "B", bytes: new Uint8Array() };

function setUp(check: (pack: PackSource) => void = () => {}) {
	const worker = new FakeWorker();
	const failures: string[] = [];
	const neural = new NeuralReranker({
		worker,
		check,
		onLoadFailed: (error) => failures.push(error.message),
	});
	return { worker, failures, neural };
}

/** 等 promise 的接續跑完。 */
const settle = () => new Promise((resolve) => setImmediate(resolve));

test("換模型 A→B、A 晚失敗：不報錯，B 照樣算正在用的", async () => {
	const { worker, failures, neural } = setUp();
	neural.load(modelA);
	neural.prepare(modelB)?.();
	worker.reply(0, { type: "failed", message: "A 壞了" });
	worker.reply(1, { type: "loaded" });
	await settle();
	assert.deepEqual(failures, []);
	assert.equal(neural.prepare(modelB), undefined, "B 不必重載");
});

test("正在換的那份失敗：報錯，下次換資料包再試", async () => {
	const { worker, failures, neural } = setUp();
	neural.load(modelA);
	neural.prepare(modelB)?.();
	worker.reply(0, { type: "loaded" });
	worker.reply(1, { type: "failed", message: "B 壞了" });
	await settle();
	assert.deepEqual(failures, ["B 壞了"]);
	assert.notEqual(neural.prepare(modelB), undefined, "B 要再載一次");
});

test("Worker 出錯時等著的載入都失敗，只報最後那份", async () => {
	const { worker, failures, neural } = setUp();
	neural.load(modelA);
	neural.prepare(modelB)?.();
	worker.onerror?.call(
		worker as unknown as Worker,
		new ErrorEvent("error", { message: "掛了" }),
	);
	await settle();
	assert.deepEqual(failures, ["停鍵層的 Worker 出錯：掛了"]);
});

test("和正在用的同一份不重載、不核對", () => {
	let checked = 0;
	const { worker, neural } = setUp(() => checked++);
	neural.load(modelA);
	assert.equal(neural.prepare(modelA), undefined);
	assert.equal(checked, 0);
	assert.equal(worker.requests.length, 1);
});

test("核心不收時丟錯，什麼都不送、正在用的不變", () => {
	const { worker, neural } = setUp((pack) => {
		throw new Error(`不收 ${pack.manifest}`);
	});
	neural.load(modelA);
	assert.throws(() => neural.prepare(modelB), /不收 B/);
	assert.equal(worker.requests.length, 1);
	assert.equal(neural.prepare(modelA), undefined);
});

test("拿掉模型：送 undefined，不核對", () => {
	let checked = 0;
	const { worker, neural } = setUp(() => checked++);
	neural.load(modelA);
	neural.prepare(undefined)?.();
	assert.equal(checked, 0);
	assert.equal(worker.requests[1]?.type, "load");
	assert.equal(
		worker.requests[1]?.type === "load" && worker.requests[1].pack,
		undefined,
	);
});
