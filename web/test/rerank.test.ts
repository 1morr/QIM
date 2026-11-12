import assert from "node:assert/strict";
import { afterEach, beforeEach, mock, test } from "node:test";
import {
	fakeOrder,
	type Outcome,
	type Rank,
	type Ranked,
	type RerankJob,
	rerankerId,
	rerankMode,
	rerankScheduler,
	STOP_MS,
} from "../src/rerank.ts";
import type { Session, Snapshot } from "../src/wasm/qim_wasm.js";

// 核心的停鍵結果快取認這個識別（plan §2.7，M4 票 03）：變了就清掉，`undefined` 不留快取。
test("重排識別：關掉與沒有模型是 undefined，神經模型帶 manifest，假實作是模式", () => {
	assert.equal(rerankerId("off", "A"), undefined);
	assert.equal(rerankerId("neural", undefined), undefined);
	assert.notEqual(rerankerId("neural", "A"), rerankerId("neural", "B"));
	assert.notEqual(rerankerId("neural", "A"), rerankerId("identity", "A"));
	assert.notEqual(rerankerId("identity", "A"), rerankerId("raise-last", "A"));
	assert.equal(rerankerId("identity", "A"), rerankerId("identity", "B"));
});

test("原樣：名次照原順序", () => {
	assert.deepEqual(fakeOrder("identity", 4), [0, 1, 2, 3]);
});

// 核心只讓它到第 2 位（ADR 0005）；這裡要的是「排最前」，上限由核心擋。
test("最後一條提前：最後一條排最前，其餘照原順序", () => {
	assert.deepEqual(fakeOrder("raise-last", 4), [3, 0, 1, 2]);
});

test("只有一條路徑時照原順序", () => {
	assert.deepEqual(fakeOrder("raise-last", 1), [0]);
});

test("選單的值轉成模式，不認得的當關", () => {
	assert.equal(rerankMode("neural"), "neural");
	assert.equal(rerankMode("identity"), "identity");
	assert.equal(rerankMode("raise-last"), "raise-last");
	assert.equal(rerankMode("raise-fifth"), "off");
	assert.equal(rerankMode("off"), "off");
	assert.equal(rerankMode("cloud"), "off");
});

/**
 * 假的核心：`generation` 由測試改（模擬按鍵）；`applyRerank` 只收目前的 `generation`，名次變了就加一。
 * 只做 `reorderer` 用到的兩個方法。
 */
class FakeSession {
	generation = 1n;
	applied: bigint[] = [];

	rerankRequest() {
		return {
			generation: this.generation,
			// 測試從前文讀出這是哪一次的要求。
			context: `g${this.generation}`,
			paths: ["甲", "乙"],
			scores: [0, -1],
			free() {},
		};
	}

	applyRerank(generation: bigint) {
		if (generation !== this.generation) {
			return undefined;
		}
		this.applied.push(generation);
		this.generation += 1n;
		return { generation: this.generation, free() {} };
	}
}

/** 模型：收到的要求記下來，由測試決定什麼時候回。 */
function heldModel() {
	const jobs: RerankJob[] = [];
	const answers: ((ranked: Ranked) => void)[] = [];
	const rank: Rank = (job) => {
		jobs.push(job);
		return new Promise((resolve) => answers.push(resolve));
	};
	const answer = (i: number) =>
		answers[i]({ order: Uint32Array.from([1, 0]), ms: 1 });
	return { jobs, rank, answer };
}

function setUp(model: Rank, mode: () => "neural" | "off" = () => "neural") {
	const session = new FakeSession();
	const outcomes: Outcome[] = [];
	const shown: bigint[] = [];
	// 假核心只做用到的兩個方法，型別對不上 wasm 的 `Session`。
	const request = rerankScheduler(session as unknown as Session, {
		mode,
		models: { neural: model, identity: model, "raise-last": model },
		show: (snapshot: Snapshot) => shown.push(snapshot.generation),
		onResult: (outcome) => outcomes.push(outcome),
	});
	return { session, outcomes, shown, request };
}

/** 讓已經完成的 Promise 接續跑完（`setImmediate` 沒被假計時器接管）。 */
const settle = () => new Promise((resolve) => setImmediate(resolve));

beforeEach(() => mock.timers.enable({ apis: ["setTimeout"] }));
afterEach(() => mock.timers.reset());

test("停鍵 STOP_MS 後才要求；這之前再按鍵就重新計時", async () => {
	const model = heldModel();
	const { session, request } = setUp(model.rank);
	request(session.generation);
	mock.timers.tick(STOP_MS - 1);
	session.generation = 2n;
	request(session.generation);
	mock.timers.tick(STOP_MS - 1);
	assert.equal(model.jobs.length, 0);
	mock.timers.tick(1);
	assert.deepEqual(
		model.jobs.map((j) => j.context),
		["g2"],
	);
});

test("名次變了才重畫；同一個 generation 不再要求", async () => {
	const model = heldModel();
	const { session, outcomes, shown, request } = setUp(model.rank);
	request(session.generation);
	mock.timers.tick(STOP_MS);
	model.answer(0);
	await settle();
	assert.deepEqual(outcomes, ["applied"]);
	assert.deepEqual(shown, [2n]);
	// 套用後的快照帶新的 generation，殼畫完會再呼叫一次：不再要求。
	request(session.generation);
	mock.timers.tick(STOP_MS);
	assert.equal(model.jobs.length, 1);
});

test("模型在算時又停鍵：同時只算一個，回來後（舊的過期）再要求最新的", async () => {
	const model = heldModel();
	const { session, outcomes, shown, request } = setUp(model.rank);
	request(session.generation);
	mock.timers.tick(STOP_MS);
	session.generation = 5n;
	request(session.generation);
	mock.timers.tick(STOP_MS);
	assert.equal(model.jobs.length, 1, "前一個還在算，先不送");
	model.answer(0);
	await settle();
	assert.deepEqual(outcomes, ["stale"]);
	assert.deepEqual(shown, []);
	assert.deepEqual(
		model.jobs.map((j) => j.context),
		["g1", "g5"],
	);
	model.answer(1);
	await settle();
	assert.deepEqual(outcomes, ["stale", "applied"]);
	assert.deepEqual(session.applied, [5n]);
});

test("關著時不要求；等待中關掉的結果不套用", async () => {
	const model = heldModel();
	let mode: "neural" | "off" = "off";
	const { session, outcomes, request } = setUp(model.rank, () => mode);
	request(session.generation);
	mock.timers.tick(STOP_MS);
	assert.equal(model.jobs.length, 0);

	mode = "neural";
	session.generation = 2n;
	request(session.generation);
	mock.timers.tick(STOP_MS);
	mode = "off";
	model.answer(0);
	await settle();
	assert.deepEqual(outcomes, ["none"]);
	assert.deepEqual(session.applied, []);
});

test("沒有模型（回 undefined）時不套用", async () => {
	const { session, outcomes, request } = setUp(async () => undefined);
	request(session.generation);
	mock.timers.tick(STOP_MS);
	await settle();
	assert.deepEqual(outcomes, ["none"]);
	assert.deepEqual(session.applied, []);
});

test("模型丟錯：記成沒有結果，之後的停鍵照常要求", async () => {
	let calls = 0;
	const failing: Rank = async () => {
		calls++;
		throw new Error("Worker 壞了");
	};
	const { session, outcomes, request } = setUp(failing);
	const quiet = mock.method(console, "error", () => {});
	request(session.generation);
	mock.timers.tick(STOP_MS);
	await settle();
	session.generation = 2n;
	request(session.generation);
	mock.timers.tick(STOP_MS);
	await settle();
	quiet.mock.restore();
	assert.deepEqual(outcomes, ["none", "none"]);
	assert.equal(calls, 2);
});

test("套用名次時丟錯（例如 show）：不出 unhandled rejection，記成沒有結果一次，busy 歸位", async () => {
	const model = heldModel();
	const session = new FakeSession();
	const outcomes: Outcome[] = [];
	let broken = true;
	const request = rerankScheduler(session as unknown as Session, {
		mode: () => "neural",
		models: {
			neural: model.rank,
			identity: model.rank,
			"raise-last": model.rank,
		},
		show: () => {
			if (broken) {
				throw new Error("畫面壞了");
			}
		},
		onResult: (outcome) => outcomes.push(outcome),
	});
	const unhandled: unknown[] = [];
	const onUnhandled = (reason: unknown) => unhandled.push(reason);
	process.on("unhandledRejection", onUnhandled);
	const quiet = mock.method(console, "error", () => {});
	try {
		request(session.generation);
		mock.timers.tick(STOP_MS);
		model.answer(0);
		await settle();
		await settle();
		assert.deepEqual(unhandled, []);
		assert.deepEqual(outcomes, ["none"]);

		// busy 歸位：之後的停鍵照常要求。
		broken = false;
		session.generation = 9n;
		request(session.generation);
		mock.timers.tick(STOP_MS);
		assert.equal(model.jobs.length, 2);
	} finally {
		quiet.mock.restore();
		process.off("unhandledRejection", onUnhandled);
	}
});

test("交給模型時回報要求的 generation 與內容（debug 面板）；沒送出的不回報", async () => {
	const model = heldModel();
	const session = new FakeSession();
	const requests: [bigint, string][] = [];
	const request = rerankScheduler(session as unknown as Session, {
		mode: () => "neural",
		models: {
			neural: model.rank,
			identity: model.rank,
			"raise-last": model.rank,
		},
		show: () => {},
		onRequest: (generation, job) => requests.push([generation, job.context]),
	});
	request(session.generation);
	mock.timers.tick(STOP_MS);
	// 模型還在算時又停鍵：先不送，也不回報。
	session.generation = 2n;
	request(session.generation);
	mock.timers.tick(STOP_MS);
	assert.deepEqual(requests, [[1n, "g1"]]);
	model.answer(0);
	await settle();
	assert.deepEqual(requests, [
		[1n, "g1"],
		[2n, "g2"],
	]);
});
