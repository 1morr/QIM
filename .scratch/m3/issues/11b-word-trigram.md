# 11b: 同步層：詞級 trigram 進核心

Status: done
Blocked by: 11
讀: `docs/research/model-ladder.md`〈同步層〉〈實驗用的核心介面〉；`scripts/experiments/model-ladder/README.md`〈同步層〉與 `src/sync.rs`；plan §2.1（實驗的語言模型那條）、§2.3、§4.3、§4.5、§4.6；brief §9、§11；`docs/research/ios-feasibility.md`〈鍵盤擴充接上核心〉〈記憶體上限〉；spec〈共同規則〉

## What to build

把票 08 量過的詞級 trigram 做成正式的資料包與核心打分，取代實驗用的 `with_segment_model`。

- `qim-databuild` 建詞級 trigram 資料包，照 `word3:8:8` 的做法：段由現行打分切訓練句、各語料各算折扣（plan §4.3 的標準 KN）、α 夾在 1 以下；進 `build_packs`、manifest（plan §4.6）、`cargo xtask data --verify` 的決定性檢查；`LICENSE`、`NOTICE` 寫四份語料與 rime-essay（LGPL-3.0）。方案檔的 `[packs]` 指名它。
- `qim-core` 載入它，lattice 的邊與整串字詞的靜態機率改用它打分，沒有統計的地方退回字級 bigram（同實驗的接法）；做成核心內建的打分，不經 `dyn` 介面。做完刪 `Engine::with_segment_model`、`SegmentModel`、`SegmentQuery` 與 `tests/segment_model.rs`，`model-ladder` README 註明 `sync` 只在刪之前的 commit 建得起來。
- 放寬版：剪枝參數照 `word3:2:4` 另建一份，用 `ios-keyboard` 實驗的量法在 iPhone 量模型的 footprint 峰值增量：核心加現有資料包（79 MB，含字級 bigram）載入後，再載入放寬版多出的峰值。這個增量連 1.4M 的 +2 MB ≤ 60 MB（spec〈採用門檻〉的模型上限）就改成預設用放寬版，超過就留預算版（預算版也照樣量一次當對照）；結果寫進 `docs/research/ios-feasibility.md` 與 brief §12.3。要接 iPhone 時停下問使用者。
- 網頁載入新的資料包；`cargo xtask bench` 的每鍵延遲（原生與 WebAssembly）和載入時間對 spec〈採用門檻〉同步層的成本那條。
- 寫回 plan §2.1、§2.3、§4.3、§4.6、brief §9（核心加資料包的記憶體）。

## 驗收

- [x] 詞級 trigram 資料包由 `cargo xtask data` 建出，`--verify` 通過決定性檢查，有 manifest、`LICENSE`、`NOTICE`。
- [x] `cargo xtask bench --check` 的前後對照附在 Comments：口語首選與修正次數比開工時的基線仍過 spec〈採用門檻〉同步層那條，個人集與維基不退步；`--update-baseline` 的理由寫在同一個 commit（個人集 baseline 是共用的本機檔，更新前問使用者）。
- [x] `with_segment_model` 一系列與它的測試已刪；plan §2.1 同步改寫。
- [x] 放寬版的 iPhone 峰值與預設用哪一版寫進 plan §4.3、`docs/research/ios-feasibility.md`、brief §12.3。
- [x] 每鍵延遲原生 p95 ≤ 1 ms、選開頭一段 p95 ≤ 5 ms、載入 ≤ 300 ms（這台 Mac）。
- [x] Playwright 在網頁打一句詞級 trigram 才選對的句子、讀回首選；`cargo xtask check` 綠燈。

## Comments

### 2026-11-11 實作（開工 `a6b4550`）

**做法與取捨**

- 新的資料包種類 `word-trigram`（`WordTrigramPack`，魔數 `QIMW`），方案 `[packs]` 的 `word_trigram` 指名預設那份。`Engine::with_word_trigram` 掛上，`Engine::open` 照方案要不要核對（`WordTrigramMismatch`），同詞庫、語言模型只核對有沒有、不核對 id；建出的是不是指名那份由 `--verify` 擋。沒改 `Engine::new` 的參數：三十幾個測試與殼都呼叫它，詞級 trigram 只有 `open` 與 `qim-bench`、`qim-ffi` 要給。manifest 加種類照 plan §4.6 不加 `manifest_version`。
- `Scheme.packs` 多一欄，方案資料包的 postcard 格式變了：`FORMAT_VERSION` 6 → 7，所有資料包的 sha256 都換、`web/e2e/fixtures/` 照測試訊息重產。
- 核心打分照實驗的 `WordNgram`：前一段不在詞表 → 現行打分；留下的 trigram 直接用，否則 α 加詞級 bigram，詞級 bigram 再退回現行打分。兩份都建（預算 8:8、放寬 2:4），都進網頁的內建清單，預設是方案指名的預算版。
- 詞表照建置端第一次出現的順序編號（同實驗）：先照文字排序重編過一次，預算版大 0.35 MB（5.17 MB，加 bigram 8.19 MB 超過 8 MB）。查文字用載入時建的開放定址雜湊表（FNV-1a，不加依賴）：先用依文字排序的二分搜尋，同一程序比每鍵 p50 31 → 64 µs、p95 43 → 88 µs，換雜湊後 45 / 62 µs、選開頭一段 0.75 → 1.20 ms。剩下的成本是每條邊與每條部分路徑查兩階的表。
- 建置：`qim-databuild` 的 `word_trigram.rs` 是實驗 `sync.rs` 詞級部分搬過來（只留各語料各算折扣），加 `rustc-hash`（plan §1.4）。資料包大小與實驗一致（4.82 MB、14.56 MB），bench 的口語數字也與票 08 test 一致。`cargo xtask data` 45 → 75 秒、峰值約 5 GB；`--verify` 80 → 130 秒。
- `cargo xtask bench` 原本不量載入，加一行「載入」：原生是 `load_engine` 解開加建 `Engine`（不含讀檔），WebAssembly 是 `new Session(packs)`（`latency.ts` 在輸出第一行 ` | ` 後帶回，Swift 的計時程式不給也照讀）。`load_engine` 改回傳 `LoadedEngine`。
- `model-ladder` 的 `sync`（`src/sync.rs`、`src/sync_cli.rs`、`tests/sync.rs`）刪掉，其餘照樣建得起來（票 11d 要用 `eval` 在新基線重量）；README 註明要重跑票 08 的數字 checkout `a6b4550`。
- 網頁的 e2e：粵典詞庫那條原本用「襟食」（`lfov`）證明預設詞庫打不出，詞級 trigram 學到「襟」「食」相連後預設也排得出，換成同樣只在粵典詞頭表、預設照樣打不出的「暗寸」（`aadi`）。

**`cargo xtask bench --check` 前後對照**（開工 `a6b4550` 與完成後，同一台 Mac）

| 集合 | 首選（前） | 首選（後） | 修正（前） | 修正（後） |
| --- | ---: | ---: | ---: | ---: |
| common-voice-yue | 619（37.54%） | 757（45.91%） | 3908 | 3249 |
| hkcancor | 339（34.28%） | 417（42.16%） | 2478 | 2148 |
| 口語合計（2,638 題） | 958（36.32%） | 1174（44.50%） | 6386（2.421/句） | 5397（2.046/句） |
| yue-wiki | 877（43.85%） | 1059（52.95%） | 4295 | 3239 |
| zh-wiki | 874（43.70%） | 1021（51.05%） | 4381 | 3493 |
| 有前文四語料合計·給 / 不給 | 1245 / 1232 對 | 1501 / 1457 對 | 6802 / 6868 | 5573 / 5651 |
| personal（4,517 題） | 3435（76.05%） | 3434（76.02%） | 1469 | 1463 |

- spec〈採用門檻〉同步層：口語首選 +8.2 點（≥ +2.0）、修正 −15.5%（≥ −3%）；口語翻好／翻壞 289 / 73（common-voice-yue 179 / 41、hkcancor 110 / 32），同票 08 的 p = 1.6e-31。個人集首選 −0.03 點（容許 −0.5）、修正 −0.4%（容許 +2%）；維基首選 +9.1、+7.4 點。前 2 名（`model-ladder eval --split test`，`load_engine` 掛著詞級 trigram）：個人 85.5% → 85.8%（容許 −0.5）、維基 55.1% → 63.4%（容許 −1.0）、口語 49.1% → 56.5%，基線的數字取 spec〈採用門檻〉。「有前文」給前文仍比不給好（答錯 1524 對 1568、修正 5573 對 5651）。
- `--update-baseline` 的理由：換成詞級 trigram 是票 11 採用的同步層模型，公開集合全部變好；`--check` 只擋在個人集答錯 1082 → 1083（+1）。使用者這輪的訊息（轉達協調者）同意這段時間更新共用的個人集 baseline，條件是列出翻盤題號與理由。個人集翻盤（行號）：
  - 錯 → 對 26 題：29、213、518、652、822、941、1112、1362、1491、1544、1867、1935、2149、2150、2170、2212、2242、2693、2778、3033、3123、3421、3573、3950、4057、4166
  - 對 → 錯 27 題：161、483、714、716、754、786、1005、1098、1311、1564、1596、1692、1868、2050、2071、2213、2291、2344、2381、2613、2656、2738、3712、4268、4282、4305、4346
  - 與票 08 的 26 / 27 相同：個人集多是兩字詞、整題是一段，詞級的增益在段與段之間，翻好翻壞相抵。

**成本（spec〈採用門檻〉同步層的成本）**：這台 Mac，load average 約 7（其他 session 在跑），`cargo xtask bench --update-baseline` 那次：原生每鍵 p95 67 µs（≤ 1 ms）、選開頭一段 p95 1.24 ms（≤ 5 ms）、載入 148 ms（≤ 300 ms）；WebAssembly 79 µs、1.49 ms、202 ms。開工時原生 45 µs、0.76 ms。預算版資料包加 bigram 7.84 MB（≤ 8 MB）。

**iPhone（放寬版要不要當預設）**：使用者同意後裝上 `ios-keyboard` 實驗、用 iphone-use 在備忘錄量，各一個新程序。六份資料包載入後 79.4 MB；放寬版留下 +23.4 MB、峰值 +71.3 MB（到 150.7 MB，收到兩次記憶體警告），超過 60 MB → **預設留預算版**；預算版留下 +5.6 MB、峰值 +20.4 MB。峰值多出的部分推論是解開時的暫存；改成就地解開後可以再量一次。寫進 `docs/research/ios-feasibility.md`〈詞級 trigram 的記憶體〉、brief §9、§12.3、§12.5、plan §2.7、§4.3。截圖 `.scratch/m3/evidence/11b-iphone-word-trigram{-large,}.png`。

**驗證**：`cargo xtask data --verify` 通過（詞級 trigram 預算版加 bigram 7,842,947 位元組、十份資料包建兩次 sha256 相同、manifest 10 份）；Playwright `web/e2e/word-trigram.spec.ts` 打 `ofrdgihnmyhn` 首選「你哋去行下街」、空白上屏（只有字級 bigram 時首選「你哋去行下行」），截圖 `.scratch/m3/evidence/11b-web-word-trigram.png`。

**code-review（`a6b4550` 起，Standards、Spec 兩軸）**

- Spec：沒有實作錯誤。指出的缺口已補：前 2 名的前後對照（上面）、個人集 baseline 的同意來自使用者的訊息（上面）。
- Standards：沒有違反文件規則的；以下是判斷題。
  - 已改：databuild 的 `WORD_BITS` 改由核心的 `WORD_TRIGRAM_MAX_WORDS` 推出、詞表上限用它；`Summary.pruning` 不再是 `Option`，`train` 收 id、不由呼叫端事後填；pipeline 帶著 `Variant` 走，不再查兩次 `VARIANTS`；`load_engine` 的計時改成泛型的 `timed`；`packs.spec.ts` 的列數收成 `BUILTIN_ROWS`。
  - 不改：databuild 與核心各有一份 `Table`（建置端要 `build` 與剪枝、量化後的 bigram 當 trigram 的低一階，核心只查表；共用要讓核心公開建表的內部）；`WordTrigram::log_prob` 用文字查（兩個 crate 的測試用，`Engine` 走編號的 `score`）；`qim-ffi` 的 `Engine::new` 多一個位置參數、`FORMAT_VERSION` 7（`qim-ffi` 只有 repo 裡的實驗在用、資料包還沒發佈過，同票 19 升 v6 的做法），改成照 manifest 的 `open` 留給 M6 起接資料包管理的平台票（plan §4.6 已寫）。
  - 範圍：網頁把放寬版也列為內建（每次載入多抓 14.56 MB，只在本機跑）：和粵典詞庫一樣在設定區可切換，給使用者在網頁比兩版；要省的話把它移出 `BUILTIN_IDS`。
- `cargo xtask check` 第一次跑時 e2e 紅兩條，都是詞級 trigram 改了排序、測試的例子不再成立，不是程式錯：`context.spec.ts` 的 `wrht` 句首與不知道前文都是「回血」了，換成 `hi`（句首「我」、不知道前文「的」）；`composition-keys.spec.ts` 按 ← 後目前分段的候選照「不知道前文時單打它的碼」（plan §2.1），和接著句首的整串候選第 5 名起不同序了，改成只比首選。
