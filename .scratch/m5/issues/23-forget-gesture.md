# 23: 撤銷學習的觸控手勢與簡體去重

Status: ready-for-agent
Blocked by: 12
讀: brief §4.4、§4.5；`ARCHITECTURE.md`〈候選的排位順序〉〈核心對殼〉；`.scratch/m4/issues/16-forget-candidate.md`、`30-association-boost.md` 的 Comments

## What to build

- 觸控上撤銷學習的手勢，能指定任一個候選（`Session::forget` 收任何位置）；長按候選維持字根提示（brief §4.4），手勢另定，先看 Gboard 與 iOS 的做法（盤點清單）。
- 簡體模式兩個原文轉成同一個簡體字時候選去重只留第一個，撤銷學習只清得到它；簡體去重時聯想的 emoji 從第 3 位變第 2 位（M4 票 16、30）。定怎麼處理，照成熟產品的做法。

## 驗收

- [ ] Playwright 用手勢撤銷第 2 位的自動造詞，再打同一串碼它不在第 2 位。
- [ ] 簡體去重的決定有測試讀回。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
