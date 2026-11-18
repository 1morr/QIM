# 24: 評估空輸入的句首建議與量詞聯想

Status: ready-for-agent
Blocked by: 12
讀: brief §4.5；`ARCHITECTURE.md`〈候選的排位順序〉；`docs/research/blackbox-baseline.md`（T8、T18）；`.scratch/m4/issues/30-association-boost.md` 的 Comments

## What to build

iOS 速成有：空輸入時給句首建議、數字與標點後給量詞聯想。和沒有組字時的工具列搶同一格（票 05 的定稿）；句首建議可用字級 bigram 的句首（`CLAUSE_START`），量詞沒有資料、要另整理量詞表或重訓；桌面上兩者都要在沒有組字時開候選窗。門檻先定再量，結論寫回 brief；採用的話開實作票。

## 驗收

- [ ] 量法與結果寫進 `docs/research/`；結論寫回 brief。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
