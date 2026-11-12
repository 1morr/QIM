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

// 個人 bigram（M3 票 12，plan §2.3）：上屏時記「前一個字詞 → 這一段」，只在同樣的前文後面加分；
// 同碼的學習計數不看前文，所以用「前文不同、名次不同」把兩者分開。
test("在「我」後面選「你」一次，只有「我」後面的 of 把「你」抬高", async ({
	page,
}) => {
	const { text } = await ready(page);
	/** 把輸入框換成 `before`、游標放在結尾（像貼上後點到那裡），打 `of`，回傳「你」在候選的位置；再取消組字。 */
	const rankOfNi = async (before: string): Promise<number> => {
		await text.evaluate((element: HTMLTextAreaElement, value: string) => {
			element.value = value;
			element.dispatchEvent(new Event("input", { bubbles: true }));
			element.setSelectionRange(value.length, value.length);
		}, before);
		await page.keyboard.type("of");
		const at = (await shown(page)).indexOf("你");
		expect(at, `前文「${before}」的候選`).not.toBe(-1);
		await page.keyboard.press("Escape");
		return at;
	};

	// 學之前：前文「我」後面「你」排得比沒有前文時後面。
	const afterWoBefore = await rankOfNi("我");
	const emptyBefore = await rankOfNi("");
	expect(afterWoBefore).toBeGreaterThan(emptyBefore);

	// 前文「我」，打 of 選「你」一次；一次不足以讓同碼的計數把「你」抬成首選。
	await text.evaluate((element: HTMLTextAreaElement) => {
		element.value = "我";
		element.dispatchEvent(new Event("input", { bubbles: true }));
		element.setSelectionRange(1, 1);
	});
	await choose(page, "of", "你");
	await expect(text).toHaveValue("我你");

	// 學了之後：「我」後面「你」比沒有前文時前面，沒有前文時名次不變。
	const afterWo = await rankOfNi("我");
	const empty = await rankOfNi("");
	expect(afterWo).toBeLessThan(afterWoBefore);
	expect(afterWo).toBeLessThan(empty);
	expect(empty).toBe(emptyBefore);

	await rankOfNi("我");
	await page.keyboard.type("of");
	await expect(
		page.getByRole("list", { name: "候選" }).getByRole("listitem").nth(afterWo),
	).toHaveText(`${afterWo + 1}你`);
	await page.screenshot({
		path: `${import.meta.dirname}/../../.scratch/m3/evidence/23-personal-bigram.png`,
	});
});
