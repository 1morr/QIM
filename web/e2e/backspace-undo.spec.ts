import { expect, type Page, test } from "@playwright/test";

// 上屏後緊接著退格撤銷這次的學習（M4 票 15，plan §2.4〈退格撤銷〉）：上屏後第一個交還宿主的鍵是退格時，
// 退格照常刪字，這次上屏學到的都撤銷；只收起聯想的退格不算夾在中間，其他鍵夾在中間就不撤銷。英文直通
// 打完一串後緊接著退格，撤銷那個英文使用者詞。排位規則由 debug 面板讀回。

function ui(page: Page) {
	const debug = page.locator("#debug");
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		association: page.getByRole("checkbox", { name: "聯想" }),
		toggle: page.getByRole("button", { name: "debug" }),
		debug,
		rows: debug.getByRole("table", { name: "候選來源" }).locator("tbody tr"),
	};
}

/** 頁面拋出的錯誤；每個測試結尾要是空的。 */
let errors: Error[] = [];

/** 開網頁（學習資料清空）、打開 debug 面板，焦點在輸入框。 */
async function open(page: Page) {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
	const { text, toggle, debug } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await toggle.click();
	await expect(debug).toBeVisible();
	await text.focus();
}

test.afterEach(() => {
	expect(errors).toEqual([]);
});

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
}

/** debug 面板候選表裡文字是 `text` 的那列。 */
function row(page: Page, text: string) {
	return ui(page).rows.filter({
		has: page.locator("td:nth-child(2)", {
			hasText: new RegExp(`^${text}$`),
		}),
	});
}

/** 打 `a` 讀首選與第 2 個候選，Esc 放棄。 */
async function topTwo(page: Page): Promise<[string, string]> {
	await page.keyboard.type("a");
	const [top, second] = await shown(page);
	expect(second, "a 要有第 2 個候選").toBeTruthy();
	await page.keyboard.press("Escape");
	return [top, second];
}

/** 打 `a` 按 2 選第 2 個候選。 */
async function chooseSecond(page: Page, second: string) {
	await page.keyboard.type("a");
	await expect(ui(page).items.nth(1)).toHaveText(`2${second}`);
	await page.keyboard.press("2");
}

test.describe("緊接著的退格", () => {
	test.beforeEach(async ({ page }) => {
		await open(page);
	});

	test("聯想關掉：選第 2 個候選兩次，第二次上屏後緊接退格，再打同一串碼首選不變", async ({
		page,
	}, testInfo) => {
		const { text, items, association } = ui(page);
		await association.uncheck();
		await text.focus();
		const [top, second] = await topTwo(page);

		await chooseSecond(page, second);
		await chooseSecond(page, second);
		await expect(text).toHaveValue(second + second);
		await page.keyboard.press("Backspace");
		await expect(text).toHaveValue(second);

		await page.keyboard.type("a");
		await expect(items.first()).toHaveText(`1${top}`);
		await expect(row(page, second)).not.toContainText("排位規則 2");
		await page.screenshot({
			path: testInfo.outputPath("undone-top.png"),
		});

		// 撤銷後也存了檔：重新整理後照樣只選過一次。
		await page.keyboard.press("Escape");
		await page.reload();
		await expect(text).toHaveAttribute("data-ready", "true");
		await text.focus();
		await page.keyboard.type("a");
		await expect(items.first()).toHaveText(`1${top}`);
	});

	test("聯想開著：第一下退格只收起聯想，第二下刪字並撤銷（存檔回到上屏前）", async ({
		page,
	}, testInfo) => {
		const { text, items } = ui(page);
		const saved = () =>
			page.evaluate(() => localStorage.getItem("qim.userdict"));
		// 「日」上屏後有聯想（「本」…）。
		await page.keyboard.type("a ");
		await page.keyboard.press("Escape");
		const before = await saved();
		await page.keyboard.type("a ");
		await expect(text).toHaveValue("日日");
		await expect(items.first()).toBeVisible();
		expect(await saved()).not.toBe(before);

		await page.keyboard.press("Backspace");
		await expect(items).toHaveCount(0);
		await expect(text).toHaveValue("日日");
		expect(await saved(), "只收起聯想，還沒撤銷").not.toBe(before);
		await page.keyboard.press("Backspace");
		await expect(text).toHaveValue("日");
		await expect.poll(saved).toBe(before);
		await page.screenshot({
			path: testInfo.outputPath("association-undone.png"),
		});
	});

	test("中間夾了別的鍵：退格只刪字，選兩次的成首選", async ({ page }) => {
		const { text, items, association } = ui(page);
		await association.uncheck();
		await text.focus();
		const [, second] = await topTwo(page);

		await chooseSecond(page, second);
		await chooseSecond(page, second);
		await page.keyboard.type("a");
		await page.keyboard.press("Escape");
		await page.keyboard.press("Backspace");
		await expect(text).toHaveValue(second);

		await page.keyboard.type("a");
		await expect(items.first()).toHaveText(`1${second}`);
		await expect(row(page, second)).toContainText("排位規則 2");
	});

	test("上屏後點了輸入框（移游標）才退格：不撤銷", async ({ page }) => {
		const { text, items, association } = ui(page);
		await association.uncheck();
		await text.focus();
		const [, second] = await topTwo(page);
		await chooseSecond(page, second);
		await chooseSecond(page, second);

		await text.click();
		await page.keyboard.press("Backspace");
		await expect(text).toHaveValue(second);
		await page.keyboard.type("a");
		await expect(items.first()).toHaveText(`1${second}`);
	});

	test("實體鍵位換字（AZERTY 的實體 A 鍵：key 是 q、code 是 KeyA）打出的上屏，緊接退格照樣撤銷", async ({
		page,
	}) => {
		const { text, items, association } = ui(page);
		await association.uncheck();
		await text.focus();
		const [top, second] = await topTwo(page);
		/** 送一個 key 與 code 不同的按下事件（同 `physical-weighted.spec.ts`）。 */
		const physicalA = () =>
			text.evaluate((element) => {
				element.dispatchEvent(
					new KeyboardEvent("keydown", {
						key: "q",
						code: "KeyA",
						bubbles: true,
						cancelable: true,
					}),
				);
			});
		for (let i = 0; i < 2; i++) {
			await physicalA();
			await expect(items.nth(1)).toHaveText(`2${second}`);
			await page.keyboard.press("2");
		}
		await expect(text).toHaveValue(second + second);

		await page.keyboard.press("Backspace");
		await expect(text).toHaveValue(second);
		await physicalA();
		await expect(items.first()).toHaveText(`1${top}`);
	});

	test("英文直通打完一串後緊接退格：那串不學，改過再打完的才學", async ({
		page,
	}, testInfo) => {
		const { text } = ui(page);
		const words = page
			.getByRole("list", { name: "英文使用者詞清單" })
			.getByRole("listitem");
		// 沒有組字時單按 Shift 切英文直通。
		await page.keyboard.press("Shift");
		await page.keyboard.type("rrrrr ");
		await expect(words).toHaveText(["rrrrr刪除"]);
		await page.keyboard.press("Backspace");
		await expect(words).toHaveText(["還沒有英文使用者詞"]);
		await page.keyboard.press("Backspace");
		await page.keyboard.type("s ");
		await expect(text).toHaveValue("rrrrs ");
		await expect(words).toHaveText(["rrrrs刪除"]);
		await page.screenshot({
			path: testInfo.outputPath("english-undone.png"),
		});
	});
});

test.describe("時限（殼計時，照 librime 的 3 秒）", () => {
	test.beforeEach(async ({ page }) => {
		await page.clock.install();
		await open(page);
	});

	for (const [title, waitMs, undone] of [
		["時限內：上屏後 2.9 秒緊接退格，撤銷", 2900, true],
		["時限到了：上屏後 3 秒才退格，不撤銷、只刪字", 3000, false],
	] as const) {
		test(title, async ({ page }) => {
			const { text, items, association } = ui(page);
			await association.uncheck();
			await text.focus();
			const [top, second] = await topTwo(page);
			await chooseSecond(page, second);
			await chooseSecond(page, second);

			await page.clock.runFor(waitMs);
			await page.keyboard.press("Backspace");
			await expect(text).toHaveValue(second);
			await page.keyboard.type("a");
			await expect(items.first()).toHaveText(`1${undone ? top : second}`);
		});
	}
});

test("localStorage 打不開（學習資料只在記憶體）：上屏後緊接退格照樣撤銷", async ({
	page,
}) => {
	errors = [];
	page.on("pageerror", (error) => errors.push(error));
	await page.addInitScript(() => {
		Object.defineProperty(window, "localStorage", {
			configurable: true,
			get() {
				throw new DOMException("封鎖了網站資料", "SecurityError");
			},
		});
	});
	await page.goto("/");
	const { text, items, association } = ui(page);
	await expect(text).toHaveAttribute("data-ready", "true");
	await association.uncheck();
	await text.focus();
	const [top, second] = await topTwo(page);

	await chooseSecond(page, second);
	await chooseSecond(page, second);
	await expect(page.locator("#status")).toContainText("不會存");
	await page.keyboard.press("Backspace");
	await expect(text).toHaveValue(second);
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${top}`);

	// 對照：記憶體裡的學習照常，再選一次（回到兩次）它就成首選。
	await page.keyboard.press("2");
	await page.keyboard.press("Space");
	await page.keyboard.type("a");
	await expect(items.first()).toHaveText(`1${second}`);
	expect(errors).toEqual([]);
});
