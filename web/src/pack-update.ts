// 資料包線上更新（plan〈線上更新〉、brief〈版本與更新〉）：讀更新索引、問核心哪些裝著的資料包有新版、下載、交給
// `PackLibrary.install` 核對後換上。在背景跑，失敗或斷網時照用現有的（brief〈隱私與安全〉：打字不依賴網路）。
//
// 更新來源是 GitHub Releases，但 release 的檔案不帶 CORS 標頭，瀏覽器讀不了，所以網頁一律向自己的
// `/pack-updates/` 要，由 Vite 開發伺服器轉發（`vite.config.ts`，同雲端 API 的做法，plan〈網頁 prototype〉）。

import { message } from "./dom.ts";
import type { PackLibrary } from "./packs.ts";
import { storage } from "./storage.ts";
import { packUpdates } from "./wasm/qim_wasm.js";

/** 更新索引在開發伺服器上的位置；資料包檔的 `url` 相對於它。 */
const INDEX_URL = "/pack-updates/index.json";
/** 讀索引的逾時：來源不回應時這次算失敗，「檢查更新」鍵才能再按。 */
const INDEX_TIMEOUT_MS = 15_000;
/** 下載一份資料包的逾時；最大的詞庫約 5 MB。 */
const PACK_TIMEOUT_MS = 120_000;
/** 線上更新的開關，存 `localStorage`；關掉時存 `"off"`，預設開（brief〈版本與更新〉）。 */
const SWITCH_KEY = "qim.packs.update";

export function updatesEnabled(): boolean {
	return storage.get(SWITCH_KEY) !== "off";
}

export function setUpdatesEnabled(on: boolean): void {
	if (on) {
		storage.remove(SWITCH_KEY);
	} else {
		storage.set(SWITCH_KEY, "off");
	}
}

/** 換上的一份：id、舊版與新版。 */
export interface Updated {
	id: string;
	from: string;
	to: string;
}

/** 有新版、但下載或核對不過而沒換的一份；照用舊版。 */
export interface Rejected {
	id: string;
	version: string;
	reason: string;
}

export interface UpdateResult {
	updated: Updated[];
	rejected: Rejected[];
}

/**
 * 讀 `url` 的整份內容，`ms` 毫秒內讀不完就中止並丟錯。逾時用 `setTimeout`（不用 `AbortSignal.timeout`），
 * e2e 才能用 Playwright 的假時鐘快轉。
 */
async function fetchWithin(url: URL, ms: number): Promise<ArrayBuffer> {
	const controller = new AbortController();
	const timer = setTimeout(() => controller.abort(), ms);
	try {
		const response = await fetch(url, {
			cache: "no-store",
			signal: controller.signal,
		});
		if (!response.ok) {
			throw new Error(`${url}：${response.status}`);
		}
		return await response.arrayBuffer();
	} catch (error) {
		throw controller.signal.aborted
			? new Error(`${url}：${ms / 1000} 秒沒有回應`)
			: error;
	} finally {
		clearTimeout(timer);
	}
}

/**
 * 檢查並換上新版。讀不到索引或索引不合格式時丟錯（什麼都沒動）；個別資料包下載或核對不過時記在
 * `rejected`，其他的照樣換。`apply` 同 `PackLibrary.install`：把目前選的資料包交給核心。
 */
export async function updatePacks(
	library: PackLibrary,
	apply: () => void,
): Promise<UpdateResult> {
	const indexUrl = new URL(INDEX_URL, location.href);
	const index = new TextDecoder().decode(
		await fetchWithin(indexUrl, INDEX_TIMEOUT_MS),
	);
	const result: UpdateResult = { updated: [], rejected: [] };
	for (const update of packUpdates(index, library.installedManifests())) {
		const { id, version, manifest } = update;
		const url = new URL(update.url, indexUrl);
		update.free();
		// 核心只回裝著的資料包的新版，所以一定找得到。
		const from = library.list().find((p) => p.id === id)?.version ?? "";
		try {
			const bytes = new Uint8Array(await fetchWithin(url, PACK_TIMEOUT_MS));
			if (await library.update(manifest, bytes, from, apply)) {
				result.updated.push({ id, from, to: version });
			}
		} catch (error) {
			result.rejected.push({ id, version, reason: message(error) });
		}
	}
	return result;
}
