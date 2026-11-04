import { expect, test } from "@playwright/test";

// 字序先看字頻、再看 Unihan kGradeLevel、再看 Big5，`a` 的首選是「日」（M0 票 08b、M1 票 06）。
test("打 a 首選是「日」，按空白上屏「日」", async ({ page }, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type("a");
	await expect(composition).toHaveText("a");
	await expect(items.first()).toHaveText("1日");
	await page.screenshot({ path: testInfo.outputPath("type-a.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("日");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await page.screenshot({ path: testInfo.outputPath("type-a-space.png") });
});
