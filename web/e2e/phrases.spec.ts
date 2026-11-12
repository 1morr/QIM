import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 常用語（M3 票 14）：設定區新增、編輯、刪除；打觸發詞是首選，沒有觸發詞的從候選列的清單選。

const ADDRESS = "九龍彌敦道 1 號\n3 樓 A 室";

async function ready(page: Page) {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	const section = page.getByRole("region", { name: "常用語" });
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		section,
		list: section
			.getByRole("list", { name: "常用語清單" })
			.getByRole("listitem"),
		status: section.getByRole("status", { name: "常用語狀態" }),
	};
}

async function addPhrase(page: Page, phrase: string, trigger: string) {
	const { section } = await ready(page);
	await section.getByLabel("常用語", { exact: true }).fill(phrase);
	await section.getByLabel("觸發詞（可不填）").fill(trigger);
	await section.getByRole("button", { name: "新增" }).click();
}

/** 打 `code` 按空白上屏，回傳輸入框的字，再清空。 */
async function typeAndCommit(page: Page, code: string): Promise<string> {
	const { text, items } = await ready(page);
	await page.keyboard.type(code);
	await expect(items.first()).toBeVisible();
	await page.keyboard.press("Space");
	const value = await text.inputValue();
	await text.fill("");
	return value;
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
});

test("在設定區新增含換行的常用語，打觸發詞上屏整段；刪除後打不出", async ({
	page,
}, testInfo) => {
	const { text, items, list, status, section } = await ready(page);
	await addPhrase(page, ADDRESS, "qim");
	await expect(status).toHaveText("已新增「九龍彌敦道 1 號⏎3 樓 A 室」");
	await expect(list).toHaveCount(1);
	await expect(list.first()).toContainText("九龍彌敦道 1 號⏎3 樓 A 室");
	await expect(list.first()).toContainText("qim");
	await section.screenshot({ path: testInfo.outputPath("phrase-list.png") });

	await text.focus();
	await page.keyboard.type("qim");
	await expect(items.first()).toHaveText(`1${ADDRESS}`);
	await page.screenshot({ path: testInfo.outputPath("phrase-trigger.png") });
	await page.keyboard.press("Space");
	await expect(text).toHaveValue(ADDRESS);
	await text.fill("");

	// 重新整理網頁後還在（存在學習資料裡）。
	await page.reload();
	expect(await typeAndCommit(page, "qim")).toBe(ADDRESS);

	await list.first().getByRole("button", { name: "刪除" }).click();
	await expect(status).toHaveText("已刪除「九龍彌敦道 1 號⏎3 樓 A 室」");
	await expect(list).toHaveCount(0);
	expect(await typeAndCommit(page, "qim")).not.toBe(ADDRESS);
});

test("沒有觸發詞的常用語從候選列的「常用語」點選上屏", async ({
	page,
}, testInfo) => {
	const { text, items } = await ready(page);
	await addPhrase(page, "多謝晒！", "");
	await text.focus();
	await page.keyboard.type("a");
	await page.keyboard.press("Space");
	const before = await text.inputValue();
	expect(before).not.toBe("");
	const open = page.getByRole("button", { name: "常用語", exact: true });
	await open.click();
	const picker = page
		.getByRole("list", { name: "選常用語" })
		.getByRole("listitem");
	await expect(picker).toHaveText(["多謝晒！"]);
	await page.screenshot({ path: testInfo.outputPath("phrase-picker.png") });
	await picker.first().click();
	await expect(text).toHaveValue(`${before}多謝晒！`);
	await expect(picker).toHaveCount(0);
	// 點完焦點還在輸入框，可以接著打。
	await page.keyboard.type("a");
	await expect(items.first()).toBeVisible();
	// 組字中藏起「常用語」鍵。
	await expect(open).toBeHidden();
});

test("編輯常用語換文字與觸發詞，舊的觸發詞打不出", async ({ page }) => {
	const { list, status, section } = await ready(page);
	await addPhrase(page, "早晨", "qim");
	await list.first().getByRole("button", { name: "編輯" }).click();
	await expect(section.getByLabel("常用語", { exact: true })).toHaveValue(
		"早晨",
	);
	await section.getByLabel("常用語", { exact: true }).fill("早晨！");
	await section.getByLabel("觸發詞（可不填）").fill("ab");
	await section.getByRole("button", { name: "儲存修改" }).click();
	await expect(status).toHaveText("已修改「早晨！」");
	await expect(list).toHaveCount(1);
	expect(await typeAndCommit(page, "ab")).toBe("早晨！");
	expect(await typeAndCommit(page, "qim")).not.toMatch(/早晨/);
});

test("觸發詞有方案以外的鍵時存不進去", async ({ page }) => {
	const { list, status } = await ready(page);
	await addPhrase(page, "早晨", "zz");
	await expect(status).toHaveText(/存不進去/);
	await expect(list).toHaveCount(0);
});

test("匯出、清空、匯入後常用語還在", async ({ page }) => {
	const { list } = await ready(page);
	await addPhrase(page, ADDRESS, "qim");
	const download = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	const file = await (await download).path();
	const exported = await readFile(file, "utf8");
	expect(exported.split("\n")[0]).toBe("# qim-userdict v3");
	expect(exported).toMatch(
		/\nentry\tquick\t九龍彌敦道 1 號\\n3 樓 A 室\t\t0\t0\t0\t0\t\d{4}-\d\d-\d\d\tphrase\tqim\n/,
	);

	await page.getByRole("button", { name: "清空學習資料" }).click();
	await expect(list).toHaveCount(0);
	await page.getByLabel("匯入學習資料（取代目前的）").setInputFiles(file);
	await page
		.getByRole("group", { name: "確認匯入" })
		.getByRole("button", { name: "確定匯入" })
		.click();
	await expect(list).toHaveCount(1);
	expect(await typeAndCommit(page, "qim")).toBe(ADDRESS);
});

test("新增或編輯成已有的常用語時擋下，不蓋掉那條的觸發詞", async ({ page }) => {
	const { list, status, section } = await ready(page);
	await addPhrase(page, "早晨", "qim");
	await addPhrase(page, "午安", "ab");
	await addPhrase(page, "早晨", "");
	await expect(status).toHaveText("已有「早晨」，要改它請按「編輯」");
	await list
		.filter({ hasText: "午安" })
		.getByRole("button", { name: "編輯" })
		.click();
	await expect(section.getByLabel("常用語", { exact: true })).toHaveValue(
		"午安",
	);
	await section.getByLabel("常用語", { exact: true }).fill("早晨");
	await section.getByRole("button", { name: "儲存修改" }).click();
	await expect(status).toHaveText("已有「早晨」，要改它請按「編輯」");
	await expect(list).toHaveCount(2);
	expect(await typeAndCommit(page, "qim")).toBe("早晨");
	expect(await typeAndCommit(page, "ab")).toBe("午安");
});
