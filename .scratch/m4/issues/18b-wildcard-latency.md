# 18b: 含萬用鍵的輸入進每鍵延遲閘門

Status: done
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

- [x] 改前的數字（含萬用鍵的輸入，空白與學過各三組 p95）與慢在哪裡的證據貼在 Comments。
- [x] 改後含萬用鍵的輸入在〈每鍵延遲上限〉內（負載低時單獨量），前後對照貼在 Comments。
- [x] `bench --check` 量並擋含萬用鍵的輸入；閘門的判斷有雙向測試（假計時器：含萬用鍵那組超過時紅、在內不紅）。
- [x] `bench --check` 各集合與翻盤清單前後相同；含萬用鍵的輸入的候選列前後逐字相同的測試或比對。
- [x] `docs/development.md`〈每鍵延遲〉、spec〈每鍵延遲上限〉、brief §4.3 萬用鍵那條的延遲數字已更新；`cargo xtask check` 綠燈。

## Comments

- 2026-11-13 失效條件與慢在哪裡。量法照票 09：基準的每鍵延遲那批（固定題與四個語料，6,722 題、123,330 鍵），每題第一碼換成 `z`，原生 release，Apple M6。負載低時（load 2.1）單獨量，改前 p95：空白 全部 961 µs、第 20 碼起 **1,010 µs**（超過 1 ms）、選開頭一段 1,255 µs；學過 1,176 µs、1,230 µs、1,824 µs。負載高時（load 17）空白全部是 1,111 µs。p50 也有 556 µs，所以不是長尾：幾乎每一鍵都慢。
- profile（macOS `sample`，空白、量改前的程式 20 秒）：每鍵的 `Session::handle` 12.4k 個樣本裡，`refresh_candidates` → `lattice::candidates` 佔 9.2k；其中開頭各段的迴圈（`(1..n).rev()` 對每個開頭長度呼叫 `Engine::span`）佔 7.7k，又分成 `wildcard_edges` 依字頻排序 3.5k（`sort_by_key` 每次比較都查一次字頻）、`matching_codes` 2.5k。`Lattice::push` 的 `edges_ending` 只有約 1.2k。票 09 的推測對了一半：重算的是開頭各段的 `span`（`z`、`zX` 這兩段各對上幾百個碼），蓋滿整串那段反而便宜；lattice 建邊不是瓶頸。
- 改法（只改速度）：(1) `Lattice` 記每個開頭長度的 `span`（`spans`），由候選列第一次用到時算，逐鍵打字時重用，退格連同那一欄刪掉；`lattice::candidates` 改收 `&mut Lattice`。(2) `wildcard_edges` 的字頻排序改 `sort_by_cached_key`（同樣是穩定排序）。(3) `Session::refresh` 丟掉留著的開頭各段（code-review 抓到：網頁 `setPacks` 換了引擎後呼叫它，不丟的話開頭各段會照舊資料包列）。
- 前後對照（原生 p95，µs，負載低時改前、只有 (1)、(1)+(2) 接連量）：

  | | 空白 全部 | 空白 第 20 碼起 | 空白 選開頭一段 | 學過 全部 | 學過 第 20 碼起 | 學過 選開頭一段 |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | 改前 | 961 | 1,010 | 1,255 | 1,176 | 1,230 | 1,824 |
  | 只有 (1) | 491 | 203 | 1,252 | 666 | 266 | 1,868 |
  | (1)+(2) | 237 | 202 | 1,246 | 381 | 267 | 1,890 |
  | 上限 | 1,000 | 1,000 | 5,000 | 3,000 | 3,000 | 10,000 |

  `cargo xtask bench --check`（合併 main 後、接上個人集，load 2.9）的含萬用鍵的輸入：空白 236 / 204 / 1,238，學過 382 / 268 / 1,874；WebAssembly（只報）空白 246 / 217 / 1,520，學過 387 / 304 / 2,221。基準的輸入也跟著快（開頭各段不再每鍵重算）：空白全部 p95 60 µs（票 18 量時 75 µs）、學過 133 µs（165 µs）；選開頭一段不變（選定後從頭建 lattice，本來每段就只算一次）。
- 候選前後逐字相同：(a) `bench --check` 各集合（16 個公開集合與個人集）的答錯、修正次數都等於 baseline，翻盤清單全是「沒有翻盤」，固定題 84/84。(b) 比對：在 scratchpad 寫一個不進 repo 的小程式，用 `qim_bench::wildcard_inputs` 取含萬用鍵的 6,722 題，空白與學過各打一次，每一鍵把 `Snapshot` 的 `Debug` 與 `Session::all_candidates()` 的全部文字餵進 `DefaultHasher`，打完有兩段以上的再照 `lefts_to_first_segment` 選開頭一段、同樣雜湊，每題一列；改前的程式與 (1)、(1)+(2) 各跑一次，13,444 列逐列相同。(c) 測試：`properties.rs` 加萬用鍵方案的「逐鍵算的候選等於從頭建的」性質測試（含退格、選字、目前分段）；把 `pop` 裡丟 `spans` 那行拿掉時它紅（找到的反例存在 `properties.proptest-regressions`）。`wildcard.rs` 加換資料包後 `refresh` 的測試，修 (3) 前紅。
- 閘門：`qim_bench::wildcard_inputs` 是規則（每個輸入第一碼換成方案的萬用鍵）；`Latency` 多兩組（`wildcard_blank`、`wildcard_learned`），原生照同一組上限擋、WebAssembly 只報，報告多一欄「批次」。雙向測試：`tests/latency.rs` 的 `wildcard_inputs_within_the_same_limits_pass`、`wildcard_inputs_over_a_limit_are_a_violation_naming_them`，`tests/run.rs` 的 `check_blocks_the_wildcard_inputs_over_their_limit`（假計時器只讓含萬用鍵那批慢：紅、只註明它；在上限內：不紅）。
- 票外的取捨：(a) 方案沒有萬用鍵時 `bench` 整個失敗（同〈每鍵延遲〉「量不了是錯誤」），所以 `tests/run.rs` 的小方案加了 `z`；之後的方案沒有萬用鍵時再改成跳過並註明。(b) 組字途中殼改了學習資料（網頁的 `mergeUserDict`、`addWord` 這類）又沒呼叫 `refresh` 時，開頭各段照舊到退格到更短或組字結束，和 lattice 早已留著的欄一樣；改前開頭各段每鍵重查會跟上。現在的殼都是一個 `Session` 配一份 `UserModel`，沒有別的 `Session` 在組字中途學習；要完全照舊得在 `UserModel` 的二十幾個寫入方法加版本號，不做。
- code-review（`main` 起，Standards 與 Spec 兩軸）已處理：`setPacks` 後開頭各段照舊資料包（上面 (3)）；`Inputs` 改名 `InputBatch`、報告欄「輸入碼」改「批次」（輸入碼是名詞表的詞）；`report::latency` 的注釋補兩批輸入；`tests/run.rs` 假計時器注釋和 `.all` 對不上；`candidates` 直接讀 `spans` 欄改成 `Lattice::cached_spans`；小方案加 `z` 的原因寫在 fixture。沒處理：`Latency` 四個平攤的欄位（審查建議 2×2 的結構，四欄有名字、`runs()` 收在一處，不改）；commit `198759e` 的 subject 一次寫了 core 與 bench（已在合併之前，不改寫歷史）；比對程式沒進 repo（一次性的，做法寫在上面）。
- 給之後的票（不在本票範圍）：`Session::refresh` 換了資料包時沒重建 lattice 的欄，多段路徑仍是舊資料包算的（改前就這樣；網頁組字中途換詞庫時可見，例如 `日口` 留著已拿掉的「日」）。要修就讓 `refresh` 從頭建 lattice（像撤銷學習那樣，目前分段對不上時回整串）；建議由動到資料包切換的票或 M4 收尾票（31）決定去處。

Session 紀錄：含萬用鍵的輸入每鍵延遲：候選列重用開頭各段的 span、字頻排序只查一次，空白全部 p95 961→237 µs；`bench --check` 加這批輸入、同一組上限
