import { expect, type Page, test } from "@playwright/test";

// brief〈桌面鍵盤操作〉 的鍵位表，每一列一項，實打網頁讀回輸入框、組字與候選（M1 票 23）。
// 細節的變化（翻頁找字、連打分段）另見 keys、composition-keys、edit-segments 各檔。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		segments: page.getByTestId("composition").locator(".segment"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		mode: page.getByTestId("mode"),
		rerank: page.getByRole("combobox", { name: "重排" }),
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

// 只驗翻頁鍵，關掉停鍵重排：它在翻頁中途回來時，回到第一頁才插入（rerank.spec），順序就和翻頁前讀的
// 不同；高負載時常碰到（M4 票 05c）。
test("- / =、PageUp / PageDown：翻頁", async ({ page }) => {
	const { text, items, rerank } = ui(page);
	await rerank.selectOption("off");
	await text.focus();
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

// 組字中單按 Shift 預設照 Rime（M1 票 25）：左 Shift 留碼接英文，右 Shift 先上屏組字再切英文。
// Playwright 的 `Shift` 是左 Shift（`KeyboardEvent.code` 為 `ShiftLeft`）。
test("Shift（單按）：組字中右 Shift 先上屏整個組字再切英文", async ({
	page,
}) => {
	const { text, composition, mode } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("ShiftRight");
	await expect(text).toHaveValue("門口");
	await expect(composition).toHaveText("");
	await expect(mode).toHaveText("英");
	await page.keyboard.type("a");
	await expect(text).toHaveValue("門口a");
});

test("Shift（單按）：組字中左 Shift 留著輸入碼當英文接下去，Enter 上屏後回中文", async ({
	page,
}, testInfo) => {
	const { text, composition, items, mode } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("ShiftLeft");
	await expect(composition).toHaveText("anr");
	await expect(items).toHaveCount(0);
	await expect(mode).toHaveText("英");
	await expect(text).toHaveValue("");
	// 臨時英文中 `-` 是字元，不是翻頁。
	await page.keyboard.type("x-1");
	await expect(composition).toHaveText("anrx-1");
	await page.screenshot({ path: testInfo.outputPath("inline-ascii.png") });
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anrx-1");
	await expect(composition).toHaveText("");
	await expect(mode).toHaveText("中");
	await page.keyboard.type("anr ");
	await expect(text).toHaveValue("anrx-1門口");
});

test("Shift（單按）：左 Shift 留碼後沒改就再按左 Shift，還原中文組字", async ({
	page,
}) => {
	const { text, composition, items, mode } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("Shift");
	await expect(composition).toHaveText("anr");
	await page.keyboard.press("Shift");
	await expect(mode).toHaveText("中");
	await expect(composition).toHaveText("日弓口");
	await expect(items.first()).toHaveText("1門口");
	await page.keyboard.press(" ");
	await expect(text).toHaveValue("門口");
});

// librime 1.17.0（M1 票 25b）：臨時英文中空白也接進英文，Enter 才上屏。
test("Shift（單按）：左 Shift 留碼後空白接進英文，Enter 上屏整段", async ({
	page,
}, testInfo) => {
	const { text, composition, mode } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("ShiftLeft");
	await page.keyboard.type("hello world");
	await expect(composition).toHaveText("anrhello world");
	await expect(mode).toHaveText("英");
	await expect(text).toHaveValue("");
	await page.screenshot({
		path: testInfo.outputPath("inline-ascii-space.png"),
	});
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anrhello world");
	await expect(composition).toHaveText("");
	await expect(mode).toHaveText("中");
});

test("Shift（單按）：留碼後打了字再按左 Shift，整串當輸入碼回中文組字", async ({
	page,
}) => {
	const { text, composition, items, mode } = ui(page);
	await page.keyboard.type("an");
	await page.keyboard.press("Shift");
	await page.keyboard.type("r");
	await expect(composition).toHaveText("anr");
	await page.keyboard.press("Shift");
	await expect(mode).toHaveText("中");
	await expect(composition).toHaveText("日弓口");
	await expect(items.first()).toHaveText("1門口");
	await page.keyboard.press(" ");
	await expect(text).toHaveValue("門口");

	// 英文含空白：中文組字裝不下，上屏英文回中文。
	await page.keyboard.type("anr");
	await page.keyboard.press("Shift");
	await page.keyboard.type(" x");
	await page.keyboard.press("Shift");
	await expect(text).toHaveValue("門口anr x");
	await expect(composition).toHaveText("");
	await expect(mode).toHaveText("中");
});

test("組字中單按 Shift 的設定：左右都先上屏、左右都留碼", async ({ page }) => {
	const { text, mode } = ui(page);
	const setting = page.getByLabel("組字中單按 Shift");
	await expect(setting).toHaveValue("rime");

	await setting.selectOption("commit-text");
	await text.focus();
	await page.keyboard.type("anr");
	await page.keyboard.press("ShiftLeft");
	await expect(text).toHaveValue("門口");
	await expect(mode).toHaveText("英");
	await page.keyboard.press("ShiftLeft");

	await setting.selectOption("inline-ascii");
	await text.focus();
	await page.keyboard.type("anr");
	await page.keyboard.press("ShiftRight");
	await page.keyboard.type("x");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("門口anrx");
	await expect(mode).toHaveText("中");
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
