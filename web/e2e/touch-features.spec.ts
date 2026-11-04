import { readFileSync } from "node:fs";
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

/** 長按字母頁的逗號鍵進常用頁（代替 Gboard 逗號鍵的表情入口，M1 票 27）。 */
async function openPhrases(page: Page): Promise<void> {
	const hand = await finger(page);
	await hand.down(...(await centre(page, "key-，")));
	await expect(page.getByTestId("long-press").getByRole("option")).toHaveText([
		"切到 常用",
	]);
	await hand.up();
	await expect(page.getByTestId("key-唔該")).toBeVisible();
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

// 使用者 2026-11-05 決定（M1 票 27）：長按只出字母，數字改由設定的數字列。
test("長按第一排的 q 只有字母、沒有數字", async ({ page }) => {
	const hand = await finger(page);
	const popup = page.getByTestId("long-press");
	await hand.down(...(await centre(page, "key-q")));
	await expect(popup.getByRole("option")).toHaveText(["q"]);
	await hand.up();
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue("q");
});

test("數字列預設關；打開後字母頁上方多一排數字，點「1」輸入 1，關掉就不見", async ({
	page,
}, testInfo) => {
	const numberRow = page.getByRole("checkbox", { name: "數字列" });
	await expect(numberRow).not.toBeChecked();
	await expect(page.getByTestId("key-1")).toHaveCount(0);
	const keyboard = page.getByTestId("keyboard");
	const before = await keyboard.boundingBox();
	await numberRow.check();
	await tapKeys(page, ["1", "a", "space"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"1日",
	);
	const after = await keyboard.boundingBox();
	expect(after?.height).toBeGreaterThan(before?.height ?? Infinity);
	await page.screenshot({ path: testInfo.outputPath("number-row.png") });
	await numberRow.uncheck();
	await expect(page.getByTestId("key-1")).toHaveCount(0);
	await tapKeys(page, ["a"]);
	await expect(page.getByTestId("composition")).toHaveText("日");
});

// Shift（brief §4.4，照 Gboard 速成實測，M1 票 27）。
test("點一下 Shift：字母鍵換成大寫，點「日」上屏 A，打完自動回到字根", async ({
	page,
}, testInfo) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	const shift = page.getByTestId("key-shift");
	await expect(shift).toHaveAttribute("data-shift", "off");
	await tapKeys(page, ["shift"]);
	await expect(shift).toHaveAttribute("data-shift", "once");
	await expect(page.getByTestId("key-a")).toHaveText("A");
	await expect(page.getByTestId("key-z")).toHaveText("Z");
	await page.screenshot({ path: testInfo.outputPath("shift-once.png") });
	await tapKeys(page, ["a"]);
	await expect(text).toHaveValue("A");
	await expect(page.getByTestId("composition")).toHaveText("");
	await expect(shift).toHaveAttribute("data-shift", "off");
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await tapKeys(page, ["a"]);
	await expect(page.getByTestId("composition")).toHaveText("日");
});

test("快速點兩下 Shift 鎖定大寫，再點一下解除；隔太久的第二下是關掉", async ({
	page,
}) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	const shift = page.getByTestId("key-shift");
	await tapKeys(page, ["shift", "shift"]);
	await expect(shift).toHaveAttribute("data-shift", "locked");
	await tapKeys(page, ["a", "s", "d"]);
	await expect(text).toHaveValue("ASD");
	await expect(shift).toHaveAttribute("data-shift", "locked");
	await tapKeys(page, ["shift"]);
	await expect(shift).toHaveAttribute("data-shift", "off");
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await tapKeys(page, ["shift"]);
	await page.waitForTimeout(600);
	await tapKeys(page, ["shift"]);
	await expect(shift).toHaveAttribute("data-shift", "off");
});

test("組字中點 Shift 再點字母：先上屏組字再接大寫字母", async ({ page }) => {
	await tapKeys(page, ["a", "n", "r", "shift"]);
	await expect(page.getByTestId("composition")).toHaveText("日弓口");
	await tapKeys(page, ["a"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"門口A",
	);
	await expect(page.getByTestId("composition")).toHaveText("");
});

test("Shift 開著時長按字母鍵出小寫，之後 Shift 解除", async ({ page }) => {
	await tapKeys(page, ["shift"]);
	const hand = await finger(page);
	await hand.down(...(await centre(page, "key-a")));
	await expect(page.getByTestId("long-press").getByRole("option")).toHaveText([
		"a",
	]);
	await hand.up();
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue("a");
	await expect(page.getByTestId("key-shift")).toHaveAttribute(
		"data-shift",
		"off",
	);
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
	await expect(page.getByTestId("composition")).toHaveText("手");
});

test("符號頁：打兩個符號仍在符號頁，按「返回」回字母頁", async ({
	page,
}, testInfo) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await tapKeys(page, ["to-symbols"]);
	await expect(page.getByTestId("key-a")).toHaveCount(0);
	await tapKeys(page, ["1", "@"]);
	await expect(text).toHaveValue("1@");
	// 打完不跳回（brief §4.4）。
	await expect(page.getByTestId("key-to-letters")).toHaveText("返回");
	await expect(page.getByTestId("key-?")).toBeVisible();
	await page.screenshot({ path: testInfo.outputPath("symbols.png") });
	await tapKeys(page, ["to-letters"]);
	await expect(page.getByTestId("key-a")).toHaveText("日a");
	await expect(page.getByTestId("key-@")).toHaveCount(0);
});

test("組字中切到符號頁組字留著，按符號先上屏組字再接符號", async ({ page }) => {
	await tapKeys(page, ["a", "n", "r", "to-symbols"]);
	await expect(page.getByTestId("composition")).toHaveText("日弓口");
	await tapKeys(page, ["?"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"門口?",
	);
	await expect(page.getByTestId("composition")).toHaveText("");
});

test("自訂頁面：長按逗號鍵切過去輸入常用語與表情，再回字母頁", async ({
	page,
}, testInfo) => {
	await openPhrases(page);
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
	await expect(page.getByTestId("composition")).toHaveText("弓");
});

// 顏色照 Gboard 實測（M1 票 27）。
test("系統深色時按鍵用佈局的深色版本", async ({ page }, testInfo) => {
	const key = page.getByTestId("key-a");
	await expect(key).toHaveCSS("background-color", "rgb(255, 255, 255)");
	// Enter 的佈局顏色淺深都和 CSS 的預設底色不同，讀得出佈局的顏色真的套上了。
	await expect(page.getByTestId("key-enter")).toHaveCSS(
		"background-color",
		"rgb(220, 226, 249)",
	);
	await page.emulateMedia({ colorScheme: "dark" });
	await expect(key).toHaveCSS("background-color", "rgb(41, 44, 52)");
	await expect(key.locator(".label")).toHaveCSS("color", "rgb(228, 229, 240)");
	await expect(page.getByTestId("key-enter")).toHaveCSS(
		"background-color",
		"rgb(53, 59, 77)",
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
	await expect(key).toHaveCSS("background-color", "rgb(41, 44, 52)");
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
	await expect(page.getByTestId("composition")).toHaveText("日弓口");
	await expect(page.getByTestId("key-preview")).toBeHidden();
});

test("核心沒吃下長按選的動作時，網頁照那個動作做，不是照鍵的 tap", async ({
	page,
}) => {
	// 自訂佈局：a 的長按清單是刪除。沒組字時核心不吃刪除，網頁要刪輸入框的字、不是插入 a。
	const layout = JSON.parse(
		readFileSync(
			new URL(
				"../../crates/qim-layout/tests/fixtures/overlap.json",
				import.meta.url,
			),
			"utf8",
		),
	);
	const a = layout.variants[0].pages[0].keys.find(
		(key: { id: string }) => key.id === "a",
	);
	a.longPress = ["backspace"];
	await page.route("**/layouts/gboard-quick.json", (route) =>
		route.fulfill({
			contentType: "application/json",
			body: JSON.stringify(layout),
		}),
	);
	await page.reload();
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await tapKeys(page, ["bang", "bang"]);
	await expect(text).toHaveValue("！！");
	const hand = await finger(page);
	const box = await page.getByTestId("key-a").boundingBox();
	if (!box) {
		throw new Error("a 鍵沒畫出來");
	}
	await hand.down(box.x + 10, box.y + box.height / 2);
	await expect(page.getByTestId("long-press").getByRole("option")).toHaveText([
		"刪除",
	]);
	await hand.up();
	await expect(text).toHaveValue("！");
});

test("觸控的空白鍵選字上屏可以回報，組字中點符號上屏不算", async ({ page }) => {
	// 「回報」（票 12）看綁定回報的實際動作：空白才是選字。
	const report = page.getByRole("button", { name: "回報", exact: true });
	const reports = page.getByRole("textbox", { name: /回報的固定題/ });
	await tapKeys(page, ["a", "n", "r", "space"]);
	await expect(report).toBeEnabled();
	await report.tap();
	await expect(reports).toHaveValue("anr\t門口\t網頁回報（首選 門口）");

	await tapKeys(page, ["a", "n", "r", "to-symbols", "?"]);
	await expect(page.getByRole("textbox", { name: "輸入框" })).toHaveValue(
		"門口門口?",
	);
	await expect(report).toBeDisabled();
});

test("點候選選字之後，長按與放大預覽照常", async ({ page }) => {
	// 票 21 的點候選選字在候選列，長按與預覽在鍵盤上，兩邊交錯用都不互相干擾。
	const text = page.getByRole("textbox", { name: "輸入框" });
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await tapKeys(page, ["a", "n", "r"]);
	const second = (await items.nth(1).textContent())?.replace(/^2/, "") ?? "";
	expect(second).not.toBe("");
	await items.nth(1).tap();
	await expect(text).toHaveValue(second);
	await expect(page.getByTestId("composition")).toHaveText("");

	const hand = await finger(page);
	await tapKeys(page, ["a"]);
	await hand.down(...(await centre(page, "key-n")));
	await expect(page.getByTestId("key-preview")).toHaveText("弓");
	await hand.up();
	await expect(page.getByTestId("composition")).toHaveText("日弓");
	await items.first().tap();
	await hand.down(...(await centre(page, "key-q")));
	await expect(page.getByTestId("long-press").getByRole("option")).toHaveText([
		"q",
	]);
	await hand.up();
	await expect(page.getByTestId("long-press")).toBeHidden();
	await expect(text).toHaveValue(new RegExp(`^${second}.+q$`));
	await expect(page.getByTestId("composition")).toHaveText("");
});

test("簡體模式下觸控打字：空白上屏簡體，組字中點符號先上屏簡體組字", async ({
	page,
}) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await page.getByLabel("簡體").tap();
	await tapKeys(page, ["a", "n", "r", "space"]);
	await expect(text).toHaveValue("门口");
	await tapKeys(page, ["a", "n", "r", "to-symbols", "?"]);
	await expect(text).toHaveValue("门口门口?");
});

// 使用者 2026-11-04 決定（M1 票 23）：「常用」頁的文字鍵在簡體模式也轉簡體。
test("「常用」頁的「多謝」：簡體模式上屏「多谢」，繁體模式「多謝」", async ({
	page,
}) => {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await openPhrases(page);
	await tapKeys(page, ["多謝"]);
	await expect(text).toHaveValue("多謝");
	await page.getByLabel("簡體").tap();
	await tapKeys(page, ["多謝"]);
	await expect(text).toHaveValue("多謝多谢");
});
