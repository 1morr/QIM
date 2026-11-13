/**
 * debug 面板（plan〈網頁 prototype〉，M4 票 01）：打字時看核心為什麼這樣排——殼給的前文、輸入碼的切段、各候選的來源、
 * 路徑分與學習加分、排位規則，與停鍵重排的要求和結果。預設關；開著時每次重畫後問核心 `Session.inspect()`，
 * 關著時不問，每鍵不多花時間（`data-inspect-count` 是問過幾次，e2e 守它）。重排的模型總分與毫秒是網頁這邊
 * 記的，核心只給候選是第幾條路徑放上來的。
 */

import type { Outcome, Ranked, RerankJob } from "./rerank.ts";
import type { Session, Inspection as WasmInspection } from "./wasm/qim_wasm.js";

/** 一段字詞從哪裡來（核心的 `SegmentSource`）。 */
export type SegmentSource = "codeTable" | "lexicon" | "userWord";
/** 候選的種類（核心的 `CandidateKind`）。 */
export type CandidateKind =
	| "word"
	| "english"
	| "emoji"
	| "englishUserWord"
	| "association";

/** 候選的一段（綁定的 `InspectedSegment` 讀成一般物件）。 */
export interface InspectedSegment {
	code: string;
	text: string;
	source: SegmentSource;
	learned: boolean;
	personalBigram: boolean;
	/** 碼有萬用鍵時實際對到的碼。 */
	matchedCode?: string;
}

/** 一個候選（綁定的 `InspectedCandidate` 讀成一般物件）。 */
export interface InspectedCandidate {
	text: string;
	kind: CandidateKind;
	segments: InspectedSegment[];
	score?: number;
	learning?: number;
	rule?: number;
	rerankPath?: number;
	rerankInserted: boolean;
	/** 鄰鍵讀法（M4 票 09c）換的每一碼：在剩下的輸入碼裡的位置（從 0 起）、打出的與換成的字母；不是鄰鍵讀法時是空的。 */
	neighborSwaps: { position: number; typed: string; code: string }[];
	/** 鄰鍵讀法的空間懲罰（已從路徑分扣掉）。 */
	neighborPenalty?: number;
}

/** 綁定的 `Inspection` 讀成一般物件：綁定的 getter 每次都複製一份，讀一次、釋放完再畫。 */
interface Inspection {
	contextState: "known" | "unknown" | "incognito";
	context: string;
	precedingKind: "clauseStart" | "char" | "unigram";
	precedingChar?: string;
	precedingWord?: string;
	segments: { code: string; text: string; selected: boolean }[];
	candidates: InspectedCandidate[];
	/** 最近一個鍵送了按鍵機率（觸控或實體鍵盤的字母鍵，M4 票 06、07）時各字母的機率，由高到低；其他鍵是空的。 */
	weights: { letter: string; probability: number }[];
	/** 候選列的停鍵重排來自核心的快取（M4 票 03）：沒等停鍵、沒跑模型。 */
	rerankCached: boolean;
}

/** 用完 `item` 就釋放（綁定的物件不會自己回收）。 */
function using<T extends { free(): void }, R>(item: T, use: (item: T) => R): R {
	try {
		return use(item);
	} finally {
		item.free();
	}
}

// 綁定的這些欄位是字串，值只有核心列舉的那幾種（`qim-wasm` 的 `From<Inspection>`）；這裡照型別收下。
/** 讀出 `inspection` 的內容，釋放它與裡面每個綁定的物件。 */
function read(inspection: WasmInspection): Inspection {
	return using(inspection, (i) => ({
		contextState: i.contextState as Inspection["contextState"],
		context: i.context,
		precedingKind: i.precedingKind as Inspection["precedingKind"],
		precedingChar: i.precedingChar,
		precedingWord: i.precedingWord,
		segments: i.segments.map((segment) =>
			using(segment, (s) => ({
				code: s.code,
				text: s.text,
				selected: s.selected,
			})),
		),
		candidates: i.candidates.map((candidate) =>
			using(candidate, (c) => ({
				text: c.text,
				kind: c.kind as CandidateKind,
				segments: c.segments.map((segment) =>
					using(segment, (s) => ({
						code: s.code,
						text: s.text,
						source: s.source as SegmentSource,
						learned: s.learned,
						personalBigram: s.personalBigram,
						matchedCode: s.matchedCode,
					})),
				),
				score: c.score,
				learning: c.learning,
				rule: c.rule,
				rerankPath: c.rerankPath,
				rerankInserted: c.rerankInserted,
				neighborSwaps: Array.from(c.neighborPositions, (position, k) => ({
					position,
					typed: c.neighborTyped[k],
					code: c.neighborCodes[k],
				})),
				neighborPenalty: c.neighborPenalty,
			})),
		),
		weights: i.weightLetters.map((letter, k) => ({
			letter,
			probability: i.weightProbabilities[k],
		})),
		rerankCached: i.rerankCached,
	}));
}

export interface DebugPanel {
	/** 組字、候選或前文可能變了：開著時重問核心再畫。 */
	refresh(): void;
	/** 停鍵重排的要求交給模型了。 */
	rerankRequested(generation: bigint, job: RerankJob): void;
	/** 重排的結果回來了（套用、名次沒變、過期作廢或沒有模型）。 */
	rerankResult(outcome: Outcome, ranked: Ranked | undefined): void;
}

/** 一次停鍵重排：要求與（回來後的）結果。 */
interface RerankRecord {
	generation: bigint;
	job: RerankJob;
	outcome?: Outcome;
	ranked?: Ranked;
	/** 套用時核心放到第 2 位起的那條路徑：結果回來當下讀，之後打字候選換了也照舊顯示。 */
	chosen?: { path: number; text: string; inserted: boolean };
	/** 結果剛回來、下次畫的時候要讀 `chosen`。 */
	readChosen?: boolean;
}

const OUTCOME_LABELS: Record<Outcome, string> = {
	applied: "已套用",
	unchanged: "名次沒變（模型排出的路徑就是首選，或已在第 2 位或之前）",
	stale: "作廢（要求之後候選變了）",
	none: "沒有結果（沒有模型或已關掉重排）",
};

const SOURCE_LABELS: Record<SegmentSource, string> = {
	codeTable: "碼表",
	lexicon: "詞庫",
	userWord: "使用者詞",
};

const KIND_LABELS: Record<Exclude<CandidateKind, "word">, string> = {
	english: "英文",
	emoji: "emoji",
	englishUserWord: "英文使用者詞",
	association: "聯想",
};

/** 分數到小數兩位；沒有時是「—」。 */
function score(value: number | undefined): string {
	return value === undefined ? "—" : value.toFixed(2);
}

/** 建一個元素，填上文字與屬性。 */
function make<K extends keyof HTMLElementTagNameMap>(
	tag: K,
	text = "",
	attributes: Record<string, string> = {},
): HTMLElementTagNameMap[K] {
	const node = document.createElement(tag);
	node.textContent = text;
	for (const [name, value] of Object.entries(attributes)) {
		node.setAttribute(name, value);
	}
	return node;
}

/** 有標題列的表格：回傳表格與放資料列的 `tbody`。 */
function table(
	label: string,
	titles: string[],
): { table: HTMLTableElement; body: HTMLTableSectionElement } {
	const node = make("table", "", { "aria-label": label });
	const head = make("tr");
	for (const title of titles) {
		head.append(make("th", title));
	}
	node.append(make("thead"));
	node.tHead?.append(head);
	const body = make("tbody");
	node.append(body);
	return { table: node, body };
}

/** 候選表「切段」欄的一段：`碼→字詞`；有萬用鍵時碼後面括號是實際對到的碼（`hz（ho）→的`）。 */
export function segmentText(segment: InspectedSegment): string {
	const matched =
		segment.matchedCode === undefined ? "" : `（${segment.matchedCode}）`;
	return `${segment.code}${matched}→${segment.text}`;
}

/**
 * 一個候選的來源標記：機器讀的代號（e2e 比對）與中文標籤。字詞是各段的來源（碼表、詞庫、使用者詞），
 * 多段的另標連打，學過的段標學習計數、接得上前一個字詞的標個人 bigram；其餘是候選的種類。
 */
export function candidateSources(candidate: InspectedCandidate): {
	codes: string[];
	labels: string[];
} {
	const codes: string[] = [];
	const labels: string[] = [];
	const add = (code: string, label: string): void => {
		if (!codes.includes(code)) {
			codes.push(code);
			labels.push(label);
		}
	};
	if (candidate.kind === "word") {
		const segments = candidate.segments;
		if (segments.length > 1) {
			add("sentence", "連打");
		}
		for (const segment of segments) {
			add(segment.source, SOURCE_LABELS[segment.source]);
		}
		if (segments.some((s) => s.learned)) {
			add("learned", "學習計數");
		}
		if (segments.some((s) => s.personalBigram)) {
			add("personalBigram", "個人 bigram");
		}
		if (segments.some((s) => s.matchedCode !== undefined)) {
			add("wildcard", "萬用鍵");
		}
	} else {
		add(candidate.kind, KIND_LABELS[candidate.kind]);
	}
	if (candidate.neighborSwaps.length > 0) {
		const swaps = candidate.neighborSwaps
			.map((s) => `第 ${s.position + 1} 碼 ${s.typed}→${s.code}`)
			.join("、");
		const penalty = candidate.neighborPenalty?.toFixed(2) ?? "";
		add("neighbor", `鄰鍵（${swaps}，懲罰 ${penalty}）`);
	}
	if (candidate.rerankPath !== undefined) {
		const inserted = candidate.rerankInserted;
		add(
			inserted ? "rerankInserted" : "rerankMoved",
			`重排${inserted ? "插入" : "移上"}（第 ${candidate.rerankPath + 1} 條路徑）`,
		);
	}
	if (candidate.rule !== undefined) {
		add(`rule${candidate.rule}`, `排位規則 ${candidate.rule}`);
	}
	return { codes, labels };
}

/**
 * 掛上面板：`toggle` 開關它（預設關），`root` 放內容，`session` 開著時才問。
 */
export function mountDebugPanel(
	root: HTMLElement,
	toggle: HTMLButtonElement,
	session: Session,
): DebugPanel {
	let open = false;
	let inspections = 0;
	let rerank: RerankRecord | undefined;
	/** 交給模型的要求有幾次（`data-rerank-requests`）；面板關著也記，e2e 讀它看快取有沒有省下一次。 */
	let requests = 0;
	root.dataset.inspectCount = "0";
	root.dataset.rerankRequests = "0";

	const contextLine = (inspection: Inspection): HTMLElement => {
		const state = inspection.contextState;
		const text =
			state === "known"
				? `前文：「${inspection.context}」`
				: state === "incognito"
					? "無痕：不讀前文"
					: "不知道前文";
		const preceding =
			inspection.precedingKind === "clauseStart"
				? "句首"
				: inspection.precedingKind === "char"
					? `「${inspection.precedingChar}」`
					: "unigram（不接前一字）";
		const line = make("p", "", {
			"data-testid": "debug-context",
			"data-state": state,
		});
		line.append(
			make("span", text, { "data-testid": "debug-context-text" }),
			make("span", `　開頭接：${preceding}`, {
				"data-testid": "debug-preceding",
				"data-kind": inspection.precedingKind,
			}),
			make(
				"span",
				`　前一個字詞：${inspection.precedingWord ? `「${inspection.precedingWord}」` : "沒有"}`,
				{ "data-testid": "debug-preceding-word" },
			),
		);
		return line;
	};

	const segmentsLine = (inspection: Inspection): HTMLElement => {
		const line = make("p", "切段：", { "data-testid": "debug-segments" });
		const segments = inspection.segments;
		if (segments.length === 0) {
			line.append("（沒有組字）");
		}
		for (const segment of segments) {
			line.append(
				make("code", `${segment.code}→${segment.text || "？"}`, {
					"data-selected": String(segment.selected),
				}),
				" ",
			);
		}
		return line;
	};

	// 按鍵機率：觸控的空間模型或實體鍵盤的鄰鍵先驗算出的各字母機率；核心只用第一個。
	const weightsLine = (inspection: Inspection): HTMLElement => {
		const line = make("p", "按鍵機率：", { "data-testid": "debug-weights" });
		if (inspection.weights.length === 0) {
			line.append("（最近一個鍵沒有送機率）");
		}
		for (const { letter, probability } of inspection.weights) {
			line.append(
				make("code", `${letter} ${(probability * 100).toFixed(1)}%`, {
					"data-letter": letter,
					"data-probability": String(probability),
				}),
				" ",
			);
		}
		return line;
	};

	const topLine = (inspection: Inspection): HTMLElement => {
		const top = inspection.candidates[0];
		const line = make("p", "", { "data-testid": "debug-top" });
		if (!top) {
			line.textContent = "首選：（沒有候選）";
			return line;
		}
		line.dataset.score = score(top.score);
		line.dataset.learning = score(top.learning);
		line.textContent = `首選「${top.text}」　路徑分 ${score(top.score)}　學習加分 ${score(top.learning)}${
			top.rule === undefined ? "" : `　排位規則 ${top.rule}`
		}`;
		return line;
	};

	const candidateTable = (inspection: Inspection): HTMLElement => {
		const { table: list, body } = table("候選來源", [
			"#",
			"候選",
			"來源",
			"路徑分",
			"學習加分",
			"切段",
		]);
		inspection.candidates.forEach((candidate, i) => {
			const { codes, labels } = candidateSources(candidate);
			const row = make("tr", "", {
				"data-text": candidate.text,
				"data-kind": candidate.kind,
				"data-sources": codes.join(" "),
				"data-learning": score(candidate.learning),
			});
			row.append(
				make("td", String(i + 1)),
				make("td", candidate.text),
				make("td", labels.join("、")),
				make("td", score(candidate.score)),
				make("td", score(candidate.learning)),
				make("td", candidate.segments.map(segmentText).join(" ")),
			);
			body.append(row);
		});
		return list;
	};

	const rerankSection = (inspection: Inspection): HTMLElement => {
		const section = make("div", "", { "data-testid": "debug-rerank" });
		const cached = inspection.rerankCached;
		section.append(
			make(
				"p",
				`停鍵結果快取：${cached ? "這份候選列用快取的結果排（沒等停鍵、沒跑模型）" : "沒用"}　模型跑過 ${requests} 次`,
				{ "data-testid": "debug-rerank-cache", "data-cached": String(cached) },
			),
		);
		if (!rerank) {
			section.append(make("p", "停鍵重排：還沒有要求"));
			return section;
		}
		const { generation, job, outcome, ranked, chosen } = rerank;
		section.append(
			make(
				"p",
				`停鍵重排的要求（generation ${generation}）　前文：「${job.context}」`,
				{
					"data-testid": "debug-rerank-request",
					"data-generation": String(generation),
				},
			),
		);
		const { table: paths, body } = table("重排的路徑", [
			"#",
			"路徑",
			"路徑分",
			"模型總分",
			"模型名次",
		]);
		const rank = ranked ? Array.from(ranked.order) : [];
		job.paths.forEach((path, i) => {
			const row = make("tr", "", { "data-path": path });
			const place = rank.indexOf(i);
			row.append(
				make("td", String(i + 1)),
				make("td", path),
				make("td", score(job.scores[i])),
				make("td", score(ranked?.totals?.[i])),
				make("td", place < 0 ? "—" : String(place + 1)),
			);
			body.append(row);
		});
		section.append(paths);
		const result = make("p", "", { "data-testid": "debug-rerank-result" });
		if (outcome === undefined) {
			result.textContent = "結果：模型算中…";
		} else {
			result.dataset.outcome = outcome;
			const picked = chosen
				? `　選了第 ${chosen.path + 1} 條「${chosen.text}」（${chosen.inserted ? "插入" : "移上"}第 2 位起）`
				: "";
			const ms = ranked ? `　${ranked.ms.toFixed(1)} ms` : "";
			result.textContent = `結果：${OUTCOME_LABELS[outcome]}${ms}${picked}`;
		}
		section.append(result);
		return section;
	};

	const draw = (): void => {
		if (!open) {
			return;
		}
		const inspection = read(session.inspect());
		inspections += 1;
		root.dataset.inspectCount = String(inspections);
		if (rerank?.readChosen) {
			rerank.readChosen = false;
			const placed = inspection.candidates.find(
				(c) => c.rerankPath !== undefined,
			);
			if (placed?.rerankPath !== undefined) {
				rerank.chosen = {
					path: placed.rerankPath,
					text: placed.text,
					inserted: placed.rerankInserted,
				};
			}
		}
		root.replaceChildren(
			contextLine(inspection),
			segmentsLine(inspection),
			weightsLine(inspection),
			topLine(inspection),
			candidateTable(inspection),
			rerankSection(inspection),
		);
	};

	const setOpen = (on: boolean): void => {
		open = on;
		root.hidden = !on;
		toggle.setAttribute("aria-pressed", String(on));
		if (on) {
			draw();
		} else {
			root.replaceChildren();
		}
	};
	setOpen(false);
	toggle.addEventListener("click", () => setOpen(!open));

	return {
		refresh: draw,
		rerankRequested(generation, job) {
			rerank = { generation, job };
			requests += 1;
			root.dataset.rerankRequests = String(requests);
			draw();
		},
		rerankResult(outcome, ranked) {
			if (rerank) {
				rerank.outcome = outcome;
				rerank.ranked = ranked;
				// 套用時核心剛把路徑放上去，這次畫的時候讀；面板關著時不問核心，就不記。
				rerank.readChosen = open && outcome === "applied";
			}
			draw();
		},
	};
}
