import { expect, test } from "@playwright/test";
import { ready, waitReady } from "./page.ts";

// 區網 http 開網頁（M4 票 31b）：手機用 `http://192.168.x.x:5174/` 開，不是安全環境（HTTPS 或 localhost），
// 瀏覽器不給 `crypto.randomUUID`、`navigator.clipboard`。這裡讓頁面的網址是不存在的主機名，請求攔下來轉給
// 開發伺服器，頁面就跑在真的非安全環境裡。
const LAN = "http://qim-lan.test";

test.beforeEach(async ({ context, baseURL }) => {
	await context.route(`${LAN}/**`, async (route) => {
		const url = route
			.request()
			.url()
			.replace(LAN, baseURL ?? "");
		await route.fulfill({ response: await route.fetch({ url }) });
	});
});

// 輸入框 ready 之後頁面還在背景載入停鍵層的 Worker（模組與 wasm）、跑線上更新檢查，測試先結束時上面的
// handler 還在轉發，context 關掉就讓 `route.fetch` 丟錯，記成這條測試或「不屬於任何測試」的錯（flow-sync
// 票 02b）。結束前拿掉轉發，轉發到一半的請求丟的錯不算：測試已經結束，沒有人看那些回應。不用
// `behavior: "wait"`：拿掉轉發的當下剛進 handler 的請求會被 Playwright 自動放行，handler 接著 fulfill
// 仍會丟「Route is already handled!」。
test.afterEach(async ({ context }) => {
	await context.unrouteAll({ behavior: "ignoreErrors" });
});

test("非安全環境開網頁：產生裝置 id、照常載入能打字", async ({
	page,
}, testInfo) => {
	await page.goto(`${LAN}/`);
	expect(await page.evaluate(() => window.isSecureContext)).toBe(false);
	const { text } = await waitReady(page);
	await expect(page.getByText("載入失敗")).toHaveCount(0);
	const device = await page.evaluate(() => localStorage.getItem("qim.device"));
	expect(device).toMatch(
		/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
	);

	await text.focus();
	await page.keyboard.type("anr ");
	await expect(text).toHaveValue("門口");
	await page.screenshot({ path: testInfo.outputPath("insecure-context.png") });
});

test("非安全環境按「複製全部」：選取回報清單、說明要自己複製", async ({
	page,
}) => {
	await ready(page, `${LAN}/`);
	const reports = page.getByRole("textbox", { name: /回報的固定題/ });
	await reports.fill("mir\t一台\t網頁回報");

	await page.getByRole("button", { name: "複製全部" }).click();
	await expect(reports).toBeFocused();
	expect(
		await reports.evaluate((area: HTMLTextAreaElement) => [
			area.selectionStart,
			area.selectionEnd,
		]),
	).toEqual([0, "mir\t一台\t網頁回報".length]);
	await expect(page.getByText(/已選取回報清單/)).toBeVisible();
});
