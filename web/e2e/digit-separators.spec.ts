import { expect, type Page, test } from "@playwright/test";

// 數字後的句點、冒號用半形（M1 票 26，brief〈英文模式與欄位類型〉）：前一個上屏的字是半形數字時直接上屏半形，
// 不待定、不出候選；逗號照舊全形（使用者決定）。開關預設開。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		toggle: page.getByRole("checkbox", { name: "數字後半形" }),
		composition: page.getByTestId("composition"),
	};
}

test("打 3.5、12:30 讀回半形、不進組字，數字後的逗號是全形", async ({
	page,
}, testInfo) => {
	const { text, toggle, composition } = await ready(page);
	await expect(toggle).toBeChecked();
	await text.focus();
	await page.keyboard.type("3.");
	await expect(text).toHaveValue("3.");
	await expect(composition).toHaveText("");
	await page.screenshot({ path: testInfo.outputPath("digit-separator.png") });
	// 冒號用 Shift 加鍵送（中文模式照實體鍵位與 Shift 換成美式字元，票 07）。
	await page.keyboard.type("5 12");
	await page.keyboard.press("Shift+Semicolon");
	await page.keyboard.type("30 1,");
	await expect(text).toHaveValue("3.5 12:30 1，");
});

test("不在數字後、或數字後連按兩下，第二個是全形", async ({ page }) => {
	const { text } = await ready(page);
	await text.focus();
	await page.keyboard.type(".");
	await expect(text).toHaveValue("。");
	await page.keyboard.type("3..");
	await expect(text).toHaveValue("。3.。");
});

test("關掉開關後數字後也是全形", async ({ page }) => {
	const { text, toggle } = await ready(page);
	await toggle.uncheck();
	await expect(text).toBeFocused();
	await page.keyboard.type("3.");
	await expect(text).toHaveValue("3。");
});

test("留碼接英文時數字與標點照原樣接在英文後", async ({ page }) => {
	const { text } = await ready(page);
	await text.focus();
	await page.keyboard.type("anr");
	// Playwright 的 `ShiftLeft` 是左 Shift，預設留碼接英文（brief〈桌面鍵盤操作〉）。
	await page.keyboard.press("ShiftLeft");
	// 空白也接進英文（票 25b）。
	await page.keyboard.type("3.5,0 1:2.");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anr3.5,0 1:2.");
});

test("數字後按了留給瀏覽器的組合鍵（Ctrl+B 移游標）就不算緊接在數字後", async ({
	page,
}) => {
	const { text } = await ready(page);
	await text.focus();
	await page.keyboard.type("3");
	await page.keyboard.press("Control+KeyB");
	await page.keyboard.type(".");
	await expect(text).not.toHaveValue(/\./);
	await expect(text).toHaveValue(/。/);
});

// 單按修飾鍵沒有送字元（Rime 也不記），數字還沒結束（M1 票 26b）。
for (const modifier of ["Control", "Meta", "Alt", "CapsLock"]) {
	test(`數字後單按 ${modifier} 仍算緊接在數字後`, async ({ page }) => {
		const { text } = await ready(page);
		await text.focus();
		await page.keyboard.type("3");
		await page.keyboard.press(modifier);
		await page.keyboard.type(".");
		await expect(text).toHaveValue("3.");
	});
}

test("組字中單按 Ctrl 吃下，組字與輸入框都不變", async ({ page }) => {
	const { text, composition } = await ready(page);
	await text.focus();
	await page.keyboard.type("a");
	await page.keyboard.press("Control");
	await expect(composition).toHaveText("日");
	await expect(text).toHaveValue("");
	await page.keyboard.press(" ");
	await expect(text).toHaveValue("日");
});
