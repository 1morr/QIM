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
	return { id, kind, version: "1.0.0", size: 10, license: "CC0-1.0", builtin };
}

const BUILTIN = [
	entry("quick", "scheme", true),
	entry("quick-lexicon", "lexicon", true),
	entry("cldr-emoji-yue-hant", "emoji", true),
];

test("沒有匯入時列出內建的，每種用內建那份", () => {
	const listed = listPacks(BUILTIN, []);
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
	const listed = listPacks(BUILTIN, [entry("e2e-lexicon", "lexicon", false)]);
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
	const newer = {
		...entry("quick-lexicon", "lexicon", false),
		version: "1.1.0",
	};
	const listed = listPacks(BUILTIN, [newer]);
	assert.deepEqual(
		listed.map((p) => [p.id, p.version, p.builtin]),
		[
			["quick", "1.0.0", true],
			["quick-lexicon", "1.1.0", false],
			["cldr-emoji-yue-hant", "1.0.0", true],
		],
	);
});

test("選的那份不在了（刪掉）或種類不對時用那種的預設", () => {
	const listed = listPacks(BUILTIN, []);
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
	assert.deepEqual(parseManifest(json, false), {
		id: "e2e-lexicon",
		kind: "lexicon",
		version: "1.0.0",
		size: 39,
		license: "CC0-1.0",
		builtin: false,
	});
	assert.throws(() => parseManifest("{}", false), /manifest/);
	assert.throws(() => parseManifest("not json", false));
});

test("匯入和已有的資料包同 id 但種類不同時拒絕，同種的可以取代", () => {
	const listed = listPacks(BUILTIN, []);
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
