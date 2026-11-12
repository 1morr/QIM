import { expect, type Page, test } from "@playwright/test";

// 實體鍵盤的鄰鍵先驗（M4 票 07）：中文模式的字母鍵送 `KeyboardEvent.code`，`qim-layout` 照實體鍵位
// 轉成各字母的機率（`Key::Weighted`）；容錯進核心之前核心只用按下的字母，打字行為不變。debug 面板讀得到
// 這組機率；換 Dvorak 等配列（`key` 不同、`code` 相同）時照實體鍵位。

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

test.beforeEach(async ({ page }) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await page.getByRole("button", { name: "debug" }).click();
	await expect(page.locator("#debug")).toBeVisible();
	await text.focus();
});

test.afterEach(() => {
	expect(errors).toEqual([]);
});

/** 面板上最近一個鍵的各字母與機率，由高到低。 */
async function keyProbabilities(page: Page) {
	return page
		.getByTestId("debug-weights")
		.locator("code")
		.evaluateAll((codes) =>
			codes.map((code) => ({
				letter: code.getAttribute("data-letter") ?? "",
				probability: Number(code.getAttribute("data-probability")),
			})),
		);
}

test("按 f：打出火，面板讀到 f 與六個實體鄰鍵的機率", async ({
	page,
}, testInfo) => {
	await page.keyboard.press("f");
	await expect(page.getByTestId("composition")).toHaveText("火");

	const weights = await keyProbabilities(page);
	expect(weights[0]).toEqual({ letter: "f", probability: expect.any(Number) });
	expect(weights[0].probability).toBeCloseTo(0.9, 5);
	expect(
		weights
			.slice(1)
			.map((w) => w.letter)
			.sort(),
	).toEqual(["c", "d", "g", "r", "t", "v"]);
	// 各排錯位：上一排的 R 比 T 近。
	const of = (letter: string) =>
		weights.find((w) => w.letter === letter)?.probability ?? 0;
	expect(of("r")).toBeGreaterThan(of("t"));
	const sum = weights.reduce((total, w) => total + w.probability, 0);
	expect(sum).toBeCloseTo(1, 5);
	await page.screenshot({
		path: testInfo.outputPath("07-physical-weights.png"),
		fullPage: true,
	});

	// 非字母鍵不送機率，面板清掉。
	await page.keyboard.press("Backspace");
	await expect(page.getByTestId("debug-weights")).toHaveText(
		"按鍵機率：（最近一個鍵沒有送機率）",
	);
});

test("Dvorak 的實體 F 鍵（key 是 u、code 是 KeyF）照實體鍵位打 f", async ({
	page,
}) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await text.evaluate((element) => {
		element.dispatchEvent(
			new KeyboardEvent("keydown", {
				key: "u",
				code: "KeyF",
				bubbles: true,
				cancelable: true,
			}),
		);
	});
	await expect(page.getByTestId("composition")).toHaveText("火");
	const weights = await keyProbabilities(page);
	expect(weights[0].letter).toBe("f");
	expect(weights).toHaveLength(7);
	await expect(text).toHaveValue("");
});

test("英文直通照配列打，不送機率", async ({ page }) => {
	await page.keyboard.down("Shift");
	await page.keyboard.up("Shift");
	await page.keyboard.press("f");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveValue("f");
	await expect(page.getByTestId("debug-weights")).toHaveText(
		"按鍵機率：（最近一個鍵沒有送機率）",
	);
});
