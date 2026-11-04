import { expect, type Page, test } from "@playwright/test";

// 數字後的句點、逗號、冒號用半形（M1 票 26，brief §4.7）：照 librime `digit_separators`，
// 分隔符先待定、接數字半形上屏，再按一次同一個鍵改回全形；開關預設開。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		toggle: page.getByRole("checkbox", { name: "數字後半形" }),
		composition: page.getByTestId("composition"),
	};
}

test("打 3.5、1,000、12:30 讀回半形", async ({ page }, testInfo) => {
	const { text, toggle, composition } = await ready(page);
	await expect(toggle).toBeChecked();
	await text.focus();
	await page.keyboard.type("3.");
	// 分隔符待定時顯示在組字裡，還沒上屏。
	await expect(composition).toHaveText(".");
	await expect(text).toHaveValue("3");
	await page.screenshot({
		path: testInfo.outputPath("digit-separator-pending.png"),
	});
	await page.keyboard.type("5 1,000 12:30");
	await expect(text).toHaveValue("3.5 1,000 12:30");
	await expect(composition).toHaveText("");
	// 分隔符不是選字，「回報」沒有東西可回報。
	await expect(
		page.getByRole("button", { name: "回報", exact: true }),
	).toBeDisabled();
});

test("不在數字後、或再按一次同一個鍵，句點是全形", async ({ page }) => {
	const { text } = await ready(page);
	await text.focus();
	await page.keyboard.type(".");
	await expect(text).toHaveValue("。");
	await page.keyboard.type("1,,");
	await expect(text).toHaveValue("。1，");
});

test("關掉開關後數字後也是全形", async ({ page }) => {
	const { text, toggle } = await ready(page);
	await toggle.uncheck();
	await expect(text).toBeFocused();
	await page.keyboard.type("3.5");
	await expect(text).toHaveValue("3。5");
});
