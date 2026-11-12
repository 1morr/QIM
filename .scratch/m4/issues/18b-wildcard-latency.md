# 18b: 含萬用鍵的輸入進每鍵延遲閘門

Status: ready-for-agent
Blocked by: 18
讀: spec〈每鍵延遲上限〉〈共同規則〉；`docs/research/tolerance.md`〈每鍵延遲〉；`docs/research/wildcard.md`；plan §2.1 萬用鍵那條、§5.2；`docs/development.md`〈每鍵延遲〉；票 11、09、18 的 Comments

## What to build

期中檢查插入的修復票（使用者 2026-11-13 決定：先優化再加進閘門，上限不變）。票 09 量到只有萬用鍵 `z`（容錯關）的輸入，空白學習資料的每鍵全部 p95 1.07 ms（負載低時重跑；第一次 1.17 ms），超過〈每鍵延遲上限〉的 1 ms；學過的 1.50 ms、選開頭一段都在上限內。`bench --check` 只量沒有 `z` 的輸入，守不到它。

- 先寫出失效條件：照票 09 的量法（基準的每鍵延遲那批輸入，每個輸入的第一碼換成 `z`）量原生 p95，負載低時單獨量，記下數字。
- 找出慢在哪裡再改。票 09 的推測（沒驗證）：候選列每鍵都對蓋滿整串與每個開頭長度的碼重做一次萬用鍵展開（`lattice::candidates` 經 `Engine::span` 到 `Engine::wildcard_edges`），lattice 建邊時已展開過一次。用 profile 或計時確認，不照推測直接改。
- 優化只改速度：所有輸入的候選與排序逐字不變（`bench --check` 各集合前後相同，另把含 `z` 的那批輸入的候選列前後比對逐字相同）。
- `cargo xtask bench` 的每鍵延遲加一組「含萬用鍵」的輸入（取法照上面，規則放在 `qim_bench`，寫進 `docs/development.md`〈每鍵延遲〉），空白、學過都量，原生 p95 照〈每鍵延遲上限〉同一組上限由 `--check` 擋，WebAssembly 只報。spec〈每鍵延遲上限〉補一句含萬用鍵的輸入也擋。
- 上限不放寬（spec〈每鍵延遲上限〉的條件）。優化後仍超過就停下回報數字與原因，不改上限。

## 驗收

- [ ] 改前的數字（含萬用鍵的輸入，空白與學過各三組 p95）與慢在哪裡的證據貼在 Comments。
- [ ] 改後含萬用鍵的輸入在〈每鍵延遲上限〉內（負載低時單獨量），前後對照貼在 Comments。
- [ ] `bench --check` 量並擋含萬用鍵的輸入；閘門的判斷有雙向測試（假計時器：含萬用鍵那組超過時紅、在內不紅）。
- [ ] `bench --check` 各集合與翻盤清單前後相同；含萬用鍵的輸入的候選列前後逐字相同的測試或比對。
- [ ] `docs/development.md`〈每鍵延遲〉、spec〈每鍵延遲上限〉已更新；`cargo xtask check` 綠燈。

## Comments
