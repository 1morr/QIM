# 04: `set_context` 與「有前文」的基準集合

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §2.1（上下文、組字中修改）、§2.3、§4.3、§5.2；brief §8、§9；spec〈範圍〉〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（票 21 那條）

## What to build

核心收得到輸入框的前文，連打的第一個字不再只用 unigram；用新的基準集合證明這有用。

- `Session::set_context(before: &str)`（plan §2.1）：lattice 開頭的字用前文最後一字的 bigram；前文裡不在連打字集的字（英文、標點、數字）怎麼處理在票內定，寫回 plan §2.3。無痕模式下核心不用前文（brief §8：不讀上下文）。
- 同一個機制：選開頭一段後，剩下輸入碼的第一個字接選定分段的最後一字（M1 票 21 的現狀由性質測試鎖住，改測試並寫回 plan §2.1、§2.3）。
- 首選字基準加「有前文」的集合：`cargo xtask data` 從語料的 test 切分取連續句子，前一兩句當前文、最後一句轉速成碼（plan §5.2 表格加一列）；基準報告同一批題給前文與不給前文並列；設 baseline。

## 驗收

- [ ] `set_context` 有單元測試：同一串碼、不同前文時首選不同的例子；無痕時前文不影響候選。
- [ ] 「有前文」集合給前文時答錯數與修正次數都比不給前文少（貼數字）。
- [ ] 其他集合 `bench --check` 不比改之前差，有變的寫理由；`--update-baseline` 同一 commit 說明。
- [ ] plan §2.1、§2.3、§5.2 與 `docs/development.md`〈首選字基準〉已更新；`cargo xtask check` 綠燈。

## Comments
