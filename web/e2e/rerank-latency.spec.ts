import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// 停鍵層模型在瀏覽器裡重排一次的延遲（M3 票 11e）：只報不擋，平常略過，設 `QIM_RERANK_LATENCY=1` 才跑
// （`docs/development.md`〈網頁〉）。兩種量法：
// - 合成：照 `cargo run --release -p qim-neural --example latency`，9 條 × 18 字、前文 50 字，字從字表前
//   3,000 字依同一個線性同餘序列取，熱身 20 次後量 500 次 `Reranker.rank`（主執行緒，同一份綁定）。
// - 實打：口語集 hkcancor 前 200 題的輸入碼照順序打，讀 Web Worker 回報的毫秒數（候選列的 `data-rerank-ms`）。

test.skip(!process.env.QIM_RERANK_LATENCY, "設 QIM_RERANK_LATENCY=1 才量");

const MODEL_ID = "zh-hant-neural-rerank";

function percentile(values: number[], p: number): number {
	const sorted = [...values].sort((a, b) => a - b);
	return sorted[
		Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1)
	];
}

function report(name: string, values: number[], extra = ""): void {
	console.log(
		`${name}：n=${values.length}，p50 ${percentile(values, 50).toFixed(1)} ms，p95 ${percentile(values, 95).toFixed(1)} ms，最大 ${Math.max(...values).toFixed(1)} ms${extra}`,
	);
}

test.beforeEach(async ({ page }) => {
	test.setTimeout(300_000);
	await page.goto("/");
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveAttribute(
		"data-ready",
		"true",
	);
});

test("合成：9 條 × 18 字、前文 50 字", async ({ page }) => {
	const vocab = readFileSync(
		new URL("../../data/neural-rerank/vocab.txt", import.meta.url),
		"utf8",
	)
		.split("\n")
		.slice(2)
		.filter((line) => line.length > 0)
		.map((line) => [...line][0]);
	const { load, times } = await page.evaluate(
		async ({ vocab, id }) => {
			// 由 Vite 開發伺服器提供的綁定，只在瀏覽器裡解析得到：網址放進變數，tsc 不去 Node 這邊找它
			// （結果的型別因此是 any）。
			const url = "/src/wasm/qim_wasm.js";
			const wasm = await import(url);
			const manifest = await (await fetch(`/packs/${id}/manifest.json`)).text();
			const bytes = new Uint8Array(
				await (await fetch(`/packs/${id}/${id}.pack`)).arrayBuffer(),
			);
			const pack = new wasm.Pack(manifest, bytes);
			const start = performance.now();
			const reranker = new wasm.Reranker(pack);
			const load = performance.now() - start;
			let seed = 11n;
			const mask = (1n << 64n) - 1n;
			const next = () => {
				seed = (seed * 6364136223846793005n + 1442695040888963407n) & mask;
				return vocab[Number(seed >> 33n) % Math.min(3000, vocab.length)];
			};
			const times: number[] = [];
			for (let i = 0; i < 520; i++) {
				const before = Array.from({ length: 50 }, next).join("");
				const paths = Array.from({ length: 9 }, () =>
					Array.from({ length: 18 }, next).join(""),
				);
				const scores = Float64Array.from(paths.map((_, k) => -k));
				const t = performance.now();
				reranker.rank(before, paths, scores);
				if (i >= 20) {
					times.push(performance.now() - t);
				}
			}
			return { load, times };
		},
		{ vocab, id: MODEL_ID },
	);
	report("合成", times, `；載入 ${load.toFixed(1)} ms`);
});

test("實打：口語集前 200 題", async ({ page }) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	const list = page.getByRole("list", { name: "候選" });
	const codes = readFileSync(
		new URL("../../bench/colloquial/hkcancor.tsv", import.meta.url),
		"utf8",
	)
		.split("\n")
		.filter((line) => line.length > 0)
		.slice(0, 200)
		.map((line) => line.split("\t")[0].replaceAll(" ", ""));
	const results = async () => {
		let total = 0;
		for (const key of ["applied", "unchanged", "stale"]) {
			total += Number((await list.getAttribute(`data-rerank-${key}`)) ?? 0);
		}
		return total;
	};
	await text.focus();
	const times: number[] = [];
	for (const code of codes) {
		const before = await results();
		await page.keyboard.type(code);
		await expect.poll(results, { timeout: 5000 }).toBeGreaterThan(before);
		times.push(Number(await list.getAttribute("data-rerank-ms")));
		await page.keyboard.press("Escape");
	}
	const lengths = codes.map((c) => c.length);
	report(
		"實打",
		times,
		`；輸入碼長度 p50 ${percentile(lengths, 50)}、p95 ${percentile(lengths, 95)}`,
	);
});
