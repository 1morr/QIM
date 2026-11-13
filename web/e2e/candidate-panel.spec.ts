import { devices, expect, type Page, test } from "@playwright/test";

// 候選展開面板（brief〈手機觸控鍵盤〉，M1 票 23）：候選列右側的展開鍵一次列全部候選，點了照常選字上屏。
// 桌面組字中按 ↓ 打開、↑ 收起。`ar` 有好幾頁候選。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		panel: page.getByRole("list", { name: "全部候選" }),
		all: page.getByRole("list", { name: "全部候選" }).getByRole("listitem"),
		expand: page.getByRole("button", { name: "展開候選" }),
		collapse: page.getByRole("button", { name: "收起候選" }),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

async function ready(page: Page): Promise<void> {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
}

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** 面板裡第一頁以外的第一個候選（全部候選的第 10 個），不在候選列上。 */
async function beyondFirstPage(page: Page): Promise<string> {
	const { all, items } = ui(page);
	expect(await all.count()).toBeGreaterThan(9);
	const tenth = (await all.nth(9).textContent()) ?? "";
	const bar = await items.allTextContents();
	expect(bar.map((t) => t.slice(1))).not.toContain(tenth);
	return tenth;
}

test.describe("實體鍵盤", () => {
	test.beforeEach(async ({ page }) => {
		await ready(page);
		await ui(page).text.focus();
	});

	test("組字中 ↓ 打開面板，點第一頁以外的候選上屏", async ({
		page,
	}, testInfo) => {
		const { text, composition, panel } = ui(page);
		await expect(ui(page).expand).toBeHidden();
		await page.keyboard.type("ar");
		await expect(panel).toBeHidden();
		await page.keyboard.press("ArrowDown");
		await expect(panel).toBeVisible();
		const tenth = await beyondFirstPage(page);
		await page.screenshot({ path: testInfo.outputPath("panel-desktop.png") });

		await ui(page).all.nth(9).click();
		await expect(text).toHaveValue(tenth);
		await expect(composition).toHaveText("");
		await expect(panel).toBeHidden();
		await expect(text).toBeFocused();
	});

	test("↑ 收起面板；展開鍵也能打開與收起；組字結束面板就收起", async ({
		page,
	}) => {
		const { panel, expand, collapse } = ui(page);
		await page.keyboard.type("ar");
		await page.keyboard.press("ArrowDown");
		await expect(panel).toBeVisible();
		await page.keyboard.press("ArrowUp");
		await expect(panel).toBeHidden();

		await expand.click();
		await expect(panel).toBeVisible();
		await expect(ui(page).text).toBeFocused();
		await collapse.click();
		await expect(panel).toBeHidden();

		await page.keyboard.press("ArrowDown");
		await page.keyboard.press("Escape");
		await expect(panel).toBeHidden();
		await expect(expand).toBeHidden();
	});

	test("面板開著時打字，面板跟著換成新的候選", async ({ page }) => {
		const { all } = ui(page);
		await page.keyboard.type("a");
		await page.keyboard.press("ArrowDown");
		await expect(all).toHaveText(["日", "曰"]);
		await page.keyboard.type("r");
		expect(await all.count()).toBeGreaterThan(9);
	});
});

test.describe("觸控", () => {
	// `defaultBrowserType` 只能在設定檔指定，這裡只取 viewport、觸控與 user agent。
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test.beforeEach(async ({ page }) => {
		await ready(page);
	});

	test("點展開鍵打開面板，點第一頁以外的候選上屏", async ({
		page,
	}, testInfo) => {
		const { text, composition, panel } = ui(page);
		for (const id of ["a", "r"]) {
			await page.getByTestId(`key-${id}`).tap();
		}
		await ui(page).expand.tap();
		await expect(panel).toBeVisible();
		const tenth = await beyondFirstPage(page);
		await page.screenshot({ path: testInfo.outputPath("panel-touch.png") });

		await ui(page).all.nth(9).tap();
		await expect(text).toHaveValue(tenth);
		await expect(composition).toHaveText("");
		await expect(panel).toBeHidden();
	});
});
