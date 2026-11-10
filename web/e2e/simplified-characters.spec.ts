import { expect, type Page, test } from "@playwright/test";

// 繁體模式的簡體字（M3 票 19，brief §4.1）：HKSCS 收的「门」「农」「阳」「头」排在同碼的繁體字
// 之後，不過濾、選兩次照常成首選；簡體模式不受影響。黑箱 T17 的 QIM 欄就是這裡量的位置。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	return {
		text,
		simplified: page.getByRole("checkbox", { name: "簡體" }),
		all: page.getByRole("list", { name: "全部候選" }).getByRole("listitem"),
	};
}

/** 候選列一頁的候選數：排在第一頁之後的位置大於它。 */
const PAGE_SIZE = 9;

/** 打 `code`、按 ↓ 打開面板，回傳全部候選的文字；組字留著。 */
async function openPanel(page: Page, code: string): Promise<string[]> {
	const all = page
		.getByRole("list", { name: "全部候選" })
		.getByRole("listitem");
	await page.keyboard.type(code);
	await page.keyboard.press("ArrowDown");
	await expect(all.first()).toBeVisible();
	return all.allTextContents();
}

test("繁體模式 ls 的「门」排在所有同碼的單字之後", async ({
	page,
}, testInfo) => {
	await ready(page);
	const texts = await openPanel(page, "ls");
	await page.screenshot({ path: testInfo.outputPath("ls-panel.png") });
	const men = texts.indexOf("门");
	expect(men).toBeGreaterThanOrEqual(PAGE_SIZE);
	// 「门」前面只有繁體字與多段路徑；`ls` 的同碼字蚱、螃都在前面。
	expect(texts.indexOf("蚱")).toBeLessThan(men);
	expect(texts.indexOf("螃")).toBeLessThan(men);
});

test("T17 的位置：门、阳、农、头、这都在第一頁之後", async ({ page }) => {
	await ready(page);
	const positions: string[] = [];
	for (const [code, char] of [
		["ls", "门"],
		["na", "阳"],
		["hv", "农"],
		["yk", "头"],
		// 不在 HKSCS，是擴展區的字。
		["yk", "这"],
	]) {
		const texts = await openPanel(page, code);
		await page.keyboard.press("Escape");
		const at = texts.indexOf(char);
		expect(at, `${code} 沒有「${char}」`).toBeGreaterThanOrEqual(PAGE_SIZE);
		positions.push(`${code} ${char} 第 ${at + 1}`);
	}
	console.log(`T17：${positions.join("、")}`);
});

test("簡體模式 ls 的「门」照分數排，比繁體模式前面", async ({ page }) => {
	const { simplified } = await ready(page);
	const traditional = (await openPanel(page, "ls")).indexOf("门");
	await page.keyboard.press("Escape");
	await simplified.check();
	await page.getByRole("textbox", { name: "輸入框" }).focus();
	const texts = await openPanel(page, "ls");
	expect(texts.indexOf("门")).toBeGreaterThanOrEqual(0);
	expect(texts.indexOf("门")).toBeLessThan(traditional);
});

test("選兩次「门」後它成 ls 的首選", async ({ page }) => {
	const { text, all } = await ready(page);
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	for (let i = 0; i < 2; i++) {
		await page.keyboard.type("ls");
		await page.keyboard.press("ArrowDown");
		await all.filter({ hasText: /^门$/ }).click();
		await expect(text).toHaveValue("门".repeat(i + 1));
	}
	await page.keyboard.type("ls");
	await expect(items.first()).toHaveText("1门");
});

test("關掉「繁體字優先」，「门」回到改前的位置；組字中切換立刻重列", async ({
	page,
}, testInfo) => {
	const { text } = await ready(page);
	const settings = page.getByRole("region", { name: "設定" });
	const toggle = settings.getByRole("checkbox", { name: "繁體字優先" });
	await expect(toggle).toBeChecked();
	// 改前的名次是不知道前文時量的；空的輸入框有前文（句首，M3 票 05），名次不同，所以在無痕
	// （不給前文）下比。
	await settings.getByRole("checkbox", { name: "無痕" }).check();
	await expect(text).toBeFocused();
	const on = await openPanel(page, "ls");
	await page.keyboard.press("ArrowUp");

	await toggle.click();
	await expect(toggle).not.toBeChecked();
	const all = page
		.getByRole("list", { name: "全部候選" })
		.getByRole("listitem");
	await page.keyboard.press("ArrowDown");
	await expect(all.first()).toBeVisible();
	const off = await all.allTextContents();
	await page.screenshot({ path: testInfo.outputPath("ls-panel-off.png") });
	// 改前（票 19 之前）是第 9。
	expect(off.indexOf("门")).toBe(8);
	expect(off.indexOf("门")).toBeLessThan(on.indexOf("门"));

	await page.keyboard.press("ArrowUp");
	await toggle.click();
	await page.keyboard.press("ArrowDown");
	await expect(all.first()).toBeVisible();
	expect((await all.allTextContents()).indexOf("门")).toBe(on.indexOf("门"));
});
