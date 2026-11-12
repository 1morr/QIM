import { expect, type Page, test } from "@playwright/test";

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		rerank: page.getByRole("combobox", { name: "重排" }),
	};
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	const { text } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
});

// 字序之後會加字頻，不寫死「問」在 `ar` 的第幾頁第幾位。
test("打 ar 翻頁找到「問」，用數字鍵上屏", async ({ page }, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("ar");
	await expect(composition).toHaveText("日口");

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
	// 候選列換成「問」的聯想（M3 票 13）。
	await expect(items.first()).not.toHaveText(/問$/);
	await page.screenshot({ path: testInfo.outputPath("ar-committed.png") });
});

// 只驗翻頁鍵，關掉停鍵重排：它在翻頁中途回來時，回到第一頁才插入（rerank.spec），順序就和翻頁前讀的
// 不同；高負載時常碰到（M4 票 05c）。
test("- = 與 PageUp PageDown 翻頁", async ({ page }, testInfo) => {
	const { text, items, rerank } = ui(page);
	await rerank.selectOption("off");
	await text.focus();
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

// `a` 只有「日」「曰」兩個字（M0 票 08b Comments）。
test("打 a 再按 2 上屏第 2 個「曰」", async ({ page }) => {
	const { text, composition } = ui(page);
	await page.keyboard.type("a2");
	await expect(text).toHaveValue("曰");
	await expect(composition).toHaveText("");
});

test("翻頁後按空白，上屏第一頁的首選", async ({ page }, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("ar");
	const first = await items.allTextContents();
	// 候選的文字是數字標籤加字，如「1問」。
	const top = first[0].slice(1);

	await page.keyboard.press("=");
	await expect(items).not.toHaveText(first);
	await page.screenshot({
		path: testInfo.outputPath("space-after-paging.png"),
	});
	await page.keyboard.type(" ");
	await expect(text).toHaveValue(top);
	await expect(composition).toHaveText("");
	await page.screenshot({
		path: testInfo.outputPath("space-after-paging-committed.png"),
	});
});

test("Backspace 刪一碼、Esc 清空組字，沒有組字時交給輸入框", async ({
	page,
}, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("ar");
	await page.keyboard.press("Backspace");
	await expect(composition).toHaveText("日");
	await expect(items.first()).toHaveText("1日");
	await page.screenshot({ path: testInfo.outputPath("backspace.png") });

	await page.keyboard.press("Escape");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await expect(text).toHaveValue("");

	// 上屏後候選列是聯想，數字鍵會選聯想（M3 票 13）：先按 Esc 收起。
	await page.keyboard.type("a ");
	await page.keyboard.press("Escape");
	await page.keyboard.type("1-");
	await expect(text).toHaveValue("日1-");
	await page.keyboard.press("Backspace");
	await page.keyboard.press("Backspace");
	await expect(text).toHaveValue("日");
	await expect(composition).toHaveText("");
});
