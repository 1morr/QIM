import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 資料包線上更新（M3 票 21，plan〈線上更新〉）：Playwright 起的本機靜態伺服器扮演 GitHub release，放
// `e2e/fixtures/pack-release/`（`index.json` 列出 `e2e-update-lexicon` 1.1.0，`anr` 是「新版詞庫」）。
// 網頁經開發伺服器的 `/pack-updates/` 轉發去讀。先從檔案匯入舊版 1.0.0（`anr` 是「舊版詞庫」）；
// 線上更新只換裝著的資料包的新版。

const UPDATES = "**/pack-updates/**";
const ID = "e2e-update-lexicon";
const OLD = new URL(`fixtures/${ID}/`, import.meta.url);

async function ready(page: Page) {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return text;
}

/** 清空輸入框、打 `code`，回傳候選列的第一個（去掉序號），再清掉組字。 */
async function firstOf(page: Page, code: string): Promise<string> {
	const text = await ready(page);
	await text.focus();
	await page.keyboard.press("Escape");
	await text.fill("");
	await page.keyboard.type(code);
	const first = page
		.getByRole("list", { name: "候選" })
		.getByRole("listitem")
		.first();
	await expect(first).toBeVisible();
	const shown = (await first.textContent())?.replace(/^\d/, "") ?? "";
	await page.keyboard.press("Escape");
	return shown;
}

/** 從檔案匯入舊版 1.0.0。 */
async function importOld(page: Page) {
	await page.getByLabel("從檔案匯入資料包").setInputFiles([
		{
			name: "manifest.json",
			mimeType: "application/json",
			buffer: await readFile(new URL("manifest.json", OLD)),
		},
		{
			name: `${ID}.pack`,
			mimeType: "application/octet-stream",
			buffer: await readFile(new URL(`${ID}.pack`, OLD)),
		},
	]);
	await expect(status(page)).toContainText(`已匯入 ${ID} 1.0.0`);
	expect(await firstOf(page, "anr")).toBe("舊版詞庫");
}

const status = (page: Page) => page.getByRole("status", { name: "資料包狀態" });
const row = (page: Page, id: string) =>
	page
		.getByRole("table", { name: "資料包清單" })
		.locator(`tbody tr[data-id="${id}"]`);
const updateSwitch = (page: Page) =>
	page.getByRole("checkbox", { name: "線上更新" });
const checkButton = (page: Page) =>
	page.getByRole("button", { name: "檢查更新" });

/** 重新整理，等載入時背景那次檢查讀完索引（之後的匯入才不會和它搶）。 */
async function reloadAndAwaitCheck(page: Page) {
	const index = page.waitForResponse("**/pack-updates/index.json");
	await page.reload();
	expect((await index).status()).toBe(200);
	await ready(page);
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await page.evaluate(async () => {
		localStorage.clear();
		await new Promise((resolve) => {
			const request = indexedDB.deleteDatabase("qim-packs");
			request.onsuccess = request.onerror = request.onblocked = resolve;
		});
	});
	await reloadAndAwaitCheck(page);
});

test("開網頁時在背景檢查到新版、下載、換上，候選跟著換", async ({
	page,
}, testInfo) => {
	await expect(updateSwitch(page)).toBeChecked();
	await importOld(page);
	await expect(row(page, ID)).toContainText("1.0.0");

	await reloadAndAwaitCheck(page);
	await expect(status(page)).toContainText(`已更新 ${ID} 1.0.0 → 1.1.0`);
	await expect(row(page, ID)).toContainText("1.1.0");
	await expect(row(page, ID)).toContainText("線上更新");
	await expect(page.getByLabel(`使用 ${ID}`)).toBeChecked();
	expect(await firstOf(page, "anr")).toBe("新版詞庫");
	await page.keyboard.type("anr");
	await page.screenshot({
		path: testInfo.outputPath("21-updated.png"),
		fullPage: true,
	});
	await page.keyboard.press("Escape");

	// 下載的存在 IndexedDB：再重新整理還是新版，也沒有東西要更新。
	await reloadAndAwaitCheck(page);
	expect(await firstOf(page, "anr")).toBe("新版詞庫");
	await expect(status(page)).not.toContainText("已更新");

	// 刪掉下載的新版就回到匯入前（沒有這份了）。
	await page.getByRole("button", { name: `刪除 ${ID}` }).click();
	expect(await firstOf(page, "anr")).toBe("門口");
});

test("按「檢查更新」換上新版；沒在用的資料包更新後也不改用它", async ({
	page,
}) => {
	await importOld(page);
	await page.getByLabel("使用 quick-lexicon", { exact: true }).check();
	expect(await firstOf(page, "anr")).toBe("門口");

	await checkButton(page).click();
	await expect(status(page)).toContainText(`已更新 ${ID} 1.0.0 → 1.1.0`);
	await expect(row(page, ID)).toContainText("1.1.0");
	await expect(
		page.getByLabel("使用 quick-lexicon", { exact: true }),
	).toBeChecked();
	expect(await firstOf(page, "anr")).toBe("門口");

	await page.getByLabel(`使用 ${ID}`).check();
	expect(await firstOf(page, "anr")).toBe("新版詞庫");

	await checkButton(page).click();
	await expect(status(page)).toHaveText("資料包都是最新的");
});

test("下載的檔案 sha256 不符時拒絕，保留舊版", async ({ page }, testInfo) => {
	await importOld(page);
	// 照樣從本機伺服器取，交給網頁前改掉最後一個位元組。
	await page.route(`**/pack-updates/${ID}.pack`, async (route) => {
		const response = await route.fetch();
		const body = Buffer.from(await response.body());
		body[body.length - 1] ^= 1;
		await route.fulfill({ response, body });
	});

	await checkButton(page).click();
	await expect(status(page)).toContainText(`${ID} 1.1.0 沒換上`);
	await expect(status(page)).toContainText("sha256");
	await page.screenshot({
		path: testInfo.outputPath("21-rejected-sha256.png"),
		fullPage: true,
	});
	await expect(row(page, ID)).toContainText("1.0.0");
	expect(await firstOf(page, "anr")).toBe("舊版詞庫");

	// IndexedDB 裡也還是舊版。
	await reloadAndAwaitCheck(page);
	await expect(status(page)).toContainText("沒換上");
	await expect(row(page, ID)).toContainText("1.0.0");
	expect(await firstOf(page, "anr")).toBe("舊版詞庫");

	// 沒在用的那份（不交給核心）下載壞了也一樣拒絕，不存進 IndexedDB。
	await page.getByLabel("使用 quick-lexicon", { exact: true }).check();
	await checkButton(page).click();
	await expect(status(page)).toContainText("sha256");
	await page.unroute(`**/pack-updates/${ID}.pack`);
	await page.getByLabel(`使用 ${ID}`).check();
	await reloadAndAwaitCheck(page);
	expect(await firstOf(page, "anr")).toBe("新版詞庫");
});

test("關掉線上更新後不連線，重新整理後也還關著", async ({ page }) => {
	await importOld(page);
	await updateSwitch(page).uncheck();
	await expect(checkButton(page)).toBeDisabled();

	const requests: string[] = [];
	await page.route(UPDATES, async (route) => {
		requests.push(route.request().url());
		await route.abort();
	});
	await page.reload();
	await ready(page);
	await expect(updateSwitch(page)).not.toBeChecked();
	await expect(checkButton(page)).toBeDisabled();
	expect(await firstOf(page, "anr")).toBe("舊版詞庫");
	// 背景的檢查在載入完立刻送出；多等一下確定真的沒送。
	await page.waitForTimeout(500);
	expect(requests).toEqual([]);
	await expect(row(page, ID)).toContainText("1.0.0");

	// 再打開就照常檢查。
	await updateSwitch(page).check();
	await page.unroute(UPDATES);
	await checkButton(page).click();
	await expect(status(page)).toContainText(`已更新 ${ID} 1.0.0 → 1.1.0`);
});

test("斷網時打字照常，說明檢查失敗", async ({ page }) => {
	await importOld(page);
	await page.route(UPDATES, (route) => route.abort("internetdisconnected"));
	await page.reload();
	await ready(page);
	await expect(status(page)).toContainText("檢查更新失敗，照用現有的資料包");
	expect(await firstOf(page, "anr")).toBe("舊版詞庫");
	expect(await firstOf(page, "mir")).not.toBe("");
	await expect(row(page, ID)).toContainText("1.0.0");
});

test("更新來源一直不回應時，網頁照樣載入、打字；15 秒後算失敗，可以再按", async ({
	page,
}) => {
	// 請求掛著不回：檢查在背景，不擋載入。用假時鐘快轉逾時。
	await page.clock.install();
	await page.route(UPDATES, () => {});
	await page.reload();
	await ready(page);
	expect(await firstOf(page, "anr")).toBe("門口");
	await expect(checkButton(page)).toBeDisabled();

	await page.clock.runFor(15_000);
	await expect(status(page)).toContainText("15 秒沒有回應");
	await expect(checkButton(page)).toBeEnabled();
});

test("下載中刪掉那份資料包，下載完不會裝回來", async ({ page }) => {
	await importOld(page);
	let release = (): void => {};
	const downloading = new Promise<void>((resolve) => {
		page.route(`**/pack-updates/${ID}.pack`, async (route) => {
			resolve();
			await new Promise<void>((r) => {
				release = r;
			});
			await route.continue();
		});
	});
	await checkButton(page).click();
	await downloading;
	await page.getByRole("button", { name: `刪除 ${ID}` }).click();
	await expect(status(page)).toContainText(`已刪除 ${ID}`);
	release();
	await expect(checkButton(page)).toBeEnabled();
	await expect(row(page, ID)).toHaveCount(0);
	expect(await firstOf(page, "anr")).toBe("門口");
	await page.reload();
	await ready(page);
	await expect(row(page, ID)).toHaveCount(0);
});

/** `e2e/fixtures/<dir>/` 裡的 manifest 與 `quick-lexicon.pack`。 */
async function quickLexicon(dir: string) {
	const base = new URL(`fixtures/${dir}/`, import.meta.url);
	return {
		manifest: await readFile(new URL("manifest.json", base), "utf8"),
		pack: await readFile(new URL("quick-lexicon.pack", base)),
	};
}

test("下載的 1.0.1 被之後內建的 1.0.2 比下去：重新整理後用內建的，下載的不刪", async ({
	page,
}) => {
	// 更新來源放內建詞庫的 1.0.1（`anr` 是「下載版詞庫」）。
	const downloaded = await quickLexicon("quick-lexicon-1.0.1");
	await page.route("**/pack-updates/index.json", (route) =>
		route.fulfill({
			json: {
				index_version: 1,
				packs: [
					{ ...JSON.parse(downloaded.manifest), url: "quick-lexicon.pack" },
				],
			},
		}),
	);
	await page.route("**/pack-updates/quick-lexicon.pack", (route) =>
		route.fulfill({ body: downloaded.pack }),
	);
	await checkButton(page).click();
	await expect(status(page)).toContainText(
		"已更新 quick-lexicon 1.0.0 → 1.0.1",
	);
	await expect(row(page, "quick-lexicon")).toContainText("線上更新");
	expect(await firstOf(page, "anr")).toBe("下載版詞庫");

	// App 之後內建了 1.0.2（`anr` 是「內建新版詞庫」）：攔內建的請求換上。
	const builtin = await quickLexicon("quick-lexicon-1.0.2");
	await page.route("**/packs/quick-lexicon/manifest.json", (route) =>
		route.fulfill({ body: builtin.manifest }),
	);
	await page.route("**/packs/quick-lexicon/quick-lexicon.pack", (route) =>
		route.fulfill({ body: builtin.pack }),
	);
	await reloadAndAwaitCheck(page);
	expect(await firstOf(page, "anr")).toBe("內建新版詞庫");
	await expect(row(page, "quick-lexicon")).toContainText("1.0.2");
	await expect(row(page, "quick-lexicon")).toContainText("內建");
	await expect(
		page.getByRole("button", { name: "刪除 quick-lexicon" }),
	).toHaveCount(0);
	// 索引上的 1.0.1 比內建的舊，不算更新。
	await expect(status(page)).not.toContainText("已更新");

	// 下載的 1.0.1 還在 IndexedDB，只是不用。
	const stored = await page.evaluate(
		() =>
			new Promise<string>((resolve, reject) => {
				const open = indexedDB.open("qim-packs");
				open.onerror = () => reject(open.error);
				open.onsuccess = () => {
					const get = open.result
						.transaction("packs", "readonly")
						.objectStore("packs")
						.get("quick-lexicon");
					get.onsuccess = () =>
						resolve(JSON.parse(get.result.manifest).version);
					get.onerror = () => reject(get.error);
				};
			}),
	);
	expect(stored).toBe("1.0.1");

	// 內建的回到 1.0.0（比下載的舊）：又用下載的 1.0.1。
	await page.unroute("**/packs/quick-lexicon/manifest.json");
	await page.unroute("**/packs/quick-lexicon/quick-lexicon.pack");
	await reloadAndAwaitCheck(page);
	expect(await firstOf(page, "anr")).toBe("下載版詞庫");
	await expect(row(page, "quick-lexicon")).toContainText("1.0.1");
});
