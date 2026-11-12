import { expect, type Page, test } from "@playwright/test";

// 停鍵結果快取（plan §2.7，M4 票 03）：同一串碼的重排結果留在核心，退格回到算過的那串碼時按鍵的快照
// 就排好了，不等停鍵、不再跑模型；debug 面板讀得到用的是快取。關掉重排、換模型後舊結果不再套用。

function ui(page: Page) {
	const debug = page.locator("#debug");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		list: page.getByRole("list", { name: "候選" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		rerank: page.getByRole("combobox", { name: "重排" }),
		toggle: page.getByRole("button", { name: "debug" }),
		debug,
		cache: debug.getByTestId("debug-rerank-cache"),
		result: debug.getByTestId("debug-rerank-result"),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
	const { text, toggle, debug } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await toggle.click();
	await expect(debug).toBeVisible();
	await text.focus();
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** 候選列目前的字（去掉序號）。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) => t.slice(1));
}

/** 交給模型的要求有幾次（debug 面板記的）。 */
async function requests(page: Page): Promise<number> {
	return Number(await ui(page).debug.getAttribute("data-rerank-requests"));
}

/** 等到第 `count` 次要求的結果回來。 */
async function resultOf(page: Page, count: number) {
	const { debug, result } = ui(page);
	await expect(debug).toHaveAttribute("data-rerank-requests", String(count));
	await expect(result).toHaveAttribute("data-outcome", /.+/);
}

/** 讀候選列，不等：按鍵後馬上讀到的就是按鍵的快照，停鍵 80 ms 的計時還沒到。 */
async function shownNow(page: Page): Promise<string[]> {
	return page.evaluate(() =>
		Array.from(
			document.querySelectorAll("#candidates li"),
			(li) => li.textContent?.slice(1) ?? "",
		),
	);
}

test("神經模型：打 hjtt 停鍵重排，多打一碼再退格回 hjtt，第 2 位不等停鍵就和先前相同、用的是快取", async ({
	page,
}, testInfo) => {
	const { cache, rerank, result } = ui(page);
	await expect(rerank).toHaveValue("neural");
	await page.keyboard.type("hjtt");
	await resultOf(page, 1);
	// 空前文下模型挑的「科藍」本來就在第 2 位（名次沒變）；名次真的變了的快取由下面 `raise-last` 的測試守。
	await expect(result).toHaveAttribute("data-outcome", /^(applied|unchanged)$/);
	const reranked = await shown(page);
	await expect(cache).toHaveAttribute("data-cached", "false");

	await page.keyboard.press("a");
	await resultOf(page, 2);
	await page.keyboard.press("Backspace");
	const back = await shownNow(page);
	expect(back[1]).toBe(reranked[1]);
	expect(back).toEqual(reranked);
	await expect(cache).toHaveAttribute("data-cached", "true");
	await page.waitForTimeout(400);
	expect(await requests(page)).toBe(2);
	expect(await shown(page)).toEqual(reranked);
	await page.screenshot({ path: testInfo.outputPath("rerank-cache.png") });
	await ui(page)
		.debug.getByTestId("debug-rerank")
		.screenshot({ path: testInfo.outputPath("rerank-cache-panel.png") });
});

/** 重排關著時 `code` 的第一頁。 */
async function unranked(page: Page, code: string): Promise<string[]> {
	const { text, rerank } = ui(page);
	const before = await rerank.inputValue();
	await rerank.selectOption("off");
	await text.focus();
	await page.keyboard.type(code);
	const first = await shown(page);
	await page.keyboard.press("Escape");
	await expect(text).toHaveValue("");
	await rerank.selectOption(before);
	await text.focus();
	return first;
}

/** 假實作 `raise-last` 把 `anr` 的最後一條路徑插到第 2 位，再多打一碼、等它的結果回來。 */
async function raiseLastThenTypeMore(page: Page): Promise<string[]> {
	const { text, rerank, list } = ui(page);
	await rerank.selectOption("raise-last");
	await text.focus();
	await page.keyboard.type("anr");
	await resultOf(page, 1);
	await expect(list).toHaveAttribute("data-rerank-applied", "1");
	const reranked = await shown(page);
	await page.keyboard.press("a");
	await resultOf(page, 2);
	return reranked;
}

test("假實作也用快取：退格回到插入過的那串碼，馬上就插好", async ({ page }) => {
	const first = await unranked(page, "anr");
	const reranked = await raiseLastThenTypeMore(page);
	expect(reranked[1]).not.toBe(first[1]);
	await page.keyboard.press("Backspace");
	expect(await shownNow(page)).toEqual(reranked);
	await page.waitForTimeout(400);
	expect(await requests(page)).toBe(2);
	await expect(ui(page).cache).toHaveAttribute("data-cached", "true");
});

test("關掉重排後退格回同一串碼：第 2 位不變（不套用舊結果）", async ({
	page,
}) => {
	const first = await unranked(page, "anr");
	const reranked = await raiseLastThenTypeMore(page);
	expect(reranked[1]).not.toBe(first[1]);
	const { rerank, text, cache } = ui(page);
	await rerank.selectOption("off");
	await text.focus();
	await page.keyboard.press("Backspace");
	await page.waitForTimeout(400);
	expect(await shown(page)).toEqual(first);
	await expect(cache).toHaveAttribute("data-cached", "false");
});

test("換模型後退格回同一串碼：舊模型的結果不套用，新模型重算", async ({
	page,
}) => {
	const first = await unranked(page, "anr");
	await raiseLastThenTypeMore(page);
	const { rerank, text, cache } = ui(page);
	await rerank.selectOption("identity");
	await text.focus();
	await page.keyboard.press("Backspace");
	expect(await shownNow(page)).toEqual(first);
	await expect(cache).toHaveAttribute("data-cached", "false");
	// 原樣回傳的新模型跑了一次（anr、anra 之後的第三次）。
	await resultOf(page, 3);
	expect(await shown(page)).toEqual(first);
});

// 舊模型的結果晚回來（網址的 `rerank-delay` 讓結果晚 1 秒）：換模式之後回來的作廢，不記到新模型名下，
// 退格回來不會套用它（code-review 找到）。
test("重排在途時換模式：舊結果作廢，退格回同一串碼不套用它", async ({
	page,
}) => {
	const first = await unranked(page, "anr");
	const { text, toggle, debug, rerank, result } = ui(page);
	await page.goto("/?rerank-delay=1000");
	await expect(text).toHaveAttribute("data-ready", "true");
	await toggle.click();
	await expect(debug).toBeVisible();
	await rerank.selectOption("raise-last");
	await text.focus();
	await page.keyboard.type("anr");
	await expect(debug).toHaveAttribute("data-rerank-requests", "1");
	await rerank.selectOption("identity");
	await text.focus();
	await expect(result).toHaveAttribute("data-outcome", "stale");
	expect(await shown(page)).toEqual(first);
	await page.keyboard.press("a");
	await page.keyboard.press("Backspace");
	expect(await shownNow(page)).toEqual(first);
	await expect(ui(page).cache).toHaveAttribute("data-cached", "false");
});
