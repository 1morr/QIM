import { expect, type Page, test } from "@playwright/test";

// 重排的假實作（plan §3.1、ADR 0005，M1 票 23、M3 票 11c）：延遲後回傳 lattice 路徑的名次，核心
// 把排出的路徑放到第 2 位，候選列沒有就插入。過期的結果丟掉由核心測試驗（`crates/qim-core/tests/rerank.rs`）。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		all: page.getByRole("list", { name: "全部候選" }).getByRole("listitem"),
		rerank: page.getByRole("combobox", { name: "重排" }),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** `anr` 有超過 9 條整句路徑，候選列只放方案的幾條句子。 */
const CODE = "anr";

/** 重排關著時 `CODE` 的第一頁與全部候選（展開面板讀）。 */
async function unranked(
	page: Page,
): Promise<{ first: string[]; all: string[] }> {
	const { text, items, all, rerank } = ui(page);
	await rerank.selectOption("off");
	await text.focus();
	await page.keyboard.type(CODE);
	const first = (await items.allTextContents()).map((t) => t.slice(1));
	await page.keyboard.press("ArrowDown");
	await expect(all.first()).toBeVisible();
	const every = await all.allTextContents();
	await page.keyboard.press("Escape");
	await expect(text).toHaveValue("");
	return { first, all: every };
}

test("預設原樣回傳：等過延遲，候選順序不變", async ({ page }) => {
	const { text, items, rerank } = ui(page);
	const { first } = await unranked(page);
	await rerank.selectOption("identity");
	await text.focus();
	await page.keyboard.type(CODE);
	await page.waitForTimeout(600);
	await expect(items).toHaveText(first.map((t, i) => `${i + 1}${t}`));
});

test("把最後一條路徑排最前：候選列沒有它，插到第 2 位，首選不變", async ({
	page,
}, testInfo) => {
	const { text, items, rerank } = ui(page);
	const { first, all } = await unranked(page);
	await rerank.selectOption("raise-last");
	await text.focus();
	await page.keyboard.type(CODE);
	// 原本的第 2 個被擠到第 3：插入的路徑到了。
	await expect(items.nth(2)).toHaveText(`3${first[1]}`);
	await expect(items.first()).toHaveText(`1${first[0]}`);
	const inserted = ((await items.nth(1).textContent()) ?? "").slice(1);
	expect(all).not.toContain(inserted);
	await page.screenshot({ path: testInfo.outputPath("raise-last.png") });

	// 選它照那條路徑上屏。
	await page.keyboard.press("2");
	await expect(text).toHaveValue(inserted);
});
