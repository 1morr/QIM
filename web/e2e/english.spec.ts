import { expect, type Page, test } from "@playwright/test";

// 中文狀態的英文候選（M1 票 15）：整串輸入碼剛好是完整英文單字時排第 2 位，按 2 上屏；
// 不容錯、不補全、不搶首選；開關預設開。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		english: page.getByRole("checkbox", { name: "英文候選" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

test("打 the，第 2 位是 the，按 2 上屏 the", async ({ page }, testInfo) => {
	const { text, english, items } = await ready(page);
	await expect(english).toBeChecked();
	await text.focus();
	await page.keyboard.type("the");
	await expect(items.nth(1)).toHaveText("2the");
	await expect(items.first()).not.toHaveText("1the");
	await page.screenshot({ path: testInfo.outputPath("english-the.png") });

	await page.keyboard.press("2");
	await expect(text).toHaveValue("the");
	await expect(items).toHaveCount(0);
});

test("拼錯一碼、只打前幾碼、2 碼的英文字都沒有英文候選", async ({ page }) => {
	const { text, items } = await ready(page);
	await text.focus();
	for (const code of ["thw", "th", "an"]) {
		await page.keyboard.type(code);
		await expect(items.first()).toBeVisible();
		for (const item of await items.allTextContents()) {
			expect(item, `${code}：${item}`).not.toMatch(/^\d[a-z]+$/i);
		}
		await page.keyboard.press("Escape");
	}
});

test("關掉英文候選就不出，組字中切換立刻重列", async ({ page }) => {
	const { text, english, items } = await ready(page);
	await text.focus();
	await page.keyboard.type("the");
	await expect(items.nth(1)).toHaveText("2the");

	await english.uncheck();
	await expect(items.filter({ hasText: /^\dthe$/ })).toHaveCount(0);
	await expect(text).toBeFocused();

	await english.check();
	await expect(items.nth(1)).toHaveText("2the");
});
