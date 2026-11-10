import assert from "node:assert/strict";
import { test } from "node:test";
import {
	activeIds,
	importConflict,
	listPacks,
	type PackEntry,
	parseManifest,
} from "../src/pack-choice.ts";

function entry(id: string, kind: string, builtin: boolean): PackEntry {
	const origin = builtin ? "builtin" : "imported";
	return { id, kind, version: "1.0.0", size: 10, license: "CC0-1.0", origin };
}

/** 測試用的「較新」：這裡的版本都是一位數，字串比就對（網頁用核心的 semver 比）。 */
const newer = (a: PackEntry, b: PackEntry) => a.version > b.version;

const BUILTIN = [
	entry("quick", "scheme", true),
	entry("quick-lexicon", "lexicon", true),
	entry("cldr-emoji-yue-hant", "emoji", true),
];

test("沒有匯入時列出內建的，每種用內建那份", () => {
	const listed = listPacks(BUILTIN, [], newer);
	assert.deepEqual(listed, BUILTIN);
	assert.deepEqual(
		activeIds(listed, {}),
		new Map([
			["scheme", "quick"],
			["lexicon", "quick-lexicon"],
			["emoji", "cldr-emoji-yue-hant"],
		]),
	);
});

test("匯入的同種資料包排在內建的後面，選了才用它", () => {
	const listed = listPacks(
		BUILTIN,
		[entry("e2e-lexicon", "lexicon", false)],
		newer,
	);
	assert.deepEqual(
		listed.map((p) => p.id),
		["quick", "quick-lexicon", "e2e-lexicon", "cldr-emoji-yue-hant"],
	);
	assert.equal(activeIds(listed, {}).get("lexicon"), "quick-lexicon");
	assert.equal(
		activeIds(listed, { lexicon: "e2e-lexicon" }).get("lexicon"),
		"e2e-lexicon",
	);
});

test("匯入和內建同 id 的取代內建那份", () => {
	const replacement = {
		...entry("quick-lexicon", "lexicon", false),
		version: "1.1.0",
	};
	const listed = listPacks(BUILTIN, [replacement], newer);
	assert.deepEqual(
		listed.map((p) => [p.id, p.version, p.origin]),
		[
			["quick", "1.0.0", "builtin"],
			["quick-lexicon", "1.1.0", "imported"],
			["cldr-emoji-yue-hant", "1.0.0", "builtin"],
		],
	);
});

test("選的那份不在了（刪掉）或種類不對時用那種的預設", () => {
	const listed = listPacks(BUILTIN, [], newer);
	const active = activeIds(listed, {
		lexicon: "e2e-lexicon",
		emoji: "quick-lexicon",
	});
	assert.equal(active.get("lexicon"), "quick-lexicon");
	assert.equal(active.get("emoji"), "cldr-emoji-yue-hant");
});

test("manifest 讀出列表要的欄位；不是 manifest 時丟錯", () => {
	const json = JSON.stringify({
		manifest_version: 1,
		id: "e2e-lexicon",
		kind: "lexicon",
		version: "1.0.0",
		min_core_version: "0.0.0",
		size: 39,
		sha256: "0".repeat(64),
		license: "CC0-1.0",
	});
	assert.deepEqual(parseManifest(json, "imported"), {
		id: "e2e-lexicon",
		kind: "lexicon",
		version: "1.0.0",
		size: 39,
		license: "CC0-1.0",
		origin: "imported",
	});
	assert.throws(() => parseManifest("{}", "imported"), /manifest/);
	assert.throws(() => parseManifest("not json", "imported"));
});

test("匯入和已有的資料包同 id 但種類不同時拒絕，同種的可以取代", () => {
	const listed = listPacks(BUILTIN, [], newer);
	assert.match(
		importConflict(listed, entry("quick", "lexicon", false)) ?? "",
		/quick.*方案/,
	);
	assert.equal(
		importConflict(listed, entry("quick-lexicon", "lexicon", false)),
		undefined,
	);
	assert.equal(
		importConflict(listed, entry("e2e-lexicon", "lexicon", false)),
		undefined,
	);
});

test("線上更新下載的同 id 資料包：版本較高才取代內建的，同版或較舊用內建的（不刪，只是不列）", () => {
	const downloaded = (version: string): PackEntry => ({
		...entry("quick-lexicon", "lexicon", false),
		origin: "downloaded",
		version,
	});
	const lexiconOf = (listed: PackEntry[]) =>
		listed
			.filter((p) => p.kind === "lexicon")
			.map((p) => [p.version, p.origin]);
	assert.deepEqual(
		lexiconOf(listPacks(BUILTIN, [downloaded("1.0.1")], newer)),
		[["1.0.1", "downloaded"]],
	);
	// 之後 App 內建了較新的 1.0.2（這裡把內建的改成 1.0.2）：用內建的。
	const upgraded = BUILTIN.map((p) =>
		p.id === "quick-lexicon" ? { ...p, version: "1.0.2" } : p,
	);
	assert.deepEqual(
		lexiconOf(listPacks(upgraded, [downloaded("1.0.1")], newer)),
		[["1.0.2", "builtin"]],
	);
	// 同版：內建的隨 App 發佈、已知可用，以它為準。
	assert.deepEqual(
		lexiconOf(listPacks(BUILTIN, [downloaded("1.0.0")], newer)),
		[["1.0.0", "builtin"]],
	);
	// 從檔案匯入的是使用者自己選的，照舊一律取代（票 20）。
	assert.deepEqual(
		lexiconOf(
			listPacks(upgraded, [entry("quick-lexicon", "lexicon", false)], newer),
		),
		[["1.0.0", "imported"]],
	);
});
