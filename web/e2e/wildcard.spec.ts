import { devices, expect, type Page, test } from "@playwright/test";

// 萬用鍵（brief §1.5、plan §2.5，M4 票 11）：速成的方案檔設 `z` 為萬用鍵，組字裡顯示「＊」，
// 每個 `z` 代任一個碼，放在字的碼的任一位置都行。「的人」是 `hi` + `o`，`hiz`、`hzo` 的首選
// 照樣是的人（`zio` 不是：一碼字的位置對得上的字太多，首選是「一次」）。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

test.describe("實體鍵盤", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
		await expect(ui(page).text).toHaveAttribute("data-ready", "true");
		await ui(page).text.focus();
	});

	test("hiz、hzo 的首選是的人，組字顯示＊，空白上屏", async ({
		page,
	}, testInfo) => {
		const { text, composition, items } = ui(page);
		for (const [typed, radicals] of [
			["hiz", "竹戈＊"],
			["hzo", "竹＊人"],
		]) {
			await page.keyboard.type(typed);
			await expect(composition).toHaveText(radicals);
			await expect(items.first()).toHaveText("1的人");
			if (typed === "hiz") {
				await page.screenshot({ path: testInfo.outputPath("hiz.png") });
			}
			await page.keyboard.press("Escape");
		}
		await page.keyboard.type("hiz ");
		await expect(text).toHaveValue("的人");
		await expect(composition).toHaveText("");
	});

	test("debug 面板：萬用鍵的候選標「萬用鍵」，切段附實際的碼", async ({
		page,
	}, testInfo) => {
		const debug = page.locator("#debug");
		await page.getByRole("button", { name: "debug" }).click();
		await expect(debug).toBeVisible();
		await ui(page).text.focus();
		await page.keyboard.type("hiz");
		await expect(ui(page).items.first()).toHaveText("1的人");
		const rows = debug
			.getByRole("table", { name: "候選來源" })
			.locator("tbody tr");
		const top = rows.filter({
			has: page.locator("td:nth-child(2)", { hasText: /^的人$/ }),
		});
		await expect(top).toHaveAttribute("data-sources", /\bwildcard\b/);
		await expect(top.locator("td").nth(2)).toContainText("萬用鍵");
		await expect(top.locator("td").nth(5)).toHaveText("hiz（hio）→的人");
		await page.screenshot({ path: testInfo.outputPath("debug-panel.png") });
		await page.keyboard.press("Escape");
		await page.keyboard.type("hio");
		await expect(top).not.toHaveAttribute("data-sources", /\bwildcard\b/);
	});

	test("沒有組字時 z 開始組字，Enter 原樣上屏 z", async ({ page }) => {
		const { text, composition } = ui(page);
		await page.keyboard.type("z");
		await expect(composition).toHaveText("＊");
		await expect(text).toHaveValue("");
		await page.keyboard.press("Enter");
		await expect(text).toHaveValue("z");
	});
});

test.describe("觸控", () => {
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test.beforeEach(async ({ page }) => {
		await page.goto("/");
		await expect(ui(page).text).toHaveAttribute("data-ready", "true");
	});

	test("點「重」鍵（z）當萬用鍵：h、z、o 首選的人", async ({ page }) => {
		const { text, composition, items } = ui(page);
		for (const id of ["h", "z", "o"]) {
			await page.getByTestId(`key-${id}`).tap();
		}
		await expect(composition).toHaveText("竹＊人");
		await expect(items.first()).toHaveText("1的人");
		await page.getByTestId("key-space").tap();
		await expect(text).toHaveValue("的人");
	});
});
