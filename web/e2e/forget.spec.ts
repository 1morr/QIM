import { expect, type Page, test } from "@playwright/test";

// 撤銷學習（M4 票 16，plan〈學習與排位〉）：組字中 Shift+Delete 或 Ctrl+Delete（librime 的綁定）清掉首選學到的，候選回到不學習時
// 的位置，debug 面板的學習加分歸零；再選兩次又成首選。沒有組字時 Shift+Delete 照常交還瀏覽器。

function ui(page: Page) {
	const debug = page.locator("#debug");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		debug,
		rows: debug.getByRole("table", { name: "候選來源" }).locator("tbody tr"),
		top: debug.getByTestId("debug-top"),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** 從空白的學習資料開始，關掉重排（候選位置只看學習），打開 debug 面板。 */
async function ready(page: Page) {
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
	const parts = ui(page);
	await expect(parts.text).toHaveAttribute("data-ready", "true");
	await page.getByRole("combobox", { name: "重排" }).selectOption("off");
	await page.getByRole("button", { name: "debug" }).click();
	await expect(parts.debug).toBeVisible();
	await parts.text.focus();
	return parts;
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
}

/** 打 `code`，按數字鍵選文字是 `word` 的候選。 */
async function choose(page: Page, code: string, word: string) {
	await page.keyboard.type(code);
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
}

/** debug 面板裡文字是 `word` 的候選的學習加分。 */
function learning(page: Page, word: string) {
	return ui(page).rows.and(page.locator(`[data-text="${word}"]`));
}

test("Shift+Delete 清掉學成首選的字：回到原位、學習加分歸零，重新整理後也是；再選兩次又成首選", async ({
	page,
}, testInfo) => {
	const { text, items, top } = await ready(page);
	await page.keyboard.type("a");
	const unlearned = await shown(page);
	const second = unlearned[1];
	expect(second, "a 要有第 2 個候選").toBeTruthy();
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
	expect(
		Number(await learning(page, second).getAttribute("data-learning")),
	).toBeGreaterThan(0);
	await page.screenshot({ path: testInfo.outputPath("forget-before.png") });

	await page.keyboard.press("Shift+Delete");
	await expect(items.first()).toHaveText(`1${unlearned[0]}`);
	expect(await shown(page)).toEqual(unlearned);
	await expect(learning(page, second)).toHaveAttribute("data-learning", "0.00");
	await expect(top).toHaveAttribute("data-learning", "0.00");
	await expect(text).toHaveValue(second + second);
	await page.screenshot({ path: testInfo.outputPath("forget-after.png") });

	// 存進學習資料：重新整理後一樣。
	await page.reload();
	await expect(text).toHaveAttribute("data-ready", "true");
	await page.getByRole("combobox", { name: "重排" }).selectOption("off");
	await text.focus();
	await page.keyboard.press("End");
	await page.keyboard.type("a");
	expect(await shown(page)).toEqual(unlearned);
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${unlearned[0]}`);
	await page.keyboard.press("Escape");
	await choose(page, "a", second);
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
});

test("Ctrl+Delete 也清首選（librime 的預設綁定）", async ({ page }) => {
	const { items } = await ready(page);
	await page.keyboard.type("a");
	const unlearned = await shown(page);
	await page.keyboard.press("Escape");
	await choose(page, "a", unlearned[1]);
	await choose(page, "a", unlearned[1]);
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${unlearned[1]}`);

	await page.keyboard.press("Control+Delete");
	await expect(items.first()).toHaveText(`1${unlearned[0]}`);
	expect(await shown(page)).toEqual(unlearned);
});

test("翻頁後清的是這頁的第一個，不是看不到的首選", async ({ page }) => {
	const { items } = await ready(page);
	// `ok` 的候選多於一頁（「a」只有兩個）。
	await page.keyboard.type("ok");
	const unlearned = await shown(page);
	await page.keyboard.press("Escape");
	await choose(page, "ok", unlearned[1]);
	await choose(page, "ok", unlearned[1]);
	await page.keyboard.type("ok");
	await expect(items.first()).toHaveText(`1${unlearned[1]}`);

	await page.keyboard.press("PageDown");
	await expect(items.first()).not.toHaveText(`1${unlearned[1]}`);
	await page.keyboard.press("Shift+Delete");
	await page.keyboard.press("PageUp");
	await expect(items.first()).toHaveText(`1${unlearned[1]}`);
	await page.keyboard.press("Escape");
	await page.keyboard.type("ok");
	await expect(items.first()).toHaveText(`1${unlearned[1]}`);
});

test("沒有組字時 Shift+Delete 交還瀏覽器，學習資料不動", async ({ page }) => {
	const { text, items } = await ready(page);
	await page.keyboard.type("a");
	const second = (await shown(page))[1];
	await page.keyboard.press("Escape");
	await choose(page, "a", second);
	await choose(page, "a", second);

	await page.keyboard.press("Shift+Delete");
	await expect(text).toHaveValue(second + second);
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
});
