import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 使用者詞與學習資料（M1 票 14）：分段選字上屏後自動造詞、存 localStorage、匯出匯入、手動加詞。

async function ready(page: Page) {
	const text = page.getByRole("textbox", { name: "輸入框" });
	await expect(text).toHaveAttribute("data-ready", "true");
	await text.focus();
	return {
		text,
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
		status: page.getByRole("status", { name: "學習資料狀態" }),
	};
}

/** 候選列的文字，去掉前面的序號。 */
async function shown(page: Page): Promise<string[]> {
	const items = page.getByRole("list", { name: "候選" }).getByRole("listitem");
	return (await items.allTextContents()).map((t) => t.replace(/^\d/, ""));
}

/** 按數字鍵選這頁文字是 `word` 的候選。 */
async function pick(page: Page, word: string) {
	const texts = await shown(page);
	const at = texts.indexOf(word);
	expect(at, `候選：${texts.join(" ")}`).not.toBe(-1);
	await page.keyboard.type(String(at + 1));
}

/** `mir` 分段選「雲」「口」上屏：「雲口」不在詞庫，學成使用者詞。 */
async function learnYunHau(page: Page) {
	const { text, items } = await ready(page);
	await page.keyboard.type("mir");
	const before = await shown(page);
	expect(before.indexOf("雲口"), `學之前：${before.join(" ")}`).not.toBe(1);
	await pick(page, "雲");
	await expect(text).toHaveValue("");
	await pick(page, "口");
	await expect(text).toHaveValue("雲口");
	// 上屏後候選列是聯想（M3 票 13），收起再接著測。
	await page.keyboard.press("Escape");
	await expect(items).toHaveCount(0);
}

async function secondOfMir(page: Page): Promise<string> {
	const { text } = await ready(page);
	await page.keyboard.type("mir");
	const second = (await shown(page))[1];
	await page.keyboard.press("Escape");
	await text.fill("");
	return second;
}

test.beforeEach(async ({ page }) => {
	await page.goto("/");
	await page.evaluate(() => localStorage.clear());
	await page.reload();
});

test("分段選字上屏後重新整理網頁，學到的詞仍在第 2 位", async ({
	page,
}, testInfo) => {
	await learnYunHau(page);
	await page.reload();
	const { items } = await ready(page);
	await page.keyboard.type("mir");
	await expect(items.nth(1)).toHaveText("2雲口");
	await page.screenshot({
		path: testInfo.outputPath("auto-phrase-reload.png"),
	});

	// 再選一次成首選（brief §4.5）。
	await pick(page, "雲口");
	await page.keyboard.type("mir");
	await expect(items.first()).toHaveText("1雲口");
});

test("匯出、清空、匯入後讀回相同", async ({ page }, testInfo) => {
	await learnYunHau(page);
	const { status } = await ready(page);

	const download = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	const file = await (await download).path();
	const exported = await readFile(file, "utf8");
	expect(exported.split("\n")[0]).toBe("# qim-userdict v3");
	expect(exported).toContain("\t雲口\tmir\t1\t");

	await page.getByRole("button", { name: "清空學習資料" }).click();
	await expect(status).toHaveText("已清空學習資料");
	expect(await secondOfMir(page)).not.toBe("雲口");

	await page.getByLabel("匯入學習資料（取代目前的）").setInputFiles(file);
	await page
		.getByRole("group", { name: "確認匯入" })
		.getByRole("button", { name: "確定匯入" })
		.click();
	await expect(status).toHaveText(/已匯入/);
	expect(await secondOfMir(page)).toBe("雲口");
	await page.screenshot({ path: testInfo.outputPath("imported.png") });

	const again = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	expect(await readFile(await (await again).path(), "utf8")).toBe(exported);
});

/** 按「匯出學習資料」下載的檔。 */
async function exportedFile(page: Page): Promise<string> {
	const download = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	return readFile(await (await download).path(), "utf8");
}

/** 匯出檔裡種類 `entry`、詞是 `word` 的列，拆成欄。 */
function entryRows(exported: string, word: string): string[][] {
	return exported
		.split("\n")
		.map((line) => line.split("\t"))
		.filter((columns) => columns[0] === "entry" && columns[2] === word);
}

// 學習資料 v3（M4 票 12）：檔頭有裝置 id，每列有最後使用的日期；日期由網頁每次操作前交給核心，
// 測試用 `page.clock` 換日。
test("匯出 v3 的檔頭有裝置 id、每列有日期，換一天上屏後日期跟著變", async ({
	page,
}, testInfo) => {
	await page.clock.setFixedTime(new Date(2026, 10, 12, 23, 50));
	await learnYunHau(page);
	const first = await exportedFile(page);
	const lines = first.split("\n");
	expect(lines[0]).toBe("# qim-userdict v3");
	expect(lines[1]).toMatch(/^#@\/device\t[0-9a-f-]{36}$/);
	const [yunHau] = entryRows(first, "雲口");
	// kind scheme word code commits last_commit_tick decayed last_tick last_used source trigger
	expect(yunHau?.slice(3, 5)).toEqual(["mir", "1"]);
	expect(yunHau?.slice(8, 10)).toEqual(["2026-11-12", "auto"]);
	const [yun] = entryRows(first, "雲");
	expect(yun?.[8]).toBe("2026-11-12");

	// 過了午夜（同一個分頁沒重新整理），選第 2 位的「雲口」。
	await page.clock.setFixedTime(new Date(2026, 10, 13, 0, 10));
	await page.keyboard.type("mir");
	await pick(page, "雲口");
	const second = await exportedFile(page);
	expect(second.split("\n")[1]).toBe(lines[1]);
	const [again] = entryRows(second, "雲口");
	expect(again?.[4]).toBe("2");
	expect(again?.[8]).toBe("2026-11-13");
	expect(entryRows(second, "雲")[0]?.[8]).toBe("2026-11-12");

	// 重新整理後裝置 id 不變（存在 localStorage）。
	await page.reload();
	await ready(page);
	expect((await exportedFile(page)).split("\n")[1]).toBe(lines[1]);
	await page.screenshot({ path: testInfo.outputPath("userdict-v3-date.png") });
});

/** v3 的檔帶一列之後版本才有的種類：略過並說明，其他列照收。 */
test("匯入 v3 時不認得的種類略過並說明，其他列照收", async ({ page }) => {
	const { status } = await ready(page);
	const file = [
		"# qim-userdict v3",
		"#@/device\tother-device",
		"#@/tick/quick\t0",
		"entry\tquick\t速成\tqim\t0\t0\t0\t0\t2026-01-02\tmanual\tqc",
		"tombstone\tquick\t速成\tqim\t2026-01-03",
		"",
	].join("\n");
	await page.getByLabel("匯入學習資料（取代目前的）").setInputFiles({
		name: "future.tsv",
		mimeType: "text/plain",
		buffer: Buffer.from(file),
	});
	const confirm = page.getByRole("group", { name: "確認匯入" });
	await expect(confirm).toContainText("略過 1 列：第 5 行");
	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await expect(status).toHaveText(
		/已匯入 future\.tsv，略過 1 列：第 5 行：不認得的種類「tombstone」/,
	);
	const exported = await exportedFile(page);
	expect(entryRows(exported, "速成")[0]?.slice(8)).toEqual([
		"2026-01-02",
		"manual",
		"qc",
	]);
	// 這台裝置的 id 留著，不換成檔裡的。
	expect(exported).not.toContain("other-device");
	expect(exported.split("\n")[1]).toMatch(/^#@\/device\t[0-9a-f-]{36}$/);
});

test("匯入不是 qim-userdict 的檔時報錯，學習資料不動", async ({ page }) => {
	await learnYunHau(page);
	const { status } = await ready(page);
	await page.getByLabel("匯入學習資料（取代目前的）").setInputFiles({
		name: "bad.tsv",
		mimeType: "text/plain",
		buffer: Buffer.from("# Rime user dictionary\n"),
	});
	await expect(status).toHaveText(/匯入失敗.*第 1 行/);
	await expect(page.getByRole("group", { name: "確認匯入" })).toBeHidden();
	expect(await secondOfMir(page)).toBe("雲口");
});

/** 只有一個手動詞的 v1 檔（v2 起仍收）：匯入後「雲口」就不在了。 */
const OTHER_DICT = [
	"# qim-userdict v1",
	"#@/tick/quick\t0",
	"quick\t速成\tqim\t0\t0\t0\t0\tmanual\tqc",
	"",
].join("\n");

test("匯入前在頁面上說明會覆蓋目前的學習資料，確定後才取代", async ({
	page,
}, testInfo) => {
	await learnYunHau(page);
	const { status } = await ready(page);
	const confirm = page.getByRole("group", { name: "確認匯入" });
	const choose = () =>
		page.getByLabel("匯入學習資料（取代目前的）").setInputFiles({
			name: "other.tsv",
			mimeType: "text/plain",
			buffer: Buffer.from(OTHER_DICT),
		});
	await expect(confirm).toBeHidden();

	await choose();
	await expect(confirm).toBeVisible();
	await expect(confirm).toContainText("other.tsv");
	await expect(confirm).toContainText(/會覆蓋目前的 \d+ 筆學習資料/);
	await page.screenshot({ path: testInfo.outputPath("confirm-import.png") });
	expect(await secondOfMir(page), "確定之前不動").toBe("雲口");

	await confirm.getByRole("button", { name: "取消" }).click();
	await expect(confirm).toBeHidden();
	await expect(status).toHaveText("沒有匯入，學習資料沒動");
	expect(await secondOfMir(page)).toBe("雲口");

	await choose();
	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await expect(confirm).toBeHidden();
	await expect(status).toHaveText("已匯入 other.tsv");
	expect(await secondOfMir(page)).not.toBe("雲口");
});

test("手動加的詞打觸發詞時是首選，重新整理後還在", async ({
	page,
}, testInfo) => {
	const { text, items, status } = await ready(page);
	await page
		.getByRole("textbox", { name: "詞", exact: true })
		.fill("速成輸入法");
	await page.getByRole("textbox", { name: "觸發詞", exact: true }).fill("qim");
	await page.getByRole("button", { name: "加詞" }).click();
	await expect(status).toHaveText("已加入「速成輸入法」");
	await expect(text).toBeFocused();

	await page.keyboard.type("qim");
	await expect(items.first()).toHaveText("1速成輸入法");
	await page.keyboard.press("Space");
	await expect(text).toHaveValue("速成輸入法");

	await page.reload();
	await ready(page);
	await page.keyboard.type("qim");
	await expect(items.first()).toHaveText("1速成輸入法");
	await page.screenshot({ path: testInfo.outputPath("trigger.png") });
});

test("觸發詞有方案以外的鍵時加不進去", async ({ page }) => {
	const { status } = await ready(page);
	await page.getByRole("textbox", { name: "詞", exact: true }).fill("QIM");
	await page.getByRole("textbox", { name: "觸發詞", exact: true }).fill("q1");
	await page.getByRole("button", { name: "加詞" }).click();
	await expect(status).toHaveText(/加不進去/);
});
