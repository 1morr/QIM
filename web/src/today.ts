// 今天的日期（plan〈學習與排位〉 殼的日期契約）：核心不讀系統時間（plan〈依賴規則〉），由殼在第一次學習前、之後每次
// 使用者操作前交給它（`Session.setToday`），學到的條目記成最後使用的日期。

/** `now` 在使用者當地的日曆日，寫成 `YYYY-MM-DD`（核心 `Date` 收的寫法）。 */
export function localToday(now: Date = new Date()): string {
	const year = String(now.getFullYear()).padStart(4, "0");
	const month = String(now.getMonth() + 1).padStart(2, "0");
	const day = String(now.getDate()).padStart(2, "0");
	return `${year}-${month}-${day}`;
}
