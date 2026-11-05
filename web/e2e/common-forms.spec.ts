import { expect, type Page, test } from "@playwright/test";

/** 打 `code`，讀回組字的字根與首選，再按空白上屏。 */
async function expectTop(
	page: Page,
	code: string,
	radicals: string,
	top: string,
) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type(code);
	await expect(composition).toHaveText(radicals);
	await expect(items.first()).toHaveText(`1${top}`);

	await page.keyboard.type(" ");
	await expect(text).toHaveValue(top);
}

// 字形用常見寫法、不用香港寫法（brief §4.1，M1 票 17b 使用者決定）：語料換回常見寫法後，
// 同碼的「溫／温」「群／羣」由常見寫法當首選（香港寫法仍在碼表裡，字頻是 0）。
for (const { code, radicals, common, replaced } of [
	{ code: "et", radicals: "水廿", common: "溫", replaced: "温" },
	{ code: "sq", radicals: "尸手", common: "群", replaced: "羣" },
]) {
	test(`打 ${code} 首選是「${common}」，不是「${replaced}」`, async ({
		page,
	}) => {
		await expectTop(page, code, radicals, common);
	});
}

// 詞庫也用常見寫法（M1 票 17c）：rime-essay 的「羣組」換成「群組」；並存寫法打哪個碼出哪個：
// 雙向的「這裏／這裡」，單向的「隨着」另收「隨著」。
for (const { code, radicals, word } of [
	{ code: "sqvm", radicals: "尸手女一", word: "群組" },
	{ code: "yryv", radicals: "卜口卜女", word: "這裏" },
	{ code: "yrlg", radicals: "卜口中土", word: "這裡" },
	{ code: "nbtu", radicals: "弓月廿山", word: "隨着" },
	{ code: "nbta", radicals: "弓月廿日", word: "隨著" },
]) {
	test(`打 ${code} 首選是「${word}」`, async ({ page }) => {
		await expectTop(page, code, radicals, word);
	});
}
