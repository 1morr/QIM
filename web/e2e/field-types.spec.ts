import { devices, expect, type Page, test } from "@playwright/test";

// 欄位類型（M4 票 26，brief〈英文模式與欄位類型〉）：工具列的「欄位」選單模擬宿主輸入框的 `type` / `inputmode`。
// email、網址、密碼欄一聚焦就是英文直通（照 Trime、青簡），沒有建議、自動修正開著也不修；密碼欄另不讀前文；
// 數字欄開數字頁；回到一般欄還原原本的中文。debug 面板讀回欄位類型與前文狀態。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
const { defaultBrowserType: __, ...desktop } = devices["Desktop Chrome"];
test.use(pixel);
// 截圖寫到 Playwright 的輸出目錄：`.scratch/m4/evidence/26-field-*.png` 是票 26 留下的證據，e2e 不改寫。

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
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
		mode: page.getByTestId("mode"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		field: page.getByTestId("debug-field"),
	};
}

/** 換「欄位」選單的選項（選項的字，例如 `email（type=email）`）。 */
async function chooseField(page: Page, label: string): Promise<void> {
	await page.locator("#field-type").selectOption({ label });
}

/** 逐字點觸控鍵盤：字母是 `key-<字母>`，空白是 `key-space`。 */
async function tapText(page: Page, typed: string): Promise<void> {
	for (const c of typed) {
		await page.getByTestId(c === " " ? "key-space" : `key-${c}`).tap();
	}
}

async function turnOnAutocorrect(page: Page): Promise<void> {
	await page
		.getByRole("region", { name: "設定" })
		.getByRole("checkbox", { name: "英文自動修正" })
		.check();
	await page.getByRole("textbox", { name: "輸入框" }).focus();
}

for (const [label, kind] of [
	["email（type=email）", "email"],
	["網址（type=url）", "url"],
] as const) {
	test(`${kind} 欄位預設英文直通、打 teh 不修正、沒有建議；回到一般欄是中文`, async ({
		page,
	}, testInfo) => {
		const { text, mode, items, field } = parts(page);
		await page.getByRole("button", { name: "debug" }).click();
		await turnOnAutocorrect(page);
		await expect(mode).toHaveText("中");

		await chooseField(page, label);
		await expect(mode).toHaveText("英");
		await expect(field).toHaveAttribute("data-field", kind);
		// 英文頁（地球鍵切過去的那頁）：小寫字母、第一排角標數字。
		await expect(page.getByTestId("key-q")).toHaveText("q1");
		await tapText(page, "teh");
		await expect(text).toHaveValue("teh");
		await expect(items).toHaveCount(0);
		await expect(page.getByTestId("debug-autocorrection")).toHaveAttribute(
			"data-autocorrection",
			"",
		);
		await tapText(page, " ");
		await expect(text).toHaveValue("teh ");
		await page.screenshot({
			path: testInfo.outputPath(`field-${kind}.png`),
			fullPage: true,
		});

		await chooseField(page, "一般（type=text）");
		await expect(mode).toHaveText("中");
		await expect(field).toHaveAttribute("data-field", "text");
		await expect(page.getByTestId("key-a")).toHaveText("日a");
		await tapText(page, "a");
		await expect(page.getByTestId("composition")).not.toHaveText("");
	});
}

for (const [label, attribute] of [
	["數字（type=number）", "type=number"],
	["數字（inputmode=numeric）", "inputmode=numeric"],
] as const) {
	test(`數字欄（${attribute}）打開是數字頁，中英不動；回到一般欄是字母頁`, async ({
		page,
	}, testInfo) => {
		const { text, mode, field } = parts(page);
		await page.getByRole("button", { name: "debug" }).click();
		await chooseField(page, label);
		await expect(field).toHaveAttribute("data-field", "number");
		await expect(page.getByTestId("key-to-letters")).toBeVisible();
		await expect(mode).toHaveText("中");
		await page.getByTestId("key-7").tap();
		await expect(text).toHaveValue("7");
		if (attribute === "type=number") {
			await page.screenshot({
				path: testInfo.outputPath("field-number.png"),
				fullPage: true,
			});
		}

		await chooseField(page, "一般（type=text）");
		await expect(page.getByTestId("key-a")).toHaveText("日a");
	});
}

test("密碼欄預設英文直通、沒有建議、不讀前文；使用者的無痕開關不動", async ({
	page,
}, testInfo) => {
	const { text, mode, items, field } = parts(page);
	await page.getByRole("button", { name: "debug" }).click();
	await chooseField(page, "密碼（type=password）");
	await expect(mode).toHaveText("英");
	await expect(field).toHaveAttribute("data-field", "password");
	await tapText(page, "hel");
	await expect(text).toHaveValue("hel");
	await expect(items).toHaveCount(0);
	await expect(page.getByTestId("debug-context")).toHaveAttribute(
		"data-state",
		"incognito",
	);
	await expect(
		page
			.getByRole("region", { name: "設定" })
			.getByRole("checkbox", { name: "無痕" }),
	).not.toBeChecked();
	await page.screenshot({
		path: testInfo.outputPath("field-password.png"),
		fullPage: true,
	});
});

test("一般欄位照舊：中文組字，打 a 首選是日", async ({ page }) => {
	const { mode, items } = parts(page);
	await chooseField(page, "email（type=email）");
	await chooseField(page, "一般（type=text）");
	await expect(mode).toHaveText("中");
	await tapText(page, "a");
	await expect(items.first()).toHaveText(/^1日/);
});

test.describe("實體鍵盤", () => {
	test.use(desktop);

	test("email 欄位預設英文直通、打 teh 空白不修正", async ({ page }) => {
		const { text, mode } = parts(page);
		await turnOnAutocorrect(page);
		await chooseField(page, "email（type=email）");
		await expect(mode).toHaveText("英");
		await page.keyboard.type("teh ");
		await expect(text).toHaveValue("teh ");
	});
});
