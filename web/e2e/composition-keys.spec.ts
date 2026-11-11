import { expect, type Page, test } from "@playwright/test";

// 組字中按任何鍵都不漏字元進輸入框、組字也不殘留（brief §4.3，M1 票 10）。

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	const { text } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
});

test("組字中按 Enter，輸入碼原樣上屏", async ({ page }, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anr");
	await expect(composition).toHaveText("");
	await expect(items).toHaveCount(0);
	await page.screenshot({ path: testInfo.outputPath("enter.png") });
});

test("選了開頭一段再按 Enter，上屏選定的字與其餘的輸入碼", async ({ page }) => {
	const { text, composition, items } = ui(page);
	// 門口有人：門 an、口 r、有 kb、人 o。
	await page.keyboard.type("anrkbo");
	const texts = (await items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
	const at = texts.indexOf("門口");
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.press(String(at + 1));
	await expect(composition).toHaveText("門口大月人");

	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("門口kbo");
	await expect(composition).toHaveText("");
});

test("組字中按 z、←、→ 吃下不動作", async ({ page }, testInfo) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("anr");
	const before = await items.allTextContents();
	for (const key of ["z", "ArrowLeft", "ArrowRight", "ArrowLeft"]) {
		await page.keyboard.press(key);
		await expect(text, key).toHaveValue("");
		await expect(composition, key).toHaveText("日弓口");
		// ← → 移動目前分段：分段的候選照不知道前文時單打它的碼（plan §2.1），不一定和接著句首的
		// 整串候選同序（M3 票 11b 起詞級 trigram 看得出句首），只比首選。
		if (key === "z") await expect(items, key).toHaveText(before);
		else await expect(items.first(), key).toHaveText(before[0]);
	}
	await page.screenshot({ path: testInfo.outputPath("swallowed.png") });
	await page.keyboard.type(" ");
	await expect(text).toHaveValue("門口");
});

// 亂按時找到的：這些鍵網頁原本不轉給核心，組字中 Tab 移走焦點、Delete 刪掉輸入框的字、
// 方向鍵與 Home / End 移動游標，上屏的字就落到別處。↑ ↓ 後來改成開關展開面板（票 23）。
test("組字中按 Tab、Delete、↑、↓、Home、End，輸入框與組字不變", async ({
	page,
}) => {
	const { text, composition, items } = ui(page);
	await page.keyboard.type("日月");
	await page.keyboard.press("ArrowLeft");
	await page.keyboard.type("anr");
	// 首選接前文「日」（M3 票 05），按這些鍵前後都一樣。
	const top = (await items.first().textContent())?.slice(1);
	for (const key of ["Tab", "Delete", "ArrowUp", "ArrowDown", "Home", "End"]) {
		await page.keyboard.press(key);
		await expect(text, key).toBeFocused();
		await expect(text, key).toHaveValue("日月");
		await expect(composition, key).toHaveText("日弓口");
	}
	await page.keyboard.type(" ");
	await expect(text).toHaveValue(`日${top}月`);
});

test("組字中按標點，先上屏首選再接全形標點", async ({ page }, testInfo) => {
	const { text, composition } = ui(page);
	await page.keyboard.type("anr,");
	await expect(text).toHaveValue("門口，");
	await expect(composition).toHaveText("");
	await page.keyboard.type("a.");
	await expect(text).toHaveValue("門口，日。");
	// 沒有全形的符號照原樣接在後面。
	await page.keyboard.type("a@");
	await expect(text).toHaveValue("門口，日。日@");
	await expect(composition).toHaveText("");
	await page.screenshot({ path: testInfo.outputPath("punct.png") });
});

test("組字中按大寫字母，先上屏首選再接那個字母", async ({ page }) => {
	const { text, composition } = ui(page);
	await page.keyboard.type("anr");
	await page.keyboard.press("Shift+A");
	await expect(text).toHaveValue("門口A");
	await expect(composition).toHaveText("");
});

test("沒有組字時：標點出全形，其他符號、大寫、Enter 照瀏覽器", async ({
	page,
}) => {
	const { text, composition } = ui(page);
	await page.keyboard.type(",?");
	await expect(text).toHaveValue("，？");
	await page.keyboard.type("@");
	await page.keyboard.press("Shift+B");
	await page.keyboard.press("Enter");
	await page.keyboard.press("ArrowLeft");
	await page.keyboard.type("z");
	await expect(text).toHaveValue("，？@Bz\n");
	await expect(composition).toHaveText("");
});
