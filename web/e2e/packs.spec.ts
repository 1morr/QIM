import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 資料包管理（M3 票 20，plan §4.6）：設定區列出資料包，從檔案匯入、切換、刪除，每步讀回候選。
// 匯入的是 `e2e/fixtures/e2e-lexicon/`：`anr` 只有「測試詞庫」一個詞（由
// `crates/qim-core/tests/web_fixture.rs` 產生）。

const FIXTURE = new URL("fixtures/e2e-lexicon/", import.meta.url);
/** 內建資料包的列數（`web/src/packs.ts` 的 `BUILTIN_IDS`）。 */
const BUILTIN_ROWS = 10;

async function fixture(): Promise<{ manifest: string; pack: Buffer }> {
	return {
		manifest: await readFile(new URL("manifest.json", FIXTURE), "utf8"),
		pack: await readFile(new URL("e2e-lexicon.pack", FIXTURE)),
	};
}

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

/** 從檔案匯入 manifest 與資料包；`manifest` 可以改過。 */
async function importPack(page: Page, manifest: string, pack: Buffer) {
	await page.getByLabel("從檔案匯入資料包").setInputFiles([
		{
			name: "manifest.json",
			mimeType: "application/json",
			buffer: Buffer.from(manifest),
		},
		{
			name: "e2e-lexicon.pack",
			mimeType: "application/octet-stream",
			buffer: pack,
		},
	]);
}

const status = (page: Page) => page.getByRole("status", { name: "資料包狀態" });
const rows = (page: Page) =>
	page.getByRole("table", { name: "資料包清單" }).locator("tbody tr");

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await page.evaluate(async () => {
		localStorage.clear();
		await new Promise((resolve) => {
			const request = indexedDB.deleteDatabase("qim-packs");
			request.onsuccess = request.onerror = request.onblocked = resolve;
		});
	});
	await page.reload();
});

test("列出內建的資料包：名稱、版本、大小、授權", async ({ page }) => {
	await ready(page);
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
	const lexicon = rows(page).and(page.locator(`[data-id="quick-lexicon"]`));
	await expect(lexicon).toContainText("詞庫");
	await expect(lexicon).toContainText("1.0.0");
	await expect(lexicon).toContainText("MB");
	await expect(lexicon).toContainText("LGPL-3.0");
	await expect(lexicon).toContainText("內建");
	await expect(rows(page).filter({ hasText: "opencc-hk2s" })).toContainText(
		"KB",
	);
	// 詞庫內建兩份（M3 票 24）：預設用 quick-lexicon，粵典那份可選；內建的都不能刪。
	await expect(
		page.getByLabel("使用 quick-lexicon", { exact: true }),
	).toBeChecked();
	await expect(
		page.getByLabel("使用 quick-lexicon-words-hk"),
	).not.toBeChecked();
	await expect(
		rows(page).filter({ hasText: "quick-lexicon-words-hk" }),
	).toContainText("LicenseRef-words-hk-NCODL-1.0");
	// 詞級 trigram 內建兩份（M3 票 11b）：預設用方案指名的預算版，放寬版可選。
	await expect(
		page.getByLabel("使用 zh-hant-word-trigram", { exact: true }),
	).toBeChecked();
	await expect(
		page.getByLabel("使用 zh-hant-word-trigram-large"),
	).not.toBeChecked();
	await expect(
		rows(page).and(page.locator(`[data-id="zh-hant-word-trigram"]`)),
	).toContainText("詞級 trigram");
	// 每種只有一份時沒得選。
	await expect(page.getByLabel("使用 zh-hant-bigram")).toBeChecked();
	await expect(page.getByLabel("使用 zh-hant-bigram")).toBeDisabled();
	await expect(page.getByRole("button", { name: /^刪除/ })).toHaveCount(0);
});

test("手機寬度下資料包清單自己橫向捲動，頁面不變寬", async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await ready(page);
	const overflow = await page.evaluate(
		() =>
			document.documentElement.scrollWidth -
			document.documentElement.clientWidth,
	);
	expect(overflow).toBe(0);
	// 只有表格橫向捲動，說明在畫面寬度內換行（M3 票 21）。
	for (const note of ["#import-pack-note", "#pack-updates-note"]) {
		const box = await page.locator(note).boundingBox();
		expect(box && box.x + box.width, note).toBeLessThanOrEqual(390);
	}
});

test("匯入、切換、刪除，每步候選都換", async ({ page }, testInfo) => {
	expect(await firstOf(page, "anr")).toBe("門口");
	const { manifest, pack } = await fixture();

	await importPack(page, manifest, pack);
	await expect(status(page)).toContainText("已匯入 e2e-lexicon 1.0.0");
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS + 1);
	await expect(page.getByLabel("使用 e2e-lexicon")).toBeChecked();
	expect(await firstOf(page, "anr")).toBe("測試詞庫");
	await page.keyboard.type("anr");
	await page.screenshot({
		path: testInfo.outputPath("20-imported.png"),
		fullPage: true,
	});
	await page.keyboard.press("Escape");

	// 重新整理後還是用匯入的那份（IndexedDB 與 localStorage）。
	await page.reload();
	expect(await firstOf(page, "anr")).toBe("測試詞庫");

	await page.getByLabel("使用 quick-lexicon", { exact: true }).check();
	await expect(status(page)).toContainText("詞庫改用 quick-lexicon");
	expect(await firstOf(page, "anr")).toBe("門口");
	await page.keyboard.type("anr");
	await page.screenshot({
		path: testInfo.outputPath("20-switched.png"),
		fullPage: true,
	});
	await page.keyboard.press("Escape");

	await page.getByLabel("使用 e2e-lexicon").check();
	expect(await firstOf(page, "anr")).toBe("測試詞庫");

	await page.getByRole("button", { name: "刪除 e2e-lexicon" }).click();
	await expect(status(page)).toContainText("已刪除 e2e-lexicon");
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
	await expect(
		page.getByLabel("使用 quick-lexicon", { exact: true }),
	).toBeChecked();
	expect(await firstOf(page, "anr")).toBe("門口");
	await page.screenshot({
		path: testInfo.outputPath("20-deleted.png"),
		fullPage: true,
	});

	await page.reload();
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
	expect(await firstOf(page, "anr")).toBe("門口");
});

test("組字中切換、刪除，候選列立刻換", async ({ page }) => {
	const text = await ready(page);
	const { manifest, pack } = await fixture();
	await importPack(page, manifest, pack);
	await expect(status(page)).toContainText("已匯入");
	const first = page
		.getByRole("list", { name: "候選" })
		.getByRole("listitem")
		.first();
	await text.focus();
	await page.keyboard.type("anr");
	await expect(first).toHaveText("1測試詞庫");
	await page.getByLabel("使用 quick-lexicon", { exact: true }).check();
	await expect(first).toHaveText("1門口");
	await page.getByLabel("使用 e2e-lexicon").check();
	await expect(first).toHaveText("1測試詞庫");
	await page.getByRole("button", { name: "刪除 e2e-lexicon" }).click();
	await expect(status(page)).toContainText("已刪除");
	await expect(first).toHaveText("1門口");
});

/** 打 `code` 時候選列的文字（去掉序號），再清掉組字。 */
async function candidatesOf(page: Page, code: string): Promise<string[]> {
	await firstOf(page, code);
	await page.keyboard.type(code);
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	const texts = (await items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
	await page.keyboard.press("Escape");
	return texts;
}

test("粵典詞庫：切過去打得出只在粵典詞頭表的詞，切回預設詞庫打不出", async ({
	page,
}, testInfo) => {
	// 「暗寸」（aadi）只在粵典的詞頭表：預設詞庫沒有，逐字拼也拼不出（M3 票 24；票 11b 起詞級 trigram 拼得出原本用的「襟食」，換成它）。
	await ready(page);
	expect(await candidatesOf(page, "aadi")).not.toContain("暗寸");

	await page.getByLabel("使用 quick-lexicon-words-hk").check();
	await expect(status(page)).toContainText("詞庫改用 quick-lexicon-words-hk");
	expect(await firstOf(page, "aadi")).toBe("暗寸");
	expect(await firstOf(page, "anr")).toBe("門口");
	await page.keyboard.type("aadi");
	await page.screenshot({
		path: testInfo.outputPath("24-words-hk.png"),
		fullPage: true,
	});
	await page.keyboard.press("Escape");

	// 重新整理後還是用粵典那份。
	await page.reload();
	expect(await firstOf(page, "aadi")).toBe("暗寸");

	await page.getByLabel("使用 quick-lexicon", { exact: true }).check();
	await expect(status(page)).toContainText("詞庫改用 quick-lexicon");
	expect(await candidatesOf(page, "aadi")).not.toContain("暗寸");
	await page.keyboard.type("aadi");
	await page.screenshot({
		path: testInfo.outputPath("24-default.png"),
		fullPage: true,
	});
	await page.keyboard.press("Escape");
});

// 建置略過粵典詞庫時 `build/packs/` 沒有它：Vite 開發伺服器對沒有的檔回 200 的 index.html（SPA
// fallback，實測），其他靜態伺服器回 404，兩種都要照樣載入。
for (const [served, missing] of [
	[
		"Vite 回 index.html",
		{ status: 200, contentType: "text/html", body: "<!doctype html>" },
	],
	["回 404", { status: 404 }],
] as const) {
	test(`建置略過了粵典詞庫（${served}）時，網頁照樣載入，選過它的改用預設詞庫`, async ({
		page,
	}) => {
		await ready(page);
		await page.getByLabel("使用 quick-lexicon-words-hk").check();
		expect(await firstOf(page, "aadi")).toBe("暗寸");
		await page.route("**/packs/quick-lexicon-words-hk/**", (route) =>
			route.fulfill(missing),
		);
		await page.reload();
		await expect(rows(page)).toHaveCount(BUILTIN_ROWS - 1);
		await expect(
			page.getByLabel("使用 quick-lexicon", { exact: true }),
		).toBeChecked();
		expect(await firstOf(page, "anr")).toBe("門口");
		expect(await candidatesOf(page, "aadi")).not.toContain("暗寸");
	});
}

test("學習資料與開關在換資料包後還在", async ({ page }) => {
	await ready(page);
	// 手動加的使用者詞：「雲口」不在任何詞庫。
	await page.getByLabel("詞", { exact: true }).fill("雲口");
	await page.getByRole("button", { name: "加詞" }).click();
	await page.getByLabel("簡體").check();
	expect(await firstOf(page, "anr")).toBe("门口");
	expect(await candidatesOf(page, "mir")).toContain("云口");

	const { manifest, pack } = await fixture();
	await importPack(page, manifest, pack);
	await expect(status(page)).toContainText("已匯入");
	expect(await firstOf(page, "anr")).toBe("测试词库");
	// 位置隨詞庫的字頻變，詞還在。
	expect(await candidatesOf(page, "mir")).toContain("云口");
});

test("改過 sha256 的資料包匯入被拒，候選不變", async ({ page }, testInfo) => {
	const { manifest, pack } = await fixture();
	const tampered = manifest.replace(
		/"sha256": "[0-9a-f]+"/,
		`"sha256": "${"0".repeat(64)}"`,
	);
	expect(tampered).not.toBe(manifest);
	await ready(page);
	await importPack(page, tampered, pack);
	await expect(status(page)).toContainText("匯入失敗");
	await expect(status(page)).toContainText("sha256");
	await page.screenshot({
		path: testInfo.outputPath("20-rejected-sha256.png"),
		fullPage: true,
	});
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
	expect(await firstOf(page, "anr")).toBe("門口");

	// 資料包檔被改過一個位元組也一樣。
	const flipped = Buffer.from(pack);
	flipped[flipped.length - 1] ^= 1;
	await importPack(page, manifest, flipped);
	await expect(status(page)).toContainText("sha256");
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
});

test("要求更新核心的資料包匯入被拒", async ({ page }) => {
	const { manifest, pack } = await fixture();
	const newer = manifest.replace(
		/"min_core_version": "[^"]+"/,
		'"min_core_version": "999.0.0"',
	);
	expect(newer).not.toBe(manifest);
	await ready(page);
	await importPack(page, newer, pack);
	await expect(status(page)).toContainText("匯入失敗");
	await expect(status(page)).toContainText("999.0.0");
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
	expect(await firstOf(page, "anr")).toBe("門口");
});

test("只選資料包檔、沒選 manifest 時說明要一起選", async ({ page }) => {
	const { pack } = await fixture();
	await ready(page);
	await page.getByLabel("從檔案匯入資料包").setInputFiles([
		{
			name: "e2e-lexicon.pack",
			mimeType: "application/octet-stream",
			buffer: pack,
		},
	]);
	await expect(status(page)).toContainText("manifest.json");
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
});

test("和內建同 id 但種類不同的資料包匯入被拒", async ({ page }) => {
	const { manifest, pack } = await fixture();
	// 一份 id 叫 quick（內建的方案）的詞庫：會取代內建的方案，種類卻不同。
	await ready(page);
	await page.getByLabel("從檔案匯入資料包").setInputFiles([
		{
			name: "manifest.json",
			mimeType: "application/json",
			buffer: Buffer.from(
				manifest.replace('"id": "e2e-lexicon"', '"id": "quick"'),
			),
		},
		{ name: "quick.pack", mimeType: "application/octet-stream", buffer: pack },
	]);
	await expect(status(page)).toContainText("匯入失敗");
	await expect(status(page)).toContainText("方案");
	await page.reload();
	await expect(rows(page)).toHaveCount(BUILTIN_ROWS);
	expect(await firstOf(page, "anr")).toBe("門口");
});

test("存著的資料包壞了時，重新整理後這次只用內建的並說明", async ({ page }) => {
	const { manifest, pack } = await fixture();
	await ready(page);
	await importPack(page, manifest, pack);
	await expect(status(page)).toContainText("已匯入");
	// 把 IndexedDB 裡那份的最後一個位元組改掉。
	await page.evaluate(
		() =>
			new Promise<void>((resolve, reject) => {
				const open = indexedDB.open("qim-packs");
				open.onerror = () => reject(open.error);
				open.onsuccess = () => {
					const table = open.result
						.transaction("packs", "readwrite")
						.objectStore("packs");
					const get = table.get("e2e-lexicon");
					get.onsuccess = () => {
						const value = get.result as { manifest: string; bytes: Uint8Array };
						const bytes = new Uint8Array(value.bytes);
						bytes[bytes.length - 1] ^= 1;
						const put = table.put({ ...value, bytes }, "e2e-lexicon");
						put.onsuccess = () => resolve();
						put.onerror = () => reject(put.error);
					};
				};
			}),
	);
	await page.reload();
	await ready(page);
	await expect(page.locator("#status")).toContainText("這次只用內建的");
	await expect(page.locator("#status")).toContainText("sha256");
	expect(await firstOf(page, "anr")).toBe("門口");
});
