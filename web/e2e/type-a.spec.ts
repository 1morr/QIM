import { expect, test } from "@playwright/test";
import { ready } from "./page.ts";

// 字序先看字頻、再看 Unihan kGradeLevel、再看 Big5，`a` 的首選是「日」（M0 票 08b、M1 票 06）。
test("打 a 首選是「日」，按空白上屏「日」", async ({ page }, testInfo) => {
	const { text, composition, items } = await ready(page);

	await text.focus();
	await page.keyboard.type("a");
	await expect(composition).toHaveText("日");
	await expect(items.first()).toHaveText("1日");
	await page.screenshot({ path: testInfo.outputPath("type-a.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("日");
	await expect(composition).toHaveText("");
	// 候選列換成「日」的聯想（M3 票 13）。
	await expect(items.first()).not.toHaveText("1日");
	await page.screenshot({ path: testInfo.outputPath("type-a-space.png") });
});
