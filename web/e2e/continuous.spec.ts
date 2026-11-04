import { expect, test } from "@playwright/test";

// 連打（M1 票 07）：輸入碼切成字與詞的 lattice，用同一把尺打分。
test("打 anr 首選是「門口」，按空白上屏「門口」", async ({
	page,
}, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	await page.keyboard.type("anr");
	await expect(items.first()).toHaveText("1門口");
	await page.screenshot({ path: testInfo.outputPath("type-anr.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue("門口");
});

test("連打一串、按數字選開頭一段、再按空白，上屏兩段合起來", async ({
	page,
}, testInfo) => {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	const composition = page.getByTestId("composition");
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	await expect(text).toHaveAttribute("data-ready", "true");

	await text.focus();
	// 門口有人：門 an、口 r、有 kb、人 o。
	await page.keyboard.type("anrkbo");
	await expect(composition).toHaveText("anrkbo");
	const texts = (await items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
	const first = texts.indexOf("門口");
	expect(first, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.screenshot({ path: testInfo.outputPath("type-anrkbo.png") });

	await page.keyboard.type(String(first + 1));
	await expect(composition).toHaveText("門口kbo");
	await expect(text).toHaveValue("");
	const rest = (await items.first().textContent())?.replace(/^1/, "") ?? "";
	expect(rest).not.toBe("");
	await page.screenshot({ path: testInfo.outputPath("select-men-kou.png") });

	await page.keyboard.type(" ");
	await expect(text).toHaveValue(`門口${rest}`);
	await expect(composition).toHaveText("");
	await page.screenshot({
		path: testInfo.outputPath("commit-two-segments.png"),
	});
});
