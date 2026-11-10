import { expect, type Page, test } from "@playwright/test";

// 記住的字串（M3 票 15）：用英文打過的 email 記在核心，之後中文打它開頭的 3 個字母，它在候選第 2 位起，
// 選了上屏整串；設定區列出、可刪，重新整理後還在；無痕不記。`iva` 剛好是英文詞表的「Iva」（人名），
// 英文候選（規則 4）在第 2 位，email（規則 5）排在它後面的第 3 位。

const EMAIL = "ivan.test@example.com";

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		list: page.getByRole("list", { name: "記住的 email 清單" }),
	};
}

/** 單按 Shift 切英文直通打 `typed`，再單按 Shift 切回中文。 */
async function typeInEnglish(page: Page, typed: string) {
	await page.keyboard.press("Shift");
	await page.keyboard.type(typed);
	await page.keyboard.press("Shift");
}

test("英文打一次 email，中文打 iva 它跟在英文候選後面，按 3 上屏整串", async ({
	page,
}, testInfo) => {
	const { text, items, list } = await ready(page);
	await expect(list).toHaveText("還沒有記住的 email");
	await text.focus();
	await typeInEnglish(page, `${EMAIL} `);
	await expect(text).toHaveValue(`${EMAIL} `);
	await expect(list.getByRole("listitem")).toHaveText([`${EMAIL}刪除`]);

	await page.keyboard.type("iva");
	await expect(items.nth(1)).toHaveText("2Iva");
	await expect(items.nth(2)).toHaveText(`3${EMAIL}`);
	await page.screenshot({
		path: testInfo.outputPath("15-remembered-email.png"),
	});

	await page.keyboard.press("3");
	await expect(text).toHaveValue(`${EMAIL} ${EMAIL}`);
	await expect(items).toHaveCount(0);
});

test("打完直接失焦也記住，重新整理後還在；刪除後不再出", async ({
	page,
}, testInfo) => {
	const { text, items, list } = await ready(page);
	await text.focus();
	await page.keyboard.press("Shift");
	await page.keyboard.type(EMAIL);
	await page.getByRole("heading", { name: "記住的 email" }).click();
	await expect(list.getByRole("listitem")).toHaveText([`${EMAIL}刪除`]);

	await page.reload();
	await expect(text).toHaveAttribute("data-ready", "true");
	await expect(list.getByRole("listitem")).toHaveText([`${EMAIL}刪除`]);
	await page.screenshot({
		path: testInfo.outputPath("15-remembered-list.png"),
		fullPage: true,
	});

	await page.getByRole("button", { name: `刪除 ${EMAIL}` }).click();
	await expect(list).toHaveText("還沒有記住的 email");
	await expect(text).toBeFocused();
	await page.keyboard.type("iva");
	await expect(items.first()).toBeVisible();
	await expect(items.filter({ hasText: EMAIL })).toHaveCount(0);
});

test("無痕時不記；關掉「記住 email」就不出", async ({ page }) => {
	const { text, items, list } = await ready(page);
	await page.getByRole("checkbox", { name: "無痕" }).check();
	await text.focus();
	await typeInEnglish(page, "ivan@private.example.org ");
	await expect(list).toHaveText("還沒有記住的 email");
	await page.getByRole("checkbox", { name: "無痕" }).uncheck();

	await text.focus();
	await typeInEnglish(page, `${EMAIL} `);
	await page.keyboard.type("iva");
	await expect(items.nth(2)).toHaveText(`3${EMAIL}`);
	const option = page.getByRole("checkbox", { name: "記住 email" });
	await expect(option).toBeChecked();
	await option.uncheck();
	await expect(items.filter({ hasText: EMAIL })).toHaveCount(0);
	await expect(text).toBeFocused();
});

test("速成打不出的 z 跳過：zz98765@example.org 打 exa 起就有", async ({
	page,
}) => {
	const { text, items } = await ready(page);
	await text.focus();
	await typeInEnglish(page, "zz98765@example.org ");
	await page.keyboard.type("exa");
	await expect(items.filter({ hasText: "zz98765@example.org" })).toHaveCount(1);
});
