import { devices, expect, type Page, test } from "@playwright/test";

// 觸控鍵盤功能（M1 票 20）：長按、符號頁、自訂頁面、放大預覽、深色、橫向。
// 長按與按住用 CDP 的觸控事件，跟手指一樣先 touchStart、停住、再 touchEnd。
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

/** 一根手指：按下、移動、放開，座標是 CSS 像素。 */
interface Finger {
	down(x: number, y: number): Promise<void>;
	move(x: number, y: number): Promise<void>;
	up(): Promise<void>;
}

async function finger(page: Page): Promise<Finger> {
	const cdp = await page.context().newCDPSession(page);
	const touch = async (
		type: "touchStart" | "touchMove" | "touchEnd",
		touchPoints: { x: number; y: number }[],
	) => {
		await cdp.send("Input.dispatchTouchEvent", { type, touchPoints });
	};
	return {
		down: (x, y) => touch("touchStart", [{ x, y }]),
		move: (x, y) => touch("touchMove", [{ x, y }]),
		up: () => touch("touchEnd", []),
	};
}

async function centre(page: Page, testId: string): Promise<[number, number]> {
	const box = await page.getByTestId(testId).boundingBox();
	if (!box) {
		throw new Error(`${testId} 沒畫出來`);
	}
	return [box.x + box.width / 2, box.y + box.height / 2];
}

async function tapKeys(page: Page, ids: string[]): Promise<void> {
	for (const id of ids) {
		await page.getByTestId(`key-${id}`).tap();
	}
}

test("長按字母鍵彈出英文字母，放開上屏預設的那個", async ({
	page,
}, testInfo) => {
	const hand = await finger(page);
	const popup = page.getByTestId("long-press");
	await hand.down(...(await centre(page, "key-a")));
	await expect(popup.getByRole("option")).toHaveText(["a"]);
	await expect(popup.getByRole("option", { selected: true })).toHaveText("a");
	await page.screenshot({ path: testInfo.outputPath("long-press-a.png") });
	await hand.up();
	await expect(popup).toBeHidden();
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue("a");
	// 英文字母直接上屏，不進組字。
	await expect(page.getByTestId("composition")).toHaveText("");
});

test("長按第一排另含數字，滑到數字放開上屏數字", async ({ page }, testInfo) => {
	const hand = await finger(page);
	const popup = page.getByTestId("long-press");
	await hand.down(...(await centre(page, "key-q")));
	await expect(popup.getByRole("option")).toHaveText(["q", "1"]);
	const [x, y] = await centre(page, "long-press-1");
	await hand.move(x, y);
	await expect(popup.getByRole("option", { selected: true })).toHaveText("1");
	await page.screenshot({ path: testInfo.outputPath("long-press-q-1.png") });
	await hand.up();
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue("1");
});

test("組字中長按選字母：先上屏組字再接字母", async ({ page }) => {
	await tapKeys(page, ["a", "n", "r"]);
	const hand = await finger(page);
	await hand.down(...(await centre(page, "key-q")));
	await expect(page.getByTestId("long-press")).toBeVisible();
	await hand.up();
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"門口q",
	);
	await expect(page.getByTestId("composition")).toHaveText("");
});

test("點一下不彈長按清單，照常組字", async ({ page }) => {
	await tapKeys(page, ["q"]);
	await expect(page.getByTestId("long-press")).toBeHidden();
	await expect(page.getByTestId("composition")).toHaveText("q");
});

test("符號頁：打兩個符號仍在符號頁，按「ABC」回字母頁", async ({
	page,
}, testInfo) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await tapKeys(page, ["to-symbols"]);
	await expect(page.getByTestId("key-a")).toHaveCount(0);
	await tapKeys(page, ["1", "@"]);
	await expect(text).toHaveValue("1@");
	// 打完不跳回（brief §4.4）。
	await expect(page.getByTestId("key-to-letters")).toHaveText("ABC");
	await expect(page.getByTestId("key-？")).toBeVisible();
	await page.screenshot({ path: testInfo.outputPath("symbols.png") });
	await tapKeys(page, ["to-letters"]);
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await expect(page.getByTestId("key-@")).toHaveCount(0);
});

test("組字中切到符號頁組字留著，按符號先上屏組字再接符號", async ({ page }) => {
	await tapKeys(page, ["a", "n", "r", "to-symbols"]);
	await expect(page.getByTestId("composition")).toHaveText("anr");
	await tapKeys(page, ["？"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"門口？",
	);
	await expect(page.getByTestId("composition")).toHaveText("");
});

test("自訂頁面：由符號頁切過去輸入常用語與表情，再回字母頁", async ({
	page,
}, testInfo) => {
	await tapKeys(page, ["to-symbols", "to-phrases"]);
	await tapKeys(page, ["唔該", "👍"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"唔該👍",
	);
	await page.screenshot({ path: testInfo.outputPath("phrases.png") });
	await tapKeys(page, ["to-letters", "a", "space"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"唔該👍日",
	);
});

test("按住按鍵顯示放大預覽，放開消失", async ({ page }, testInfo) => {
	const hand = await finger(page);
	const preview = page.getByTestId("key-preview");
	await expect(preview).toBeHidden();
	await hand.down(...(await centre(page, "key-n")));
	await expect(preview).toHaveText("弓");
	// 預覽比按鍵大、在按鍵上方。
	const key = await page.getByTestId("key-n").boundingBox();
	const shown = await preview.boundingBox();
	if (!key || !shown) {
		throw new Error("預覽或按鍵沒畫出來");
	}
	expect(shown.height).toBeGreaterThan(key.height);
	expect(shown.y).toBeLessThan(key.y);
	await page.screenshot({ path: testInfo.outputPath("preview.png") });
	await hand.up();
	await expect(preview).toBeHidden();
	await expect(page.getByTestId("composition")).toHaveText("n");
});

test("系統深色時按鍵用佈局的深色版本", async ({ page }, testInfo) => {
	const key = page.getByTestId("key-a");
	await expect(key).toHaveCSS("background-color", "rgb(255, 255, 255)");
	await page.emulateMedia({ colorScheme: "dark" });
	await expect(key).toHaveCSS("background-color", "rgb(60, 64, 67)");
	await expect(key.locator(".label")).toHaveCSS("color", "rgb(227, 227, 227)");
	await expect(page.getByTestId("key-enter")).toHaveCSS(
		"background-color",
		"rgb(95, 99, 104)",
	);
	await expect(page.getByRole("button", { name: "深色" })).toHaveAttribute(
		"aria-pressed",
		"true",
	);
	await page.screenshot({ path: testInfo.outputPath("dark.png") });
});

test("網頁的深色開關蓋過系統設定", async ({ page }) => {
	const key = page.getByTestId("key-a");
	const toggle = page.getByRole("button", { name: "深色" });
	await expect(toggle).toHaveAttribute("aria-pressed", "false");
	await toggle.tap();
	await expect(toggle).toHaveAttribute("aria-pressed", "true");
	await expect(key).toHaveCSS("background-color", "rgb(60, 64, 67)");
	await toggle.tap();
	await expect(key).toHaveCSS("background-color", "rgb(255, 255, 255)");
	// 手動切成淺色後，系統改深色也不跟。
	await page.emulateMedia({ colorScheme: "dark" });
	await expect(key).toHaveCSS("background-color", "rgb(255, 255, 255)");
});

test("轉成橫向換橫向的佈局版本，仍打得出「門口」", async ({
	page,
}, testInfo) => {
	const keyboard = page.getByTestId("keyboard");
	const portrait = await keyboard.boundingBox();
	await page.setViewportSize({ width: 839, height: 412 });
	await expect
		.poll(async () => {
			const box = await keyboard.boundingBox();
			return box ? box.width / box.height : 0;
		})
		.toBeCloseTo(844 / 160, 1);
	const landscape = await keyboard.boundingBox();
	if (!portrait || !landscape) {
		throw new Error("鍵盤沒畫出來");
	}
	expect(landscape.height).toBeLessThan(portrait.height);
	// 鍵盤沒蓋住輸入框：頁面底部留了鍵盤的高度。
	const padding = await page.evaluate(() =>
		Number.parseFloat(getComputedStyle(document.body).paddingBottom),
	);
	expect(padding).toBeCloseTo(landscape.height, 0);

	await tapKeys(page, ["a", "n", "r"]);
	await expect(
		page.getByRole("list", { name: "候選" }).getByRole("listitem").first(),
	).toHaveText("1門口");
	await page.screenshot({ path: testInfo.outputPath("landscape.png") });
	await tapKeys(page, ["space"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"門口",
	);
});

test("轉向時留在同一頁", async ({ page }) => {
	await tapKeys(page, ["to-symbols"]);
	await page.setViewportSize({ width: 839, height: 412 });
	await expect(page.getByTestId("key-to-letters")).toBeVisible();
	await expect(page.getByTestId("key-a")).toHaveCount(0);
});

test("快打時前一指還沒放開就按下一鍵，兩個鍵都打得出、順序不亂", async ({
	page,
}) => {
	const cdp = await page.context().newCDPSession(page);
	const [ax, ay] = await centre(page, "key-a");
	const [nx, ny] = await centre(page, "key-n");
	const [rx, ry] = await centre(page, "key-r");
	const a = { x: ax, y: ay, id: 1 };
	const n = { x: nx, y: ny, id: 2 };
	const r = { x: rx, y: ry, id: 3 };
	const send = (
		type: "touchStart" | "touchEnd",
		touchPoints: { x: number; y: number; id: number }[],
	) => cdp.send("Input.dispatchTouchEvent", { type, touchPoints });
	// CDP 的 touchStart 列出按著的手指（含新按下的），touchEnd 列出這次放開的手指。
	// a 按著時按 n，放開 a；n 按著時按 r，放開 n；放開 r。
	await send("touchStart", [a]);
	await send("touchStart", [a, n]);
	await send("touchEnd", [a]);
	await send("touchStart", [n, r]);
	await send("touchEnd", [n]);
	await send("touchEnd", [r]);
	await expect(page.getByTestId("composition")).toHaveText("anr");
	await expect(page.getByTestId("key-preview")).toBeHidden();
});
