import { expect, type Page, test } from "@playwright/test";

// 學習（M1 票 13）：同一個碼選同一個非首選兩次，它就成首選；無痕模式下不學。
// 學習開關（M3 票 12）：關掉時不學，已學的照用。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		incognito: page.getByRole("checkbox", { name: "無痕" }),
		learning: page.getByRole("checkbox", { name: "學習" }),
	};
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	return (await items.allTextContents()).map((t) => t.replace(/^\d/, ""));
}

/** 打 `code`，按數字鍵選文字是 `word` 的候選。 */
async function choose(page: Page, code: string, word: string) {
	await page.keyboard.type(code);
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
}

test("同一個碼選同一個非首選兩次，它成首選", async ({ page }, testInfo) => {
	const { text, items } = await ready(page);
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	expect(second, "a 要有第 2 個候選").toBeTruthy();
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);

	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
	expect(second).not.toBe(top);
	await page.screenshot({ path: testInfo.outputPath("learned-top.png") });
});

test("開無痕後同樣選兩次，首選不變", async ({ page }, testInfo) => {
	const { text, items, incognito } = await ready(page);
	await incognito.check();
	await expect(text).toBeFocused();
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);

	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${top}`);
	await page.screenshot({ path: testInfo.outputPath("incognito-top.png") });
});

test("關掉學習後選兩次同一候選，它不成首選", async ({ page }, testInfo) => {
	const { text, items, learning } = await ready(page);
	await learning.uncheck();
	await expect(text).toBeFocused();
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	await page.keyboard.press("Escape");

	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);

	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${top}`);
	await page.screenshot({ path: testInfo.outputPath("learning-off-top.png") });
});

test("學過的在關掉學習後照用", async ({ page }) => {
	const { text, items, learning } = await ready(page);
	await page.keyboard.type("a");
	const [, second] = await shown(page);
	await page.keyboard.press("Escape");
	await choose(page, "a", second);
	await choose(page, "a", second);

	await learning.uncheck();
	await expect(text).toBeFocused();
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
});

/** 把輸入框換成 `before`、游標放在結尾（像貼上後點到那裡）。 */
async function placeBefore(page: Page, before: string) {
	await page
		.getByRole("textbox", { name: "輸入框" })
		.evaluate((element: HTMLTextAreaElement, value: string) => {
			element.value = value;
			element.dispatchEvent(new Event("input", { bubbles: true }));
			element.setSelectionRange(value.length, value.length);
		}, before);
}

/** 前文 `before` 後打 `of`，回傳「你」在候選的位置；再取消組字。 */
async function rankOfNiAfter(page: Page, before: string): Promise<number> {
	await placeBefore(page, before);
	await page.keyboard.type("of");
	const at = (await shown(page)).indexOf("你");
	expect(at, `前文「${before}」的候選`).not.toBe(-1);
	await page.keyboard.press("Escape");
	return at;
}

// 個人 bigram（M3 票 12，plan〈排序：同一把尺〉）：上屏時記「前一個字詞 → 這一段」，只在同樣的前文後面加分。
// 選一次「你」也會加同碼的學習計數（不看前文），所以開兩個乾淨的瀏覽器狀態，一個在「我」後面選、一個在
// 「他」後面選：兩邊的同碼計數一樣，「我」後面的名次差別只來自 bigram。
test("在「我」後面選「你」一次，「我」後面的 of 比在「他」後面選時把「你」排得更前", async ({
	page,
	browser,
}, testInfo) => {
	/** 新頁面在前文 `before` 後打 of 選「你」一次，回傳之後前文「我」時「你」的位置。 */
	const learnAfter = async (target: Page, before: string): Promise<number> => {
		const { text } = await ready(target);
		await placeBefore(target, before);
		await choose(target, "of", "你");
		await expect(text).toHaveValue(`${before}你`);
		return rankOfNiAfter(target, "我");
	};

	await ready(page);
	const beforeLearning = await rankOfNiAfter(page, "我");
	const afterMe = await learnAfter(page, "我");

	const other = await browser.newContext();
	const afterHe = await learnAfter(await other.newPage(), "他");
	await other.close();

	expect(afterMe).toBeLessThan(afterHe);
	expect(afterHe).toBeLessThanOrEqual(beforeLearning);

	await placeBefore(page, "我");
	await page.keyboard.type("of");
	await expect(
		page.getByRole("list", { name: "候選" }).getByRole("listitem").nth(afterMe),
	).toHaveText(`${afterMe + 1}你`);
	// 截圖寫到 Playwright 的輸出目錄：M3 已封存，不再改寫 `.scratch/m3/evidence/` 的證據。
	await page.screenshot({ path: testInfo.outputPath("personal-bigram.png") });
});
