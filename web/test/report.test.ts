import assert from "node:assert/strict";
import { test } from "node:test";
import { fixedLine, initialReport, observe } from "../src/report.ts";

/** 網頁從 `Snapshot` 讀的欄位；`chosen` 是這個鍵是不是選字（空白、數字鍵）。 */
function snap(
	input: string,
	candidates: string[],
	extra: { selected?: string; commit?: string; chosen?: boolean } = {},
) {
	return { input, candidates, selected: "", chosen: true, ...extra };
}

test("上屏時記下整串輸入碼、上屏的字與打完時的首選", () => {
	let state = initialReport;
	state = observe(state, snap("m", ["一"]));
	state = observe(state, snap("mi", ["丟"]));
	state = observe(state, snap("mir", ["歹口", "一台"]));
	// 選第 2 個，組字清空、上屏「一台」。
	state = observe(state, snap("", [], { commit: "一台" }));
	assert.deepEqual(state.last, { input: "mir", wanted: "一台", top: "歹口" });
	assert.equal(
		fixedLine(state.last ?? assert.fail()),
		"mir\t一台\t網頁回報（首選 歹口）",
	);
});

test("翻頁與選開頭一段不改打完時的首選", () => {
	let state = initialReport;
	state = observe(state, snap("anr", ["門口", "問"]));
	state = observe(state, snap("anr", ["第二頁"]));
	state = observe(state, snap("anr", ["口"], { selected: "門" }));
	state = observe(state, snap("", [], { commit: "門口" }));
	assert.equal(state.last?.top, "門口");
});

test("選了開頭一段後再打字，首選不明時來源不寫首選", () => {
	let state = initialReport;
	state = observe(state, snap("an", ["門"]));
	state = observe(state, snap("an", [], { selected: "門" }));
	state = observe(state, snap("anr", ["口"], { selected: "門" }));
	state = observe(state, snap("", [], { commit: "門口" }));
	assert.deepEqual(state.last, { input: "anr", wanted: "門口", top: "" });
	assert.equal(fixedLine(state.last ?? assert.fail()), "anr\t門口\t網頁回報");
});

test("沒有組字時的上屏（例如標點）不算一次組字", () => {
	let state = initialReport;
	state = observe(state, snap("", [], { commit: "，", chosen: false }));
	assert.equal(state.last, undefined);
	state = observe(state, snap("a", ["日"]));
	state = observe(state, snap("", [], { commit: "日" }));
	state = observe(state, snap("", [], { commit: "，", chosen: false }));
	assert.equal(state.last?.wanted, "日", "之後的標點不蓋掉上一次組字");
});

test("不是選字的上屏（組字中按標點、Enter）不算，也清掉上一次", () => {
	let state = initialReport;
	state = observe(state, snap("a", ["日"]));
	state = observe(state, snap("", [], { commit: "日" }));
	// 組字中按「,」：上屏「日，」，裡面有標點，不是使用者要的首選。
	state = observe(state, snap("a", ["日"]));
	state = observe(state, snap("", [], { commit: "日，", chosen: false }));
	assert.equal(state.last, undefined);
	// Enter 上屏輸入碼。
	state = observe(state, snap("mir", ["一台"]));
	state = observe(state, snap("", [], { commit: "mir", chosen: false }));
	assert.equal(state.last, undefined);
});

test("Esc 清掉組字不算上屏", () => {
	let state = initialReport;
	state = observe(state, snap("a", ["日"]));
	state = observe(state, snap("", []));
	assert.equal(state.last, undefined);
});

test("要的字裡的 tab 與換行拿掉，一行才不會斷", () => {
	assert.equal(
		fixedLine({ input: "a", wanted: "日\t\n", top: "日" }),
		"a\t日\t網頁回報（首選 日）",
	);
});
