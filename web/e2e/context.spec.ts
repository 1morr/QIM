import { devices, expect, type Page, test } from "@playwright/test";

// 上下文（M3 票 05，plan〈介面〉）：網頁把輸入框游標前的文字交給核心，候選照前文排；數字後半形看
// 前文；無痕時不給（brief〈隱私與安全〉）。輸入框是空的或游標在開頭時前文是空字串，接句首。

async function ready(page: Page) {
	await page.goto("/");
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return {
		text,
		first: page
			.getByRole("list", { name: "候選" })
			.getByRole("listitem")
			.first(),
	};
}

/** 換掉輸入框的內容、游標放在第 `cursor` 個 UTF-16 碼元，像使用者貼上後點到那裡。 */
async function place(page: Page, value: string, cursor = value.length) {
	await page.getByRole("textbox", { name: "輸入框" }).evaluate(
		(element: HTMLTextAreaElement, [value, cursor]) => {
			element.value = value as string;
			element.dispatchEvent(new Event("input", { bubbles: true }));
			element.setSelectionRange(cursor as number, cursor as number);
		},
		[value, cursor] as const,
	);
}

test.describe("實體鍵盤", () => {
	test("同一串碼在不同前文首選不同，游標移到別處候選跟著變", async ({
		page,
	}, testInfo) => {
		const { text, first } = await ready(page);
		await text.focus();
		// 空的輸入框：句首。
		await page.keyboard.type("of");
		await expect(first).toHaveText("1係");
		await page.keyboard.press("Escape");

		await place(page, "冇");
		await page.keyboard.type("of");
		await expect(first).toHaveText("1你");
		await page.screenshot({
			path: testInfo.outputPath("context-after-mou.png"),
		});
		await page.keyboard.press("Escape");

		// 游標移到開頭（方向鍵交還瀏覽器）：前文是空的，回到句首的首選。
		await page.keyboard.press("Home");
		await page.keyboard.type("of");
		await expect(first).toHaveText("1係");
		await page.screenshot({
			path: testInfo.outputPath("context-at-start.png"),
		});
		await page.keyboard.press("Escape");

		// 點回結尾（不經按鍵）。
		await text.evaluate((element: HTMLTextAreaElement) =>
			element.setSelectionRange(1, 1),
		);
		await page.keyboard.type("of");
		await expect(first).toHaveText("1你");
	});

	test("上屏的字是下一個組字的前文", async ({ page }) => {
		const { text, first } = await ready(page);
		await text.focus();
		await page.keyboard.type("hi");
		await expect(first).toHaveText("1我");
		await page.keyboard.press("Space");
		await expect(text).toHaveValue("我");
		await page.keyboard.type("hi");
		await expect(first).toHaveText("1的");
	});

	test("游標沒動的交還鍵（結尾按 End）之後照樣照前文", async ({ page }) => {
		const { text, first } = await ready(page);
		await text.focus();
		await place(page, "冇");
		// End 交還瀏覽器、核心忘掉前文；游標本來就在結尾，不會有 selectionchange。
		await page.keyboard.press("End");
		await page.keyboard.type("of");
		await expect(first).toHaveText("1你");
	});

	test("前文以半形數字結尾時 . 上屏半形，游標移離數字後是全形", async ({
		page,
	}) => {
		const { text } = await ready(page);
		await text.focus();
		await place(page, "共3");
		await page.keyboard.type(".");
		await expect(text).toHaveValue("共3.");

		// 游標移到「共」後面。
		await place(page, "共3", 1);
		await page.keyboard.type(".");
		await expect(text).toHaveValue("共。3");
	});

	test("無痕時前文不影響候選，關掉後又照前文", async ({ page }, testInfo) => {
		const { text, first } = await ready(page);
		const incognito = page.getByRole("checkbox", { name: "無痕" });
		await text.focus();
		// `hi`：句首是「我」，不知道前文是「的」（M3 票 11b 起詞級 trigram 讓原本的 `wrht` 兩者都是「回血」）。
		await page.keyboard.type("hi");
		await expect(first).toHaveText("1我");
		await page.keyboard.press("Escape");

		await incognito.check();
		await expect(text).toBeFocused();
		await page.keyboard.type("hi");
		await expect(first).toHaveText("1的");
		await page.screenshot({
			path: testInfo.outputPath("context-incognito.png"),
		});
		await page.keyboard.press("Escape");

		// 有前文「冇」：平時 `of` 首選「你」，無痕時照不知道前文是「係」。
		await place(page, "冇");
		await page.keyboard.type("of");
		await expect(first).toHaveText("1係");
		await page.keyboard.press("Escape");

		await incognito.uncheck();
		await expect(text).toBeFocused();
		await page.keyboard.type("of");
		await expect(first).toHaveText("1你");
		await page.keyboard.press("Escape");
		await place(page, "");
		await page.keyboard.type("hi");
		await expect(first).toHaveText("1我");
	});

	test("無痕時不看前文的數字，只看自己送出去的字", async ({ page }) => {
		const { text } = await ready(page);
		await page.getByRole("checkbox", { name: "無痕" }).check();
		await place(page, "共3");
		await page.keyboard.type(".");
		await expect(text).toHaveValue("共3。");
		await page.keyboard.type("5.");
		await expect(text).toHaveValue("共3。5.");
	});
});

test.describe("無痕期間打的字", () => {
	test("無痕時上屏的字不是前文；關掉後第一次打字照游標處的前文，不接無痕時打的字", async ({
		page,
	}) => {
		const { text, first } = await ready(page);
		const incognito = page.getByRole("checkbox", { name: "無痕" });
		await incognito.check();
		await expect(text).toBeFocused();
		// `kb` 的第 2 個候選是「冇」；無痕不學，選了也不改名次。
		await page.keyboard.type("kb2");
		await expect(text).toHaveValue("冇");
		await page.keyboard.type("of");
		await expect(first).toHaveText("1係");
		await page.keyboard.press("Escape");

		// 還在無痕時點到開頭，再關掉無痕：前文是游標處的（空的、接句首），不是無痕時打的「冇」。
		await text.evaluate((element: HTMLTextAreaElement) =>
			element.setSelectionRange(0, 0),
		);
		await incognito.uncheck();
		await expect(text).toBeFocused();
		await expect
			.poll(() => text.evaluate((e: HTMLTextAreaElement) => e.selectionStart))
			.toBe(0);
		await page.keyboard.type("of");
		await expect(first).toHaveText("1係");
	});
});

test.describe("觸控鍵盤", () => {
	// 觸控鍵不經實體鍵盤的 keydown：前文靠輸入框的 input 與 selectionchange 給。
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test("點鍵打字也照前文，游標移到別處跟著變", async ({ page }) => {
		const { first } = await ready(page);
		await place(page, "冇");
		for (const id of ["o", "f"]) {
			await page.getByTestId(`key-${id}`).tap();
		}
		await expect(first).toHaveText("1你");
		await page.getByTestId("key-backspace").tap();
		await page.getByTestId("key-backspace").tap();

		await place(page, "冇", 0);
		for (const id of ["o", "f"]) {
			await page.getByTestId(`key-${id}`).tap();
		}
		await expect(first).toHaveText("1係");
	});
});
