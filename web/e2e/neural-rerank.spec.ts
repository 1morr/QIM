import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 停鍵層的神經模型（plan §2.7、§3.1，M3 票 11e）：停鍵後 Web Worker 裡的 1.4M 模型給 lattice 的前 9 條
// 路徑排名次，核心把排出的路徑放到第 2 位，候選列沒有就插入；首選不動。
//
// `fukjctd`（口語集的「光大實業」）：同步層的首選是「光大額葉」、第 2 個是「火山大額葉」，全部候選裡沒有
// 「光大實業」；模型把它排到第 2 位。換了模型或資料包時這組字可能要換。

const CODE = "fukjctd";
const TOP = "光大額葉";
const SECOND = "火山大額葉";
const RERANKED = "光大實業";
const MODEL_ID = "zh-hant-neural-rerank";
/** `cargo xtask web` 放的模型資料包，匯入的測試用它。 */
const MODEL_DIR = new URL(`../public/packs/${MODEL_ID}/`, import.meta.url);

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		list: page.getByRole("list", { name: "候選" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		all: page.getByRole("list", { name: "全部候選" }).getByRole("listitem"),
		rerank: page.getByRole("combobox", { name: "重排" }),
		rows: page.getByRole("table", { name: "資料包清單" }).locator("tbody tr"),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

async function open(page: Page, url = "/") {
	await page.goto(url);
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
	await ui(page).text.focus();
}

/** 候選列目前的字（去掉序號）。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) => t.slice(1));
}

/** 重排關著時打 `CODE` 的全部候選（展開面板讀），讀完清掉。 */
async function unranked(page: Page): Promise<string[]> {
	const { text, all, rerank } = ui(page);
	const before = await rerank.inputValue();
	await rerank.selectOption("off");
	await text.focus();
	await page.keyboard.type(CODE);
	await page.keyboard.press("ArrowDown");
	await expect(all.first()).toBeVisible();
	const every = await all.allTextContents();
	await page.keyboard.press("Escape");
	await expect(text).toHaveValue("");
	await rerank.selectOption(before);
	await text.focus();
	return every;
}

test("預設開著神經模型：停鍵後把候選列沒有的路徑排到第 2 位，首選不動", async ({
	page,
}, testInfo) => {
	await open(page);
	const { text, items, list, rerank, rows } = ui(page);
	await expect(rerank).toHaveValue("neural");
	await expect(rows.filter({ hasText: MODEL_ID })).toContainText("停鍵層模型");
	const every = await unranked(page);
	expect(every.slice(0, 2)).toEqual([TOP, SECOND]);
	expect(every).not.toContain(RERANKED);

	await page.keyboard.type(CODE);
	await expect(items.nth(1)).toHaveText(`2${RERANKED}`);
	await expect(items.first()).toHaveText(`1${TOP}`);
	await expect(items.nth(2)).toHaveText(`3${SECOND}`);
	await expect(list).toHaveAttribute("data-rerank-applied", "1");
	await page.screenshot({ path: testInfo.outputPath("neural.png") });

	// 選它照那條路徑上屏。
	await page.keyboard.press("2");
	await expect(text).toHaveValue(RERANKED);
});

test("關掉重排：停鍵後候選不再變動", async ({ page }) => {
	await open(page);
	const { items, list, rerank } = ui(page);
	await rerank.selectOption("off");
	await ui(page).text.focus();
	await page.keyboard.type(CODE);
	await page.waitForTimeout(800);
	expect((await shown(page)).slice(0, 2)).toEqual([TOP, SECOND]);
	await expect(items.nth(1)).toHaveText(`2${SECOND}`);
	await expect(list).not.toHaveAttribute("data-rerank-applied");
});

// 開關改了之後：已經套用的那次之後不再重排；送出要求後才關，回來的結果不套用（網址的 `rerank-delay`
// 讓結果晚 1 秒回來）。
test("重排套用後或等待中才關掉：之後不再變動", async ({ page }) => {
	await open(page, "/?rerank-delay=1000");
	const { text, items, list, rerank } = ui(page);
	await page.keyboard.type(CODE);
	await expect(items.nth(1)).toHaveText(`2${RERANKED}`);
	await rerank.selectOption("off");
	await text.focus();
	await page.keyboard.press("Escape");
	await page.keyboard.type(CODE);
	await page.waitForTimeout(1500);
	await expect(items.nth(1)).toHaveText(`2${SECOND}`);

	await rerank.selectOption("neural");
	await text.focus();
	await page.keyboard.press("Escape");
	await page.keyboard.type(CODE);
	// 停鍵（80 ms）後要求已送出，還沒回來就關掉。
	await page.waitForTimeout(300);
	await rerank.selectOption("off");
	await expect(list).toHaveAttribute("data-rerank-none", "1", {
		timeout: 5000,
	});
	await expect(items.nth(1)).toHaveText(`2${SECOND}`);
	await expect(list).toHaveAttribute("data-rerank-applied", "1");
});

// 網址的 `rerank-delay` 讓結果回來後再等 1.5 秒才套用（模擬慢的模型）。停鍵後送出要求，結果回來前退格再
// 打回同一碼：輸入碼一樣，候選卻已經換過一輪（`generation` 變了），舊結果由核心丟掉；之後新的要求照常套用。
test("重排回來前繼續打字：舊結果不套用", async ({ page }, testInfo) => {
	await open(page, "/?rerank-delay=1500");
	const { items, list } = ui(page);
	await page.keyboard.type(CODE);
	// 停鍵（80 ms）後要求已送出、還沒回來。
	await page.waitForTimeout(300);
	await page.keyboard.press("Backspace");
	await page.keyboard.type(CODE.at(-1) ?? "");
	await expect(list).toHaveAttribute("data-rerank-stale", "1", {
		timeout: 5000,
	});
	await expect(items.nth(1)).toHaveText(`2${SECOND}`);
	await expect(list).not.toHaveAttribute("data-rerank-applied");
	await page.screenshot({ path: testInfo.outputPath("stale.png") });

	await expect(items.nth(1)).toHaveText(`2${RERANKED}`, { timeout: 5000 });
	await expect(list).toHaveAttribute("data-rerank-applied", "1");
});

test.describe("沒有內建的模型資料包", () => {
	test.beforeEach(async ({ page }) => {
		await page.route(`**/packs/${MODEL_ID}/**`, (route) =>
			route.fulfill({ status: 404, body: "" }),
		);
		await page.goto("/");
		await page.evaluate(async () => {
			localStorage.clear();
			await new Promise((resolve) => {
				const request = indexedDB.deleteDatabase("qim-packs");
				request.onsuccess = request.onerror = request.onblocked = resolve;
			});
		});
	});

	test("打字與候選照常，不重排", async ({ page }, testInfo) => {
		await open(page);
		const { items, list, rows } = ui(page);
		await expect(rows.filter({ hasText: MODEL_ID })).toHaveCount(0);
		await page.keyboard.type(CODE);
		await expect(list).toHaveAttribute("data-rerank-none", "1");
		expect((await shown(page)).slice(0, 2)).toEqual([TOP, SECOND]);
		await expect(items.nth(1)).toHaveText(`2${SECOND}`);
		await page.screenshot({ path: testInfo.outputPath("no-model.png") });
		await page.keyboard.press("Space");
		await expect(ui(page).text).toHaveValue(TOP);
	});

	// 模型資料包不交給 `Session`（核心會拒收），交給 Web Worker（M3 票 11d 的發現）。
	test("從檔案匯入模型資料包：列出來，停鍵後照它重排", async ({ page }) => {
		await open(page);
		const { items, rows } = ui(page);
		await page.getByLabel("從檔案匯入資料包").setInputFiles([
			{
				name: "manifest.json",
				mimeType: "application/json",
				buffer: await readFile(new URL("manifest.json", MODEL_DIR)),
			},
			{
				name: `${MODEL_ID}.pack`,
				mimeType: "application/octet-stream",
				buffer: await readFile(new URL(`${MODEL_ID}.pack`, MODEL_DIR)),
			},
		]);
		await expect(rows.filter({ hasText: MODEL_ID })).toContainText("匯入");
		await ui(page).text.focus();
		await page.keyboard.type(CODE);
		await expect(items.nth(1)).toHaveText(`2${RERANKED}`);
		await expect(items.first()).toHaveText(`1${TOP}`);
	});
});
