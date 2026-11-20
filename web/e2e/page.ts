// e2e 共用的 helper：多數 spec 都要的定位器、等網頁就緒、讀候選與按鍵機率、換輸入框內容。
// 各 spec 特有的定位器與步驟留在 spec，用展開（`{ ...ui(page), … }`）或先呼叫這裡的再加步驟組起來。

import { expect, type Page } from "@playwright/test";

/** 輸入框、組字區、候選列的候選。 */
export function ui(page: Page) {
	return {
		text: page.getByRole("textbox", { name: "輸入框" }),
		composition: page.getByTestId("composition"),
		items: page.getByRole("list", { name: "候選" }).getByRole("listitem"),
	};
}

/** 等網頁接上核心（輸入框的 `data-ready`），不換頁；回傳 `ui(page)`。 */
export async function waitReady(page: Page) {
	const parts = ui(page);
	await expect(parts.text).toHaveAttribute("data-ready", "true");
	return parts;
}

/** 開 `url` 並等它就緒；回傳 `ui(page)`。 */
export async function ready(page: Page, url = "/") {
	await page.goto(url);
	return waitReady(page);
}

/** 開 `url`、清掉 `localStorage` 再載入一次（從沒學過、沒設定過開始），等它就緒；回傳 `ui(page)`。 */
export async function readyCleared(page: Page, url = "/") {
	await page.goto(url);
	await page.evaluate(() => localStorage.clear());
	await page.reload();
	return waitReady(page);
}

/** 候選列上的候選，去掉開頭的序號。 */
export async function shown(page: Page): Promise<string[]> {
	return (await ui(page).items.allTextContents()).map((t) =>
		t.replace(/^\d/, ""),
	);
}

/** 換掉輸入框的內容、游標放在第 `cursor` 個 UTF-16 碼元，像使用者貼上後點到那裡。 */
export async function place(
	page: Page,
	value: string,
	cursor = value.length,
): Promise<void> {
	await ui(page).text.evaluate(
		(element: HTMLTextAreaElement, [value, cursor]) => {
			element.value = value as string;
			element.dispatchEvent(new Event("input", { bubbles: true }));
			element.setSelectionRange(cursor as number, cursor as number);
		},
		[value, cursor] as const,
	);
}

/** debug 面板上最近一個鍵（實體鍵或觸控點擊）的各字母與機率，由高到低。 */
export async function keyProbabilities(page: Page) {
	return page
		.getByTestId("debug-weights")
		.locator("code")
		.evaluateAll((codes) =>
			codes.map((code) => ({
				letter: code.getAttribute("data-letter") ?? "",
				probability: Number(code.getAttribute("data-probability")),
			})),
		);
}
