# 18: 期中檢查

Status: done
Blocked by: 03, 04, 11, 17（10 依票 09 的結論 wontfix）
讀: spec 全部；`docs/progress.md`；票 01–17 的 Comments

## What to build

第一期做完，確認可用才開第二期（使用者 2026-11-12 決定）：

- 跑 `cargo xtask check`、`bench --check`（含每鍵延遲上限）與 `--learning`。
- 使用者在桌面與手機瀏覽器（區網）試用第一期：空間模型（容錯票 09 沒採用、不試）、學習時限、撤銷、英文使用者詞、同步。
- 回覆裡要改的，第一期的補票或記去處；影響第二期的，改 spec 與票 19–30。

## 驗收

- [x] 三個指令的輸出貼在 Comments。
- [x] 使用者試用並回覆；回覆與要改的地方記在 Comments，progress〈偏差與決定〉記一條。
- [x] 要改第二期的，spec 與票 19–30 已改。
- [x] 票 01–17 Comments 沒處理的發現逐條註明去處。

## Comments

- 2026-11-13（票 09 留下，協調者轉自票 11）：含萬用鍵 `z` 的輸入，容錯關時空白資料的每鍵全部 p95 1.07 ms（load 3–6 重跑；第一次 1.17 ms），超過〈每鍵延遲上限〉的 1 ms（學過 1.50 ms、選開頭一段都在上限內）；閘門只量沒有 `z` 的輸入，守不到它。要不要把含 `z` 的輸入加進閘門、超過要不要先優化萬用鍵（推測是候選列每鍵對整串與每個開頭長度重做萬用鍵展開），期中檢查決定。數字：`docs/research/tolerance.md`〈每鍵延遲〉。容錯與 `z` 一起的組合不存在（容錯沒採用）。
- 2026-11-13 三個指令（main `ee90dab`，本 worktree `cargo xtask data` 重建資料包 v8，個人集用主 checkout 的軟連結；load average 5–12）：
  - `cargo xtask bench --check`：`bench 通過`，固定題 84/84，17 個集合（含個人集）都等於 baseline、每個「沒有翻盤」；有前文給／不給答錯 1524／1568、修正 5573／5651；四個打錯集合的改錯都是 0（容錯沒採用）。答錯／修正：yue-wiki 941／3239、zh-wiki 979／3493、common-voice-yue 892／3249、hkcancor 572／2148、personal 1083／1463。
  - 每鍵延遲（原生 release p95，上限在括號）：空白 全部 75 µs（1 ms）、第 20 碼起 80 µs（1 ms）、選開頭一段 1.31 ms（5 ms）；學過 165 µs（3 ms）、262 µs（3 ms）、1.98 ms（10 ms）。WebAssembly（只報）空白 92 µs／100 µs／1.60 ms、學過 215 µs／407 µs／2.33 ms。載入原生 115 ms、WebAssembly 200 ms。閘門不量含 `z` 的輸入（見上一條）。
  - `cargo xtask bench --learning`：學習關／開的修正 yue-wiki 3239／3216、zh-wiki 3493／3578、common-voice-yue 3249／3108、hkcancor 2148／1719、yue-wiki·前文 1738／1715、zh-wiki·前文 1729／1764、common-voice-yue·前文 594／550、hkcancor·前文 1512／1179、personal 1463／1390；和票 04、14、15 相同（zh-wiki 學習開比關多 85 次是既有的）。
  - `cargo xtask check`：fmt、clippy、test（Rust 1162 passed、0 failed）、deps、docs、wasm、tsc、biome、unit（79 pass）、e2e（281 passed、2 skipped）全過，`check 通過`。第一次跑因 worktree 沒有 `web/node_modules` 停在 tsc，`npm ci` 後重跑。
- 2026-11-13 票 01–17 Comments 沒處理的發現的去處（逐條；「不修」指票上已判斷不改或接受的代價，理由在原票）：
  - 01：組字中途開無痕，面板「不讀前文」卻開頭接「句首」（lattice 不重建）→ 票 31 處理或註明；`learning.spec.ts:153` 每次 e2e 改寫已封存的 `.scratch/m3/evidence/23-personal-bigram.png` → 票 31；`inspect_path` 與 `Lattice::push` 平行一份 → 不修（路徑分相同的測試守）。
  - 02：IndexedDB 打不開時 `qim.packs.active` 記了不在的資料包、`checkModelPack` 位置、`PackLibrary` 聯合型別、`localStorage` 整份只留記憶體 → 不修。
  - 03：同一 `generation` 要求兩次且中間前文變了的快取縫隙、`RerankKey` 與 `RerankRequest` 重疊、重排識別用字串 → 不修。
  - 04、07、13、16、03：e2e helper（`place`、`shown`、`choose`、`ui()`、讀面板機率）各 spec 一份 → M5（重做觸控鍵盤時 e2e 大改，那時收成共用模組）；04 的 TSPhrases 退回逐字轉、詞級 trigram 缺直接測試 → 不修（既有行為，原票註明）。
  - 05：Node 延遲路徑沒有自動測試 → 不修（每次 `cargo xtask bench` 都實跑它，量不了就整個失敗）；`learned_model` 與學習迴圈相似、高負載下學過那組餘裕 2–3 倍 → 不修（容錯 wontfix，見下面〈每鍵延遲〉的決定）。
  - 05b、05c：`native_timing_with` 從根匯出、`fake_timing` 刻意不共用、翻頁中的結果記成 `unchanged` 與回第一頁多要求一次重排 → 不修。
  - 06、06b、07：空間模型「上層加成 2」、σ、鄰鍵先驗 0.9／σ 0.5 都沒有來源、沒調；空白鍵不在 `Weighted`；橫向鍵外的平手點 → 容錯重評時（plan §2.6 已註明「還沒在基準上調過」）；`Key::Weighted` 分支、`Key::parse` 錯誤型別、CSS px 當 1/160 吋、測試格子與 `gboard.rs` 尺寸差 → 不修。
  - 07：macOS Chrome 在 ISO 鍵盤 `Backquote`／`IntlBackslash` 可能對調、dead key 換字元後系統狀態不一定取消 → M7（寫進 plan §7.7 平台殼接實體鍵盤那條）；臨時英文照配列 → 使用者已確認（協調者 ledger）。
  - 08：觸控模擬靠近 Shift、退格的鍵出錯率偏低、一般集合不報改錯、typo／touch Data Clumps → 不修。
  - 09：只有 `z` 的空白全部 p95 超過上限 → 本票使用者決定（下面）；`Tolerance` 與 plan §0 `Correction` 命名、實驗程式重複、沒跑個人集 → 容錯重評時；brief §7 容錯開關的名稱 → 票 21、25 沿用（票 25 已引）。
  - 10：重評容錯要補的四項 → 容錯重評時（票 10 留著）。
  - 11：單字萬用鍵首選 1–2% → 本票使用者決定（下面）；觸控 `z` 鍵標「重」還是「＊」→ M5（寫進 plan §7.9）；`matched_code` 重跑推回、`has_key_with_prefix` 位置 → 不修。
  - 12：`#@/device`、`#@/tick` 夾在資料列中間也收、名詞表要不要收「列的種類」→ 票 31（v3 定稿，寫進票 31）；日期欄 parser 重複、裝置 id 不是 newtype → 不修（票 13、17 沒再加重複）。
  - 13：時鐘錯的期間上屏記成未來日子 → 不修（碰到再開票）；條目上限 2 萬／3 萬 → 本票使用者決定（下面）；`pack-update.spec.ts`「sha256 不符時拒絕」偶發紅 → 票 31（寫進票 31）；每次學習數條目 58 µs、`enforce_limits` 兩段相似、停用自動造詞的詞表匯入 → 不修。
  - 14：上限淘汰把打過一次的詞典詞當沒到門檻（`UserModel` 看不到英文詞典）→ 票 24（寫進票 24）；v3 草稿 `string` 列不轉 → 不修（v3 未發佈）；`enforce_limits` Feature Envy → 不修（票 17 沒再動它）；英文切詞 → 票 21（已收）。
  - 15：`UndoOwner::default`、單按 Shift 從嚴 → 不修（已註明）；有聯想時第一下退格只收起聯想 → 照 librime-predict 與 Gboard（plan §2.1、brief §12.2），本票使用者確認（下面）。
  - 16：簡體模式兩個原文轉成同一簡體字只清得到第一個 → M5（寫進 plan §7.9 撤銷學習手勢那條）；自動造詞第 2 位桌面清不到 → M5（plan §7.9 已有）；debug 面板開著打過字後第一次點「無痕」沒反應 → 票 31（寫進票 31）；句子候選各段不清、測試 fixture 重複 → 不修。
  - 17：合併迴圈各寫一份、三套平行表示、墓碑只在合併時回收、墓碑雜湊可字典比對、上限淘汰不留墓碑 → 不修（ADR 0007、plan §2.4 已寫）；舊覆蓋匯入同步後次數算兩次、清空後別台未同步的新學習被清 → 接受的代價，本票請使用者確認（下面）。
  - 協調者 ledger：progress.md union 合併後的重複段落 → 票 31（寫進票 31）。
- 2026-11-13 使用者試用與決定（桌面 http://localhost:5173/、手機區網 http://192.168.50.98:5173/，main `ee90dab`、資料包 v8，協調者開的伺服器）：
  - 試用：桌面與手機試過第一期（空間模型、萬用鍵、退格撤銷、英文使用者詞、撤銷學習、同步），回覆「可以開第二期」，沒有其他要改的。
  - 含萬用鍵的輸入超過每鍵延遲上限：先優化再加進閘門，上限不變 → 插修復票 18b，擋 22、23、30 與 31（19、20 是研究票，不碰核心，不擋）。
  - 容錯：暫時放棄，維持票 10 wontfix，〈容錯採用門檻〉不改（spec 已註明）；重評的條件與要補的事在 brief §4.8、票 10。
  - 萬用鍵單字首選 1–2%：不補強（brief §4.3 萬用鍵那條已改）。
  - 條目上限：維持 2 萬列／3 萬組（plan §2.4 不變）。
  - 已知代價：有聯想時第一下退格只收起聯想 → 改成第一下就刪字並收起聯想，插修復票 18c（偏離 librime-predict 與 Gboard，使用者決定），擋 30、31；舊覆蓋匯入同步後次數算兩次、清空後別台未同步的新學習被清 → 接受。
  - 臨時英文照配列：使用者已確認（協調者 ledger）。Dvorak、AZERTY 真實配列與 dead key 沒實測 → M7（plan §7.7 已補）。
  - 第二期的票：只改擋的關係（22、23、30、31）與票 24 收票 14 留下的上限淘汰問題，範圍不變。
