import { expect, test } from "@playwright/test";

// 沒有字頻前字序照 libcangjie 的 Big5 排序值，`a` 的首選是「曰」、「日」第 2（M0 票 07 Comments）。
test("打 a 列出「日」，按空白上屏首選", async ({ page }, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type("a");
	await expect(composition).toHaveText("a");
	await expect(items.filter({ hasText: "日" })).toHaveCount(1);
	await expect(items.first()).toHaveText("1曰");
	await page.screenshot({ path: testInfo.outputPath("type-a.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("曰");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await page.screenshot({ path: testInfo.outputPath("type-a-space.png") });
});
