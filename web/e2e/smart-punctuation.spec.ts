import { devices, expect, type Page, test } from "@playwright/test";

// 智慧標點（M4 票 27，brief〈英文模式與欄位類型〉）：英文模式字後連按兩下空白輸出「. 」（照 Gboard、LatinIME，
// 1100 ms 內），中文狀態實體鍵盤的 " ' 左右輪流出 “” ‘’（使用者定，照 Rime）。兩個各有開關、預設開。
// 數字後半形在 `digit-separators.spec.ts`。

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

async function ready(page: Page) {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	const settings = page.getByRole("region", { name: "設定" });
	return {
		text,
		doubleSpace: settings.getByRole("checkbox", { name: "兩下空白輸出句點" }),
		quotes: settings.getByRole("checkbox", { name: "引號左右輪流" }),
	};
}

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** 游標的位置（`selectionStart`、`selectionEnd`）。 */
async function cursor(page: Page): Promise<[number, number]> {
	return page
		.getByRole("textbox", { name: "輸入框" })
		.evaluate((el: HTMLTextAreaElement) => [
			el.selectionStart,
			el.selectionEnd,
		]);
}

test.describe("連按兩下空白", () => {
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	/** 逐字點觸控鍵盤：字母是 `key-<字母>`，空白是 `key-space`。 */
	async function tapText(page: Page, typed: string): Promise<void> {
		for (const c of typed) {
			await page.getByTestId(c === " " ? "key-space" : `key-${c}`).tap();
		}
	}

	test("英文模式字後連按兩下空白讀回「. 」，下一個字接在後面", async ({
		page,
	}, testInfo) => {
		const { text, doubleSpace } = await ready(page);
		await expect(doubleSpace).toBeChecked();
		await page.getByTestId("key-globe").tap();
		await tapText(page, "hi  ");
		await expect(text).toHaveValue("hi. ");
		expect(await cursor(page)).toEqual([4, 4]);
		await page.screenshot({
			path: testInfo.outputPath("27-double-space-period.png"),
			fullPage: true,
		});
		await tapText(page, "ok   ");
		await expect(text).toHaveValue("hi. ok.  ");
	});

	test("兩下之間隔超過 1.1 秒照打兩個空白", async ({ page }) => {
		const { text } = await ready(page);
		await page.getByTestId("key-globe").tap();
		await tapText(page, "hi ");
		await page.waitForTimeout(1500);
		await tapText(page, " ");
		await expect(text).toHaveValue("hi  ");
	});

	test("兩下之間點了輸入框（可能移了游標）照打兩個空白", async ({ page }) => {
		const { text } = await ready(page);
		await page.getByTestId("key-globe").tap();
		await tapText(page, "hi ");
		await text.tap();
		await tapText(page, " ");
		await expect(text).toHaveValue("hi  ");
	});

	test("關掉開關後照打兩個空白", async ({ page }, testInfo) => {
		const { text, doubleSpace } = await ready(page);
		await doubleSpace.uncheck();
		await page.getByTestId("key-globe").tap();
		await tapText(page, "hi  ");
		await expect(text).toHaveValue("hi  ");
		await page.screenshot({
			path: testInfo.outputPath("27-double-space-off.png"),
			fullPage: true,
		});
	});

	test("中文狀態連按兩下空白不換句點", async ({ page }) => {
		const { text } = await ready(page);
		await text.focus();
		await page.keyboard.type("a  ");
		await expect(text).toHaveValue("日 ");
		await page.keyboard.type(" ");
		await expect(text).toHaveValue("日  ");
	});
});

test.describe("引號左右輪流", () => {
	test('中文狀態的 " \' 各自輪流出 “” ‘’，游標在引號後；組字中 " 先上屏首選', async ({
		page,
	}, testInfo) => {
		const { text, quotes } = await ready(page);
		await expect(quotes).toBeChecked();
		await text.focus();
		// 中文模式照實體鍵位與 Shift 換成美式字元（票 07），`"` 要用 Shift 加鍵送。
		const doubleQuote = () => page.keyboard.press("Shift+Quote");
		await doubleQuote();
		await page.keyboard.type("'");
		await expect(text).toHaveValue("“‘");
		expect(await cursor(page)).toEqual([2, 2]);
		await page.keyboard.type("a '");
		await doubleQuote();
		await expect(text).toHaveValue("“‘日’”");
		expect(await cursor(page)).toEqual([5, 5]);
		await page.keyboard.type("a");
		await doubleQuote();
		await expect(text).toHaveValue("“‘日’”日“");
		await page.screenshot({
			path: testInfo.outputPath("27-paired-quotes.png"),
			fullPage: true,
		});
	});

	test("組字中 ' 照舊是分隔記號", async ({ page }) => {
		const { text } = await ready(page);
		await text.focus();
		await page.keyboard.type("mi'r");
		await expect(page.getByTestId("composition")).toContainText("'");
		await expect(text).toHaveValue("");
	});

	test("關掉開關後引號上屏半形", async ({ page }, testInfo) => {
		const { text, quotes } = await ready(page);
		await quotes.uncheck();
		await text.focus();
		await page.keyboard.press("Shift+Quote");
		await page.keyboard.type("'");
		await page.keyboard.press("Shift+Quote");
		await expect(text).toHaveValue('"\'"');
		await page.screenshot({
			path: testInfo.outputPath("27-paired-quotes-off.png"),
			fullPage: true,
		});
	});

	test("英文直通的引號照原樣", async ({ page }) => {
		const { text } = await ready(page);
		await text.focus();
		await page.keyboard.press("Shift");
		await page.keyboard.type('"hi"');
		await expect(text).toHaveValue('"hi"');
	});
});
