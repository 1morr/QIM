import { devices, expect, test } from "@playwright/test";

// 觸控模式（M1 票 18）：行動裝置 viewport 下顯示由佈局資料畫出的 Gboard 速成直向鍵盤，
// 點擊位置交給 qim-layout 判定。
// `defaultBrowserType` 只能在設定檔指定，這裡只取 viewport、觸控與 user agent。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

test("依序點 a、n、r，首選「門口」；點空白上屏「門口」", async ({
	page,
}, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	const keyboard = page.getByTestId("keyboard");
	await expect(text).toHaveAttribute("data-ready", "true");
	await expect(keyboard).toBeVisible();
	// 字母鍵的主標籤是方案的字根，角標是字母。
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await page.screenshot({ path: testInfo.outputPath("touch-keyboard.png") });

	for (const id of ["a", "n", "r"]) {
		await page.getByTestId(`key-${id}`).tap();
	}
	await expect(page.getByTestId("composition")).toHaveText("anr");
	await expect(items.first()).toHaveText("1門口");
	await page.screenshot({ path: testInfo.outputPath("touch-anr.png") });

	await page.getByTestId("key-space").tap();
	await expect(text).toHaveValue("門口");
	await page.screenshot({ path: testInfo.outputPath("touch-commit.png") });
});

test("點在兩鍵之間的縫，由 qim-layout 取最近的鍵", async ({ page }) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	const a = await page.getByTestId("key-a").boundingBox();
	const s = await page.getByTestId("key-s").boundingBox();
	if (!a || !s) {
		throw new Error("a、s 鍵沒畫出來");
	}
	const gapX = a.x + a.width + 0.5;
	expect(gapX).toBeLessThan(s.x);
	await page.touchscreen.tap(gapX, a.y + a.height / 2);
	await expect(page.getByTestId("composition")).toHaveText("a");
});

test("核心沒吃下的鍵照宿主的預設動作：空白插入空格、刪除刪字、Enter 換行", async ({
	page,
}) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	for (const id of ["a", "space", "space", "enter", "backspace", "backspace"]) {
		await page.getByTestId(`key-${id}`).tap();
	}
	await expect(text).toHaveValue("日");
	for (const id of ["space", "enter"]) {
		await page.getByTestId(`key-${id}`).tap();
	}
	await expect(text).toHaveValue("日 \n");
});

test("觸控模式可以關掉，關掉後不顯示鍵盤", async ({ page }) => {
	await page.goto("/");
	const toggle = page.getByRole("button", { name: "觸控鍵盤" });
	await expect(toggle).toHaveAttribute("aria-pressed", "true");
	await toggle.tap();
	await expect(toggle).toHaveAttribute("aria-pressed", "false");
	await expect(page.getByTestId("keyboard")).toBeHidden();
	await toggle.tap();
	await expect(page.getByTestId("keyboard")).toBeVisible();
});

test("組字與候選出現、上屏後，鍵盤的位置不動", async ({ page }) => {
	await page.goto("/");
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveAttribute(
		"data-ready",
		"true",
	);
	const keyboard = page.getByTestId("keyboard");
	const before = await keyboard.boundingBox();
	for (const id of ["a", "n", "r"]) {
		await page.getByTestId(`key-${id}`).tap();
	}
	await expect(page.getByTestId("composition")).toHaveText("anr");
	expect(await keyboard.boundingBox()).toEqual(before);
	await page.getByTestId("key-space").tap();
	await expect(page.getByTestId("composition")).toHaveText("");
	expect(await keyboard.boundingBox()).toEqual(before);
});

test("手機寬度下候選列是單列：「門口」橫排不折行（brief §4.4）", async ({
	page,
}) => {
	await page.goto("/");
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveAttribute(
		"data-ready",
		"true",
	);
	for (const id of ["a", "n", "r"]) {
		await page.getByTestId(`key-${id}`).tap();
	}
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(items.first()).toHaveText("1門口");
	const boxes = await Promise.all(
		(await items.all()).map((item) => item.boundingBox()),
	);
	const first = boxes[0];
	if (!first) {
		throw new Error("首選沒畫出來");
	}
	expect(first.width).toBeGreaterThan(first.height);
	// 全部候選在同一列。
	for (const box of boxes) {
		expect(box?.y).toBe(first.y);
	}
});
