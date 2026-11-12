import assert from "node:assert/strict";
import { test } from "node:test";
import {
	candidateSources,
	type InspectedCandidate,
	type InspectedSegment,
	type SegmentSource,
} from "../src/debug-panel.ts";

function segment(
	code: string,
	text: string,
	source: SegmentSource,
	learned = false,
	personalBigram = false,
): InspectedSegment {
	return { code, text, source, learned, personalBigram };
}

function word(segments: InspectedSegment[]): InspectedCandidate {
	return {
		text: segments.map((s) => s.text).join(""),
		kind: "word",
		segments,
		rerankInserted: false,
	};
}

test("單段的詞是它的來源", () => {
	const { codes, labels } = candidateSources(
		word([segment("anr", "門口", "lexicon")]),
	);
	assert.deepEqual(codes, ["lexicon"]);
	assert.deepEqual(labels, ["詞庫"]);
});

test("多段的是連打，各段的來源只列一次；學過的段與個人 bigram 另標", () => {
	const { codes } = candidateSources(
		word([
			segment("a", "日", "codeTable"),
			segment("b", "月", "codeTable", true),
			segment("cd", "丁天", "userWord", false, true),
		]),
	);
	assert.deepEqual(codes, [
		"sentence",
		"codeTable",
		"userWord",
		"learned",
		"personalBigram",
	]);
});

test("英文、emoji、記住的字串、聯想照種類；重排與排位規則另標", () => {
	const english: InspectedCandidate = {
		text: "the",
		kind: "english",
		segments: [],
		rule: 4,
		rerankInserted: false,
	};
	assert.deepEqual(candidateSources(english).codes, ["english", "rule4"]);
	const inserted: InspectedCandidate = {
		...word([segment("a", "日", "codeTable"), segment("b", "月", "codeTable")]),
		rerankPath: 8,
		rerankInserted: true,
	};
	const { codes, labels } = candidateSources(inserted);
	assert.deepEqual(codes, ["sentence", "codeTable", "rerankInserted"]);
	assert.equal(labels[2], "重排插入（第 9 條路徑）");
	const moved = { ...inserted, rerankInserted: false };
	assert.deepEqual(candidateSources(moved).codes.at(-1), "rerankMoved");
});
