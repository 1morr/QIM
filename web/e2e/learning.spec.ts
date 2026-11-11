import { expect, type Page, test } from "@playwright/test";

// 學習（M1 票 13）：同一個碼選同一個非首選兩次，它就成首選；無痕模式下不學。
// 學習開關（M3 票 12）：關掉時不學，已學的照用。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		incognito: page.getByRole("checkbox", { name: "無痕" }),
		learning: page.getByRole("checkbox", { name: "學習" }),
	};
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	return (await items.allTextContents()).map((t) => t.replace(/^\d/, ""));
}

/** 打 `code`，按數字鍵選文字是 `word` 的候選。 */
async function choose(page: Page, code: string, word: string) {
	await page.keyboard.type(code);
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
}

test("同一個碼選同一個非首選兩次，它成首選", async ({ page }, testInfo) => {
	const { text, items } = await ready(page);
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	expect(second, "a 要有第 2 個候選").toBeTruthy();
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);

	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
	expect(second).not.toBe(top);
	await page.screenshot({ path: testInfo.outputPath("learned-top.png") });
});

test("開無痕後同樣選兩次，首選不變", async ({ page }, testInfo) => {
	const { text, items, incognito } = await ready(page);
	await incognito.check();
	await expect(text).toBeFocused();
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);

	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${top}`);
	await page.screenshot({ path: testInfo.outputPath("incognito-top.png") });
});

test("關掉學習後選兩次同一候選，它不成首選", async ({ page }, testInfo) => {
	const { text, items, learning } = await ready(page);
	await learning.uncheck();
	await expect(text).toBeFocused();
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);

	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${top}`);
	await page.screenshot({ path: testInfo.outputPath("learning-off-top.png") });
});

test("學過的在關掉學習後照用", async ({ page }) => {
	const { text, items, learning } = await ready(page);
	await page.keyboard.type("a");
	const [, second] = await shown(page);
	await page.keyboard.press("Escape");
	await choose(page, "a", second);
	await choose(page, "a", second);

	await learning.uncheck();
	await expect(text).toBeFocused();
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
});
