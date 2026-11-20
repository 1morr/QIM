import { expect, type Page, test } from "@playwright/test";
import { ready as baseReady } from "./page.ts";

// 中文狀態的英文候選（M1 票 15）：整串輸入碼剛好是完整英文單字時排第 2 位，按 2 上屏；
// 不容錯、不補全、不搶首選；開關預設開。

async function ready(page: Page) {
	return {
		...(await baseReady(page)),
		english: page.getByRole("checkbox", { name: "英文候選" }),
	};
}

test("打 the，第 2 位是 the，按 2 上屏 the", async ({ page }, testInfo) => {
	const { text, english, items } = await ready(page);
	await expect(english).toBeChecked();
	await text.focus();
	await page.keyboard.type("the");
	await expect(items.nth(1)).toHaveText("2the");
	await expect(items.first()).not.toHaveText("1the");
	await page.screenshot({ path: testInfo.outputPath("english-the.png") });

	await page.keyboard.press("2");
	await expect(text).toHaveValue("the");
	await expect(items).toHaveCount(0);
});

test("拼錯一碼、只打前幾碼、2 碼的英文字都沒有英文候選", async ({ page }) => {
	const { text, items } = await ready(page);
	await text.focus();
	for (const code of ["thw", "th", "an"]) {
		await page.keyboard.type(code);
		await expect(items.first()).toBeVisible();
		for (const item of await items.allTextContents()) {
			expect(item, `${code}：${item}`).not.toMatch(/^\d[a-z]+$/i);
		}
		await page.keyboard.press("Escape");
	}
});

test("關掉英文候選就不出，組字中切換立刻重列", async ({ page }) => {
	const { text, english, items } = await ready(page);
	await text.focus();
	await page.keyboard.type("the");
	await expect(items.nth(1)).toHaveText("2the");

	await english.uncheck();
	await expect(items.filter({ hasText: /^\dthe$/ })).toHaveCount(0);
	await expect(text).toBeFocused();

	await english.check();
	await expect(items.nth(1)).toHaveText("2the");
});

test("單按 Shift 切英文時打 the 直接進輸入框、沒有候選；切回中文 the 仍在第 2 位", async ({
	page,
}) => {
	const { text, english, items } = await ready(page);
	await text.focus();
	await page.keyboard.press("Shift");
	await page.keyboard.type("the");
	await expect(text).toHaveValue("the");
	await expect(items).toHaveCount(0);
	// 英文狀態下切英文候選開關也不出候選。
	await english.uncheck();
	await english.check();
	await expect(items).toHaveCount(0);

	await text.focus();
	await page.keyboard.press("Shift");
	await page.keyboard.type("the");
	await expect(items.nth(1)).toHaveText("2the");
	await page.keyboard.press("2");
	await expect(text).toHaveValue("thethe");
});

// 英文資料包換成 AOSP LatinIME 的詞表（M4 票 22）：常用詞照樣在第 2 位，debug 面板讀得到來源是英文；
// 錯拼不在詞表（LatinIME 只把它當 shortcut 的錯拼不收），沒有英文候選。
test("打 receive 第 2 位是 receive、來源是英文；打 recieve 沒有英文候選", async ({
	page,
}, testInfo) => {
	const { text, items } = await ready(page);
	await page.getByRole("button", { name: "debug" }).click();
	const debug = page.locator("#debug");
	await expect(debug).toBeVisible();
	await text.focus();

	await page.keyboard.type("receive");
	await expect(items.nth(1)).toHaveText("2receive");
	const row = debug
		.getByRole("table", { name: "候選來源" })
		.locator("tbody tr")
		.filter({ has: page.locator("td:nth-child(2)", { hasText: /^receive$/ }) });
	await expect(row).toHaveAttribute("data-kind", "english");
	await page.screenshot({ path: testInfo.outputPath("english-receive.png") });
	await page.keyboard.press("Escape");

	await page.keyboard.type("recieve");
	await expect(items.first()).toBeVisible();
	for (const item of await items.allTextContents()) {
		expect(item, `recieve：${item}`).not.toMatch(/^\d[a-z]+$/i);
	}
	await expect(debug.locator('tbody tr[data-kind="english"]')).toHaveCount(0);
});
