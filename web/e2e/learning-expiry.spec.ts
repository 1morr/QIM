import { expect, type Page, test } from "@playwright/test";
import { readyCleared, shown } from "./page.ts";

// 學習的時限（M4 票 13）：「選兩次成首選」的兩次要在一年內；最近一次整串上屏過了
// 365 天就不再是首選；一年沒用的條目停用（不刪）。用 `page.clock` 換日（網頁在 keydown 前把今天交給核心）。

/** 打 `code`，按數字鍵選文字是 `word` 的候選。 */
async function choose(page: Page, code: string, word: string) {
	await page.keyboard.type(code);
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
}

/** `localStorage` 存的學習資料裡，碼是 `code`、詞是 `word` 的 `entry` 列，拆成欄。 */
async function storedEntry(
	page: Page,
	code: string,
	word: string,
): Promise<string[] | undefined> {
	const stored = await page.evaluate(() =>
		localStorage.getItem("qim.userdict"),
	);
	return (stored ?? "")
		.split("\n")
		.map((line) => line.split("\t"))
		.find((c) => c[0] === "entry" && c[2] === word && c[3] === code);
}

test("選過兩次的候選，最近一次過了 365 天就不再是首選", async ({
	page,
}, testInfo) => {
	await page.clock.setFixedTime(new Date(2026, 0, 1, 12, 0));
	const { text } = await readyCleared(page);
	await text.focus();
	const first = page
		.getByRole("list", { name: "候選" })
		.getByRole("listitem")
		.first();

	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	expect(second, "a 要有第 2 個候選").toBeTruthy();
	await page.keyboard.press("Escape");
	await choose(page, "a", second);
	await choose(page, "a", second);
	await expect(text).toHaveValue(second + second);
	await expect
		// counts（第 5 欄）「編號:用過:整串上屏:不算的:最後整串上屏的日期」、last_used（第 12 欄）。
		.poll(async () => (await storedEntry(page, "a", second))?.[4])
		.toBe("0:2:2:0:2026-01-01");
	expect((await storedEntry(page, "a", second))?.[11]).toBe("2026-01-01");

	// 剛好 365 天：還在一年內。
	await page.clock.setFixedTime(new Date(2027, 0, 1, 12, 0));
	await page.keyboard.type("a");
	await expect(first).toHaveText(`1${second}`);
	await page.keyboard.press("Escape");

	// 366 天：兩次都在一年前，回到不學習時的首選。
	await page.clock.setFixedTime(new Date(2027, 0, 2, 12, 0));
	await page.keyboard.type("a");
	await expect(first).toHaveText(`1${top}`);
	await page.screenshot({ path: testInfo.outputPath("expired-top.png") });
});
