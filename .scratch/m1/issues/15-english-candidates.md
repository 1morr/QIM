# 15: 英文候選

Status: done
Blocked by: 13
讀: plan §2.4（規則 4）、§4.4；brief §4.3、§4.7、§5.2、§12.1；`scripts/experiments/english-edit1/`；spec〈共同規則〉

## What to build

中文狀態下，輸入碼剛好是完整英文單字時，第 2 位出現那個英文字；不容錯、不補全，不搶首選。

- SCOWL / ESDB（`en-wl/wordlist`）size ≤ 60 的詞釘進 `sources.toml`，建成英文資料包（自己的 `LICENSE`、`NOTICE`）；只收能用 a–y 打出的字（`z` 不在速成字母表）。大小寫怎麼處理在本票定，參考黑箱 T7（已做完的話）。
- 排位規則 4：放第 2 位；和規則 1–3 的先後照 plan §2.4。
- `Options.english_candidates` 開關（預設開），網頁加開關。
- bench 前後對照：英文候選只佔第 2 位，首選正確率不應改變；有改變就查明。

## 驗收

- [x] 核心測試：`the` 第 2 位是 `the`；非完整單字（如拼錯一碼）不出英文候選；關掉選項不出。
- [x] `--verify` 報英文詞數；建兩次 sha256 相同。
- [x] Playwright：中文狀態打 `the`，讀回第 2 位是 `the`，按 `2` 上屏 `the`。
- [x] bench 前後對照貼在 `## Comments`。

## Comments

### 決定（2026-11-04，實作時）

- **資料來源**：ESDB release `rel-2026.02.25` 的 `hunspell-en_US` zip（README：`mk-list --accents=strip en_US 60`，即官方 size 60 美式預設字典），`qim-databuild` 照 `en_US.aff` 展開 `en_US.dic`（`crates/qim-databuild/src/english.rs`）。不從 repo 的 `data/` 自己組：要跑 ESDB 的 Python + SQLite 工具（`combine.py` 合併十幾份檔），建置會多一種語言的依賴。代價是沒有逐詞的 size，plan §4.4 刪掉「size 級別當常見度」；英文候選只看是否完整單字，用不到。
- **禁忌詞**：`.dic` 標 NOSUGGEST 的 27 個詞幹整個不收（票沒寫，中文狀態冒出這些字不妥）。
- **大小寫**（參考黑箱 T7）：鍵是小寫；同一個小寫拼法有幾種時取全小寫（`may`），沒有才取大寫（`english` → English、`nasa` → NASA）。macOS 對 `ipad` 列 iPad、ipad 兩個，QIM 只佔第 2 位一格，有意只給一個；要小寫原樣可按 Enter。
- **長度**：輸入碼長過一個字的最長碼才出（速成 3 碼起）。1–2 碼的英文字（`an`、`it`、`a`）會常佔單字碼的第 2 位；macOS 速成也是第 3 碼起（T7「第 3 碼起有英文候選」）。
- **範圍**：只給剩下的整串輸入碼，目前分段的候選列沒有；沒有任何中文候選時不出（不搶首選）；第 2 位已被規則 1–3 排定時放它後面最前的空位（`pin_at_or_after`，不用 `pin_at_most`：後者會往前移到首選）。
- **選了不學**：和 Enter 一樣選定的分段上屏字詞、其餘上屏英文字，整次不學、不造詞。學的話照規則 2 選兩次就成首選，違反「不搶首選」。
- **資料包**：`scowl-en-us`（魔數 `QIME`），不由方案指名、`Engine::with_english` 掛上，同簡繁。只收轉成小寫後全是 a–y 的詞：85,959 詞（詞幹 47,551、展開 109,863 個詞形、打不出 22,384 個）、796 KB；WebAssembly 載入多約 10 ms。讀回時重新排序整理，不靠資料包的順序。
- 本機沒有 hunspell / unmunch，展開結果沒和官方工具逐詞比對；靠單元測試照 `en_US.aff` 的實際規則（`D`、`G`、`S`、`U`、交叉、條件）與 `--verify` 的 `the` 抽查。

### bench 前後對照（`cargo xtask bench`；前＝沒掛英文資料包，後＝掛上、預設開）

| 集合 | 整句首選正確 前 → 後 | 答錯 前 → 後 | 修正 前 → 後 | KSPC |
| --- | --- | --- | --- | --- |
| fixed | 19/19 → 19/19 | 0 → 0 | 0 → 0 | 2.35 → 2.35 |
| yue-wiki | 869 → 869 | 1131 → 1131 | 4400 → 4400 | 2.23 → 2.23 |
| zh-wiki | 858 → 858 | 1142 → 1142 | 4689 → 4689 | 2.23 → 2.23 |
| common-voice-yue | 606 → 606 | 1032 → 1032 | 4101 → 4101 | 2.31 → 2.31 |
| hkcancor | 321 → 321 | 671 → 671 | 2636 → 2637 | 2.39 → 2.39 |
| personal | 3421 → 3421 | 1096 → 1096 | 1492 → 1493 | 2.50 → 2.50 |

- 首選正確率、依長度分組、翻盤清單都不變（沒有翻盤）。
- 修正 +1 的原因（逐題比對開關英文的結果）：hkcancor 第 646 題 `al of rg or bg be rn`（即係哩個雞販喇），選到最後剩下 `bern`，是英文專有名詞 Bern，佔第 2 位把原本第 9 位的候選擠到第 2 頁，多翻一頁。個人集第 3944 題同樣是剩下的碼尾段是英文字、多翻一頁（內容不記）。這是英文佔第 2 位的必然代價，`--update-baseline` 接受。
- 每鍵 p95（µs）：原生 39.25 → 39.79、WebAssembly 51.46 → 51.79（全部輸入），只報不擋。

### 驗收與亂按

- 核心測試在 `crates/qim-core/tests/english.rs`（18 題），規則 4 換回 `pin_at_most`、分段也給英文、選英文會學、拿掉「沒有中文候選不出」與選項比對，各自讓對應的測試紅過。
- Playwright `web/e2e/english.spec.ts`：中文狀態打 `the`，讀回第 2 位 `2the`，按 `2` 上屏 `the`；截圖 `.scratch/m1/evidence/15-english-the.png`。
- 亂按：網頁上用腳本送 300 輪隨機按鍵（英文字、`thw`、`athe`、2、空白、Esc、Backspace、← →、翻頁、標點、Enter、Tab、`z`、大寫）並隨機切英文候選與簡體開關，每輪檢查：沒有例外、沒有組字時沒有候選、英文從不在首選、關掉時沒有英文、不會有兩個英文候選。都沒有違反；選過幾次 `the` 後它仍在第 2 位。

### code-review 未處理的發現

- 多一種資料包要改 pipeline、verify、packing、main、wasm、bench 六處（Shotgun Surgery），`Outcome`、`Report` 的 sha256 用位置欄位：沿用既有結構，留給里程碑收尾（票 24）判斷要不要收成一張資料包清單。
- 英文資料包標「各方案共用」，卻用速成方案的字母表過濾：目前各方案都是 a–y，加了字母表不同的方案時再改成各方案自己過濾。
- `lattice::candidates(…, false)` 的布林參數在呼叫點讀不出意思：只有三處，參數名 `english_candidates` 與註解已說明，暫不改型別。
