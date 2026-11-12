import { expect, type Page, test } from "@playwright/test";

// debug 面板（plan §3.1，M4 票 01）：預設關；開著時讀得到殼給的前文（有前文、不知道前文、無痕不讀）、
// 輸入碼的切段、首選的路徑分與學習加分、每個候選的來源，與停鍵重排的要求和結果。關著時網頁不問核心
// （`data-inspect-count` 不動）。

const EMAIL = "ivan.test@example.com";

function ui(page: Page) {
	// 關著時是 hidden，用角色找不到；`data-inspect-count` 要在關著時也讀得到。
	const debug = page.locator("#debug");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		toggle: page.getByRole("button", { name: "debug" }),
		debug,
		context: debug.getByTestId("debug-context"),
		contextText: debug.getByTestId("debug-context-text"),
		preceding: debug.getByTestId("debug-preceding"),
		segments: debug.getByTestId("debug-segments"),
		top: debug.getByTestId("debug-top"),
		rows: debug.getByRole("table", { name: "候選來源" }).locator("tbody tr"),
		request: debug.getByTestId("debug-rerank-request"),
		paths: debug.getByRole("table", { name: "重排的路徑" }).locator("tbody tr"),
		result: debug.getByTestId("debug-rerank-result"),
		rerank: page.getByRole("combobox", { name: "重排" }),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

async function ready(page: Page, url = "/") {
	await page.goto(url);
	await page.evaluate(() => localStorage.clear());
	await page.reload();
	const parts = ui(page);
	await expect(parts.text).toHaveAttribute("data-ready", "true");
	return parts;
}

/** 打開面板、焦點回輸入框。 */
async function openPanel(page: Page) {
	const { toggle, text, debug } = ui(page);
	await toggle.click();
	await expect(debug).toBeVisible();
	await text.focus();
}

/** 換掉輸入框的內容、游標放在結尾，像使用者貼上後點到那裡。 */
async function place(page: Page, value: string) {
	await ui(page).text.evaluate((element: HTMLTextAreaElement, value) => {
		element.value = value;
		element.dispatchEvent(new Event("input", { bubbles: true }));
		element.setSelectionRange(value.length, value.length);
		element.focus();
	}, value);
}

/** 面板候選表裡文字是 `text` 的那列。 */
function row(page: Page, text: string) {
	return ui(page).rows.filter({
		has: page.locator(`td:nth-child(2)`, { hasText: new RegExp(`^${text}$`) }),
	});
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
}

/** 按數字鍵選這頁文字是 `word` 的候選。 */
async function pick(page: Page, word: string) {
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
}

test.beforeEach(({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

// 守的是面板自己：關著時不呼叫 `Session.inspect()`。網頁只有 `debug-panel.ts` 呼叫它。
test("預設關，面板關著時不問核心；打開才問，關掉又不問", async ({ page }) => {
	const { text, toggle, debug, rerank } = await ready(page);
	await rerank.selectOption("off");
	await expect(debug).toBeHidden();
	await expect(toggle).toHaveAttribute("aria-pressed", "false");
	await text.focus();
	await page.keyboard.type("anr");
	await page.keyboard.press("Escape");
	await expect(debug).toHaveAttribute("data-inspect-count", "0");

	await openPanel(page);
	await expect(toggle).toHaveAttribute("aria-pressed", "true");
	await page.keyboard.type("anr");
	expect(
		Number(await debug.getAttribute("data-inspect-count")),
	).toBeGreaterThan(1);

	// 按鈕會讓輸入框失焦，失焦時面板還開著；關掉之後才記次數。
	await toggle.click();
	await expect(debug).toBeHidden();
	const asked = Number(await debug.getAttribute("data-inspect-count"));
	await text.focus();
	await page.keyboard.press("Escape");
	await page.keyboard.type("anr");
	await expect(debug).toHaveAttribute("data-inspect-count", String(asked));
});

test("前文的三種狀態：有前文、失焦後不知道前文、無痕不讀", async ({
	page,
}, testInfo) => {
	const { text, context, contextText, preceding } = await ready(page);
	await openPanel(page);
	await place(page, "你好，");
	await expect(context).toHaveAttribute("data-state", "known");
	await expect(contextText).toHaveText("前文：「你好，」");
	// 前文以標點結尾，組字開頭接句首。
	await page.keyboard.type("of");
	await expect(preceding).toHaveAttribute("data-kind", "clauseStart");
	await page.screenshot({ path: testInfo.outputPath("debug-context.png") });
	await page.keyboard.press("Escape");

	await text.blur();
	await expect(context).toHaveAttribute("data-state", "unknown");
	await expect(contextText).toHaveText("不知道前文");

	await page.getByRole("checkbox", { name: "無痕" }).check();
	await text.focus();
	await expect(context).toHaveAttribute("data-state", "incognito");
	await expect(contextText).toHaveText("無痕：不讀前文");
	await page.keyboard.type("of");
	await expect(preceding).toHaveAttribute("data-kind", "unigram");
});

test("切段與首選的路徑分；上屏一次後首選有學習加分", async ({ page }) => {
	const { text, segments, top } = await ready(page);
	await openPanel(page);
	await page.keyboard.type("anr");
	await expect(segments).toContainText("anr→門口");
	await expect(top).toContainText("首選「門口」");
	expect(Number(await top.getAttribute("data-score"))).toBeLessThan(0);
	await expect(top).toHaveAttribute("data-learning", "0.00");

	await page.keyboard.press("Space");
	await expect(text).toHaveValue("門口");
	await page.keyboard.press("Escape");
	await page.keyboard.type("anr");
	await expect(top).toContainText("首選「門口」");
	expect(Number(await top.getAttribute("data-learning"))).toBeGreaterThan(0);

	// 多段的切段：打一串連打。
	await page.keyboard.press("Escape");
	// continuous.spec 的連打：`anrkbo` 上屏時是兩段。
	await page.keyboard.type("anrkbo");
	await expect(segments.locator("code")).toHaveCount(2);
});

test("候選的來源每種都讀得到", async ({ page }, testInfo) => {
	const { text, rerank } = await ready(page);
	await rerank.selectOption("off");
	await openPanel(page);
	const sources = (word: string) => row(page, word).first();

	// 詞庫與碼表。
	await page.keyboard.type("anr");
	await expect(sources("門口")).toHaveAttribute("data-sources", /\blexicon\b/);
	await page.keyboard.press("Escape");
	await page.keyboard.type("a");
	await expect(sources("日")).toHaveAttribute("data-sources", /\bcodeTable\b/);
	await page.keyboard.press("Escape");

	// 英文。
	await page.keyboard.type("the");
	await expect(sources("the")).toHaveAttribute("data-kind", "english");
	await page.keyboard.press("Escape");

	// emoji。
	await page.keyboard.type("hk");
	await expect(row(page, "😆")).toHaveAttribute("data-kind", "emoji");
	await page.keyboard.press("Escape");

	// 使用者詞：分段選「雲」「口」上屏，「雲口」自動造詞。
	await page.keyboard.type("mir");
	await pick(page, "雲");
	await pick(page, "口");
	await expect(text).toHaveValue("雲口");
	await page.keyboard.press("Escape");
	await page.keyboard.type("mir");
	await expect(sources("雲口")).toHaveAttribute("data-sources", /\buserWord\b/);
	await page.keyboard.press("Escape");

	// 個人 bigram：前文「雲」後面選過「口」，之後同樣的前文後面「口」接得上。
	await place(page, "雲");
	await page.keyboard.type("r");
	await pick(page, "口");
	await expect(text).toHaveValue("雲口");
	await page.keyboard.press("Escape");
	await place(page, "雲");
	await page.keyboard.type("r");
	await expect(sources("口")).toHaveAttribute(
		"data-sources",
		/\bpersonalBigram\b/,
	);
	await page.keyboard.press("Escape");

	// 記住的字串：英文打一次 email，中文打開頭的字母。
	await place(page, "");
	await page.keyboard.press("Shift");
	await page.keyboard.type(`${EMAIL} `);
	await page.keyboard.press("Shift");
	await page.keyboard.type("iva");
	await expect(sources(EMAIL)).toHaveAttribute("data-kind", "remembered");
	await page.screenshot({ path: testInfo.outputPath("debug-sources.png") });
	await page.keyboard.press("Escape");

	// 重排插入：最後一條路徑提前，候選列沒有它，插到第 2 位。
	await rerank.selectOption("raise-last");
	await text.focus();
	await page.keyboard.type("anr");
	await expect(ui(page).rows.nth(1)).toHaveAttribute(
		"data-sources",
		/\brerankInserted\b/,
	);
});

test("停鍵後讀得到重排的要求與結果；神經模型有各路徑的總分", async ({
	page,
}, testInfo) => {
	const { text, request, paths, result, rerank, rows } = await ready(page);
	await rerank.selectOption("raise-last");
	await openPanel(page);
	await page.keyboard.type("anr");
	await expect(result).toHaveAttribute("data-outcome", "applied");
	await expect(request).toContainText("前文：「」");
	await expect(paths).toHaveCount(9);
	// 最後一條排最前：名次欄是 1。
	await expect(paths.nth(8).locator("td").nth(4)).toHaveText("1");
	await expect(result).toContainText("選了第 9 條");
	await expect(result).toContainText(" ms");
	// 選了哪條是那次結果的：之後翻頁、候選列看不到它，紀錄照舊。
	await page.keyboard.press("PageDown");
	await expect(result).toContainText("選了第 9 條");
	await page.keyboard.press("PageUp");
	await expect(rows.nth(1)).toHaveAttribute(
		"data-sources",
		/\brerankInserted\b/,
	);
	await page.keyboard.press("Escape");

	await rerank.selectOption("neural");
	await text.focus();
	await page.keyboard.type("fukjctd");
	// 先等到要求換成這串碼的（neural-rerank.spec 的首選「光大額葉」），上一次的「已套用」不算。
	await expect(paths.first()).toHaveAttribute("data-path", "光大額葉");
	await expect(result).toHaveAttribute("data-outcome", "applied");
	await expect(result).toContainText("選了第");
	const totals = await paths.locator("td:nth-child(4)").allTextContents();
	expect(totals.length).toBeGreaterThan(1);
	for (const total of totals) {
		expect(Number(total), `模型總分 ${total}`).toBeLessThan(0);
	}
	await page.screenshot({ path: testInfo.outputPath("debug-rerank.png") });
});

test("?rerank-delay：結果回來前候選變了，讀到作廢", async ({ page }) => {
	const { request, result, rerank } = await ready(page, "/?rerank-delay=600");
	await rerank.selectOption("raise-last");
	await openPanel(page);
	await page.keyboard.type("anr");
	await expect(request).toBeVisible();
	await expect(result).toContainText("模型算中");
	// 要求送出後、結果回來前清掉組字：候選變了，結果作廢；沒有組字就不再要求。
	await page.keyboard.press("Escape");
	await expect(result).toHaveAttribute("data-outcome", "stale", {
		timeout: 5000,
	});
	await expect(result).toContainText("作廢");
});

// 實際亂按時發現（票 01）：候選多的碼（`hk` 有三百多個）把整張表列出，手機上頁面長到上萬 px、鍵盤與設定
// 被推得很遠。面板自己最高半個畫面，內容在面板裡捲。
test("候選很多時面板不撐長頁面，最高半個畫面、在面板內捲", async ({ page }) => {
	await page.setViewportSize({ width: 412, height: 900 });
	const { debug, rows } = await ready(page);
	await openPanel(page);
	await page.keyboard.type("hk");
	await expect.poll(() => rows.count()).toBeGreaterThan(50);
	const box = await debug.boundingBox();
	expect(box?.height).toBeLessThanOrEqual(450);
	await expect(rows.first()).toBeVisible();
});
