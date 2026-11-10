import { expect, type Page, test } from "@playwright/test";

// 設定區（M3 票 01）：改核心選項的開關都在「設定」這一區，新開關加在 `settings.ts` 的 `ROWS`。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return { text, settings: page.getByRole("region", { name: "設定" }) };
}

test("設定區列出改核心選項的開關，預設值照 brief §7", async ({
	page,
}, testInfo) => {
	const { settings } = await ready(page);
	await expect(
		settings.getByRole("checkbox", { name: "無痕" }),
	).not.toBeChecked();
	await expect(settings.getByRole("checkbox", { name: "學習" })).toBeChecked();
	await expect(
		settings.getByRole("checkbox", { name: "簡體" }),
	).not.toBeChecked();
	await expect(
		settings.getByRole("checkbox", { name: "繁體字優先" }),
	).toBeChecked();
	await expect(
		settings.getByRole("checkbox", { name: "英文候選" }),
	).toBeChecked();
	await expect(
		settings.getByRole("checkbox", { name: "emoji 候選" }),
	).toBeChecked();
	await expect(
		settings.getByRole("checkbox", { name: "數字後半形" }),
	).toBeChecked();
	await expect(settings.getByRole("combobox", { name: "重排" })).toHaveValue(
		"identity",
	);
	await expect(
		settings.getByRole("combobox", { name: "組字中單按 Shift" }),
	).toHaveValue("rime");
	await settings.screenshot({ path: testInfo.outputPath("settings.png") });
});

test("改了開關之後焦點回到輸入框，可以接著打字", async ({ page }) => {
	const { text, settings } = await ready(page);
	await settings.getByRole("checkbox", { name: "簡體" }).click();
	await expect(text).toBeFocused();
	// 用滑鼠選選項時焦點在選單上；`selectOption` 不移動焦點，先放過去。
	const rerank = settings.getByRole("combobox", { name: "重排" });
	await rerank.focus();
	await rerank.selectOption("off");
	await expect(text).toBeFocused();
	await page.keyboard.type("ls ");
	await expect(text).not.toHaveValue("");
});
