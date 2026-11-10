// 回報區：把剛才那次組字記成 `bench/fixed.tsv` 的一行（`report.ts` 判定哪次算數）。

import { element } from "./dom.ts";
import { fixedLine, initialReport, observe } from "./report.ts";
import type { Snapshot } from "./wasm/qim_wasm.js";

/** 回報清單存在 `localStorage`，重新整理不會不見。 */
const REPORTS_KEY = "qim.reports";

export interface ReportPanel {
	/** 核心吃下一個鍵之後呼叫；`chosen` 是這個鍵是不是選字（空白、數字鍵）。 */
	observe(snapshot: Snapshot, chosen: boolean): void;
}

/**
 * 接上回報區。複製失敗寫到 `status`；`afterAdd` 在加了一行之後呼叫（網頁把焦點還給輸入框）。
 */
export function mountReportPanel(
	status: HTMLElement,
	afterAdd: () => void,
): ReportPanel {
	const reportButton = element<HTMLButtonElement>("#report");
	const reports = element<HTMLTextAreaElement>("#reports");
	let report = initialReport;

	reports.value = localStorage.getItem(REPORTS_KEY) ?? "";
	reports.addEventListener("input", () => {
		localStorage.setItem(REPORTS_KEY, reports.value);
	});
	// 把剛才那次組字加到回報清單最後一行；同一次只加一次。
	reportButton.addEventListener("click", () => {
		if (!report.last) {
			return;
		}
		const lines = reports.value === "" ? [] : reports.value.split("\n");
		lines.push(fixedLine(report.last));
		reports.value = lines.join("\n");
		localStorage.setItem(REPORTS_KEY, reports.value);
		reportButton.disabled = true;
		afterAdd();
	});
	element<HTMLButtonElement>("#copy-reports").addEventListener("click", () => {
		navigator.clipboard.writeText(reports.value).catch((error: unknown) => {
			status.textContent = `複製失敗：${String(error)}`;
		});
	});

	return {
		/**
		 * `Snapshot` 是 WebAssembly 的物件、用完就釋放，回報只留要的欄位。
		 * 候選與上屏取繁體原文：`fixed.tsv` 的期望首選是繁體，簡體模式下也一樣。
		 */
		observe(snapshot, chosen) {
			const before = report.last;
			report = observe(report, {
				input: snapshot.input,
				selected: snapshot.selected,
				candidates: snapshot.candidatesOriginal,
				commit: snapshot.commitOriginal,
				chosen,
			});
			if (report.last !== before) {
				reportButton.disabled = report.last === undefined;
			}
		},
	};
}
