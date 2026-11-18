# 21: 英文滑動輸入

Status: ready-for-agent
Blocked by: 20
讀: 票 20 的 ADR；brief §4.7；`ARCHITECTURE.md`〈佈局格式〉〈核心對殼〉

## What to build

照票 20 的 ADR：英文頁上畫過字母出英文詞，建議列給其他候選；用英文詞表與英文使用者詞。開關進設定頁。

## 驗收

- [ ] Playwright 用模擬軌跡在英文頁畫出 `hello`、`world` 上屏；換一份按鍵位置不同的佈局也解得出。
- [ ] flick 鍵的滑動動作照舊（測試讀回）。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
