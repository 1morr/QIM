import { readFile } from "node:fs/promises";
import { expect, type Page, test } from "@playwright/test";

// 使用者詞與學習資料（M1 票 14）：分段選字上屏後自動造詞、存 localStorage、匯出匯入、手動加詞。匯入是合併
// （M4 票 17）；兩個瀏覽器設定檔互相同步在 `sync.spec.ts`。

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

	// 再選一次成首選（brief〈學習與排位〉）。
	await pick(page, "雲口");
	await page.keyboard.type("mir");
	await expect(items.first()).toHaveText("1雲口");
});

test("匯出、清空、從備份還原後讀回相同（多了清空記號）", async ({
	page,
}, testInfo) => {
	await learnYunHau(page);
	const { status } = await ready(page);

	const download = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	const file = await (await download).path();
	const exported = await readFile(file, "utf8");
	expect(exported.split("\n")[0]).toBe("# qim-userdict v3");
	expect(exported).toContain("\t雲口\tmir\t");

	await page.getByRole("button", { name: "清空學習資料" }).click();
	await expect(status).toHaveText("已清空學習資料");
	expect(await secondOfMir(page)).not.toBe("雲口");

	// 清空會同步（清空記號），清空前的檔用「匯入」合併不收；要用「從備份還原」。
	await page.getByLabel("從備份還原（取代目前的）").setInputFiles(file);
	await page
		.getByRole("group", { name: "確認匯入" })
		.getByRole("button", { name: "確定還原" })
		.click();
	await expect(status).toHaveText(/^已從 .* 還原$/);
	expect(await secondOfMir(page)).toBe("雲口");
	await page.screenshot({ path: testInfo.outputPath("imported.png") });

	const again = page.waitForEvent("download");
	await page.getByRole("button", { name: "匯出學習資料" }).click();
	const restored = (await readFile(await (await again).path(), "utf8")).split(
		"\n",
	);
	const cleared = restored.findIndex((l) => l.startsWith("#@/cleared\t"));
	expect(restored[cleared]).toMatch(/^#@\/cleared\t2\t/);
	restored.splice(cleared, 1);
	expect(restored.join("\n")).toBe(exported);
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
	// 裝置表（M4 票 17）：各台次數欄的編號 0 就是這台。
	expect(lines[2]).toBe(lines[1].replace("#@/device\t", "#@/device/0\t"));
	const [yunHau] = entryRows(first, "雲口");
	// kind scheme word code counts … last_used(11) source(12)；counts 是「編號:用過:整串上屏:不算的:最後整串上屏的日期」
	expect(yunHau?.slice(3, 5)).toEqual(["mir", "0:1:1:0:2026-11-12"]);
	expect(yunHau?.slice(11, 13)).toEqual(["2026-11-12", "auto"]);
	const [yun] = entryRows(first, "雲");
	expect(yun?.[11]).toBe("2026-11-12");

	// 過了午夜（同一個分頁沒重新整理），選第 2 位的「雲口」。
	await page.clock.setFixedTime(new Date(2026, 10, 13, 0, 10));
	await page.keyboard.type("mir");
	await pick(page, "雲口");
	const second = await exportedFile(page);
	expect(second.split("\n")[1]).toBe(lines[1]);
	const [again] = entryRows(second, "雲口");
	expect(again?.[4]).toBe("0:2:2:0:2026-11-13");
	expect(again?.[11]).toBe("2026-11-13");
	expect(entryRows(second, "雲")[0]?.[11]).toBe("2026-11-12");

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
		"#@/device/0\tother-device",
		"#@/tick/quick\t0",
		"entry\tquick\t速成\tqim\t0:1:0:0:\t\t\t\t\t\t\t\tmanual\tqc\t1\t2026-01-02",
		"future\tquick\t速成\tqim\t2026-01-03",
		"",
	].join("\n");
	await page.getByLabel("匯入學習資料（合併進目前的）").setInputFiles({
		name: "future.tsv",
		mimeType: "text/plain",
		buffer: Buffer.from(file),
	});
	const confirm = page.getByRole("group", { name: "確認匯入" });
	await expect(confirm).toContainText("略過 1 列：第 6 行");
	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await expect(status).toHaveText(
		/已合併 future\.tsv：新增 1 筆、更新 0 筆、刪除 0 筆（對方刪掉的），略過 1 列：第 6 行：不認得的種類「future」/,
	);
	const exported = await exportedFile(page);
	expect(entryRows(exported, "速成")[0]?.slice(12)).toEqual([
		"manual",
		"qc",
		"1",
		"2026-01-02",
	]);
	// 檔頭是這台裝置的 id，不換成檔裡的；別台記的次數照樣是別台的（裝置表有它）。
	expect(exported.split("\n")[1]).toMatch(/^#@\/device\t[0-9a-f-]{36}$/);
	expect(exported).toContain("\tother-device\n");
});

test("匯入不是 qim-userdict 的檔時報錯，學習資料不動", async ({ page }) => {
	await learnYunHau(page);
	const { status } = await ready(page);
	await page.getByLabel("匯入學習資料（合併進目前的）").setInputFiles({
		name: "bad.tsv",
		mimeType: "text/plain",
		buffer: Buffer.from("# Rime user dictionary\n"),
	});
	await expect(status).toHaveText(/匯入失敗.*第 1 行/);
	await expect(page.getByRole("group", { name: "確認匯入" })).toBeHidden();
	expect(await secondOfMir(page)).toBe("雲口");
});

/** 只有一個手動詞的 v1 檔（v2 起仍收）：合併進來後「雲口」照樣在，多了「速成」。 */
const OTHER_DICT = [
	"# qim-userdict v1",
	"#@/tick/quick\t0",
	"quick\t速成\tqim\t0\t0\t0\t0\tmanual\tqc",
	"",
].join("\n");

test("匯入前在頁面上說明會合併幾筆，確定後才合併", async ({
	page,
}, testInfo) => {
	await learnYunHau(page);
	const { status } = await ready(page);
	const confirm = page.getByRole("group", { name: "確認匯入" });
	const choose = () =>
		page.getByLabel("匯入學習資料（合併進目前的）").setInputFiles({
			name: "other.tsv",
			mimeType: "text/plain",
			buffer: Buffer.from(OTHER_DICT),
		});
	await expect(confirm).toBeHidden();

	await choose();
	await expect(confirm).toBeVisible();
	await expect(confirm).toContainText("other.tsv");
	await expect(confirm).toContainText(
		"會合併進目前的學習資料：新增 1 筆、更新 0 筆、刪除 0 筆（對方刪掉的），其餘不變",
	);
	await page.screenshot({ path: testInfo.outputPath("confirm-import.png") });
	expect(await secondOfMir(page), "確定之前不動").toBe("雲口");

	await confirm.getByRole("button", { name: "取消" }).click();
	await expect(confirm).toBeHidden();
	await expect(status).toHaveText("沒有匯入，學習資料沒動");
	expect(await secondOfMir(page)).toBe("雲口");

	await choose();
	await confirm.getByRole("button", { name: "確定匯入" }).click();
	await expect(confirm).toBeHidden();
	await expect(status).toHaveText(
		"已合併 other.tsv：新增 1 筆、更新 0 筆、刪除 0 筆（對方刪掉的）",
	);
	expect(await secondOfMir(page), "合併不丟原本的").toBe("雲口");
	const { items } = await ready(page);
	await page.keyboard.type("qc");
	await expect(items.first()).toHaveText("1速成");
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

// M4 票 12–16 存下的 v3 草稿（main 7ad01da 匯出的檔）：網頁存在 localStorage 的學習資料照樣讀得進來，不搬到
// qim.userdict.broken（協調者 2026-11-12 決定；fixture 與核心測試共用）。
test("localStorage 存的是 7ad01da 的 v3 草稿時照樣讀進來", async ({ page }) => {
	const draft = await readFile(
		new URL(
			"../../crates/qim-core/tests/fixtures/userdict-v3-draft-7ad01da.tsv",
			import.meta.url,
		),
		"utf8",
	);
	await page.evaluate(
		(text) => localStorage.setItem("qim.userdict", text),
		draft,
	);
	await page.reload();
	const { items, status } = await ready(page);
	await expect(status).toHaveText("");
	expect(
		await page.evaluate(() => localStorage.getItem("qim.userdict.broken")),
	).toBeNull();
	await page.keyboard.type("kk");
	await expect(items.first()).toHaveText("1口口");
	const exported = await exportedFile(page);
	expect(exported).toContain(
		"#@/device/0\t6f1c2d3e-4b5a-4c6d-8e7f-0a1b2c3d4e5f\n",
	);
});
