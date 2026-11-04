import { expect, type Page, test } from "@playwright/test";

// 重排的假實作（plan §3.1、ADR 0005，M1 票 23）：延遲後回傳名次，核心只讓它到第 2 位。
// 過期的結果丟掉由核心測試驗（`crates/qim-core/tests/rerank.rs`）。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		rerank: page.getByRole("combobox", { name: "重排" }),
	};
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
});

/** 重排關著時 `ar` 的第一頁。 */
async function unranked(page: Page): Promise<string[]> {
	const { text, items, rerank } = ui(page);
	await rerank.selectOption("off");
	await text.focus();
	await page.keyboard.type("ar");
	const first = await items.allTextContents();
	await page.keyboard.press("Escape");
	return first.map((t) => t.slice(1));
}

test("預設原樣回傳：等過延遲，候選順序不變", async ({ page }) => {
	const { text, items, rerank } = ui(page);
	const first = await unranked(page);
	await rerank.selectOption("identity");
	await text.focus();
	await page.keyboard.type("ar");
	await page.waitForTimeout(600);
	await expect(items).toHaveText(first.map((t, i) => `${i + 1}${t}`));
});

test("要把第 5 個提到最前，結果只到第 2 位，首選不變", async ({
	page,
}, testInfo) => {
	const { text, items, rerank } = ui(page);
	const first = await unranked(page);
	await rerank.selectOption("raise-fifth");
	await text.focus();
	await page.keyboard.type("ar");
	await expect(items.nth(1)).toHaveText(`2${first[4]}`);
	await expect(items.first()).toHaveText(`1${first[0]}`);
	await expect(items.nth(2)).toHaveText(`3${first[1]}`);
	await page.screenshot({ path: testInfo.outputPath("raise-fifth.png") });

	// 選字照重排後的順序。
	await page.keyboard.press("2");
	await expect(text).toHaveValue(first[4]);
});
