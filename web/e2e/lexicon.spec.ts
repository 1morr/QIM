import { expect, test } from "@playwright/test";

// 詞庫資料包（M1 票 06）：整串輸入碼查得到詞；連打（票 07）之前，`anr` 只有詞庫的詞。
test("打 anr 候選有「門口」，按空白上屏首選", async ({ page }, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type("anr");
	await expect(composition).toHaveText("anr");
	await expect(items.filter({ hasText: "門口" })).toHaveCount(1);
	await page.screenshot({ path: testInfo.outputPath("type-anr.png") });

	const top = (await items.first().textContent())?.replace(/^1/, "");
	await page.keyboard.type(" ");
	await expect(text).toHaveValue(top ?? "");
	await expect(composition).toHaveText("");
});
