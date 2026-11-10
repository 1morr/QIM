// 資料包庫（plan §4.6）：內建的從 `/packs/<id>/` fetch，匯入的存在 IndexedDB；每種用哪份存
// `localStorage`。交給核心的都是 manifest 加位元組（`Pack`），由核心核對。

import { createStore, del, entries, set } from "idb-keyval";
import {
	activeIds,
	importConflict,
	listPacks,
	type PackEntry,
	type PackOrigin,
	parseManifest,
	RADICAL_HINT,
} from "./pack-choice.ts";
import { Pack } from "./wasm/qim_wasm.js";

/** 隨網頁發佈的資料包，由 `cargo xtask web` 放到 `web/public/packs/<id>/`。 */
const BUILTIN_IDS = [
	"quick",
	"quick-lexicon",
	"zh-hant-bigram",
	"cangjie-hint",
	"opencc-hk2s",
	"scowl-en-us",
	"cldr-emoji-yue-hant",
] as const;
/** manifest 在資料包目錄裡的檔名（`qim_core::MANIFEST_FILE`）。 */
const MANIFEST_FILE = "manifest.json";
/** 每種用哪份：`{ 種類: id }`，沒選的種類用內建那份。 */
const ACTIVE_KEY = "qim.packs.active";
/** 匯入的資料包：IndexedDB `qim-packs` 庫的 `packs` 表，鍵是 id。 */
const store = createStore("qim-packs", "packs");

/** 一份資料包的原料：manifest 的 JSON 與資料包檔。存進 IndexedDB 的另記從哪來（舊的紀錄沒有，算匯入的）。 */
interface Stored {
	manifest: string;
	bytes: Uint8Array;
	origin?: PackOrigin;
}

interface Loaded extends Stored {
	entry: PackEntry;
}

/** `cargo xtask web` 放到 `web/public/` 的檔；沒有就叫人先跑它。 */
async function fetchOk(url: string): Promise<Response> {
	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`${url}：${response.status}，先跑 cargo xtask web`);
	}
	return response;
}

async function fetchBuiltin(id: string): Promise<Loaded> {
	const manifest = await (
		await fetchOk(`/packs/${id}/${MANIFEST_FILE}`)
	).text();
	const bytes = new Uint8Array(
		await (await fetchOk(`/packs/${id}/${id}.pack`)).arrayBuffer(),
	);
	return { manifest, bytes, entry: parseManifest(manifest, "builtin") };
}

function readChosen(): Record<string, string> {
	try {
		const value: unknown = JSON.parse(localStorage.getItem(ACTIVE_KEY) ?? "{}");
		return typeof value === "object" && value !== null
			? (value as Record<string, string>)
			: {};
	} catch {
		return {};
	}
}

/** 一份要給使用者看的資料包：列表的欄位加上它是不是正在用。 */
export interface ListedPack extends PackEntry {
	active: boolean;
}

export class PackLibrary {
	private builtin: Loaded[];
	private imported: Map<string, Loaded>;
	private chosen: Record<string, string>;

	private constructor(builtin: Loaded[], imported: Map<string, Loaded>) {
		this.builtin = builtin;
		this.imported = imported;
		this.chosen = readChosen();
	}

	/** 讀內建的與 IndexedDB 裡匯入的；IndexedDB 裡讀不懂的那筆略過。 */
	static async open(): Promise<PackLibrary> {
		const builtin = await Promise.all(BUILTIN_IDS.map(fetchBuiltin));
		const imported = new Map<string, Loaded>();
		for (const [id, value] of await entries<string, Stored>(store)) {
			try {
				const entry = parseManifest(value.manifest, value.origin ?? "imported");
				imported.set(id, { ...value, entry });
			} catch {
				// 舊版或壞掉的紀錄：不列出，也不刪，留著查。
			}
		}
		return new PackLibrary(builtin, imported);
	}

	private listedEntries(): PackEntry[] {
		return listPacks(
			this.builtin.map((p) => p.entry),
			[...this.imported.values()].map((p) => p.entry),
		);
	}

	/** 設定頁列出的資料包。 */
	list(): ListedPack[] {
		const listed = this.listedEntries();
		const active = activeIds(listed, this.chosen);
		return listed.map((p) => ({ ...p, active: active.get(p.kind) === p.id }));
	}

	/** 列出的每份資料包的 `manifest.json`：檢查線上更新時和更新索引比（被取代的內建那份不算）。 */
	installedManifests(): string[] {
		return this.listedEntries().flatMap((p) => this.find(p.id)?.manifest ?? []);
	}

	private find(id: string): Loaded | undefined {
		return this.imported.get(id) ?? this.builtin.find((p) => p.entry.id === id);
	}

	/** 正在用的 `kind` 那份，做成新的 `Pack`；交出去之後由收的一方負責（`Session` 會拿走）。 */
	private pack(kind: string): Pack | undefined {
		const id = activeIds(this.listedEntries(), this.chosen).get(kind);
		const found = id === undefined ? undefined : this.find(id);
		return found && new Pack(found.manifest, found.bytes);
	}

	/** 交給 `Session` 的資料包：每種正在用的那份，字根提示除外。 */
	sessionPacks(): Pack[] {
		const kinds = new Set(this.listedEntries().map((p) => p.kind));
		kinds.delete(RADICAL_HINT);
		return [...kinds].flatMap((kind) => this.pack(kind) ?? []);
	}

	/** 正在用的字根提示資料包；用完要 `free()`。 */
	radicalHintPack(): Pack {
		const pack = this.pack(RADICAL_HINT);
		if (!pack) {
			throw new Error("沒有字根提示資料包");
		}
		return pack;
	}

	/** 換 `kind` 用的那份。 */
	choose(kind: string, id: string): void {
		this.chosen = { ...this.chosen, [kind]: id };
		localStorage.setItem(ACTIVE_KEY, JSON.stringify(this.chosen));
	}

	/**
	 * 載入失敗時：每種回到內建那份，這次也不用匯入的（和內建同 id 的匯入會取代內建那份）。
	 * IndexedDB 裡的不刪，留著查。
	 */
	fallBackToBuiltin(): void {
		this.chosen = {};
		localStorage.removeItem(ACTIVE_KEY);
		this.imported.clear();
	}

	/**
	 * 從檔案匯入：在 `files` 裡找 manifest（`manifest.json`，或 release 上的 `<id>.manifest.json`）與它說的
	 * `<id>.pack`，裝上並換成那一種正在用的那份。任一步不過時丟錯，什麼都不動。回傳匯入的資料包。
	 */
	async import(files: readonly File[], apply: () => void): Promise<PackEntry> {
		const manifestFile = files.find(
			(f) => f.name === MANIFEST_FILE || f.name.endsWith(`.${MANIFEST_FILE}`),
		);
		if (!manifestFile) {
			throw new Error(`要一起選 ${MANIFEST_FILE} 與資料包檔`);
		}
		const manifest = await manifestFile.text();
		const entry = parseManifest(manifest, "imported");
		const packFile = files.find((f) => f.name === `${entry.id}.pack`);
		if (!packFile) {
			throw new Error(`沒有選 ${entry.id}.pack`);
		}
		const bytes = new Uint8Array(await packFile.arrayBuffer());
		await this.install(entry, manifest, bytes, apply, () => {
			this.chosen = { ...this.chosen, [entry.kind]: entry.id };
		});
		return entry;
	}

	/**
	 * 換上線上更新下載的新版（§4.7）：`from` 是開始下載時那份的版本。那份已經刪掉或換了版本（下載時使用者
	 * 動過）就不裝、回傳 `false`；不改每種用哪份（同 id 的本來在用就照用）。其餘同 `import`。
	 */
	async update(
		manifest: string,
		bytes: Uint8Array,
		from: string,
		apply: () => void,
	): Promise<boolean> {
		const entry = parseManifest(manifest, "downloaded");
		if (
			!this.listedEntries().some((p) => p.id === entry.id && p.version === from)
		) {
			return false;
		}
		await this.install(entry, manifest, bytes, apply, () => {});
		return true;
	}

	/**
	 * 裝一份資料包（匯入與線上更新共用，brief §5.3）：經核心核對 manifest 與位元組，和已有的同 id 就取代
	 * 它，`choose` 改每種用哪份；`apply` 交給核心收下了才存進 IndexedDB。任一步不過時丟錯，什麼都不動。
	 */
	private async install(
		entry: PackEntry,
		manifest: string,
		bytes: Uint8Array,
		apply: () => void,
		choose: () => void,
	): Promise<void> {
		const conflict = importConflict(this.listedEntries(), entry);
		if (conflict) {
			throw new Error(conflict);
		}
		const pack = new Pack(manifest, bytes);
		try {
			pack.verify();
		} finally {
			pack.free();
		}
		const origin = entry.origin;
		await this.change(
			() => {
				this.imported.set(entry.id, { manifest, bytes, entry });
				choose();
			},
			apply,
			() => set(entry.id, { manifest, bytes, origin } satisfies Stored, store),
		);
	}

	/** 刪掉匯入的那份；正在用的話那種回到內建的。`apply` 同 `import`，核心收下了才從 IndexedDB 刪。 */
	async remove(id: string, apply: () => void): Promise<void> {
		await this.change(
			() => this.imported.delete(id),
			apply,
			() => del(id, store),
		);
	}

	/**
	 * 改清單（`edit`）、交給核心（`apply`）、再寫進儲存空間（`persist`）；前兩步之後任一步失敗就
	 * 退回改之前的清單與選擇，核心那邊也換回去。
	 */
	private async change(
		edit: () => void,
		apply: () => void,
		persist: () => Promise<void>,
	): Promise<void> {
		const imported = new Map(this.imported);
		const chosen = this.chosen;
		edit();
		try {
			apply();
		} catch (error) {
			// 核心沒收，原本的 Engine 還在。
			this.imported = imported;
			this.chosen = chosen;
			throw error;
		}
		try {
			await persist();
		} catch (error) {
			this.imported = imported;
			this.chosen = chosen;
			apply();
			throw error;
		}
		localStorage.setItem(ACTIVE_KEY, JSON.stringify(this.chosen));
	}
}
