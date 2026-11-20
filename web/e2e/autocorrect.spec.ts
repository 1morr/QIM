import { devices, expect, type Page, test } from "@playwright/test";
import { ready, ui } from "./page.ts";

// 英文自動修正（M4 票 25，brief〈英文模式與欄位類型〉）：預設關；設定打開後，英文模式打完一個詞（空白、句讀、
// Enter）時換掉拼錯的詞，緊接著退格還原、之後不再修它。英文容錯（預設開）讓建議列也列拼錯的修正，debug
// 面板讀得到會修正成什麼。用真的資料包（AOSP 英文詞表）。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await ready(page);
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** 逐字點觸控鍵盤：字母是 `key-<字母>`，空白是 `key-space`。 */
async function tapText(page: Page, typed: string): Promise<void> {
	for (const c of typed) {
		await page.getByTestId(c === " " ? "key-space" : `key-${c}`).tap();
	}
}

async function turnOnAutocorrect(page: Page): Promise<void> {
	await page
		.getByRole("region", { name: "設定" })
		.getByRole("checkbox", { name: "英文自動修正" })
		.check();
	await page.getByRole("textbox", { name: "輸入框" }).focus();
}

test("預設不修正 teh；建議列照樣列出修正 the", async ({ page }) => {
	const { text, items } = ui(page);
	await page.getByTestId("key-globe").tap();
	await tapText(page, "teh");
	await expect(items.first()).toHaveText("1the");
	await tapText(page, " ");
	await expect(text).toHaveValue("teh ");
});

test("打開自動修正後英文模式打 teh 修正成 the，退格還原，之後不再修", async ({
	page,
}, testInfo) => {
	const { text, items } = ui(page);
	await page.getByRole("button", { name: "debug" }).click();
	await page.getByTestId("key-globe").tap();
	await turnOnAutocorrect(page);

	await tapText(page, "teh");
	await expect(page.getByTestId("debug-autocorrection")).toHaveAttribute(
		"data-autocorrection",
		"the",
	);
	await expect(
		page.getByRole("table", { name: "候選來源" }).locator("tbody tr").first(),
	).toHaveAttribute("data-kind", "englishCorrection");
	await tapText(page, " ");
	await expect(text).toHaveValue("the ");
	await page.screenshot({
		path: testInfo.outputPath("25-autocorrect-the.png"),
		fullPage: true,
	});

	await page.getByTestId("key-backspace").tap();
	await expect(text).toHaveValue("teh");
	await expect(page.getByTestId("debug-autocorrection")).toHaveAttribute(
		"data-autocorrection",
		"",
	);
	await expect(items.first()).toHaveText("1the");
	await page.screenshot({
		path: testInfo.outputPath("25-autocorrect-reverted.png"),
		fullPage: true,
	});
	await tapText(page, " ");
	await expect(text).toHaveValue("teh ");
});

test("Enter 也觸發：換掉詞之後照樣換行", async ({ page }) => {
	const { text } = ui(page);
	await page.getByTestId("key-globe").tap();
	await turnOnAutocorrect(page);
	await tapText(page, "recieve");
	await page.getByTestId("key-enter").tap();
	await expect(text).toHaveValue("receive\n");
});

test("實體鍵盤的 Enter 在英文模式也先換掉詞再換行", async ({ page }) => {
	const { text } = ui(page);
	await page.getByTestId("key-globe").tap();
	await turnOnAutocorrect(page);
	await tapText(page, "recieve");
	await text.press("Enter");
	await expect(text).toHaveValue("receive\n");
});
