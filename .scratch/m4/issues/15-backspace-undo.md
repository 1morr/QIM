# 15: 上屏後退格撤銷學習

Status: ready-for-agent
Blocked by: 13, 14
讀: brief §4.5、§12.2（librime）；plan §2.4；spec〈細節〉（退格撤銷）；`docs/research/learning-forgetting-sync.md`；spec〈共同規則〉

## What to build

選錯上屏後馬上退格，這次的學習不該留下（照 librime `memory.cc` 的 `DiscardSession`，中英共用）：

- 中文：撤銷這次上屏的整串上屏次數、衰減值、自動造詞、個人 bigram。
- 英文：撤銷剛打完的那個英文使用者詞。英文直通時「打完」的界線（空白、標點）與退格的語意由票定並測。
- 「緊接著」照 librime：上屏後的第一個鍵是退格；中間夾了其他鍵就不撤銷。退格照常交給宿主刪字。

## 驗收

- [ ] 核心測試：上屏後緊接退格，學習資料回到上屏前（匯出比對）；中間夾別的鍵就不撤銷；英文的情況照票定的語意測。
- [ ] Playwright：同一串碼選第 2 個候選兩次，第二次上屏後緊接退格，再打同一串碼首選不變。
- [ ] plan §2.4 寫明；`cargo xtask check` 綠燈。
