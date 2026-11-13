import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 儲存打不開時照樣能打字（M4 票 02，plan〈網頁 prototype〉）：IndexedDB（匯入、下載的資料包）或 `localStorage`
// （學習資料、設定、每種資料包用哪份）打不開，網頁只用內建的資料包、存不了的留在記憶體，狀態列說明。
// 瀏覽器封鎖網站資料時（例如 Chrome 封鎖所有 Cookie）`indexedDB.open` 與讀 `window.localStorage` 都丟
// `SecurityError`；這裡在網頁的程式跑之前換掉它們。

/** 打 `a`、按空白，上屏「日」（同 `type-a.spec.ts`）。 */
async function typeA(page: Page) {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	await page.keyboard.type("a ");
	await expect(text).toHaveValue("日");
}

/** 讓 `indexedDB.open` 直接丟 `SecurityError`（瀏覽器封鎖網站資料時的樣子）。 */
async function blockIndexedDb(page: Page) {
	await page.addInitScript(() => {
		IDBFactory.prototype.open = () => {
			throw new DOMException("封鎖了網站資料", "SecurityError");
		};
	});
}

test("IndexedDB 打不開（open 直接丟錯）：照樣打得出字，狀態列說明只用內建的資料包", async ({
	page,
}, testInfo) => {
	await blockIndexedDb(page);
	await page.goto("/");
	await typeA(page);
	await expect(page.locator("#status")).toContainText("IndexedDB 打不開");
	await expect(page.locator("#status")).toContainText("封鎖了網站資料");
	await page.screenshot({ path: testInfo.outputPath("indexeddb-blocked.png") });
});

// 匯入的是 `packs.spec.ts` 那份 `e2e-lexicon`：`anr` 只有「測試詞庫」一個詞。
test("IndexedDB 打不開時匯入資料包：這次用得到，重新整理後回到內建的", async ({
	page,
}) => {
	await blockIndexedDb(page);
	await page.goto("/");
	await typeA(page);
	const fixture = new URL("fixtures/e2e-lexicon/", import.meta.url);
	await page.getByLabel("從檔案匯入資料包").setInputFiles([
		{
			name: "manifest.json",
			mimeType: "application/json",
			buffer: await readFile(new URL("manifest.json", fixture)),
		},
		{
			name: "e2e-lexicon.pack",
			mimeType: "application/octet-stream",
			buffer: await readFile(new URL("e2e-lexicon.pack", fixture)),
		},
	]);
	await expect(page.getByRole("status", { name: "資料包狀態" })).toContainText(
		"已匯入 e2e-lexicon 1.0.0",
	);
	const first = page
		.getByRole("list", { name: "候選" })
		.getByRole("listitem")
		.first();
	const text = page.getByRole("textbox", { name: "輸入框" });
	await text.focus();
	await page.keyboard.type("anr");
	await expect(first).toHaveText("1測試詞庫");

	await page.reload();
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	await page.keyboard.type("anr");
	await expect(first).toHaveText("1門口");
});

test("IndexedDB 打不開（open 的要求回報錯誤）：照樣打得出字，狀態列有說明", async ({
	page,
}) => {
	await page.addInitScript(() => {
		IDBFactory.prototype.open = () => {
			// 只做 idb-keyval 用到的：它設好 `onerror` 之後才回報。
			const request = {
				error: new DOMException("打不開資料庫", "InvalidStateError"),
				onerror: null as (() => void) | null,
			};
			setTimeout(() => request.onerror?.());
			return request as unknown as IDBOpenDBRequest;
		};
	});
	await page.goto("/");
	await typeA(page);
	await expect(page.locator("#status")).toContainText("IndexedDB 打不開");
});

test("localStorage 打不開：照樣打得出字，狀態列說明這次不會存", async ({
	page,
}, testInfo) => {
	await page.addInitScript(() => {
		Object.defineProperty(window, "localStorage", {
			configurable: true,
			get() {
				throw new DOMException("封鎖了網站資料", "SecurityError");
			},
		});
	});
	const errors: string[] = [];
	page.on("pageerror", (error) => errors.push(error.message));
	await page.goto("/");
	await typeA(page);
	await expect(page.locator("#status")).toContainText("這次不會存");
	await expect(page.locator("#status")).toContainText("封鎖了網站資料");
	await page.screenshot({
		path: testInfo.outputPath("localstorage-blocked.png"),
	});
	// 學習資料存在記憶體：上屏後存檔不丟錯。
	expect(errors).toEqual([]);
});

test("localStorage 存不下（寫入丟錯）：照樣打得出字，狀態列說明這次不會存", async ({
	page,
}) => {
	await page.addInitScript(() => {
		Storage.prototype.setItem = () => {
			throw new DOMException("滿了", "QuotaExceededError");
		};
	});
	const errors: string[] = [];
	page.on("pageerror", (error) => errors.push(error.message));
	await page.goto("/");
	await typeA(page);
	await expect(page.locator("#status")).toContainText("這次不會存");
	expect(errors).toEqual([]);
});

// 停鍵層模型載入失敗的說明（Worker 回報，可能在載入完之前或之後）不蓋掉儲存的說明。
test("localStorage 打不開、停鍵層模型又載入不了：狀態列兩條說明都留著", async ({
	page,
}) => {
	await page.addInitScript(() => {
		Object.defineProperty(window, "localStorage", {
			configurable: true,
			get() {
				throw new DOMException("封鎖了網站資料", "SecurityError");
			},
		});
	});
	// 模型資料包只給前 100 位元組：核心核對不過，Worker 回報載入失敗。
	await page.route("**/zh-hant-neural-rerank.pack", async (route) => {
		const response = await route.fetch();
		await route.fulfill({
			response,
			body: (await response.body()).subarray(0, 100),
		});
	});
	await page.goto("/");
	await typeA(page);
	const status = page.locator("#status");
	await expect(status).toContainText("停鍵層模型載入不了");
	await expect(status).toContainText("這次不會存");
});
