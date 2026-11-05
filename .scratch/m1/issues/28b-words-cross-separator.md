# 28b: 詞庫的詞可跨過落在字界上的分隔記號

Status: ready-for-agent
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

- [ ] 核心測試：`mir`→一台；`mi'r` 不出一台；`an'r`→門口；分隔落在詞內字界時詞可出現（用真實資料的例子，如 `vi'vdjg`）；分隔落在單字碼內時該字不出現；退格先刪分隔記號。
- [ ] 語料實驗前後數字寫進 Comments；Playwright 補觸控「'詞」後退格。
- [ ] brief、plan 已更新；`cargo xtask check`、`bench --check`（數字變了 `--update-baseline` 寫理由）綠燈；/code-review 跑完。
