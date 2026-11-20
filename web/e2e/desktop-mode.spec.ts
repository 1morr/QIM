import { expect, test } from "@playwright/test";
import { ready } from "./page.ts";

// 桌面 viewport 預設是實體鍵盤模式，不顯示觸控鍵盤，可以手動打開（M1 票 18）。
test("桌面預設不顯示觸控鍵盤，按「觸控鍵盤」打開", async ({ page }) => {
	await ready(page);
	const toggle = page.getByRole("button", { name: "觸控鍵盤" });
	await expect(toggle).toHaveAttribute("aria-pressed", "false");
	await expect(page.getByTestId("keyboard")).toBeHidden();
	await toggle.click();
	await expect(page.getByTestId("keyboard")).toBeVisible();
	await page.getByTestId("key-a").click();
	await expect(page.getByTestId("composition")).toHaveText("日");
});
