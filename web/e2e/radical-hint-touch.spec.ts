import { devices, expect, type Page, test } from "@playwright/test";

// 觸控叫出字根提示（M1 票 22，brief §4.4）：行動裝置 viewport 下長按候選。
const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
test.use(pixel);

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveAttribute(
		"data-ready",
		"true",
	);
	for (const id of ["o", "j"]) {
		await page.getByTestId(`key-${id}`).tap();
	}
	await expect(
		page.getByRole("list", { name: "候選" }).getByRole("listitem").first(),
	).toHaveText("1俾");
});

/** 在第一個候選上按下手指，`hold` 毫秒內往右移 `dx` 後放開；Playwright 的 touchscreen 只有 tap，改用 CDP 送觸控事件。 */
async function press(page: Page, hold: number, dx = 0): Promise<void> {
	const box = await page
		.getByRole("list", { name: "候選" })
		.getByRole("listitem")
		.first()
		.boundingBox();
	if (!box) {
		throw new Error("候選沒有畫出來");
	}
	const cdp = await page.context().newCDPSession(page);
	const at = (x: number) => [
		{ x: box.x + box.width / 2 + x, y: box.y + box.height / 2 },
	];
	await cdp.send("Input.dispatchTouchEvent", {
		type: "touchStart",
		touchPoints: at(0),
	});
	if (dx !== 0) {
		await cdp.send("Input.dispatchTouchEvent", {
			type: "touchMove",
			touchPoints: at(dx),
		});
	}
	await page.waitForTimeout(hold);
	await cdp.send("Input.dispatchTouchEvent", {
		type: "touchEnd",
		touchPoints: [],
	});
}

test("長按候選列出首選的倉頡五代碼，再長按收起", async ({ page }, testInfo) => {
	const hint = page.getByTestId("hint-1");
	await press(page, 700);
	await expect(hint).toHaveText("人竹田十(ohwj)");
	await page.screenshot({ path: testInfo.outputPath("touch-hint.png") });
	await press(page, 700);
	await expect(hint).toHaveCount(0);
});

test("點一下或長按中滑動都不叫出提示", async ({ page }) => {
	const hint = page.getByTestId("hint-1");
	await press(page, 50);
	await press(page, 700, 30);
	await expect(hint).toHaveCount(0);
});
