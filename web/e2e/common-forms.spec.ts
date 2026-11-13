import { expect, type Page, test } from "@playwright/test";

/** 打 `code`，讀回組字的字根；交出輸入框與候選。 */
async function typeCode(page: Page, code: string, radicals: string) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type(code);
	await expect(page.getByTestId("composition")).toHaveText(radicals);
	return { text, items };
}

/** 打 `code`，讀回組字的字根與首選，再按空白上屏。 */
async function expectTop(
	page: Page,
	code: string,
	radicals: string,
	top: string,
) {
	const { text, items } = await typeCode(page, code, radicals);
	await expect(items.first()).toHaveText(`1${top}`);

	await page.keyboard.type(" ");
	await expect(text).toHaveValue(top);
}

// 字形用常見寫法、不用香港寫法（brief〈字集與字形〉，M1 票 17b 使用者決定）：語料換回常見寫法後，
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

/** 打 `code`，讀回組字的字根與第一頁的候選（去掉前面的序號）。 */
async function firstPage(page: Page, code: string, radicals: string) {
	const { items } = await typeCode(page, code, radicals);
	await expect(items.first()).toBeVisible();
	return (await items.allTextContents()).map((t) => t.replace(/^\d/, ""));
}

// 票 17d：t2hk 換掉 essay 的「鉤」→「鈎」補成雙向，打「鉤」的碼出得了「鉤子」；「臺」「菸」單向，
// 語料照原文保留，「臺」有自己的字頻（改前 gg 排第 15）、「臺灣」在自己的碼上，「台州」不生出「臺州」。
test("打 crnd 第一頁有「鉤子」", async ({ page }) => {
	expect(await firstPage(page, "crnd", "金口弓木")).toContain("鉤子");
});

test("打 gg 第一頁有「臺」", async ({ page }) => {
	expect(await firstPage(page, "gg", "土土")).toContain("臺");
});

test("打 ggil 不出「臺州」", async ({ page }) => {
	expect(await firstPage(page, "ggil", "土土戈中")).not.toContain("臺州");
});

for (const { code, radicals, word } of [
	{ code: "ggen", radicals: "土土水弓", word: "臺灣" },
	{ code: "iren", radicals: "戈口水弓", word: "台灣" },
	{ code: "haty", radicals: "竹日廿卜", word: "香菸" },
]) {
	test(`打 ${code} 首選是「${word}」`, async ({ page }) => {
		await expectTop(page, code, radicals, word);
	});
}
