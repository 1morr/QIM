# 28b: 詞庫的詞可跨過落在字界上的分隔記號

Status: done
Blocked by: 28
讀: 票 28 全文與 Comments；brief §4.3、§4.4、§12.2（Rime 分隔符）；plan §2.1、§2.3、§2.4；spec〈共同規則〉

## What to build

M1 封存後補票的審查（2026-11-05，獨立審查）發現：票 28 讓段與詞都不跨分隔記號，理由「速成的一段就是一個字或詞，跨過去就等於沒分隔」有破綻——分隔落在詞內兩字的交界上時，詞跨過去不違背使用者要的斷點。審查查證：

- Rime 速成 rime-quick（`5dcdb9e`）`quick5.schema.yaml` 約 47、55 行用 `script_translator`、分隔符 `" ;'"`，每個字的碼當一個音節，詞可以跨分隔。
- Rime 倉頡 `cangjie5` 用 `table_translator`，詞不跨（librime `9777cb2` `table_translator.cc` 約 258、521–527、640–650 行），但它的分隔符沒有 `'`。brief 原本只查了拼音（luna_pinyin）。
- 實測：公開語料 `bench/corpus` 取 2–4 字句子、在正確字界插一個 `'`，1429 例中不分隔對、分隔後變錯的 81 例，分隔後才變對的 51 例。例：`vi'vdjg`「絃樂室」→「幾好塞」，`yk'tutktj`「文苑英華」→「放着艾草」。探測程式在 `/private/tmp/claude-501/-Users-roxy-orca-projects-QIM/4808186f-7ee6-44a4-9688-748b966c8ba8/scratchpad/review-b/probe/src/main.rs`（`corpus` 參數）。

使用者 2026-11-05 決定照 Rime 速成：**詞庫的詞可以跨過分隔記號，但分隔位置必須剛好落在該詞的字界上**（詞的每個字的碼對齊輸入碼，分隔記號只能在兩字的碼之間）；單字的碼內不可有分隔記號；分隔記號仍是「這裡一定要斷字」。

- 核心 lattice：照上面規則改（詞的邊可跨分隔，條件是分隔落在字界）；`mir` 仍是一台；`mi'r` 仍不出「一台」（m｜ir 的字界不在 `'` 的位置）；`an'r` 仍是門口；`vi'vdjg` 首選「絃樂室」（或與不分隔時相同）。
- 學習與自動造詞的碼仍去掉分隔記號（票 28 已定），選定、退格、組字顯示照舊。
- 用審查的方法重跑語料實驗（字界插 `'`），結果寫進 Comments：目標是「分隔後變錯」大幅少於 81；挑幾題進 `bench/fixed.tsv`。
- 補觸控「'詞」後退格的 Playwright（目前只有實體鍵版本）。
- 文件：brief §4.3、§4.4、§12.2 與 plan §2.3 改成新規則，§12.2 補 Rime quick5（script_translator，詞可跨）與 cangjie5（table_translator，詞不跨）的事實與來源；拿掉「比 Rime 嚴」。progress〈偏差與決定〉記一行（使用者決定，推翻票 28 的「詞不跨」），session 紀錄票欄寫「M2（補 M1 28b）」。

## 驗收

- [x] 核心測試：`mir`→一台；`mi'r` 不出一台；`an'r`→門口；分隔落在詞內字界時詞可出現（用真實資料的例子，如 `vi'vdjg`）；分隔落在單字碼內時該字不出現；退格先刪分隔記號。
- [x] 語料實驗前後數字寫進 Comments；Playwright 補觸控「'詞」後退格。
- [x] brief、plan 已更新；`cargo xtask check`、`bench --check`（數字變了 `--update-baseline` 寫理由）綠燈；/code-review 跑完。

## Comments

- 2026-11-05 做法：`Engine::exact` 收的輸入碼中間可以有分隔記號，去掉它再查詞庫與使用者詞，沒有字；`Engine::splits_at` 照碼表把詞一字一碼地對齊輸入碼（字有幾個碼時任一種拆法都算），每個分隔記號都要落在兩字的碼之間。lattice 的分隔記號一欄照舊抄前一欄，邊不從它開始，往回找邊的上限改數字母。候選列、目前分段、學習都經同一個 `span` / `edges_ending`，不必另改；學習與自動造詞的碼照舊去掉分隔記號。碼不是由字的碼組成的使用者詞（只有觸發詞的手動詞）拆不開，不跨。
- 2026-11-05 Rime 查證（現查，補進 brief §12.2）：rime-quick `5dcdb9e` `quick5.schema.yaml` 用 `script_translator`、分隔符 `" ;'"`、algebra 取倉頡首尾當音節，組字把 `;` `'` 顯示成「·」（QIM 照 Gboard 顯示 `'`，不改）；rime-cangjie `fcb3d7d` `cangjie5.schema.yaml` 用 `table_translator`、分隔符 `" ;"`；librime `9777cb2` `table_translator.cc` 的前綴查詢只在查到碼之後吃掉分隔符。
- 2026-11-05 語料實驗（審查的方法，`bench/corpus` 的 zh-wiki、yue-wiki 取 2–4 字句子，在每個正確字界插一個 `'`，無痕、空白上屏）：

  | | 例數 | 不分隔對 | 分隔後對 | 分隔後變錯 | 分隔後才變對 |
  | --- | ---: | ---: | ---: | ---: | ---: |
  | 改動前（票 28） | 1429 | 670 | 640 | 81 | 51 |
  | 改動後 | 1429 | 670 | 675 | 0 | 5 |

  分隔落在正確字界時只會刪掉字界不合的路徑，所以不分隔對的題分隔後都還對。改動前分隔後才變對的 51 題裡，46 題是巧合：分隔位置也是錯答的字界，舊規則把詞拆成單字剛好選到對的字；現在詞照樣跨過去，和不分隔時同樣錯（如 `qbypmm'oq` 要「青龍二年」，不分隔與現在都是「青龍三年」，「三年」「二年」都在 `mm`｜`oq` 斷）。分隔記號只說斷在哪、不說斷成什麼，這類題本來就要靠選字。剩下 5 題是分隔排除了字界不合的錯答（如 `m'vutjme`「一統華夏」，不分隔是「環山草酸」）。探測程式在 scratchpad，不進 repo。
- 2026-11-05 固定題加 4 題：`vi'vdjg 絃樂室`、`yk'tutktj 文苑英華`（審查的退步例）、`holcer'yb 從順治帝`（兩個詞之間）、`m'vutjme 一統華夏`（分隔後才對）；`an'r 門口` 的來源欄改寫成「門口」現在是一個跨分隔記號的詞。
- 2026-11-05 bench 前後對照（改動前 = 開工時 `bench/baseline.json`）：`bench --check` exit 0；yue-wiki、zh-wiki、common-voice-yue、hkcancor 答錯 1131 / 1127 / 1030 / 661、修正 4466 / 4541 / 4036 / 2582，個人集答錯與修正也同 baseline，五個集合都沒有翻盤：語料題沒有分隔記號，路徑不變，是預期的。固定題 75/75（原 71 題加 4 題）。p95 原生 41.67 µs、WebAssembly 50.96 µs（票 28 收尾 40.38 / 51.83）。baseline 不必更新。code-review 修正後重跑兩次，數字同上，p95 原生 45.29 / 58.17 µs、WebAssembly 52.17 / 75.38 µs：同時有其他票在跑，兩次之間 wasm 沒改也跳了 23 µs。同一時段交替量開工時與現在的核心各三輪（全部語料題逐鍵打，scratchpad 的計時程式）：p50 舊 29.7–33.6、新 30.3–37.5 µs，p95 兩邊都在 41–79 µs 間跳，看不出差別。
- 2026-11-05 測試：核心 `tests/separator.rs` 的測試資料加詞庫（「一台」「幾好」「絃樂室」「甲乙」，碼照速成碼表），加跨字界、單字碼內不跨、多碼字任一拆法、退格、學習的碼；拿掉 `splits_at` 的「分隔記號不在單字碼內」條件時 8 題紅（變異驗證）。Playwright：實體鍵盤 `vi'vdjg` 首選絃樂室、`an'r` 改成斷言「門口」是一段；觸控點「'詞」後退格先刪分隔記號、刪光後 ⇧ 變回 Shift。
- 2026-11-05 亂按（Playwright 暫存腳本，跑完刪掉）：實體鍵盤 5 個種子 × 150 步（字母、`'` 加權、退格、Esc、空白、Enter、數字、← →、Tab、↓、`,`、Shift），觸控 Pixel 7 5 個種子 × 150 步（字母、⇧ 加權、退格、空白、Enter）：沒有 page error 或 console 錯誤；中文組字裡的分隔記號不在開頭、不連續；組字中 ⇧ 都是「'詞」。第一次跑紅是腳本誤判：臨時英文裡連著兩個 `'` 是票 28 的既有行為，腳本改成臨時英文時不檢查。
- 2026-11-05 票外補的：使用者詞（手動詞、自動造詞）也照同一條件跨，同 Rime 使用者詞典的碼由音節組成；只有觸發詞、碼不是字的碼組成的拆不開，不跨。brief §4.3 寫明。
- 2026-11-05 code-review（`7a23944` 起，兩軸）已處理：`fixed.tsv` 的 `an'r` 來源欄與 `qim-bench` 測試註解還寫「段不跨」，改成新規則；`exact` / `span` 的參數照 plan §0 叫 `input`；`split_at_separators` 收成只算位置的 `separator_cuts`，碼交給既有的 `without_separators`，沒有分隔記號時走原本的 `exact_code`；`splits_at` 的變數改名；lattice 註解的連結指到 `Engine::exact`；補核心測試 `an'r` → 門口（一段）；progress 偏差與 session 紀錄、票 28 那條偏差標「票 28b 推翻」。
- 2026-11-05 code-review 沒處理：
  - 「核心測試用真實資料的例子」：核心測試的碼表是仿真的小表，但字與碼照速成碼表（絃 `vi`、樂 `vd`、室 `jg`）；真實資料包上的 `vi'vdjg`、`yk'tutktj` 由 `bench/fixed.tsv`（`bench --check` 讀 `build/packs/`）與 Playwright 驗。核心測試不讀建置產物。
  - 「去掉分隔記號的碼＋位置」包成型別：只在 `exact` 與 `splits_at` 之間傳一次，不另立型別。
