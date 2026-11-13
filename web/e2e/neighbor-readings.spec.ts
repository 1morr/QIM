import { devices, expect, type Page, test } from "@playwright/test";

// 觸控鄰鍵讀法（M4 票 09c，brief〈容錯〉）：觸控點一下的鄰鍵字母換進輸入碼，換出來的讀法從候選列第 2 格起插、
// 不搶首選；實體鍵盤與設定關掉時沒有。debug 面板標「鄰鍵」與換了哪一碼。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

function ui(page: Page) {
	const debug = page.locator("#debug");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		composition: page.getByTestId("composition"),
		settings: page.getByRole("region", { name: "設定" }),
		rows: debug.getByRole("table", { name: "候選來源" }).locator("tbody tr"),
	};
}

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
	// 停鍵重排也會往第 2 格插路徑，關掉才看得出鄰鍵讀法的位置。
	await ui(page)
		.settings.getByRole("combobox", { name: "重排" })
		.selectOption("off");
	await page.getByRole("button", { name: "debug" }).tap();
	await expect(page.locator("#debug")).toBeVisible();
});

test.afterEach(() => {
	expect(errors).toEqual([]);
});

/** 點 `letter` 鍵上橫向 `fx`（0 是左緣、1 是右緣）、縱向中間的位置。 */
async function tapAt(page: Page, letter: string, fx = 0.5) {
	const box = await page.getByTestId(`key-${letter}`).boundingBox();
	if (!box) {
		throw new Error(`${letter} 鍵沒畫出來`);
	}
	await page.touchscreen.tap(box.x + box.width * fx, box.y + box.height / 2);
}

/** 候選列（第一頁）的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
}

/**
 * 想打 `anr`（門口），第一下點錯在 s 鍵上靠 a 那側（左緣進來 20%）：觸控點錯一碼。點在 s 鍵正中心時 a 的機率只有
 * s 的 0.08 倍，低於 floor 0.2、不出鄰鍵讀法（協調者定：維持票 09b 在 dev 挑的 floor，和 Gboard 黑箱 T2a 點在
 * 錯鍵中心也列不同）。
 */
async function tapSnr(page: Page) {
	await tapAt(page, "s", 0.2);
	await tapAt(page, "n");
	await tapAt(page, "r");
	await expect(ui(page).composition).toHaveText("尸弓口");
}

test("觸控點錯在錯鍵靠想打的鍵那側：想打的詞在候選列第 2 格起，首選照原樣；debug 面板標鄰鍵", async ({
	page,
}, testInfo) => {
	await tapSnr(page);
	const list = await shown(page);
	const at = list.indexOf("門口");
	expect(at, `${list}`).toBeGreaterThanOrEqual(1);
	const row = ui(page).rows.filter({
		has: page.locator("td:nth-child(2)", { hasText: /^門口$/ }),
	});
	await expect(row.first()).toHaveAttribute("data-sources", /\bneighbor\b/);
	await expect(row.first()).toContainText("鄰鍵（第 1 碼 s→a");
	await page.screenshot({
		path: testInfo.outputPath("09c-neighbor-reading.png"),
		fullPage: true,
	});

	// 選了它上屏門口。
	await ui(page).items.nth(at).tap();
	await expect(ui(page).text).toHaveValue("門口");
});

test("設定的容錯關掉後，同樣點沒有鄰鍵讀法", async ({ page }, testInfo) => {
	const toggle = ui(page).settings.getByRole("checkbox", {
		name: "容錯（觸控鄰鍵）",
	});
	await expect(toggle).toBeChecked();
	await toggle.uncheck();
	await tapSnr(page);
	expect(await shown(page)).not.toContain("門口");
	await expect(
		ui(page).rows.filter({ has: page.locator("[data-sources~=neighbor]") }),
	).toHaveCount(0);
	await page.screenshot({
		path: testInfo.outputPath("09c-neighbor-off.png"),
		fullPage: true,
	});
});

test("實體鍵盤打同一串沒有鄰鍵讀法", async ({ page }) => {
	await ui(page).text.focus();
	await page.keyboard.type("snr");
	await expect(ui(page).composition).toHaveText("尸弓口");
	expect(await shown(page)).not.toContain("門口");
});

test("停鍵重排開著（預設）：重排的路徑排在第 2 格，鄰鍵讀法在它後面、仍在手機候選列看得到的範圍內", async ({
	page,
}, testInfo) => {
	// 兩者都從第 2 格起插時，重排的在前（重排是使用者確認的第 2 位，協調者定）。
	const rerank = ui(page).settings.getByRole("combobox", { name: "重排" });
	await rerank.selectOption("neural");
	await tapSnr(page);
	const list = page.getByRole("list", { name: "候選" });
	// 停鍵後神經模型把一條路徑插到第 2 格（這個例子是「刷口」）。
	await expect(list).toHaveAttribute("data-rerank-applied", "1");
	const reranked = ui(page).rows.filter({ hasText: "重排插入" });
	await expect(reranked).toHaveCount(1);
	const items = ui(page).items;
	const texts = await shown(page);
	const at = texts.indexOf("門口");
	expect(at, `${texts}`).toBeGreaterThanOrEqual(2);
	await expect(items.nth(1)).toHaveText(
		`2${await reranked.locator("td:nth-child(2)").textContent()}`,
	);
	await expect(items.nth(at)).toBeInViewport({ ratio: 1 });
	await page.screenshot({
		path: testInfo.outputPath("09c-neighbor-with-rerank.png"),
	});
});
