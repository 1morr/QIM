import { expect, test } from "@playwright/test";

// 字形用常見寫法、不用香港寫法（brief §4.1，M1 票 17b 使用者決定）：語料換回常見寫法後，
// 同碼的「溫／温」「群／羣」由常見寫法當首選（香港寫法仍在碼表裡，字頻是 0）。
for (const { code, radicals, common, replaced } of [
	{ code: "et", radicals: "水廿", common: "溫", replaced: "温" },
	{ code: "sq", radicals: "尸手", common: "群", replaced: "羣" },
]) {
	test(`打 ${code} 首選是「${common}」，不是「${replaced}」`, async ({
		page,
	}) => {
		await page.goto("/");
		const text = page.getByRole("textbox", { name: "輸入框" });
		const composition = page.getByTestId("composition");
		const items = page
			.getByRole("list", { name: "候選" })
			.getByRole("listitem");
		await expect(text).toHaveAttribute("data-ready", "true");

		await text.focus();
		await page.keyboard.type(code);
		await expect(composition).toHaveText(radicals);
		await expect(items.first()).toHaveText(`1${common}`);

		await page.keyboard.type(" ");
		await expect(text).toHaveValue(common);
	});
}
