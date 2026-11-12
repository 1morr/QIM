# 25: 英文自動修正

Status: ready-for-agent
Blocked by: 24, 18c
讀: brief §4.7、§7；spec〈細節〉〈英文模式〉；票 19、21；spec〈共同規則〉

## What to build

英文模式的自動修正（開關，預設關，brief §7）：

- 修正的時機照票 19 的 Gboard 實測；用 `delete_before` 換掉打錯的詞。
- 對原樣讀得通的要保守；照 AOSP 不自動修的條件（含數字、全大寫、email 字元等）。
- 退格可還原；還原時不學修正字（原樣的那串學不學，由票定）。
- 英文的容錯開關（預設開，brief §7）照票 21 的 ADR。

## 驗收

- [ ] 核心測試：AOSP 不修的各條件；退格還原後不學修正字。
- [ ] Playwright：預設不修正 `teh`；打開自動修正後英文模式打 `teh` 修正成 `the`，退格還原；截圖。
- [ ] `cargo xtask check` 綠燈。
