import { devices, expect, type Page, test } from "@playwright/test";

// 聯想（M3 票 13）：上屏之後候選列給接下來可能要打的字詞，數字鍵或點選選了就上屏、接著出下一輪；
// 空白不選、Esc 只收起、退格刪字並收起（M4 票 18c）、打字母照常組字，設定區的「聯想」關掉時沒有
// （plan §2.1）。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

/** 候選列各項的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	const labels = await ui(page).items.allTextContents();
	return labels.map((label) => label.replace(/^\d/, ""));
}

async function ready(page: Page) {
	await page.goto("/");
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
}

test.describe("實體鍵盤", () => {
	test.beforeEach(async ({ page }) => {
		await ready(page);
		await ui(page).text.focus();
	});

	test("上屏「日」→ 數字鍵選聯想「本」→ 再選聯想「人」", async ({
		page,
	}, testInfo) => {
		const { text, composition, items } = ui(page);
		await page.keyboard.type("a ");
		await expect(text).toHaveValue("日");
		await expect(composition).toHaveText("");
		await expect(items.first()).toHaveText("1本");
		await page.screenshot({
			path: testInfo.outputPath("desktop-after-ri.png"),
		});

		await page.keyboard.press("1");
		await expect(text).toHaveValue("日本");
		// 接著照「日本」補完。
		const afterBen = await shown(page);
		const ren = afterBen.indexOf("人");
		expect(ren, `聯想：${afterBen.join(" ")}`).not.toBe(-1);
		await page.screenshot({
			path: testInfo.outputPath("desktop-after-ben.png"),
		});

		await page.keyboard.press(String(ren + 1));
		await expect(text).toHaveValue("日本人");
		await expect(items.first()).toBeVisible();
		await page.screenshot({
			path: testInfo.outputPath("desktop-after-ren.png"),
		});
	});

	test("打字母時聯想消失、照常組字", async ({ page }) => {
		const { text, composition, items } = ui(page);
		await page.keyboard.type("an ");
		await expect(text).toHaveValue("門");
		await expect(items.filter({ hasText: "口" })).toHaveCount(1);
		await page.keyboard.type("a");
		await expect(composition).toHaveText("日");
		await expect(items.first()).toHaveText("1日");
		await page.keyboard.type(" ");
		await expect(text).toHaveValue("門日");
	});

	test("空白不選聯想、照打空格；Esc 只收起聯想，退格刪字並收起", async ({
		page,
	}, testInfo) => {
		const { text, items } = ui(page);
		await page.keyboard.type("a  ");
		await expect(text).toHaveValue("日 ");
		await expect(items).toHaveCount(0);

		await page.keyboard.type("a");
		await page.keyboard.press("Space");
		await expect(items.first()).toBeVisible();
		await page.keyboard.press("Escape");
		await expect(items).toHaveCount(0);
		await expect(text).toHaveValue("日 日");

		await page.keyboard.type("a ");
		await expect(items.first()).toBeVisible();
		await expect(text).toHaveValue("日 日日");
		await page.keyboard.press("Backspace");
		await expect(text).toHaveValue("日 日");
		await expect(items).toHaveCount(0);
		await page.screenshot({
			path: testInfo.outputPath("desktop-backspace-deletes.png"),
		});
	});

	test("設定區關掉「聯想」時上屏後沒有聯想，數字照打", async ({ page }) => {
		const { text, items } = ui(page);
		await page.getByRole("checkbox", { name: "聯想" }).uncheck();
		await text.focus();
		await page.keyboard.type("a ");
		await expect(text).toHaveValue("日");
		await expect(items).toHaveCount(0);
		await page.keyboard.type("1");
		await expect(text).toHaveValue("日1");
	});

	test("聯想顯示中關掉開關，聯想立刻收起", async ({ page }) => {
		const { text, items } = ui(page);
		await page.keyboard.type("a ");
		await expect(items.first()).toBeVisible();
		await page.getByRole("checkbox", { name: "聯想" }).uncheck();
		await expect(items).toHaveCount(0);
		await text.focus();
		await page.keyboard.type("1");
		await expect(text).toHaveValue("日1");
	});

	test("游標移走時聯想收起，數字照打在游標處", async ({ page }) => {
		const { text, items } = ui(page);
		await page.keyboard.type("a ");
		await expect(items.first()).toBeVisible();
		// 像滑鼠點到開頭：只有 selectionchange，沒有按鍵。
		await text.evaluate((el: HTMLTextAreaElement) =>
			el.setSelectionRange(0, 0),
		);
		await expect(items).toHaveCount(0);
		await page.keyboard.type("1");
		await expect(text).toHaveValue("1日");
	});

	test("無痕時上屏後沒有聯想", async ({ page }) => {
		const { text, items } = ui(page);
		await page.getByRole("checkbox", { name: "無痕" }).check();
		await text.focus();
		await page.keyboard.type("a ");
		await expect(text).toHaveValue("日");
		await expect(items).toHaveCount(0);
	});

	test("輸入框失焦時聯想收起", async ({ page }) => {
		const { text, items } = ui(page);
		await page.keyboard.type("a ");
		await expect(items.first()).toBeVisible();
		await text.blur();
		await expect(items).toHaveCount(0);
	});
});

test.describe("觸控", () => {
	// `defaultBrowserType` 只能在設定檔指定，這裡只取 viewport、觸控與 user agent。
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test.beforeEach(async ({ page }) => {
		await ready(page);
	});

	test("點 a、空白上屏「日」→ 點聯想「本」→ 再點聯想「人」", async ({
		page,
	}, testInfo) => {
		const { text, items } = ui(page);
		await page.getByTestId("key-a").tap();
		await page.getByTestId("key-space").tap();
		await expect(text).toHaveValue("日");
		await expect(items.first()).toHaveText("1本");
		await page.screenshot({ path: testInfo.outputPath("touch-after-ri.png") });

		await items.filter({ hasText: /^\d本$/ }).tap();
		await expect(text).toHaveValue("日本");
		await page.screenshot({ path: testInfo.outputPath("touch-after-ben.png") });

		await items.filter({ hasText: /^\d人$/ }).tap();
		await expect(text).toHaveValue("日本人");
		await page.screenshot({ path: testInfo.outputPath("touch-after-ren.png") });
	});

	test("觸控的刪除鍵刪字並收起聯想，空白照打空格", async ({
		page,
	}, testInfo) => {
		const { text, items } = ui(page);
		await page.getByTestId("key-a").tap();
		await page.getByTestId("key-space").tap();
		await page.getByTestId("key-a").tap();
		await page.getByTestId("key-space").tap();
		await expect(text).toHaveValue("日日");
		await expect(items.first()).toBeVisible();
		await page.getByTestId("key-backspace").tap();
		await expect(text).toHaveValue("日");
		await expect(items).toHaveCount(0);
		await page.screenshot({
			path: testInfo.outputPath("touch-backspace-deletes.png"),
		});

		await page.getByTestId("key-a").tap();
		await page.getByTestId("key-space").tap();
		await expect(items.first()).toBeVisible();
		await page.getByTestId("key-space").tap();
		await expect(items).toHaveCount(0);
		await expect(text).toHaveValue("日日 ");
	});
});
