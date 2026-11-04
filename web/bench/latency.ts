// WebAssembly 的每鍵延遲（plan §5.2）：由 `qim-bench` 用 Node 跑，量法見 `docs/development.md`〈首選字基準〉。
//
// 用法：node bench/latency.ts <資料包路徑> [<詞庫資料包路徑> [<語言模型資料包路徑>]]，stdin 一行一個輸入碼。
// stdout 第一行是 runtime，之後每個輸入一行、各鍵的毫秒數以空白分隔。
// 用網頁同一份綁定（`web/src/wasm/`）；每鍵計時含網頁畫組字與候選要讀的欄位。

import { readFileSync } from "node:fs";
import { initSync, Session } from "../src/wasm/qim_wasm.js";

const [packPath, lexiconPath, languageModelPath, ...extra] =
	process.argv.slice(2);
if (packPath === undefined || extra.length > 0) {
	console.error(
		"用法：node bench/latency.ts <資料包路徑> [<詞庫資料包路徑> [<語言模型資料包路徑>]]，stdin 一行一個輸入碼",
	);
	process.exit(2);
}

initSync({
	module: readFileSync(
		new URL("../src/wasm/qim_wasm_bg.wasm", import.meta.url),
	),
});
const session = new Session(
	readFileSync(packPath),
	lexiconPath === undefined ? undefined : readFileSync(lexiconPath),
	languageModelPath === undefined ? undefined : readFileSync(languageModelPath),
);
const inputs = readFileSync(0, "utf8")
	.split("\n")
	.filter((line) => line.length > 0);

const lines = [`Node ${process.version}`];
for (const input of inputs) {
	const times: string[] = [];
	for (const c of input) {
		const start = performance.now();
		const snapshot = session.letter(c);
		for (const segment of snapshot.segments) {
			segment.free();
		}
		void snapshot.focus;
		void snapshot.candidates;
		times.push(String(performance.now() - start));
		snapshot.free();
	}
	lines.push(times.join(" "));
	// 下一個輸入從空的組字打起；Esc 不計時。
	session.escape().free();
}
process.stdout.write(`${lines.join("\n")}\n`);
