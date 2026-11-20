import { expect, test } from "@playwright/test";
import { ready } from "./page.ts";

// 字級 bigram（M1 票 11）：網頁載入方案指名的語言模型資料包，連打依前後字排序。
test("打 mir 首選是「一台」，按空白上屏「一台」", async ({
	page,
}, testInfo) => {
	const { text, items } = await ready(page);

	await text.focus();
	await page.keyboard.type("mir");
	await expect(items.first()).toHaveText("1一台");
	await page.screenshot({ path: testInfo.outputPath("type-mir.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("一台");
});
