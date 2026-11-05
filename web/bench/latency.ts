// WebAssembly 的每鍵延遲（plan §5.2）：由 `qim-bench` 用 Node 跑，量法見 `docs/development.md`〈首選字基準〉。
//
// 用法：node bench/latency.ts <資料包路徑>…，stdin 一行一個輸入碼。資料包依檔頭的魔數分辨
// （`crates/qim-core/src/pack.rs`），順序不拘：速成方案一份必給，詞庫、語言模型、英文各至多一份。
// stdout 第一行是 runtime，之後每個輸入一行、各鍵的毫秒數以空白分隔。
// 用網頁同一份綁定（`web/src/wasm/`）；每鍵計時含網頁畫組字與候選要讀的欄位。

import { readFileSync } from "node:fs";
import { initSync, Session } from "../src/wasm/qim_wasm.js";

const USAGE =
	"用法：node bench/latency.ts <資料包路徑>…（速成方案必給，詞庫、語言模型、英文選給），stdin 一行一個輸入碼";
/** 檔頭魔數 → 資料包種類。 */
const KINDS: Record<string, "pack" | "lexicon" | "languageModel" | "english"> =
	{ QIMP: "pack", QIML: "lexicon", QIMB: "languageModel", QIME: "english" };

const packs: Partial<Record<(typeof KINDS)[string], Uint8Array>> = {};
for (const path of process.argv.slice(2)) {
	const bytes = readFileSync(path);
	const kind = KINDS[bytes.subarray(0, 4).toString("latin1")];
	if (kind === undefined || packs[kind] !== undefined) {
		console.error(`${path}：不認得的資料包或同種給了兩份。${USAGE}`);
		process.exit(2);
	}
	packs[kind] = bytes;
}
if (packs.pack === undefined) {
	console.error(USAGE);
	process.exit(2);
}

initSync({
	module: readFileSync(
		new URL("../src/wasm/qim_wasm_bg.wasm", import.meta.url),
	),
});
const session = new Session(
	packs.pack,
	packs.lexicon,
	packs.languageModel,
	undefined,
	packs.english,
);
const inputs = readFileSync(0, "utf8")
	.split("\n")
	.filter((line) => line.length > 0);

const lines = [`Node ${process.version}`];
for (const input of inputs) {
	const times: string[] = [];
	for (const c of input) {
		const start = performance.now();
		// 輸入碼裡的分隔記號是實體鍵盤的 `'`，同 `qim-bench` 的原生計時。
		const snapshot = c === "'" ? session.punct(c) : session.letter(c);
		for (const segment of snapshot.segments) {
			void segment.radicals;
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
