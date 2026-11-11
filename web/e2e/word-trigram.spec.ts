import { expect, test } from "@playwright/test";

// 詞級 trigram（M3 票 11b）：網頁載入方案指名的詞級 trigram 資料包，連打依前面的詞排序。
// 只有字級 bigram 時這串的首選是「你哋去行下行」：最後一段「街」要看前面的「行下」才選得對。
test("打「你哋去行下街」的碼，首選是它，按空白上屏", async ({
	page,
}, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type("ofrdgihnmyhn");
	await expect(items.first()).toHaveText("1你哋去行下街");
	await page.screenshot({ path: testInfo.outputPath("word-trigram.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("你哋去行下街");
});
