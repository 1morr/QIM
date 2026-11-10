import { expect, type Page, test } from "@playwright/test";

// 詞表匯入（M3 票 17）：設定區「匯入詞庫」先列出會加幾個、略過哪些列，確認後加進使用者詞，
// 不動原本的學習資料。

/** Gboard 個人字典的 `dictionary.txt`：`kk` → 雲口、詞庫已有的「口」、壞列。 */
const GBOARD =
	"# Gboard Dictionary version:1\nkk\t雲口\tzh-HK\n\t口\tzh-HK\nbroken\n";

async function ready(page: Page) {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	const section = page.getByRole("region", { name: "匯入詞庫" });
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		section,
		confirm: section.getByRole("group", { name: "確認匯入詞庫" }),
		skipped: section
			.getByRole("list", { name: "略過的列" })
			.getByRole("listitem"),
		status: section.getByRole("status", { name: "詞庫匯入狀態" }),
	};
}

async function choose(page: Page, name: string, content: string) {
	await page.getByLabel("詞表檔案").setInputFiles({
		name,
		mimeType: "text/plain",
		buffer: Buffer.from(content),
	});
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
});

test("匯入 Gboard 詞表：先列出會加幾個、略過哪些，確認後打得出", async ({
	page,
}, testInfo) => {
	const { text, items, section, confirm, skipped, status } = await ready(page);
	await choose(page, "dictionary.txt", GBOARD);
	await expect(confirm).toBeVisible();
	await expect(confirm).toContainText(
		"dictionary.txt（Gboard 個人字典）會加 1 個詞、略過 2 列",
	);
	await expect(skipped).toHaveText([
		"第 3 行「口」：詞庫已有",
		"第 4 行「broken」：Gboard 的列要是「快捷鍵<Tab>詞<Tab>語言」",
	]);
	await section.screenshot({
		path: testInfo.outputPath("word-list-confirm.png"),
	});

	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await expect(status).toHaveText("已從 dictionary.txt 加入 1 個詞");
	await expect(confirm).toBeHidden();

	// 觸發詞 `kk` 是首選；連打的碼 `mir` 也打得出。
	await text.focus();
	await page.keyboard.type("kk");
	await expect(items.first()).toHaveText("1雲口");
	await page.keyboard.press("Space");
	await expect(text).toHaveValue("雲口");
	await page.screenshot({ path: testInfo.outputPath("word-list-typed.png") });
	await page.keyboard.type("mir");
	await expect(items.filter({ hasText: /^\d雲口$/ })).toHaveCount(1);
	await page.keyboard.press("Escape");

	// 存進 localStorage：重新整理後還在。
	await page.reload();
	const again = await ready(page);
	await page.keyboard.type("kk");
	await expect(again.items.first()).toHaveText("1雲口");
});

test("取消不加；沒有可加的詞時不問、直接列出略過的列", async ({ page }) => {
	const { text, items, confirm, skipped, status } = await ready(page);
	await choose(page, "words.txt", "雲口\n");
	await expect(confirm).toContainText("會加 1 個詞、略過 0 列");
	await confirm.getByRole("button", { name: "取消" }).click();
	await expect(status).toHaveText("沒有匯入，使用者詞沒動");
	await text.focus();
	await page.keyboard.type("kk");
	await expect(items.first()).not.toHaveText("1雲口");
	await page.keyboard.press("Escape");

	await choose(page, "only-chars.txt", "口\nhello\n");
	await expect(confirm).toBeHidden();
	await expect(status).toHaveText("only-chars.txt 沒有可加的詞");
	await expect(skipped).toHaveCount(2);
});

test("整份讀不了的 Rime 檔報行號、什麼都不加", async ({ page }) => {
	const { confirm, status } = await ready(page);
	await choose(page, "bad.dict.yaml", "---\nname: x\n雲口\n");
	await expect(status).toHaveText(/匯入失敗，使用者詞沒動：第 1 行/);
	await expect(confirm).toBeHidden();
});
