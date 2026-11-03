import { expect, type Page, test } from "@playwright/test";

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	const { text } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
});

// 沒有字頻前字序照 Big5，「問」不一定在 `ar` 第一頁（spec〈範圍〉）。
test("打 ar 翻頁找到「問」，用數字鍵上屏", async ({ page }, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("ar");
	await expect(composition).toHaveText("ar");

	for (
		let turned = 0;
		(await items.filter({ hasText: "問" }).count()) === 0;
		turned++
	) {
		expect(turned, "翻到最後一頁還沒有「問」").toBeLessThan(20);
		await page.keyboard.press("=");
	}
	const labels = await items.allTextContents();
	const position = labels.findIndex((label) => label.endsWith("問")) + 1;
	await page.screenshot({ path: testInfo.outputPath("ar-found.png") });

	await page.keyboard.press(String(position));
	await expect(text).toHaveValue("問");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await page.screenshot({ path: testInfo.outputPath("ar-committed.png") });
});

test("- = 與 PageUp PageDown 翻頁", async ({ page }, testInfo) => {
	const { items } = ui(page);
	await page.keyboard.type("ar");
	const first = await items.allTextContents();
	await page.screenshot({ path: testInfo.outputPath("page-1.png") });

	await page.keyboard.press("=");
	await expect(items).not.toHaveText(first);
	const second = await items.allTextContents();
	await page.screenshot({ path: testInfo.outputPath("page-2.png") });
	await page.keyboard.press("-");
	await expect(items).toHaveText(first);
	await page.keyboard.press("PageDown");
	await expect(items).toHaveText(second);
	await page.keyboard.press("PageUp");
	await expect(items).toHaveText(first);
});

// `a` 的首選是「曰」、「日」第 2（M0 票 07 Comments）。
test("打 a 再按 2 上屏「日」", async ({ page }) => {
	const { text, composition } = ui(page);
	await page.keyboard.type("a2");
	await expect(text).toHaveValue("日");
	await expect(composition).toHaveText("");
});

test("Backspace 刪一碼、Esc 清空組字，沒有組字時交給輸入框", async ({
	page,
}, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("ar");
	await page.keyboard.press("Backspace");
	await expect(composition).toHaveText("a");
	await expect(items.first()).toHaveText("1曰");
	await page.screenshot({ path: testInfo.outputPath("backspace.png") });

	await page.keyboard.press("Escape");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await expect(text).toHaveValue("");

	await page.keyboard.type("a 1-");
	await expect(text).toHaveValue("曰1-");
	await page.keyboard.press("Backspace");
	await page.keyboard.press("Backspace");
	await expect(text).toHaveValue("曰");
	await expect(composition).toHaveText("");
});
