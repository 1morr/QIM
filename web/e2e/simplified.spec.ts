import { expect, type Page, test } from "@playwright/test";

// 簡體模式（M1 票 16）：候選、組字與上屏轉成簡體（OpenCC hk2s），學習仍記繁體。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		simplified: page.getByRole("checkbox", { name: "簡體" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

test("開簡體後打 anr，首選與空白上屏都是「门口」", async ({
	page,
}, testInfo) => {
	const { text, simplified, items } = await ready(page);
	await simplified.check();
	await text.focus();
	await page.keyboard.type("anr");
	await expect(items.first()).toHaveText("1门口");
	await page.screenshot({ path: testInfo.outputPath("simplified-anr.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("门口");
	await expect(items).toHaveCount(0);
	await page.screenshot({ path: testInfo.outputPath("simplified-commit.png") });
});

test("組字中切換簡繁，候選立刻跟著換、組字不變", async ({ page }) => {
	const { text, simplified, items } = await ready(page);
	const composition = page.getByTestId("composition");
	await text.focus();
	await page.keyboard.type("anr");
	await expect(items.first()).toHaveText("1門口");

	await simplified.check();
	await expect(items.first()).toHaveText("1门口");
	await expect(composition).toHaveText("anr");
	await expect(text).toBeFocused();

	await simplified.uncheck();
	await expect(items.first()).toHaveText("1門口");
	await page.keyboard.type(" ");
	await expect(text).toHaveValue("門口");
});

test("簡體模式下選定開頭一段，組字顯示簡體、上屏也是簡體", async ({ page }) => {
	const { text, simplified, items } = await ready(page);
	const composition = page.getByTestId("composition");
	await simplified.check();
	await text.focus();
	await page.keyboard.type("anrkbo");
	const menKou = items.filter({ hasText: /^\d门口$/ });
	await expect(menKou).toHaveCount(1);
	const label = (await menKou.textContent()) ?? "";
	await page.keyboard.type(label.charAt(0));
	await expect(composition).toHaveText("门口kbo");
	await page.keyboard.type(" ");
	await expect(text).toHaveValue(/^门口./);
	await expect(text).not.toHaveValue(/[門們]/);
});
