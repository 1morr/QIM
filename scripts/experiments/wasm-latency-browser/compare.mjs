// 同一份綁定與工作量，在 Playwright 的 Chromium（無頭）裡量每鍵延遲，和 `cargo xtask bench` 的 Node 數字對照。
// 用法見同目錄 README.md。

import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";

const root = new URL("../../../", import.meta.url);
const require = createRequire(new URL("web/package.json", root));
const { chromium } = require("@playwright/test");

// 與 qim-bench 同序：固定題、bench/corpus/*.tsv、bench/colloquial/*.tsv（各目錄依檔名）。
function inputs() {
	const out = [];
	for (const line of readFileSync(new URL("bench/fixed.tsv", root), "utf8").split("\n")) {
		if (line.trim() && !line.startsWith("#")) out.push(line.split("\t")[0]);
	}
	for (const dir of ["bench/corpus/", "bench/colloquial/"]) {
		for (const name of readdirSync(new URL(dir, root)).filter((n) => n.endsWith(".tsv")).sort()) {
			for (const line of readFileSync(new URL(dir + name, root), "utf8").split("\n")) {
				if (line.trim()) out.push(line.split("\t")[0].replaceAll(" ", ""));
			}
		}
	}
	return out;
}

function percentile(values, p) {
	const sorted = [...values].sort((a, b) => a - b);
	return sorted[Math.max(1, Math.ceil((sorted.length * p) / 100)) - 1];
}

// 基準載入的四份資料包（qim-bench 的 load_engine），依 `Session` 建構子的參數順序；簡繁不載。
const PACKS = [
	"build/packs/quick/quick.pack",
	"build/packs/quick-lexicon/quick-lexicon.pack",
	"build/packs/zh-hant-bigram/zh-hant-bigram.pack",
	"build/packs/scowl-en-us/scowl-en-us.pack",
];
const files = {
	"/qim_wasm.js": ["web/src/wasm/qim_wasm.js", "text/javascript"],
	"/qim_wasm_bg.wasm": ["web/src/wasm/qim_wasm_bg.wasm", "application/wasm"],
	"/": [null, "text/html"],
};
for (const [i, path] of PACKS.entries()) files[`/pack${i}`] = [path, "application/octet-stream"];

// 同 web/bench/latency.ts（qim-bench 的 WebAssembly 計時）：每題逐鍵打，每鍵計 inputChar 加讀出網頁畫
// 組字與候選的欄位；打完有兩段以上時按 ← 到第一段（不計時）再選它的首選，計那一次；每題之間 Esc。
// 在瀏覽器與 Node 各跑一次，所以寫成字串、兩邊都 new Function 出來。
const WORKLOAD = `
	const read = (snapshot) => {
		const segments = snapshot.segments;
		for (const segment of segments) { void segment.radicals; segment.free(); }
		void snapshot.focus;
		void snapshot.candidates;
		return segments.length;
	};
	const begin = performance.now();
	const perKey = [];
	const selects = [];
	for (const input of inputs) {
		const times = [];
		let segments = 0;
		for (const c of input) {
			const start = performance.now();
			const snapshot = session.inputChar(c);
			segments = read(snapshot);
			times.push(performance.now() - start);
			snapshot.free();
		}
		perKey.push(times);
		if (segments >= 2) {
			for (let i = 0; i < segments; i++) session.press("left").free();
			const start = performance.now();
			const snapshot = session.select(0);
			read(snapshot);
			selects.push(performance.now() - start);
			snapshot.free();
		}
		session.press("escape").free();
	}
	return { perKey, selects, total: performance.now() - begin };
`;

const browser = await chromium.launch();
const page = await browser.newPage();
await page.route("http://localhost:5199/**", (route) => {
	const [path, type] = files[new URL(route.request().url()).pathname];
	route.fulfill({
		body: path ? readFileSync(new URL(path, root)) : "<!doctype html>",
		contentType: type,
		// crossOriginIsolated 才有 5 µs 的 performance.now()，否則 Chromium 粗化到 100 µs。
		headers: {
			"Cross-Origin-Opener-Policy": "same-origin",
			"Cross-Origin-Embedder-Policy": "require-corp",
		},
	});
});
await page.goto("http://localhost:5199/");
const all = inputs();
const result = await page.evaluate(
	async ({ inputs, count, workload }) => {
		const { default: init, Session } = await import("/qim_wasm.js");
		await init({ module_or_path: "/qim_wasm_bg.wasm" });
		const packs = [];
		for (let i = 0; i < count; i++) {
			packs.push(new Uint8Array(await (await fetch(`/pack${i}`)).arrayBuffer()));
		}
		const session = new Session(packs[0], packs[1], packs[2], undefined, packs[3]);
		const run = new Function("session", "inputs", workload);
		return { isolated: crossOriginIsolated, ua: navigator.userAgent, ...run(session, inputs) };
	},
	{ inputs: all, count: PACKS.length, workload: WORKLOAD },
);
await browser.close();

// 同一個迴圈在 Node 跑一次：數字應與 `cargo xtask bench` 的 WebAssembly 列相近，整份工作量的總時間 ÷ 鍵數
// 不受計時解析度影響，兩邊比得起來。
const { initSync, Session } = await import(new URL("web/src/wasm/qim_wasm.js", root).href);
initSync({ module: readFileSync(new URL("web/src/wasm/qim_wasm_bg.wasm", root)) });
const [scheme, lexicon, model, english] = PACKS.map((path) => readFileSync(new URL(path, root)));
const node = new Function("session", "inputs", WORKLOAD)(
	new Session(scheme, lexicon, model, undefined, english),
	all,
);
const keys = all.reduce((n, input) => n + input.length, 0);

console.log(`${result.ua}；crossOriginIsolated ${result.isolated}`);
console.log(
	`平均每鍵（含選段、Esc 與計時本身）：Chromium ${((result.total / keys) * 1000).toFixed(2)} µs、Node ${process.version} ${((node.total / keys) * 1000).toFixed(2)} µs`,
);
console.log("| 環境 | 輸入 | 輸入數 | 鍵數 | p50 µs | p95 µs |");
console.log("| --- | --- | ---: | ---: | ---: | ---: |");
for (const [environment, run] of [
	["Chromium", result],
	[`Node ${process.version}`, node],
]) {
	// 與 qim-bench 同口徑：「第 20 碼起」只算每個輸入的第 20 鍵起。
	const long = run.perKey.filter((times) => times.length >= 20);
	for (const [group, inputs, values] of [
		["全部", run.perKey.length, run.perKey.flat()],
		["第 20 碼起", long.length, long.flatMap((times) => times.slice(19))],
		["選開頭一段", run.selects.length, run.selects],
	]) {
		const us = (p) => (percentile(values, p) * 1000).toFixed(2);
		console.log(`| ${environment} | ${group} | ${inputs} | ${values.length} | ${us(50)} | ${us(95)} |`);
	}
}
