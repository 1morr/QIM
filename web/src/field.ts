// 欄位類型（M4 票 26，plan〈介面〉）：網頁從輸入框的 `type` 與 `inputmode` 讀出核心的欄位類型
// （`Session.setField`），換欄位類型時切中英（email、網址、密碼欄是英文直通）與數字頁。這個原型只有一個
// 輸入框，工具列的「欄位」選單模擬宿主的欄位：每個選項的 `data-type`、`data-inputmode` 是它模擬的 `input` 屬性。

import type { Session, Snapshot } from "./wasm/qim_wasm.js";

/** 核心收的欄位類型（`Session.setField`）。 */
export type FieldKind = "text" | "email" | "url" | "number" | "password";

/**
 * 照 Chrome 轉給 Android 的 `EditorInfo.inputType`（事實 P24）：`inputmode` 是 email、url、numeric、decimal、tel 時
 * 蓋過 `type`（Chrome 的 `inputmode=numeric` 是 NUMBER），否則看 `type`；`tel` 也開數字頁。`type=password` 不論
 * `inputmode` 都是密碼（不學、不讀前文不能被蓋掉）。不認得的照一般欄。
 */
export function fieldKind(
	type: string | null,
	inputMode: string | null,
): FieldKind {
	const kind = type?.toLowerCase();
	if (kind === "password") {
		return "password";
	}
	switch (inputMode?.toLowerCase()) {
		case "email":
			return "email";
		case "url":
			return "url";
		case "numeric":
		case "decimal":
		case "tel":
			return "number";
	}
	switch (kind) {
		case "email":
			return "email";
		case "url":
			return "url";
		case "number":
		case "tel":
			return "number";
		default:
			return "text";
	}
}

/**
 * 接上「欄位」選單：換選項等於聚焦一個新的欄位。核心先換欄位（`setField`：上一個欄位的組字丟掉、正在打的串
 * 打完，email、網址、密碼欄切到英文直通），`changed` 收到帶新中英的快照與欄位類型（畫面照它重畫、觸控鍵盤
 * 開或關數字頁），最後 `refocus` 讓輸入框重新聚焦、給前文。
 */
export function mountFieldType(
	select: HTMLSelectElement,
	session: Session,
	changed: (snapshot: Snapshot, field: FieldKind) => void,
	refocus: () => void,
): void {
	select.addEventListener("change", () => {
		const option = select.selectedOptions[0];
		const field = fieldKind(
			option?.dataset.type ?? null,
			option?.dataset.inputmode ?? null,
		);
		const snapshot = session.setField(field);
		try {
			changed(snapshot, field);
		} finally {
			snapshot.free();
		}
		refocus();
	});
}
