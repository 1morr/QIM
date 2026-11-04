import { readFileSync } from "node:fs";
import { devices, expect, test } from "@playwright/test";

// 兩鍵重疊的測試佈局（M1 票 19）：上層的 n 排在陣列前面、z 較大，蓋住下層 a 的右半邊。
// 同一份檔案也給 `qim-layout` 的 Rust 測試用（`crates/qim-layout/tests/overlap.rs`）。
const OVERLAP = readFileSync(
	new URL(
		"../../crates/qim-layout/tests/fixtures/overlap.json",
		import.meta.url,
	),
	"utf8",
);

const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

test.beforeEach(async ({ page }) => {
	// 網頁載入的預設佈局換成重疊佈局。
	await page.route("**/layouts/gboard-quick.json", (route) =>
		route.fulfill({ contentType: "application/json", body: OVERLAP }),
	);
	await page.goto("/");
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveAttribute(
		"data-ready",
		"true",
	);
});

test("點兩鍵重疊處，輸出上層的鍵", async ({ page }, testInfo) => {
	const a = await page.getByTestId("key-a").boundingBox();
	const n = await page.getByTestId("key-n").boundingBox();
	if (!a || !n) {
		throw new Error("a、n 鍵沒畫出來");
	}
	// 重疊處：n 的左邊到 a 的右邊。
	const overlapRight = a.x + a.width;
	expect(n.x).toBeLessThan(overlapRight);
	const x = (n.x + overlapRight) / 2;
	const y = n.y + n.height / 2;
	await page.screenshot({ path: testInfo.outputPath("overlap-layout.png") });

	await page.touchscreen.tap(x, y);
	await expect(page.getByTestId("composition")).toHaveText("n");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(items.first()).toHaveText("1弓");
	await page.screenshot({ path: testInfo.outputPath("overlap-tap.png") });

	// 只有下層的地方是 a。
	await page.touchscreen.tap(a.x + 10, y);
	await expect(page.getByTestId("composition")).toHaveText("na");
});

test("組字中空白鍵換成 composing 的標籤，上屏後換回", async ({ page }) => {
	const space = page.getByTestId("key-space");
	await expect(space).toHaveText("");
	await expect(space).toHaveAttribute("aria-label", "空白");
	await page.getByTestId("key-n").tap();
	await expect(space).toHaveText("選字");
	await space.tap();
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue("弓");
	await expect(space).toHaveText("");
});

test("照佈局的外觀畫：上層鍵的顏色、透明度與圓角", async ({ page }) => {
	const n = page.getByTestId("key-n");
	await expect(n).toHaveCSS("background-color", "rgb(194, 231, 255)");
	await expect(n).toHaveCSS("opacity", "0.85");
	// 圓角 8（設計尺寸 390 寬）依鍵盤寬度縮放，不是 0。
	const radius = await n.evaluate((element) =>
		Number.parseFloat(getComputedStyle(element).borderTopLeftRadius),
	);
	const keyboard = await page.getByTestId("keyboard").boundingBox();
	if (!keyboard) {
		throw new Error("鍵盤沒畫出來");
	}
	expect(radius).toBeCloseTo((8 / 390) * keyboard.width, 1);
	await expect(n).toHaveAttribute("aria-label", "弓，n");
});
