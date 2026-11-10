import { expect, type Page, test } from "@playwright/test";

// emoji 候選（M3 票 16）：整串的候選是 CLDR 標註裡的詞時，它的 emoji 接在它後面（rime-emoji 的
// 做法），整列最多 3 個；按數字鍵上屏 emoji；開關預設開。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		emoji: page.getByRole("checkbox", { name: "emoji 候選" }),
		simplified: page.getByRole("checkbox", { name: "簡體" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	return (await items.allTextContents()).map((t) => t.replace(/^\d/, ""));
}

test("打「笑」（hk），「笑」後面接著 😆，按它的序號上屏 😆", async ({
	page,
}, testInfo) => {
	const { text, emoji, items } = await ready(page);
	await expect(emoji).toBeChecked();
	await text.focus();
	await page.keyboard.type("hk");
	await expect(items.first()).toBeVisible();
	const list = await shown(page);
	const laugh = list.indexOf("笑");
	expect(laugh, list.join(" ")).toBeGreaterThanOrEqual(0);
	expect(list.slice(laugh + 1, laugh + 4)).toEqual(["😆", "🤣", "😂"]);
	await page.screenshot({ path: testInfo.outputPath("emoji-laugh.png") });

	await page.keyboard.press(String(laugh + 2));
	await expect(text).toHaveValue("😆");
	await expect(items).toHaveCount(0);
});

test("打「多謝」（niyi）點 🙏 上屏；簡體模式顯示「多谢」也一樣接 🙏", async ({
	page,
}, testInfo) => {
	const { text, simplified, items } = await ready(page);
	await text.focus();
	await page.keyboard.type("niyi");
	await expect(items.first()).toHaveText("1多謝");
	await expect(items.nth(1)).toHaveText("2🙏");
	await items.nth(1).click();
	await expect(text).toHaveValue("🙏");

	await simplified.check();
	await text.focus();
	await page.keyboard.type("niyi");
	await expect(items.first()).toHaveText("1多谢");
	await expect(items.nth(1)).toHaveText("2🙏");
	await page.screenshot({ path: testInfo.outputPath("emoji-simplified.png") });
	await page.keyboard.press("2");
	await expect(text).toHaveValue("🙏🙏");
});

test("關掉 emoji 候選就不出，組字中切換立刻重列", async ({ page }) => {
	const { text, emoji, items } = await ready(page);
	await text.focus();
	await page.keyboard.type("niyi");
	await expect(items.nth(1)).toHaveText("2🙏");

	await emoji.uncheck();
	await expect(items.filter({ hasText: "🙏" })).toHaveCount(0);
	await expect(text).toBeFocused();

	await emoji.check();
	await expect(items.nth(1)).toHaveText("2🙏");
});

test("選了開頭一段後，剩下的整串的候選也接 emoji，上屏選定的字加 emoji", async ({
	page,
}) => {
	const { text, items } = await ready(page);
	await text.focus();
	// 「多謝」+「笑」：先選蓋住 niyi 的「多謝」，剩下 hk。
	await page.keyboard.type("niyihk");
	await expect(items.first()).toBeVisible();
	const list = await shown(page);
	const thanks = list.indexOf("多謝");
	expect(thanks, list.join(" ")).toBeGreaterThanOrEqual(0);
	expect(list).not.toContain("🙏");
	await page.keyboard.press(String(thanks + 1));

	const rest = await shown(page);
	const laugh = rest.indexOf("笑");
	expect(rest[laugh + 1], rest.join(" ")).toBe("😆");
	await page.keyboard.press(String(laugh + 2));
	await expect(text).toHaveValue("多謝😆");
});
