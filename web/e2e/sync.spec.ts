import { readFile } from "node:fs/promises";
import {
	type Browser,
	type BrowserContext,
	expect,
	type Page,
	test,
} from "@playwright/test";

// 跨裝置同步（M4 票 17，plan〈學習與排位〉〈跨裝置合併〉）：網頁沒有傳輸層，兩個瀏覽器設定檔（各自的 localStorage 與
// 裝置 id）的同步就是各自匯出、匯入對方的檔（匯入是合併）。次數各台分開記、合計時相加；刪除留墓碑傳到別台。

/** 一個瀏覽器設定檔：自己的 context（localStorage 分開）與頁面。 */
interface Profile {
	context: BrowserContext;
	page: Page;
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(() => {
	errors = [];
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

/** 開一個新的設定檔：空白的學習資料、關掉重排（候選位置只看學習）、打開 debug 面板。 */
async function profile(browser: Browser): Promise<Profile> {
	const context = await browser.newContext();
	const page = await context.newPage();
	page.on("pageerror", (error) => errors.push(error));
	await page.clock.setFixedTime(new Date(2026, 10, 13, 12, 0));
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await page.getByRole("combobox", { name: "重排" }).selectOption("off");
	await page.getByRole("button", { name: "debug" }).click();
	await expect(page.locator("#debug")).toBeVisible();
	await text.focus();
	return { context, page };
}

function ui(page: Page) {
	const debug = page.locator("#debug");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		top: debug.getByTestId("debug-top"),
		rows: debug.getByRole("table", { name: "候選來源" }).locator("tbody tr"),
		status: page.getByRole("status", { name: "學習資料狀態" }),
	};
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
}

/** 打 `code`，按數字鍵選文字是 `word` 的候選，清掉聯想。 */
async function choose(page: Page, code: string, word: string) {
	await page.keyboard.type(code);
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
	await page.keyboard.press("Escape");
}

/** 打 `code` 看首選（之後清掉組字）。 */
async function topOf(page: Page, code: string): Promise<string> {
	await page.keyboard.type(code);
	const first = (await shown(page))[0];
	await page.keyboard.press("Escape");
	return first;
}

/** 按「匯出學習資料」下載的檔的路徑與內容。 */
async function exportFile(page: Page): Promise<{ path: string; text: string }> {
	const download = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	const path = await (await download).path();
	await ui(page).text.focus();
	return { path, text: await readFile(path, "utf8") };
}

/** `into` 匯入 `from` 匯出的檔並確定（合併），回傳確認畫面的說明。 */
async function syncFrom(into: Page, from: Page): Promise<string> {
	const { path } = await exportFile(from);
	await into.getByLabel("匯入學習資料（合併進目前的）").setInputFiles(path);
	const confirm = into.getByRole("group", { name: "確認匯入" });
	await expect(confirm).toBeVisible();
	const said = (await confirm.textContent()) ?? "";
	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await expect(ui(into).status).toHaveText(/^已合併/);
	await ui(into).text.focus();
	await into.keyboard.press("End");
	return said;
}

/** 學習資料：匯出檔去掉這台的裝置 id（兩台的 id 不同，學到的東西要一樣）。 */
async function learning(page: Page): Promise<string> {
	const { text } = await exportFile(page);
	return text
		.split("\n")
		.filter((line) => !line.startsWith("#@/device\t"))
		.join("\n");
}

/** `a` 第 2 位的候選（不學習時）。 */
async function secondOfA(
	page: Page,
): Promise<{ first: string; second: string }> {
	await page.keyboard.type("a");
	const [first, second] = await shown(page);
	await page.keyboard.press("Escape");
	expect(second, "a 要有第 2 個候選").toBeTruthy();
	return { first, second };
}

test("兩個瀏覽器設定檔各選一次同一個候選，互相匯出匯入後它成首選", async ({
	browser,
}, testInfo) => {
	const a = await profile(browser);
	const b = await profile(browser);
	const { first, second } = await secondOfA(a.page);
	for (const { page } of [a, b]) {
		await choose(page, "a", second);
		expect(await topOf(page, "a"), "各選一次還不前移").toBe(first);
	}

	const said = await syncFrom(b.page, a.page);
	expect(said).toMatch(/會合併進目前的學習資料：新增 \d+ 筆、更新 [1-9]\d* 筆/);
	await syncFrom(a.page, b.page);
	for (const { page } of [a, b]) {
		await page.keyboard.type("a");
		await expect(ui(page).items.first()).toHaveText(`1${second}`);
		// debug 面板讀回原因：兩台各一次合計兩次，排位規則 2 排到首選。
		await expect(ui(page).top).toContainText(second);
		await expect(ui(page).top).toContainText("排位規則 2");
		await page.screenshot({
			path: testInfo.outputPath(
				`synced-top-${page === a.page ? "a" : "b"}.png`,
			),
		});
		await page.keyboard.press("Escape");
	}
	expect(await learning(a.page)).toBe(await learning(b.page));

	// 重新整理後（讀回 localStorage 存的檔）照樣是首選。
	await b.page.reload();
	await expect(ui(b.page).text).toHaveAttribute("data-ready", "true");
	await b.page.getByRole("combobox", { name: "重排" }).selectOption("off");
	await ui(b.page).text.focus();
	expect(await topOf(b.page, "a")).toBe(second);
	await a.context.close();
	await b.context.close();
});

test("一個新使用者詞與一筆刪除同步後兩邊讀回一致", async ({
	browser,
}, testInfo) => {
	const a = await profile(browser);
	const b = await profile(browser);
	const { first, second } = await secondOfA(a.page);
	await choose(a.page, "a", second);
	await choose(a.page, "a", second);
	await syncFrom(b.page, a.page);
	expect(await topOf(b.page, "a")).toBe(second);

	// B 刪掉（撤銷學習）學成首選的字；A 加一個手動詞。
	await b.page.keyboard.type("a");
	await b.page.keyboard.press("Shift+Delete");
	await expect(ui(b.page).items.first()).toHaveText(`1${first}`);
	await b.page.keyboard.press("Escape");
	await a.page
		.getByRole("textbox", { name: "詞", exact: true })
		.fill("速成輸入法");
	await a.page
		.getByRole("textbox", { name: "觸發詞", exact: true })
		.fill("qim");
	await a.page.getByRole("button", { name: "加詞" }).click();
	await expect(ui(a.page).status).toHaveText("已加入「速成輸入法」");

	const toA = await syncFrom(a.page, b.page);
	expect(toA).toMatch(/刪除 [1-9]\d* 筆（對方刪掉的）/);
	await syncFrom(b.page, a.page);
	for (const { page } of [a, b]) {
		// 清掉輸入框：沒有前文，學習加分只看那一列的計數（個人 bigram 只清刪除時前文那組，M4 票 16）。
		await ui(page).text.fill("");
		await page.keyboard.type("a");
		await expect(ui(page).items.first()).toHaveText(`1${first}`);
		await expect(
			ui(page).rows.and(page.locator(`[data-text="${second}"]`)),
		).toHaveAttribute("data-learning", "0.00");
		await page.keyboard.press("Escape");
		await page.keyboard.type("qim");
		await expect(ui(page).items.first()).toHaveText("1速成輸入法");
		await page.keyboard.press("Escape");
	}
	await b.page.screenshot({ path: testInfo.outputPath("synced-deletion.png") });
	const [learnedA, learnedB] = [await learning(a.page), await learning(b.page)];
	expect(learnedA).toBe(learnedB);
	// 墓碑只存雜湊：刪掉的字不在檔裡當成一列（只剩刪除的條目）。
	expect(learnedA).toMatch(/\ndeleted\tentry\tquick\t[0-9a-f]{32}\t/);
	expect(learnedA).not.toContain(`entry\tquick\t${second}\ta\t`);
	await a.context.close();
	await b.context.close();
});

// 清空學習資料要同步到別台（使用者 2026-11-13 決定）：清空留清空記號，匯入清空前的檔不收；別台匯入清空後的
// 檔，清空前學到的也清掉。
test("A 清空後匯入 B 的舊檔仍是空的；B 匯入 A 的檔後，清空前學的也清掉", async ({
	browser,
}, testInfo) => {
	const a = await profile(browser);
	const b = await profile(browser);
	const { first, second } = await secondOfA(a.page);
	for (const { page } of [a, b]) {
		await choose(page, "a", second);
	}
	await syncFrom(a.page, b.page);
	await syncFrom(b.page, a.page);
	expect(await topOf(a.page, "a")).toBe(second);
	const oldB = await exportFile(b.page);

	await a.page.getByRole("button", { name: "清空學習資料" }).click();
	await expect(ui(a.page).status).toHaveText("已清空學習資料");
	await ui(a.page).text.focus();
	await a.page
		.getByLabel("匯入學習資料（合併進目前的）")
		.setInputFiles(oldB.path);
	const confirm = a.page.getByRole("group", { name: "確認匯入" });
	await expect(confirm).toContainText("新增 0 筆、更新 0 筆、刪除 0 筆");
	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await ui(a.page).text.fill("");
	expect(await topOf(a.page, "a"), "A 仍是空的").toBe(first);
	const cleared = (await exportFile(a.page)).text;
	expect(cleared).toMatch(/\n#@\/cleared\t1\t2026-11-13\t[0-9a-f-]{36}\n/);
	expect(cleared).not.toMatch(/\nentry\t/);

	const said = await syncFrom(b.page, a.page);
	expect(said).toMatch(/刪除 [1-9]\d* 筆/);
	await ui(b.page).text.fill("");
	expect(await topOf(b.page, "a"), "B 清空前學的也清掉").toBe(first);
	await b.page.keyboard.type("a");
	await expect(
		ui(b.page).rows.and(b.page.locator(`[data-text="${second}"]`)),
	).toHaveAttribute("data-learning", "0.00");
	await b.page.keyboard.press("Escape");
	await b.page.screenshot({ path: testInfo.outputPath("synced-clear.png") });

	// 收到清空之後學的照收，同步回 A。
	await choose(b.page, "a", second);
	await choose(b.page, "a", second);
	await syncFrom(a.page, b.page);
	await ui(a.page).text.fill("");
	expect(await topOf(a.page, "a")).toBe(second);
	expect(await learning(a.page)).toBe(await learning(b.page));
	await a.context.close();
	await b.context.close();
});

// 從備份還原（協調者 2026-11-13 決定）：和匯入（合併）分開的鍵。清空前做的備份也還原得回來；還原留新的
// 清空記號，跟別台合併後別台原本的不回來、換成這份備份。
test("清空後從備份還原，資料回來；還原後跟別台合併，別台的舊資料不回來", async ({
	browser,
}, testInfo) => {
	const a = await profile(browser);
	const b = await profile(browser);
	const { first, second } = await secondOfA(a.page);
	await choose(a.page, "a", second);
	await choose(a.page, "a", second);
	const backup = await exportFile(a.page);
	// B 學了別的：「a」的首選以外的另一個（只有兩個候選時選首選，學了也看不出，最後只比學習資料）。
	await b.page.keyboard.type("a");
	const other = (await shown(b.page))[2] ?? first;
	await b.page.keyboard.press("Escape");
	await choose(b.page, "a", other);
	await choose(b.page, "a", other);

	await a.page.getByRole("button", { name: "清空學習資料" }).click();
	await expect(ui(a.page).status).toHaveText("已清空學習資料");
	await a.page
		.getByLabel("從備份還原（取代目前的）")
		.setInputFiles(backup.path);
	const confirm = a.page.getByRole("group", { name: "確認匯入" });
	await expect(confirm).toContainText("還原會取代目前的");
	await a.page.screenshot({
		path: testInfo.outputPath("restore-confirm.png"),
		fullPage: true,
	});
	await confirm.getByRole("button", { name: "確定還原" }).click();
	await expect(ui(a.page).status).toHaveText(/^已從 .* 還原$/);
	await ui(a.page).text.fill("");
	expect(await topOf(a.page, "a"), "備份的學習回來").toBe(second);
	expect((await exportFile(a.page)).text).toMatch(/\n#@\/cleared\t2\t/);

	await syncFrom(a.page, b.page);
	await ui(a.page).text.fill("");
	expect(await topOf(a.page, "a"), "別台還原前的不收").toBe(second);
	await syncFrom(b.page, a.page);
	await ui(b.page).text.fill("");
	expect(await topOf(b.page, "a"), "別台換成這份備份").toBe(second);
	if (other !== first) {
		await b.page.keyboard.type("a");
		await expect(
			ui(b.page).rows.and(b.page.locator(`[data-text="${other}"]`)),
		).toHaveAttribute("data-learning", "0.00");
		await b.page.keyboard.press("Escape");
	}
	expect(await learning(a.page)).toBe(await learning(b.page));
	await a.context.close();
	await b.context.close();
});
