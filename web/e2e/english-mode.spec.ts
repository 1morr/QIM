import { devices, expect, type Page, test } from "@playwright/test";

// 英文模式（ADR 0008，M4 票 24）：觸控的地球鍵切到英文頁時，字母逐個上屏，候選列是正在打的串的建議
// （英文詞典加英文使用者詞，照使用頻率排；詞典沒有的打過 2 次才出）。點建議換掉打的詞並補一個空白。
// 開了句首自動大寫，句子開頭 Shift 自動亮起。實體鍵盤單按 Shift 照舊是英文直通、沒有建議。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

const EMAIL = "jean.test@example.com";

/** 頁面拋出的錯誤；每個測試結尾要是空的（例如用到已釋放的 WebAssembly 物件）。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveAttribute(
		"data-ready",
		"true",
	);
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

function parts(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

/** 逐字點觸控鍵盤：字母是 `key-<字母>`，空白是 `key-space`。 */
async function tapText(page: Page, typed: string): Promise<void> {
	for (const c of typed) {
		await page.getByTestId(c === " " ? "key-space" : `key-${c}`).tap();
	}
}

/** 打開 debug 面板；回傳文字是 `word` 的候選那列。 */
async function openDebug(page: Page) {
	await page.getByRole("button", { name: "debug" }).click();
	const rows = page
		.getByRole("table", { name: "候選來源" })
		.locator("tbody tr");
	return (word: string) =>
		rows.filter({
			has: page.locator("td:nth-child(2)", {
				hasText: new RegExp(`^${word}$`),
			}),
		});
}

test("地球鍵切到英文頁打 rrrrrr 一次後，打 rrr 建議列沒有它；打第二次後有，面板標英文使用者詞", async ({
	page,
}, testInfo) => {
	const { text, items } = parts(page);
	const row = await openDebug(page);
	await page.getByTestId("key-globe").tap();

	await tapText(page, "rrrrrr ");
	await expect(text).toHaveValue("rrrrrr ");
	await tapText(page, "rrr");
	await expect(text).toHaveValue("rrrrrr rrr");
	await expect(items.filter({ hasText: /rrrrrr$/ })).toHaveCount(0);
	await page.screenshot({
		path: testInfo.outputPath("24-english-mode-once.png"),
		fullPage: true,
	});

	await tapText(page, "rrr ");
	await tapText(page, "rrr");
	await expect(items.first()).toHaveText("1rrrrrr");
	await expect(row("rrrrrr")).toHaveAttribute("data-kind", "englishUserWord");
	await page.screenshot({
		path: testInfo.outputPath("24-english-mode-twice.png"),
		fullPage: true,
	});
});

/** M3 定稿的 v2 檔：一條記住的 email（方案欄、前一個字詞欄都空的列）。 */
const V2_WITH_EMAIL = [
	"# qim-userdict v2",
	"#@/tick\t1",
	"# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger",
	`\t\t${EMAIL}\t1\t1`,
	"",
].join("\n");

test("v2 匯入的 email 打開頭的字母，英文建議列就有它；點了換掉打的字", async ({
	page,
}, testInfo) => {
	const { text, items } = parts(page);
	await page.getByLabel("匯入學習資料（合併進目前的）").setInputFiles({
		name: "v2.tsv",
		mimeType: "text/plain",
		buffer: Buffer.from(V2_WITH_EMAIL),
	});
	await page
		.getByRole("group", { name: "確認匯入" })
		.getByRole("button", { name: "確定匯入" })
		.click();
	const row = await openDebug(page);
	await text.focus();
	await page.getByTestId("key-globe").tap();

	await tapText(page, "je");
	await expect(items.filter({ hasText: EMAIL })).toHaveCount(1);
	await expect(row(EMAIL)).toHaveAttribute("data-kind", "englishUserWord");
	await page.screenshot({
		path: testInfo.outputPath("24-english-mode-email.png"),
		fullPage: true,
	});
	await items.filter({ hasText: EMAIL }).tap();
	await expect(text).toHaveValue(`${EMAIL} `);
	await expect(items).toHaveCount(0);
});

test("建議照詞典頻率排；點建議刪掉打的字、補空白，緊接著的逗號換掉那個空白", async ({
	page,
}) => {
	const { text, items } = parts(page);
	const row = await openDebug(page);
	await page.getByTestId("key-globe").tap();
	await tapText(page, "thi");
	await expect(items.first()).toHaveText("1this");
	await expect(row("this")).toHaveAttribute("data-kind", "english");
	await items.first().tap();
	await expect(text).toHaveValue("this ");
	await page.getByTestId("key-,").tap();
	await expect(text).toHaveValue("this,");
});

test("句首自動大寫打開後，英文模式在空的欄位與句號加空白之後自動開 Shift", async ({
	page,
}, testInfo) => {
	const { text } = parts(page);
	const shift = page.getByTestId("key-shift");
	await page.getByTestId("key-globe").tap();
	await expect(shift).toHaveAttribute("data-shift", "off");

	await page
		.getByRole("region", { name: "設定" })
		.getByRole("checkbox", { name: "句首自動大寫" })
		.check();
	await text.focus();
	await expect(shift).toHaveAttribute("data-shift", "once");
	await tapText(page, "ok");
	await expect(text).toHaveValue("Ok");
	await expect(shift).toHaveAttribute("data-shift", "off");
	await page.getByTestId("key-.").tap();
	await tapText(page, " ");
	await expect(shift).toHaveAttribute("data-shift", "once");
	await page.screenshot({
		path: testInfo.outputPath("24-english-mode-auto-shift.png"),
		fullPage: true,
	});
	// 使用者點掉的 Shift 不被開回來：句首也打得出小寫。
	await shift.tap();
	await tapText(page, "i");
	await expect(text).toHaveValue("Ok. i");
});

test("實體鍵盤單按 Shift 照舊是英文直通：字母交還瀏覽器、沒有建議", async ({
	page,
}) => {
	const { text, items } = parts(page);
	await text.focus();
	await page.keyboard.press("Shift");
	await page.keyboard.type("hel-lo");
	await expect(text).toHaveValue("hel-lo");
	await expect(items).toHaveCount(0);
});

test("用真的英文資料包：打過一次的 helmet 打 hel 時在前 3 格；打過一次的 tumors 不擠進 t 的前 3 格", async ({
	page,
}) => {
	const { items } = parts(page);
	await page.getByTestId("key-globe").tap();
	await tapText(page, "hel");
	await expect(items.first()).toBeVisible();
	await expect(items.filter({ hasText: /^[123]helmet$/ })).toHaveCount(0);
	await tapText(page, "met tumors ");
	await tapText(page, "hel");
	await expect(items.filter({ hasText: /^[123]helmet$/ })).toHaveCount(1);
	await tapText(page, " t");
	await expect(items.first()).toBeVisible();
	await expect(items.filter({ hasText: /^[123]tumors$/ })).toHaveCount(0);
});
