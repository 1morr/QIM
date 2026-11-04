import { expect, type Page, test } from "@playwright/test";

// 組字中的字根與字根提示（M1 票 22，brief §4.3、§4.4）。
function ui(page: Page) {
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items,
		hint: page.getByTestId("hint-1"),
		version: page.getByRole("combobox", { name: "字根提示" }),
	};
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	const { text } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
});

test("打 anr 組字顯示「日弓口」，選定開頭一段後顯示字詞接其餘的字根", async ({
	page,
}, testInfo) => {
	const { composition, items } = ui(page);
	await page.keyboard.type("anr");
	await expect(composition).toHaveText("日弓口");
	await page.screenshot({ path: testInfo.outputPath("composition-anr.png") });

	const labels = await items.allTextContents();
	const men = labels.findIndex((label) => label.endsWith("門")) + 1;
	expect(men, `候選沒有「門」：${labels}`).toBeGreaterThan(0);
	await page.keyboard.press(String(men));
	await expect(composition).toHaveText("門口");
});

test("Tab 列出首選的倉頡五代碼，改三代後列三代碼，再按 Tab 收起", async ({
	page,
}, testInfo) => {
	const { text, items, hint, version } = ui(page);
	// 「俾」的五代是 ohwj、三代是 ohhj（libcangjie）。
	await page.keyboard.type("oj");
	await expect(items.first()).toHaveText("1俾");
	await expect(hint).toHaveCount(0);

	await page.keyboard.press("Tab");
	await expect(text).toBeFocused();
	await expect(hint).toHaveText("人竹田十(ohwj)");
	await page.screenshot({ path: testInfo.outputPath("hint-cangjie5.png") });

	await version.selectOption("cangjie3");
	await expect(hint).toHaveText("人竹竹十(ohhj)");
	await page.screenshot({ path: testInfo.outputPath("hint-cangjie3.png") });

	await text.focus();
	await page.keyboard.press("Tab");
	await expect(hint).toHaveCount(0);
});

test("詞逐字列出提示；組字結束就收起", async ({ page }) => {
	const { text, items, hint } = ui(page);
	await page.keyboard.type("anr");
	await expect(items.first()).toHaveText("1門口");
	await page.keyboard.press("Tab");
	await expect(hint).toHaveText("日弓(an) 口(r)");

	await page.keyboard.press("Escape");
	await page.keyboard.type("anr");
	await expect(hint).toHaveCount(0);
	await page.keyboard.press("Tab");
	await page.keyboard.type(" ");
	await expect(text).toHaveValue("門口");
	await page.keyboard.type("a");
	await expect(hint).toHaveCount(0);
});

test("沒有組字時 Tab 交還瀏覽器：焦點離開輸入框", async ({ page }) => {
	const { text } = ui(page);
	await page.keyboard.press("Tab");
	await expect(text).not.toBeFocused();
});

test("長按候選列出字根提示，點一下不會", async ({ page }) => {
	const { items, hint } = ui(page);
	await page.keyboard.type("oj");
	await items.first().click();
	await expect(hint).toHaveCount(0);

	const box = await items.first().boundingBox();
	if (!box) {
		throw new Error("候選沒有畫出來");
	}
	await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
	await page.mouse.down();
	await expect(hint).toHaveText("人竹田十(ohwj)");
	await page.mouse.up();
});
