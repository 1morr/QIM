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

// 中文模式整個當美式鍵盤（使用者 2026-11-12 決定）：換配列時字母與標點都照實體鍵位，英文直通照配列。
// Playwright 的鍵盤固定美式，別的配列用合成的 keydown（`key` 照那個配列、`code` 是實體鍵位）；合成事件
// 沒有瀏覽器的預設動作，所以「交還瀏覽器」從事件有沒有被攔下（`defaultPrevented`）讀。

/** 在輸入框送一個合成的 keydown；回傳網頁有沒有攔下（攔下就是網頁或核心處理了，不交給瀏覽器照 `key` 打）。 */
async function dispatch(
	page: Page,
	code: string,
	key: string,
	init: KeyboardEventInit = {},
) {
	return page.getByRole("textbox", { name: "輸入框" }).evaluate(
		(element, [code, key, init]) =>
			!element.dispatchEvent(
				new KeyboardEvent("keydown", {
					...(init as KeyboardEventInit),
					key: key as string,
					code: code as string,
					bubbles: true,
					cancelable: true,
				}),
			),
		[code, key, init] as const,
	);
}

test("Dvorak：實體 W、Q 鍵打碼，逗號、句號、分號鍵打美式的標點", async ({
	page,
}) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	expect(await dispatch(page, "KeyW", ",")).toBe(true);
	expect(await dispatch(page, "KeyQ", "'")).toBe(true);
	await expect(composition).toHaveText("田手");
	// Dvorak 的實體逗號鍵打 w：照美式是逗號，先上屏組字再接全形逗號。
	expect(await dispatch(page, "Comma", "w")).toBe(true);
	await expect(composition).toHaveText("");
	await expect(text).toHaveValue(/，$/);
	expect(await dispatch(page, "Period", "v")).toBe(true);
	await expect(text).toHaveValue(/，。$/);
	expect(await dispatch(page, "Semicolon", "s")).toBe(true);
	await expect(text).toHaveValue(/，。；$/);
	// 實體 E、Z 鍵（Dvorak 的 . 與 ;）是字母 e、z，不是標點。
	expect(await dispatch(page, "KeyE", ".")).toBe(true);
	await expect(composition).toHaveText("水");
	const weights = await keyProbabilities(page);
	expect(weights[0].letter).toBe("e");
	expect(await dispatch(page, "KeyZ", ";")).toBe(true);
	await expect(
		page.getByTestId("debug-weights").locator("code").first(),
	).toHaveAttribute("data-letter", "z");
	await expect(text).toHaveValue(/，。；$/);
});

test("AZERTY：實體 Q、W、M 鍵打美式的字母，數字列不按 Shift 就是數字", async ({
	page,
}) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	// AZERTY 的實體 Q、W、M 鍵打 a、z、,。
	await dispatch(page, "KeyQ", "a");
	await dispatch(page, "KeyW", "z");
	await expect(composition).toHaveText("手田");
	await page.keyboard.press("Escape");
	await dispatch(page, "KeyM", ",");
	await expect(composition).toHaveText("一");
	// 實體逗號鍵（AZERTY 的 ;）照美式是逗號。
	await dispatch(page, "Comma", ";");
	await expect(composition).toHaveText("");
	await expect(text).toHaveValue(/，$/);
	// 沒有組字時實體 1 鍵（AZERTY 的 &）照美式是 1：核心交還宿主，網頁插入 1，不是 &。
	expect(await dispatch(page, "Digit1", "&")).toBe(true);
	await expect(text).toHaveValue(/，1$/);
	// Shift+實體 1 鍵（AZERTY 的 1）照美式是 !，中文模式上屏全形驚嘆號。
	expect(await dispatch(page, "Digit1", "1", { shiftKey: true })).toBe(true);
	await expect(text).toHaveValue(/，1！$/);
});

test("英文直通照配列：Dvorak 與 AZERTY 的鍵交還瀏覽器照 key 打", async ({
	page,
}) => {
	await page.keyboard.down("Shift");
	await page.keyboard.up("Shift");
	await expect(page.getByTestId("mode")).toHaveAttribute("aria-label", "英文");
	// 不攔下：瀏覽器照 `key`（Dvorak 的 ,、AZERTY 的 &）打。
	expect(await dispatch(page, "KeyW", ",")).toBe(false);
	expect(await dispatch(page, "Digit1", "&")).toBe(false);
	expect(await dispatch(page, "Comma", "w")).toBe(false);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue("");
	await expect(page.getByTestId("composition")).toHaveText("");
	await expect(page.getByTestId("debug-weights")).toHaveText(
		"按鍵機率：（最近一個鍵沒有送機率）",
	);
});

test("臨時英文照配列：Dvorak 的實體 W 鍵接上逗號，不是 w", async ({ page }) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await page.keyboard.type("a");
	// 組字中單按左 Shift：留碼接英文（brief §4.3）。
	await page.keyboard.press("ShiftLeft");
	await dispatch(page, "KeyW", ",");
	await dispatch(page, "KeyF", "u");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("a,u");
});

test("萬用鍵 z 照實體鍵位：Dvorak 與 AZERTY 的實體 Z 鍵照樣是萬用鍵", async ({
	page,
}) => {
	const composition = page.getByTestId("composition");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	// Dvorak 的實體 H、Z、O 鍵打 d、;、r；照美式是 h、z、o，首選和美式打 hzo 一樣是「的人」。
	await dispatch(page, "KeyH", "d");
	await dispatch(page, "KeyZ", ";");
	await expect(composition).toHaveText("竹＊");
	await expect(
		page.getByTestId("debug-weights").locator("code").first(),
	).toHaveAttribute("data-letter", "z");
	await dispatch(page, "KeyO", "r");
	await expect(composition).toHaveText("竹＊人");
	await expect(items.first()).toHaveText("1的人");
	await page.keyboard.press("Escape");
	// AZERTY 的實體 Z 鍵打 w。
	await dispatch(page, "KeyH", "h");
	await dispatch(page, "KeyZ", "w");
	await dispatch(page, "KeyO", "o");
	await expect(composition).toHaveText("竹＊人");
	await expect(items.first()).toHaveText("1的人");
});
