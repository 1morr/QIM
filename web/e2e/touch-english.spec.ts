import { devices, expect, type Page, test } from "@playwright/test";

// 觸控的地球鍵與英文鍵盤（M4 票 23，設計稿 docs/design/23-touch-english-keyboard.html）：點一下地球鍵在速成與
// 英文頁之間切換，長按開切換鍵盤的選單；英文頁接英文直通，字母直接上屏。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

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

async function tapKeys(page: Page, ids: string[]): Promise<void> {
	for (const id of ids) {
		await page.getByTestId(`key-${id}`).tap();
	}
}

/** 按住 `testId` 的中心過了長按的時間再放開（CDP 觸控事件，跟手指一樣）。 */
async function longPress(page: Page, testId: string): Promise<void> {
	const box = await page.getByTestId(testId).boundingBox();
	if (!box) {
		throw new Error(`${testId} 沒畫出來`);
	}
	const cdp = await page.context().newCDPSession(page);
	const point = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
	await cdp.send("Input.dispatchTouchEvent", {
		type: "touchStart",
		touchPoints: [point],
	});
	await page.waitForTimeout(600);
	await cdp.send("Input.dispatchTouchEvent", {
		type: "touchEnd",
		touchPoints: [],
	});
}

test("點地球鍵切到英文頁、打 hello 直接上屏，再點切回速成頁", async ({
	page,
}, testInfo) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	const mode = page.getByTestId("mode");
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await expect(page.getByTestId("key-globe")).toHaveAccessibleName("切換鍵盤");
	await expect(mode).toHaveText("中");

	await page.getByTestId("key-globe").tap();
	await expect(mode).toHaveText("英");
	// 英文頁：小寫字母，第一排角標 1–0（設計稿〈每顆鍵怎麼動〉）。
	await expect(page.getByTestId("key-q")).toHaveText("q1");
	await expect(page.getByTestId("key-a")).toHaveText("a");
	await expect(page.getByTestId("key-space")).toHaveText("English");
	await page.screenshot({ path: testInfo.outputPath("english-page.png") });

	await tapKeys(page, ["h", "e", "l", "l", "o"]);
	await expect(text).toHaveValue("hello");
	await expect(page.getByTestId("composition")).toHaveText("");
	await page.screenshot({ path: testInfo.outputPath("english-hello.png") });

	await page.getByTestId("key-globe").tap();
	await expect(mode).toHaveText("中");
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await page.screenshot({ path: testInfo.outputPath("back-to-quick.png") });
});

test("英文頁打得出半形的 ? ! :：符號頁與長按句號", async ({
	page,
}, testInfo) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await page.getByTestId("key-globe").tap();

	await page.getByTestId("key-to-english-symbols").tap();
	await expect(page.getByTestId("key-to-english")).toHaveText("ABC");
	await page.screenshot({ path: testInfo.outputPath("english-symbols.png") });
	await tapKeys(page, ["?", "!", ":"]);
	await expect(text).toHaveValue("?!:");

	await page.getByTestId("key-to-english").tap();
	await longPress(page, "key-.");
	// 放開打出預設的 `,`；清單裡有半形的 ? ! :。
	await expect(text).toHaveValue("?!:,");
	await page.screenshot({ path: testInfo.outputPath("english-period.png") });
});

test("英文時數字鍵盤的「返回」回到英文頁", async ({ page }) => {
	await page.getByTestId("key-globe").tap();
	await tapKeys(page, ["to-english-symbols", "to-numbers"]);
	await page.getByTestId("key-to-letters").tap();
	await expect(page.getByTestId("key-q")).toHaveText("q1");
});

test("長按地球鍵開切換鍵盤的選單：選 English 切過去，點外面收起", async ({
	page,
}, testInfo) => {
	const menu = page.getByRole("menu", { name: "切換鍵盤" });
	await longPress(page, "key-globe");
	await expect(menu).toBeVisible();
	await expect(menu.getByRole("menuitemradio")).toHaveText([
		/速成/,
		/English/,
		/系統鍵盤/,
	]);
	await expect(
		menu.getByRole("menuitemradio", { name: /速成/ }),
	).toHaveAttribute("aria-checked", "true");
	await expect(
		menu.getByRole("menuitemradio", { name: /系統鍵盤/ }),
	).toHaveAttribute("aria-disabled", "true");
	await page.screenshot({ path: testInfo.outputPath("keyboard-menu.png") });

	await menu.getByRole("menuitemradio", { name: /English/ }).tap();
	await expect(menu).toBeHidden();
	await expect(page.getByTestId("mode")).toHaveText("英");
	await expect(page.getByTestId("key-q")).toHaveText("q1");

	// 占位的系統鍵盤點了沒有作用；點選單外面收起，什麼都不變。
	await longPress(page, "key-globe");
	await expect(
		menu.getByRole("menuitemradio", { name: /English/ }),
	).toHaveAttribute("aria-checked", "true");
	// 占位那列是 aria-disabled，Playwright 預設不點停用的元素；照樣在它上面點一下。
	await menu
		.getByRole("menuitemradio", { name: /系統鍵盤/ })
		.tap({ force: true });
	await expect(menu).toBeVisible();
	await page
		.getByTestId("keyboard-menu-scrim")
		.tap({ position: { x: 4, y: 4 } });
	await expect(menu).toBeHidden();
	await expect(page.getByTestId("mode")).toHaveText("英");
});

test("實體鍵盤單按 Shift 切中英時觸控鍵盤跟著換頁", async ({ page }) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await text.focus();
	await page.keyboard.press("Shift");
	await expect(page.getByTestId("mode")).toHaveText("英");
	await expect(page.getByTestId("key-q")).toHaveText("q1");
	await page.keyboard.press("Shift");
	await expect(page.getByTestId("key-a")).toHaveText("日a");
});

test("組字中長按地球鍵也開選單：按住時按鍵重畫了也一樣", async ({ page }) => {
	// 停鍵重排回來會重畫按鍵、釋放舊的；長按到時不能再讀按下時拿到的那個鍵（手動亂按找到的，
	// 這個順序穩定重現）。
	const menu = page.getByRole("menu", { name: "切換鍵盤" });
	await longPress(page, "key-globe");
	await menu.getByRole("menuitemradio", { name: /速成/ }).tap();
	await tapKeys(page, ["a", "n"]);
	await expect(page.getByTestId("composition")).toHaveText("日弓");
	await longPress(page, "key-globe");
	await expect(menu).toBeVisible();
	// 只開選單：組字還在、沒有切中英。
	await expect(page.getByTestId("composition")).toHaveText("日弓");
	await expect(page.getByTestId("mode")).toHaveText("中");
});

test("臨時英文時觸控留在速成頁", async ({ page }) => {
	// 組字中單按左 Shift 留碼接英文：核心是英文，觸控仍是速成頁（使用者 2026-11-17 定），
	// 按得到全形的「，」「。」。
	const text = page.getByRole("textbox", { name: "輸入框" });
	await tapKeys(page, ["a", "n", "r"]);
	await text.focus();
	await page.keyboard.press("ShiftLeft");
	await expect(page.getByTestId("composition")).toHaveText("anr");
	await expect(page.getByTestId("mode")).toHaveText("英");
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await expect(page.getByTestId("key-。")).toBeVisible();
});
