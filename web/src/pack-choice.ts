// 資料包的清單與每種用哪份（plan §4.6）：內建的加上匯入的，設定頁列出、切換。純邏輯，不碰瀏覽器
// 的儲存與核心；manifest 的核對在核心（`Pack.verify`、交給 `Session` 時）。

/** 不交給 `Session`、由 `RadicalHints` 另外載入的種類。 */
export const RADICAL_HINT = "radical-hint";

/** 種類給人看的名字（manifest 的 `kind`，`qim_core::PackKind`）。 */
const KIND_NAMES: Readonly<Record<string, string>> = {
	scheme: "方案",
	lexicon: "詞庫",
	"language-model": "語言模型",
	[RADICAL_HINT]: "字根提示",
	simplifier: "簡繁",
	english: "英文",
	emoji: "emoji",
};

/** 種類給人看的名字；不認得的種類照 manifest 原樣顯示。 */
export function kindName(kind: string): string {
	return KIND_NAMES[kind] ?? kind;
}

/**
 * 資料包從哪來：隨網頁發佈的（`builtin`）、從檔案匯入的（`imported`）、線上更新下載的（`downloaded`）。
 * 後兩種存在 IndexedDB，可以刪。
 */
export type PackOrigin = "builtin" | "imported" | "downloaded";

/** 設定頁列出的一份資料包：manifest 裡給人看的欄位，加上它從哪來。 */
export interface PackEntry {
	id: string;
	/** manifest 的 `kind`（`lexicon`、`language-model`…）。 */
	kind: string;
	version: string;
	/** 資料包檔的位元組數。 */
	size: number;
	/** SPDX 運算式。 */
	license: string;
	origin: PackOrigin;
}

/** 讀出 manifest 裡列表要的欄位；不是 JSON 或少了欄位時丟錯（完整的核對在核心）。 */
export function parseManifest(json: string, origin: PackOrigin): PackEntry {
	const value: unknown = JSON.parse(json);
	const field = (name: string, type: "string" | "number") => {
		const found =
			typeof value === "object" && value !== null
				? (value as Record<string, unknown>)[name]
				: undefined;
		if (typeof found !== type) {
			throw new Error(`manifest 少了 ${name}`);
		}
		return found;
	};
	return {
		id: field("id", "string") as string,
		kind: field("kind", "string") as string,
		version: field("version", "string") as string,
		size: field("size", "number") as number,
		license: field("license", "string") as string,
		origin,
	};
}

/**
 * 列出的資料包：內建的照原順序，同種的匯入的接在那種後面；和內建同 id 的匯入（含下載的新版）取代
 * 內建那份（刪掉後回到內建的）。
 */
export function listPacks(
	builtin: readonly PackEntry[],
	imported: readonly PackEntry[],
): PackEntry[] {
	const replaced = new Map(imported.map((p) => [p.id, p]));
	const listed = builtin.map((p) => replaced.get(p.id) ?? p);
	for (const pack of imported) {
		if (listed.includes(pack)) {
			continue;
		}
		const lastOfKind = listed.findLastIndex((p) => p.kind === pack.kind);
		listed.splice(lastOfKind === -1 ? listed.length : lastOfKind + 1, 0, pack);
	}
	return listed;
}

/**
 * 每種用哪份 → id：`chosen`（種類 → id）選的那份還在、種類也對就用它，否則用清單裡那種的第一份
 * （內建的預設）。
 */
export function activeIds(
	listed: readonly PackEntry[],
	chosen: Readonly<Record<string, string>>,
): Map<string, string> {
	const active = new Map<string, string>();
	for (const pack of listed) {
		if (!active.has(pack.kind)) {
			active.set(pack.kind, pack.id);
		}
	}
	for (const [kind, id] of Object.entries(chosen)) {
		if (listed.some((p) => p.id === id && p.kind === kind)) {
			active.set(kind, id);
		}
	}
	return active;
}

/**
 * 匯入 `entry` 會不會撞上已有的資料包：同 id 不同種類時回傳說明（同 id 的匯入會取代那份，種類不同就
 * 換不了），沒撞上是 `undefined`。
 */
export function importConflict(
	listed: readonly PackEntry[],
	entry: PackEntry,
): string | undefined {
	const same = listed.find((p) => p.id === entry.id);
	return same && same.kind !== entry.kind
		? `id ${entry.id} 已經是${kindName(same.kind)}資料包，不能換成${kindName(entry.kind)}`
		: undefined;
}
