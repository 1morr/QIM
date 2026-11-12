import { expect, type Page, test } from "@playwright/test";

// 英文使用者詞（M4 票 14，取代 M3 的記住的字串）：英文直通打過的串（4 字元起）都學。英文詞典沒有的打過
// 2 次，中文組字打整串時排第 2（規則 4）；打過 3 次又是最常用的 20 條之一，打開頭 3 碼就有（規則 5，
// 詞典有的詞不給）。設定區列出、可刪，重新整理後還在；無痕不學。v2 匯入的記住的字串是已過門檻的。
// `iva` 剛好是英文詞典的「Iva」（人名），它在第 2 位，email 的補完排在它後面的第 3 位。

const EMAIL = "ivan.test@example.com";

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		list: page.getByRole("list", { name: "英文使用者詞清單" }),
	};
}

/** 單按 Shift 切英文直通打 `typed`，再單按 Shift 切回中文。 */
async function typeInEnglish(page: Page, typed: string) {
	await page.keyboard.press("Shift");
	await page.keyboard.type(typed);
	await page.keyboard.press("Shift");
}

/** 打開 debug 面板、焦點回輸入框；回傳文字是 `word` 的候選那列。 */
async function openDebug(page: Page) {
	await page.getByRole("button", { name: "debug" }).click();
	await page.getByRole("textbox", { name: "輸入框" }).focus();
	const rows = page
		.getByRole("table", { name: "候選來源" })
		.locator("tbody tr");
	return (word: string) =>
		rows.filter({
			has: page.locator("td:nth-child(2)", {
				hasText: new RegExp(`^${word}$`),
			}),
		});
}

test("英文直通打 rrrrrr 一次，中文打 rrrrrr 沒有它；打第二次後第 2 位有，面板標英文使用者詞", async ({
	page,
}, testInfo) => {
	const { text, items, list } = await ready(page);
	const row = await openDebug(page);
	await expect(list).toHaveText("還沒有英文使用者詞");

	await typeInEnglish(page, "rrrrrr ");
	await expect(text).toHaveValue("rrrrrr ");
	await expect(list.getByRole("listitem")).toHaveText(["rrrrrr刪除"]);
	await page.keyboard.type("rrrrrr");
	await expect(items.first()).toBeVisible();
	await expect(items.filter({ hasText: /^\drrrrrr$/ })).toHaveCount(0);
	await page.keyboard.press("Escape");

	await typeInEnglish(page, "rrrrrr ");
	await page.keyboard.type("rrrrrr");
	await expect(items.nth(1)).toHaveText("2rrrrrr");
	await expect(row("rrrrrr")).toHaveAttribute("data-kind", "englishUserWord");
	await expect(row("rrrrrr")).toContainText("英文使用者詞");
	await page.screenshot({
		path: testInfo.outputPath("14-english-user-word-second.png"),
		fullPage: true,
	});
	await page.keyboard.press("2");
	await expect(text).toHaveValue("rrrrrr rrrrrr rrrrrr");
});

/** M3 定稿的 v2 檔：一條記住的 email（方案欄、前一個字詞欄都空的列）。 */
const V2_WITH_EMAIL = [
	"# qim-userdict v2",
	"#@/tick\t1",
	"# scheme\tword\tcode\tcommits\tlast_commit_tick\tdecayed\tlast_tick\tsource\ttrigger",
	`\t\t${EMAIL}\t1\t1`,
	"",
].join("\n");

test("v2 匯入的 email 在中文打開頭 3 碼就有，面板標英文使用者詞", async ({
	page,
}, testInfo) => {
	const { text, items, list } = await ready(page);
	await page.getByLabel("匯入學習資料（合併進目前的）").setInputFiles({
		name: "v2.tsv",
		mimeType: "text/plain",
		buffer: Buffer.from(V2_WITH_EMAIL),
	});
	await page
		.getByRole("group", { name: "確認匯入" })
		.getByRole("button", { name: "確定匯入" })
		.click();
	await expect(list.getByRole("listitem")).toHaveText([`${EMAIL}刪除`]);

	const row = await openDebug(page);
	await text.focus();
	await page.keyboard.type("iva");
	await expect(items.nth(1)).toHaveText("2Iva");
	await expect(items.nth(2)).toHaveText(`3${EMAIL}`);
	await expect(row(EMAIL)).toHaveAttribute("data-kind", "englishUserWord");
	await expect(row(EMAIL)).toContainText("英文使用者詞");
	await page.screenshot({
		path: testInfo.outputPath("14-v2-email-completion.png"),
		fullPage: true,
	});
	await page.keyboard.press("3");
	await expect(text).toHaveValue(EMAIL);
});

test("打 3 次的 email 才有前綴補完；常打的詞典詞不把它擠掉", async ({
	page,
}) => {
	const { items } = await ready(page);
	await typeInEnglish(page, `${EMAIL} ${EMAIL} `);
	await page.keyboard.type("iva");
	await expect(items.nth(1)).toHaveText("2Iva");
	await expect(items.filter({ hasText: EMAIL })).toHaveCount(0);
	await page.keyboard.press("Escape");

	await typeInEnglish(page, `${EMAIL} `);
	for (let i = 0; i < 4; i++) {
		await typeInEnglish(page, "that have with there about ");
	}
	await page.keyboard.type("iva");
	await expect(items.nth(2)).toHaveText(`3${EMAIL}`);
	await page.keyboard.press("Escape");
	await page.keyboard.type("tha");
	await expect(items.first()).toBeVisible();
	await expect(items.filter({ hasText: /^\dthat$/ })).toHaveCount(0);
});

test("打完直接失焦也學，重新整理後還在；刪除後不再出", async ({
	page,
}, testInfo) => {
	const { text, items, list } = await ready(page);
	await typeInEnglish(page, "rrrrrr ");
	await page.keyboard.press("Shift");
	await page.keyboard.type("rrrrrr");
	await page.getByRole("heading", { name: "英文使用者詞" }).click();
	await expect(list.getByRole("listitem")).toHaveText(["rrrrrr刪除"]);

	await page.reload();
	await expect(text).toHaveAttribute("data-ready", "true");
	await expect(list.getByRole("listitem")).toHaveText(["rrrrrr刪除"]);
	await text.focus();
	await page.keyboard.type("rrrrrr");
	await expect(items.nth(1)).toHaveText("2rrrrrr");
	await page.keyboard.press("Escape");
	await page.screenshot({
		path: testInfo.outputPath("14-english-user-words-list.png"),
		fullPage: true,
	});

	await page.getByRole("button", { name: "刪除 rrrrrr" }).click();
	await expect(list).toHaveText("還沒有英文使用者詞");
	await expect(text).toBeFocused();
	await page.keyboard.type("rrrrrr");
	await expect(items.first()).toBeVisible();
	await expect(items.filter({ hasText: /^\drrrrrr$/ })).toHaveCount(0);
});

test("無痕時不學；關掉「英文使用者詞」就不出", async ({ page }) => {
	const { text, items, list } = await ready(page);
	await page.getByRole("checkbox", { name: "無痕" }).check();
	await text.focus();
	await typeInEnglish(page, "rrrrrr rrrrrr ");
	await expect(list).toHaveText("還沒有英文使用者詞");
	await page.getByRole("checkbox", { name: "無痕" }).uncheck();

	await text.focus();
	await typeInEnglish(page, "rrrrrr rrrrrr ");
	await page.keyboard.type("rrrrrr");
	await expect(items.nth(1)).toHaveText("2rrrrrr");
	const option = page.getByRole("checkbox", { name: "英文使用者詞" });
	await expect(option).toBeChecked();
	await option.uncheck();
	await expect(items.filter({ hasText: /^\drrrrrr$/ })).toHaveCount(0);
	await expect(text).toBeFocused();
});
