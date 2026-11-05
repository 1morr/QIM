import { expect, test } from "@playwright/test";

// 「回報」（M1 票 12）：剛才那次組字的輸入碼與上屏的字，加成回報清單的一行
// （`bench/fixed.tsv` 的格式：輸入碼<TAB>期望首選<TAB>來源），重新整理後還在。
test("選第 2 個候選上屏後按「回報」，清單多一行 fixed.tsv 格式的題", async ({
	page,
}, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	const report = page.getByRole("button", { name: "回報", exact: true });
	const reports = page.getByRole("textbox", { name: /回報的固定題/ });
	await expect(text).toHaveAttribute("data-ready", "true");
	await expect(report).toBeDisabled();

	await text.focus();
	await page.keyboard.type("mir");
	await expect(items.first()).toHaveText("1一台");
	const second = (await items.nth(1).textContent())?.replace(/^2/, "") ?? "";
	expect(second).not.toBe("");
	await page.keyboard.press("2");
	await expect(text).toHaveValue(second);

	await report.click();
	await expect(reports).toHaveValue(`mir\t${second}\t網頁回報（首選 一台）`);
	await expect(report).toBeDisabled();
	await expect(text).toBeFocused();
	await page.screenshot({ path: testInfo.outputPath("report.png") });

	// 再打一次、空白上屏首選，回報接在下一行。
	await page.keyboard.type("anr ");
	await report.click();
	await expect(reports).toHaveValue(
		`mir\t${second}\t網頁回報（首選 一台）\nanr\t門口\t網頁回報（首選 門口）`,
	);

	// 組字中按標點：上屏「日，」，不是選出來的字，不能回報。
	await page.keyboard.type("a,");
	await expect(text).toHaveValue(`${second}門口日，`);
	await expect(report).toBeDisabled();

	await page.reload();
	await expect(text).toHaveAttribute("data-ready", "true");
	await expect(reports).toHaveValue(
		`mir\t${second}\t網頁回報（首選 一台）\nanr\t門口\t網頁回報（首選 門口）`,
	);
});

// 臨時英文（組字中單按左 Shift，M1 票 25、25b）中的空白與數字是字元：上屏的是英文，不是選出來的字。
test("臨時英文打了空白與數字再按 Enter 上屏，不能回報", async ({ page }) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const report = page.getByRole("button", { name: "回報", exact: true });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	await page.keyboard.type("anr");
	await page.keyboard.press("ShiftLeft");
	await page.keyboard.type("1 ");
	await expect(text).toHaveValue("");
	await page.keyboard.press("Enter");
	await expect(text).toHaveValue("anr1 ");
	await expect(report).toBeDisabled();
});
