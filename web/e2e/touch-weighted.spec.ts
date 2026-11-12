import { devices, expect, type Page, test } from "@playwright/test";

// 觸控的空間模型（M4 票 06）：字母鍵依點擊位置送各字母的機率（`Key::Weighted`），核心只用機率
// 最高的字母，打字行為不變；debug 面板讀得到這組機率。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

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
	await page.getByRole("button", { name: "debug" }).tap();
	await expect(page.locator("#debug")).toBeVisible();
});

test.afterEach(() => {
	expect(errors).toEqual([]);
});

/** 面板上最近一下點擊的各字母與機率，由高到低。 */
async function touchProbabilities(page: Page) {
	const items = page.getByTestId("debug-weights").locator("code");
	return items.evaluateAll((codes) =>
		codes.map((code) => ({
			letter: code.getAttribute("data-letter") ?? "",
			probability: Number(code.getAttribute("data-probability")),
		})),
	);
}

async function box(page: Page, id: string) {
	const found = await page.getByTestId(`key-${id}`).boundingBox();
	if (!found) {
		throw new Error(`${id} 鍵沒畫出來`);
	}
	return found;
}

test("點在 a、s 兩鍵中間：打出 a（點擊判定按到的），面板讀到兩個字母的機率", async ({
	page,
}, testInfo) => {
	const a = await box(page, "a");
	const s = await box(page, "s");
	// a 的右緣與 s 的左緣之間的縫，稍微靠 a。
	const gapX = a.x + a.width + (s.x - a.x - a.width) * 0.4;
	await page.touchscreen.tap(gapX, a.y + a.height / 2);
	await expect(page.getByTestId("composition")).toHaveText("日");

	const touch = await touchProbabilities(page);
	expect(touch.map((t) => t.letter).slice(0, 2)).toEqual(["a", "s"]);
	expect(touch[0].probability).toBeGreaterThan(touch[1].probability);
	expect(touch[1].probability).toBeGreaterThan(0.2);
	const sum = touch.reduce((total, t) => total + t.probability, 0);
	expect(sum).toBeCloseTo(1, 5);
	await page.screenshot({
		path: testInfo.outputPath("06-weighted-touch.png"),
		fullPage: true,
	});
});

test("點在鍵中心：那個字母的機率最高；接著點文字鍵，面板清掉", async ({
	page,
}) => {
	const n = await box(page, "n");
	await page.touchscreen.tap(n.x + n.width / 2, n.y + n.height / 2);
	await expect(page.getByTestId("composition")).toHaveText("弓");
	const touch = await touchProbabilities(page);
	expect(touch[0].letter).toBe("n");
	expect(touch[0].probability).toBeGreaterThan(0.7);

	await page.getByTestId("key-space").tap();
	await expect(page.getByTestId("debug-weights")).toHaveText(
		"按鍵機率：（最近一個鍵沒有送機率）",
	);
});
