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

const files = {
	"/qim_wasm.js": ["web/src/wasm/qim_wasm.js", "text/javascript"],
	"/qim_wasm_bg.wasm": ["web/src/wasm/qim_wasm_bg.wasm", "application/wasm"],
	"/quick.pack": ["build/packs/quick/quick.pack", "application/octet-stream"],
	"/": [null, "text/html"],
};

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
const result = await page.evaluate(async (inputs) => {
	const { default: init, Session } = await import("/qim_wasm.js");
	await init({ module_or_path: "/qim_wasm_bg.wasm" });
	const pack = new Uint8Array(await (await fetch("/quick.pack")).arrayBuffer());
	const session = new Session(pack);
	const begin = performance.now();
	const perKey = inputs.map((input) => {
		const times = [];
		for (const c of input) {
			const start = performance.now();
			const snapshot = session.letter(c);
			void snapshot.input;
			void snapshot.candidates;
			times.push(performance.now() - start);
			snapshot.free();
		}
		session.escape().free();
		return times;
	});
	const total = performance.now() - begin;
	return { isolated: crossOriginIsolated, ua: navigator.userAgent, perKey, total };
}, all);
await browser.close();

// 同一個迴圈在 Node 跑一次，整份工作量的總時間 ÷ 鍵數不受計時解析度影響，兩邊比得起來。
const { initSync, Session } = await import(new URL("web/src/wasm/qim_wasm.js", root).href);
initSync({ module: readFileSync(new URL("web/src/wasm/qim_wasm_bg.wasm", root)) });
const session = new Session(readFileSync(new URL("build/packs/quick/quick.pack", root)));
const begin = performance.now();
const nodeTimes = [];
for (const input of all) {
	for (const c of input) {
		const start = performance.now();
		const snapshot = session.letter(c);
		void snapshot.input;
		void snapshot.candidates;
		nodeTimes.push(performance.now() - start);
		snapshot.free();
	}
	session.escape().free();
}
const nodeTotal = performance.now() - begin;
const keys = all.reduce((n, input) => n + input.length, 0);

console.log(`${result.ua}；crossOriginIsolated ${result.isolated}`);
console.log(
	`平均每鍵（含 Esc 與計時本身）：Chromium ${((result.total / keys) * 1000).toFixed(2)} µs、Node ${process.version} ${((nodeTotal / keys) * 1000).toFixed(2)} µs`,
);
console.log("| 輸入 | 輸入數 | 鍵數 | p50 µs | p95 µs |");
for (const [label, keep] of [
	["全部", () => true],
	["20 碼以上", (input) => input.length >= 20],
]) {
	const picked = result.perKey.filter((_, i) => keep(all[i]));
	const values = picked.flat();
	const us = (p) => (percentile(values, p) * 1000).toFixed(2);
	console.log(`| ${label} | ${picked.length} | ${values.length} | ${us(50)} | ${us(95)} |`);
}
