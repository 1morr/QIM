import { devices, expect, type Page, test } from "@playwright/test";

// 組字中修改（M1 票 21）：移到中間的分段改選，其餘沒選定的分段重算（plan §2.1）。
// `anrkboa` 的首選路徑是「門口｜有｜智」三段，中間一段 `kb` 有別的字可選。
const INPUT = "anrkboa";

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

/** 單打 `code` 時的候選：在空的組字打完讀回，再按 Esc 清掉。 */
async function candidatesOf(page: Page, code: string): Promise<string[]> {
	await page.keyboard.type(code);
	await expect(ui(page).composition).toHaveText(code);
	const texts = await candidateTexts(page);
	await page.keyboard.press("Escape");
	await expect(ui(page).composition).toHaveText("");
	return texts;
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
		const [oa] = await candidatesOf(page, "oa");

		await page.keyboard.type(INPUT);
		await expect(segments).toHaveText(["anr", "kb", "oa"]);
		await expect(items.first()).toHaveText("1門口有智");

		await page.keyboard.press("ArrowLeft");
		await expect(segments.nth(2)).toHaveAttribute("aria-current", "true");
		await page.keyboard.press("ArrowLeft");
		await expect(segments.nth(1)).toHaveAttribute("aria-current", "true");
		// 候選改列中間一段 `kb` 的字詞，和單打 `kb` 相同。
		expect(await candidateTexts(page)).toEqual(kb);
		await page.screenshot({ path: testInfo.outputPath("focus-middle.png") });

		const other = kb[1];
		await page.keyboard.type("2");
		// 前面的「門口」照顯示的字固定，改選的段固定，`oa` 重算。
		await expect(segments).toHaveText(["門口", other, "oa"]);
		await expect(text).toHaveValue("");
		await page.screenshot({ path: testInfo.outputPath("rechosen.png") });

		await page.keyboard.type(" ");
		await expect(text).toHaveValue(`門口${other}${oa}`);
		await expect(composition).toHaveText("");
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
			await expect(composition).toHaveText(code);
			const texts = await candidateTexts(page);
			for (const _ of code) {
				await page.getByTestId("key-backspace").tap();
			}
			await expect(composition).toHaveText("");
			return texts;
		};
		const kb = await tappedCandidates("kb");
		const [oa] = await tappedCandidates("oa");

		await tapKeys(INPUT);
		await expect(segments).toHaveText(["anr", "kb", "oa"]);
		await segments.nth(1).tap();
		await expect(segments.nth(1)).toHaveAttribute("aria-current", "true");
		expect(await candidateTexts(page)).toEqual(kb);
		await page.screenshot({ path: testInfo.outputPath("touch-focus.png") });

		const other = kb[1];
		await items.nth(1).tap();
		await expect(segments).toHaveText(["門口", other, "oa"]);
		await expect(items.first()).toHaveText(`1${oa}`);

		await page.getByTestId("key-space").tap();
		await expect(text).toHaveValue(`門口${other}${oa}`);
		await expect(composition).toHaveText("");
		await page.screenshot({ path: testInfo.outputPath("touch-commit.png") });
	});
});
