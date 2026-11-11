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
	// 候選列換成「门口」的聯想（M3 票 13），不再是「门口」。
	await expect(items.first()).not.toHaveText("1门口");
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
	await expect(composition).toHaveText("日弓口");
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
	await expect(composition).toHaveText("门口大月人");
	await page.keyboard.type(" ");
	await expect(text).toHaveValue(/^门口./);
	await expect(text).not.toHaveValue(/[門們]/);
});

// 回報與字根提示用繁體原文（協調者決定）：`fixed.tsv` 的期望首選是繁體，字根提示要對應打得出的字。
test("簡體模式選「门口」後按回報，存下的是繁體「門口」", async ({ page }) => {
	const { text, simplified, items } = await ready(page);
	const report = page.getByRole("button", { name: "回報", exact: true });
	const reports = page.getByRole("textbox", { name: /回報的固定題/ });
	await simplified.check();
	await text.focus();
	await page.keyboard.type("anr");
	await expect(items.first()).toHaveText("1门口");
	await page.keyboard.press("1");
	await expect(text).toHaveValue("门口");

	await report.click();
	await expect(reports).toHaveValue("anr\t門口\t網頁回報（首選 門口）");
});

test("簡體模式按 Tab，字根提示對應繁體「門口」", async ({ page }) => {
	const { text, simplified, items } = await ready(page);
	await simplified.check();
	await text.focus();
	await page.keyboard.type("anr");
	await expect(items.first()).toHaveText("1门口");
	await page.keyboard.press("Tab");
	// 「門」的五代是 an，不是「门」的 lis。
	await expect(page.getByTestId("hint-1")).toHaveText("日弓(an) 口(r)");
});
