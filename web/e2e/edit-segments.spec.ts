import { devices, expect, type Page, test } from "@playwright/test";

// 組字中修改（M1 票 21）：移到中間的分段改選，其餘沒選定的分段重算（plan §2.1）。
// 組字裡選定的分段顯示字詞，沒選定的顯示字根。
// `anrkbof` 的首選路徑是「門口｜有｜無」三段，中間一段 `kb` 有別的字可選；
// 改選中間一段後，最後一段 `of` 重算成單打的首選（M1 票 17b 換掉 `anrkboa`：資料包改了之後
// 它的首選路徑變成「門｜口布｜會」）。
const INPUT = "anrkbof";
/** 組字裡沒選定的分段顯示字根（`data/schemes/quick.toml` 的 `[radicals]`）。 */
const RADICALS: Record<string, string> = {
	anr: "日弓口",
	kb: "大月",
	of: "人火",
};

function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		segments: page.getByTestId("composition").locator(".segment"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

/** 候選列的文字，去掉前面的序號。 */
async function candidateTexts(page: Page): Promise<string[]> {
	const texts = await ui(page).items.allTextContents();
	return texts.map((t) => t.replace(/^\d/, ""));
}

/**
 * 單打時接在字詞後面的 emoji 只給剩下的整串，目前分段的候選列沒有（plan §2.4），和分段的候選列
 * 比較前去掉。
 */
function withoutEmoji(texts: string[]): string[] {
	return texts.filter((t) => !/\p{Extended_Pictographic}/u.test(t));
}

/** 單打 `code` 時的字詞候選：在空的組字打完讀回、去掉 emoji，再按 Esc 清掉。 */
async function candidatesOf(page: Page, code: string): Promise<string[]> {
	await page.keyboard.type(code);
	await expect(ui(page).composition).toHaveText(RADICALS[code]);
	const texts = await candidateTexts(page);
	await page.keyboard.press("Escape");
	await expect(ui(page).composition).toHaveText("");
	return withoutEmoji(texts);
}

test.describe("實體鍵盤", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
		await expect(ui(page).text).toHaveAttribute("data-ready", "true");
		await ui(page).text.focus();
	});

	test("← 移到中間分段改選，空白上屏的其餘部分等於重算結果", async ({
		page,
	}, testInfo) => {
		const { text, composition, segments, items } = ui(page);
		const kb = await candidatesOf(page, "kb");
		const [last] = await candidatesOf(page, "of");

		await page.keyboard.type(INPUT);
		await expect(segments).toHaveText(["日弓口", "大月", "人火"]);
		await expect(items.first()).toHaveText("1門口有無");

		await page.keyboard.press("ArrowLeft");
		await expect(segments.nth(2)).toHaveAttribute("aria-current", "true");
		await page.keyboard.press("ArrowLeft");
		await expect(segments.nth(1)).toHaveAttribute("aria-current", "true");
		// 候選改列中間一段 `kb` 的字詞，和單打 `kb` 的字詞同序；單打的第一頁有 emoji 時佔了幾格，
		// 分段的第一頁多出後面幾個字。
		expect((await candidateTexts(page)).slice(0, kb.length)).toEqual(kb);
		await page.screenshot({ path: testInfo.outputPath("focus-middle.png") });

		const other = kb[1];
		await page.keyboard.type("2");
		// 前面的「門口」照顯示的字固定，改選的段固定，`of` 重算。
		await expect(segments).toHaveText(["門口", other, "人火"]);
		await expect(text).toHaveValue("");
		await page.screenshot({ path: testInfo.outputPath("rechosen.png") });

		await page.keyboard.type(" ");
		await expect(text).toHaveValue(`門口${other}${last}`);
		await expect(composition).toHaveText("");
	});

	test("點候選改選中間分段後上屏，「回報」記下整串輸入碼與上屏的字", async ({
		page,
	}) => {
		const { text, segments, items } = ui(page);
		const report = page.getByRole("button", { name: "回報", exact: true });
		const reports = page.getByRole("textbox", { name: /回報的固定題/ });
		const kb = await candidatesOf(page, "kb");
		const [last] = await candidatesOf(page, "of");

		await page.keyboard.type(INPUT);
		await segments.nth(1).click();
		await expect(segments.nth(1)).toHaveAttribute("aria-current", "true");
		await items.nth(1).click();
		await expect(segments).toHaveText(["門口", kb[1], "人火"]);
		await items.first().click();
		const wanted = `門口${kb[1]}${last}`;
		await expect(text).toHaveValue(wanted);

		await report.click();
		await expect(reports).toHaveValue(
			`${INPUT}\t${wanted}\t網頁回報（首選 門口有無）`,
		);
	});

	test("→ 走過最後一段回到整串，候選回到整串的", async ({ page }) => {
		const { segments, items } = ui(page);
		await page.keyboard.type(INPUT);
		const whole = await candidateTexts(page);
		await page.keyboard.press("ArrowLeft");
		await expect(segments.nth(2)).toHaveAttribute("aria-current", "true");
		await page.keyboard.press("ArrowRight");
		await expect(segments.locator("[aria-current]")).toHaveCount(0);
		await expect(items.first()).toHaveText(`1${whole[0]}`);
		expect(await candidateTexts(page)).toEqual(whole);
	});
});

test.describe("觸控", () => {
	// `defaultBrowserType` 只能在設定檔指定，這裡只取 viewport、觸控與 user agent。
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test.beforeEach(async ({ page }) => {
		await page.goto("/");
		await expect(ui(page).text).toHaveAttribute("data-ready", "true");
	});

	test("點組字裡的一段，候選改列該段；點候選改選後上屏", async ({
		page,
	}, testInfo) => {
		const { text, composition, segments, items } = ui(page);
		const tapKeys = async (code: string) => {
			for (const id of code) {
				await page.getByTestId(`key-${id}`).tap();
			}
		};
		/** 單打 `code` 時的候選，讀回後點退格刪光。 */
		const tappedCandidates = async (code: string) => {
			await tapKeys(code);
			await expect(composition).toHaveText(RADICALS[code]);
			const texts = await candidateTexts(page);
			for (const _ of code) {
				await page.getByTestId("key-backspace").tap();
			}
			await expect(composition).toHaveText("");
			return withoutEmoji(texts);
		};
		const kb = await tappedCandidates("kb");
		const [last] = await tappedCandidates("of");

		await tapKeys(INPUT);
		await expect(segments).toHaveText(["日弓口", "大月", "人火"]);
		await segments.nth(1).tap();
		await expect(segments.nth(1)).toHaveAttribute("aria-current", "true");
		expect((await candidateTexts(page)).slice(0, kb.length)).toEqual(kb);
		await page.screenshot({ path: testInfo.outputPath("touch-focus.png") });

		const other = kb[1];
		await items.nth(1).tap();
		await expect(segments).toHaveText(["門口", other, "人火"]);
		await expect(items.first()).toHaveText(`1${last}`);

		await page.getByTestId("key-space").tap();
		await expect(text).toHaveValue(`門口${other}${last}`);
		await expect(composition).toHaveText("");
		await page.screenshot({ path: testInfo.outputPath("touch-commit.png") });
	});
});
