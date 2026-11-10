import { devices, expect, type Page, test } from "@playwright/test";

// 手動分隔鍵（brief §4.3、§4.4，M1 票 28、28b）：組字中按 `'`（觸控是組字中的「'詞」）插入分隔
// 記號，這裡一定要斷字：單字的碼不跨過它，詞只在它落在兩字的碼之間時跨過它（照 Rime 速成）。
// `mir` 的首選「一台」是 `m` + `ir`，`mi'r` 只能是 `mi` 的字詞接 `r` 的字詞；`vi'vdjg` 的
// 「絃樂室」是 `vi` + `vd` + `jg`，分隔落在字界上，仍是一個詞。

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

test.describe("實體鍵盤", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");
		await expect(ui(page).text).toHaveAttribute("data-ready", "true");
		await ui(page).text.focus();
	});

	test("打 mi'r：組字顯示分隔記號，首選照分段排，上屏的是 mi 的字接 r 的字", async ({
		page,
	}, testInfo) => {
		const { text, composition, segments, items } = ui(page);
		await page.keyboard.type("mir");
		await expect(items.first()).toHaveText("1一台");
		await page.keyboard.press("Escape");
		await page.keyboard.type("mi");
		await expect(composition).toHaveText("一戈");
		const mi = await candidateTexts(page);
		await page.keyboard.press("Escape");
		await page.keyboard.type("r");
		await expect(composition).toHaveText("口");
		const r = await candidateTexts(page);
		await page.keyboard.press("Escape");

		await page.keyboard.type("mi'r");
		await expect(segments).toHaveText(["一戈'", "口"]);
		await expect(text).toHaveValue("");
		const separated = await candidateTexts(page);
		expect(separated).not.toContain("一台");
		const top = separated[0];
		expect(
			mi.some(
				(head) => r.includes(top.slice(head.length)) && top.startsWith(head),
			),
			`${top} 不是 mi 的字詞接 r 的字詞`,
		).toBe(true);
		await page.screenshot({ path: testInfo.outputPath("separated.png") });

		await page.keyboard.type(" ");
		await expect(text).toHaveValue(top);
		await expect(composition).toHaveText("");
	});

	test("退格先刪分隔記號；沒有組字時 ' 照打", async ({ page }) => {
		const { text, composition } = ui(page);
		await page.keyboard.type("mi'r");
		await expect(composition).toHaveText("一戈'口");
		await page.keyboard.press("Backspace");
		await expect(composition).toHaveText("一戈'");
		await page.keyboard.press("Backspace");
		await expect(composition).toHaveText("一戈");
		await page.keyboard.press("Escape");
		await page.keyboard.type("'");
		await expect(text).toHaveValue("'");
		await expect(composition).toHaveText("");
	});

	test("分隔落在詞內的字界上：vi'vdjg 仍是一個詞「絃樂室」", async ({
		page,
	}) => {
		const { text, composition, items } = ui(page);
		// 不給前文（無痕）：在輸入框開頭時句首的 bigram 讓「將樂室」勝過詞「絃樂室」，和分隔記號無關
		// （M3 票 05）。
		await page.getByRole("checkbox", { name: "無痕" }).check();
		await page.keyboard.type("vi'vdjg");
		await expect(composition).toHaveText("女戈'女木十土");
		await expect(items.first()).toHaveText("1絃樂室");
		await page.keyboard.type(" ");
		await expect(text).toHaveValue("絃樂室");
	});

	test("an'r 首選仍是門口（同 Gboard 速成）：詞「門口」跨過字界上的分隔記號", async ({
		page,
	}) => {
		await page.keyboard.type("an'r");
		await expect(ui(page).segments).toHaveText(["日弓'口"]);
		await expect(ui(page).items.first()).toHaveText("1門口");
	});
});

test.describe("觸控", () => {
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test.beforeEach(async ({ page }) => {
		await page.goto("/");
		await expect(ui(page).text).toHaveAttribute("data-ready", "true");
	});

	async function tapKeys(page: Page, ids: string[]): Promise<void> {
		for (const id of ids) {
			await page.getByTestId(`key-${id}`).tap();
		}
	}

	test("組字中 ⇧ 顯示「'詞」，點了插入分隔記號；上屏後變回 Shift", async ({
		page,
	}, testInfo) => {
		const { text, composition, segments } = ui(page);
		const shift = page.getByTestId("key-shift");
		await expect(shift).toHaveText("⇧");
		await expect(shift).toHaveAttribute("aria-label", "大寫");

		await tapKeys(page, ["m", "i"]);
		await expect(composition).toHaveText("一戈");
		await expect(shift).toHaveText("'詞");
		await expect(shift).toHaveAttribute("aria-label", "分隔");
		await tapKeys(page, ["shift"]);
		await expect(composition).toHaveText("一戈'");
		// 不是大寫：Shift 沒開，下一個字母鍵照樣進組字。
		await expect(shift).toHaveAttribute("data-shift", "off");
		await tapKeys(page, ["r"]);
		await expect(segments).toHaveText(["一戈'", "口"]);
		const [top] = await candidateTexts(page);
		expect(top).not.toBe("一台");
		await page.screenshot({ path: testInfo.outputPath("touch-separated.png") });

		await tapKeys(page, ["space"]);
		await expect(text).toHaveValue(top);
		await expect(shift).toHaveText("⇧");
		await tapKeys(page, ["shift"]);
		await expect(shift).toHaveAttribute("data-shift", "once");
	});

	test("點「'詞」後退格先刪分隔記號，刪光組字後 ⇧ 變回 Shift", async ({
		page,
	}) => {
		const { composition } = ui(page);
		const shift = page.getByTestId("key-shift");
		await tapKeys(page, ["m", "i", "shift", "r"]);
		await expect(composition).toHaveText("一戈'口");
		await tapKeys(page, ["backspace"]);
		await expect(composition).toHaveText("一戈'");
		await tapKeys(page, ["backspace"]);
		await expect(composition).toHaveText("一戈");
		await expect(shift).toHaveText("'詞");
		await tapKeys(page, ["backspace", "backspace"]);
		await expect(composition).toHaveText("");
		await expect(shift).toHaveText("⇧");
		await expect(shift).toHaveAttribute("aria-label", "大寫");
	});

	test("Shift 開著時用實體鍵盤開始組字，⇧ 仍換成「'詞」並送分隔記號", async ({
		page,
	}) => {
		const { composition } = ui(page);
		const shift = page.getByTestId("key-shift");
		await tapKeys(page, ["shift"]);
		await expect(shift).toHaveAttribute("data-shift", "once");
		await ui(page).text.focus();
		await page.keyboard.type("mi");
		await expect(composition).toHaveText("一戈");
		await expect(shift).toHaveText("'詞");
		await expect(shift).toHaveAttribute("aria-pressed", "false");
		await expect(shift).toHaveAttribute("aria-label", "分隔");
		await tapKeys(page, ["shift"]);
		await expect(composition).toHaveText("一戈'");
	});
});
