import { devices, expect, type Page, test } from "@playwright/test";

// 殼的日期契約（plan §2.4，M4 票 12）：網頁在第一次學習前、之後每次使用者操作前把今天交給核心，
// 學到的條目記成最後使用的日期。除了實體鍵盤打字加選候選（`user-dict.spec.ts`），觸控鍵盤與
// 「失焦時把正在打的串學成英文使用者詞」也是學習的路徑；後者可能沒有前導的按鍵或點擊（Alt+Tab、切分頁）。
// 用 `page.clock` 換日。

const EMAIL = "ivan.test@example.com";

/** `localStorage` 存的學習資料裡，第一欄是 `kind`、第三欄起是 `key` 的那列，拆成欄。 */
async function storedRow(
	page: Page,
	kind: string,
	key: string,
): Promise<string[] | undefined> {
	const stored = await page.evaluate(() =>
		localStorage.getItem("qim.userdict"),
	);
	return (stored ?? "")
		.split("\n")
		.map((line) => line.split("\t"))
		.find((columns) => columns[0] === kind && columns.includes(key));
}

async function ready(page: Page) {
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	return text;
}

test("打 email 打到午夜後直接失焦（沒有按鍵或點擊），學到的那天是失焦當天", async ({
	page,
}) => {
	await page.clock.setFixedTime(new Date(2026, 10, 12, 23, 58));
	const text = await ready(page);
	await text.focus();
	await page.keyboard.press("Shift");
	await page.keyboard.type(EMAIL);

	await page.clock.setFixedTime(new Date(2026, 10, 13, 0, 2));
	// 切到別的視窗：焦點離開輸入框，前面沒有 keydown、pointerdown。
	await text.evaluate((element) => element.blur());
	await expect
		// kind word count decayed last_tick last_used（第 6 欄）
		.poll(async () => (await storedRow(page, "english", EMAIL))?.[5])
		.toBe("2026-11-13");
});

test.describe("觸控", () => {
	const { defaultBrowserType: _, ...pixel } = devices["Pixel 7"];
	test.use(pixel);

	test("點鍵上屏學到的條目記成點的那天，換日後再點跟著變", async ({ page }) => {
		await page.clock.setFixedTime(new Date(2026, 10, 12, 12, 0));
		const text = await ready(page);
		const commit = async () => {
			for (const id of ["a", "n", "r"]) {
				await page.getByTestId(`key-${id}`).tap();
			}
			await page.getByTestId("key-space").tap();
		};
		await commit();
		await expect(text).toHaveValue("門口");
		// kind scheme word code commits … last_used（第 9 欄）
		const first = await storedRow(page, "entry", "anr");
		expect(first?.slice(2, 5)).toEqual(["門口", "anr", "1"]);
		expect(first?.[8]).toBe("2026-11-12");

		await page.clock.setFixedTime(new Date(2026, 10, 14, 8, 0));
		await commit();
		await expect(text).toHaveValue("門口門口");
		const second = await storedRow(page, "entry", "anr");
		expect(second?.[4]).toBe("2");
		expect(second?.[8]).toBe("2026-11-14");
	});
});
