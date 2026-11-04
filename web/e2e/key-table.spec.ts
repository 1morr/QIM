import { expect, type Page, test } from "@playwright/test";

// brief §4.3 的鍵位表，每一列一項，實打網頁讀回輸入框、組字與候選（M1 票 23）。
// 細節的變化（翻頁找字、連打分段）另見 keys、composition-keys、edit-segments 各檔。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		segments: page.getByTestId("composition").locator(".segment"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		mode: page.getByTestId("mode"),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await expect(ui(page).text).toHaveAttribute("data-ready", "true");
	await ui(page).text.focus();
});

test.afterEach(() => {
	expect(errors.map(String)).toEqual([]);
});

test("空白：上屏首選，翻頁後也一樣", async ({ page }) => {
	const { text, items } = ui(page);
	await page.keyboard.type("ar");
	const top = (await items.first().textContent())?.slice(1);
	await page.keyboard.press("=");
	await expect(items.first()).not.toHaveText(`1${top}`);
	await page.keyboard.press(" ");
	await expect(text).toHaveValue(top ?? "");
});

test("1–9：選對應候選", async ({ page }) => {
	await page.keyboard.type("a2");
	await expect(ui(page).text).toHaveValue("曰");
});

test("- / =、PageUp / PageDown：翻頁", async ({ page }) => {
	const { items } = ui(page);
	await page.keyboard.type("ar");
	const first = await items.allTextContents();
	for (const [next, back] of [
		["=", "-"],
		["PageDown", "PageUp"],
	]) {
		await page.keyboard.press(next);
		await expect(items).not.toHaveText(first);
		await page.keyboard.press(back);
		await expect(items).toHaveText(first);
	}
});

test("Enter：輸入碼原樣上屏；已選定的分段上屏字詞", async ({ page }) => {
	const { text, segments } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anr");
	// `anrkbof`（首選「門口｜有｜無」）選開頭一段「門口」後按 Enter：門口＋其餘輸入碼。
	await page.keyboard.type("anrkbof");
	await page.keyboard.press("ArrowLeft");
	await page.keyboard.press("ArrowLeft");
	await page.keyboard.press("ArrowLeft");
	await expect(segments.nth(0)).toHaveAttribute("aria-current", "true");
	await page.keyboard.press("1");
	await expect(segments.nth(0)).toHaveText("門口");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anr門口kbof");
});

test("Esc：清除輸入碼", async ({ page }) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("Escape");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await expect(text).toHaveValue("");
});

test("← / →：移到前一段或後一段，→ 走過最後一段回到整串", async ({ page }) => {
	const { segments, items } = ui(page);
	await page.keyboard.type("anrkbof");
	await expect(segments).toHaveCount(3);
	await page.keyboard.press("ArrowLeft");
	await expect(segments.nth(2)).toHaveAttribute("aria-current", "true");
	await page.keyboard.press("ArrowLeft");
	await expect(segments.nth(1)).toHaveAttribute("aria-current", "true");
	await page.keyboard.press("ArrowRight");
	await page.keyboard.press("ArrowRight");
	await expect(segments.locator("[aria-current]")).toHaveCount(0);
	await expect(items.first()).toHaveText("1門口有無");
});

test("Tab：組字中列出或收起字根提示；沒有組字時交還瀏覽器", async ({
	page,
}) => {
	const { text } = ui(page);
	const hint = page.getByTestId("hint-1");
	await page.keyboard.type("anr");
	await page.keyboard.press("Tab");
	await expect(hint).toHaveText("日弓(an) 口(r)");
	await page.keyboard.press("Tab");
	await expect(hint).toHaveCount(0);
	await expect(text).toBeFocused();
	await page.keyboard.press("Escape");
	await page.keyboard.press("Tab");
	await expect(text).not.toBeFocused();
});

test("Shift（單按）：切換中英文，英文直接輸出，再按一次回中文", async ({
	page,
}, testInfo) => {
	const { text, composition, mode } = ui(page);
	await expect(mode).toHaveText("中");
	await page.keyboard.press("Shift");
	await expect(mode).toHaveText("英");
	await page.keyboard.type("hello, ");
	await expect(text).toHaveValue("hello, ");
	await expect(composition).toHaveText("");
	await page.screenshot({ path: testInfo.outputPath("ascii.png") });

	await page.keyboard.press("Shift");
	await expect(mode).toHaveText("中");
	await page.keyboard.type("anr ");
	await expect(text).toHaveValue("hello, 門口");
});

test("Shift（單按）：組字中先上屏整個組字再切英文", async ({ page }) => {
	const { text, composition, mode } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("Shift");
	await expect(text).toHaveValue("門口");
	await expect(composition).toHaveText("");
	await expect(mode).toHaveText("英");
	await page.keyboard.type("a");
	await expect(text).toHaveValue("門口a");
});

test("Shift 加字母打大寫，不算單按、不切換", async ({ page }) => {
	const { text, mode } = ui(page);
	await page.keyboard.press("Shift+A");
	await expect(text).toHaveValue("A");
	await expect(mode).toHaveText("中");
});

test("標點：組字中先上屏整個組字再接標點；沒有組字時直接上屏", async ({
	page,
}) => {
	await page.keyboard.type(",anr,");
	await expect(ui(page).text).toHaveValue("，門口，");
});

test("大寫字母：組字中先上屏整個組字再接字母；沒有組字時照打", async ({
	page,
}) => {
	await page.keyboard.press("Shift+B");
	await page.keyboard.type("anr");
	await page.keyboard.press("Shift+A");
	await expect(ui(page).text).toHaveValue("B門口A");
});

// 票 10 留下：組字中帶修飾鍵的組合也吃下，不然 Cmd+A 選取全部、上屏時蓋掉輸入框的字。
test("組字中 Ctrl / Cmd 組合鍵吃下，組字與輸入框都不變", async ({ page }) => {
	const { text, composition } = ui(page);
	await page.keyboard.press("Shift");
	await page.keyboard.type("xyz");
	await page.keyboard.press("Shift");
	await expect(text).toHaveValue("xyz");
	await page.keyboard.type("an");
	await page.keyboard.press("ControlOrMeta+A");
	await expect(composition).toHaveText("日弓");
	await page.keyboard.press(" ");
	await expect(text).toHaveValue("xyz門");
	// 沒有組字時照常交給瀏覽器。
	await page.keyboard.press("ControlOrMeta+A");
	await page.keyboard.type("a ");
	await expect(text).toHaveValue("日");
});

test.describe("剪貼簿", () => {
	test.use({ permissions: ["clipboard-read", "clipboard-write"] });

	// 協調者轉入（票 10 留下）：組字中 Cmd+V 原本貼進輸入框而組字還在。
	test("組字中 Cmd+V 不貼上、組字留著；沒有組字時照常貼上", async ({
		page,
	}) => {
		const { text, composition } = ui(page);
		await page.evaluate(() => navigator.clipboard.writeText("貼"));
		await page.keyboard.type("an");
		await page.keyboard.press("ControlOrMeta+V");
		await expect(composition).toHaveText("日弓");
		await expect(text).toHaveValue("");
		await page.keyboard.press(" ");
		await page.keyboard.press("ControlOrMeta+V");
		await expect(text).toHaveValue("門貼");
	});
});
