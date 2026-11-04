# 24: M1 收尾

Status: done
Blocked by: 01–23
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §5.2、§7；`docs/development.md`〈全部檢查〉；`docs/research/blackbox-baseline.md`；票 01–23 的 `## Comments`

## What to build

M1 的驗收（使用者任務書的階段 5）全部重驗並留下證據；黑箱測試表能在網頁做的項目在 QIM 上重跑、和競品並列；M1 封存，M2 標進行中。

- 里程碑 code-review：`/code-review <M1 開工時的 commit>`，Spec 軸讀 spec 與票 01–23，Standards 軸讀 `CLAUDE.md`。小問題修掉；沒修的與各票 Comments 裡沒處理的發現，整理成清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- 在乾淨 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑 spec〈驗收〉每一項：Playwright 實打（截圖存 `.scratch/m1/evidence/`）、`bench --check`、延遲、`data --verify`、`check` 全套。
- 黑箱：`docs/research/blackbox-baseline.md` 的 T1–T18 中能在網頁做的項目在 QIM 上重跑，照 `closed-source-imes.md` §4 的三類（缺功能、行為不同但合理、行為更差）記錄、和競品並列。
- 封存（〈寫法與封存〉，同一個 commit）：M1 的 session 紀錄與偏差移到 `docs/archive/M1.md`；里程碑表 M1 連到封存檔、M2 進行中；plan §7 表 M1 完成、M2 進行中，§7.2 指向 spec 與封存檔；本 session 的紀錄標 `M2`，下一步寫「新 session 下 `/to-tickets docs/plan.md 拆 M2 的票`」。

## 驗收

- [x] 里程碑 code-review 跑完；沒處理的發現清單寫在 `## Comments`，每條有去處或理由。
- [x] spec〈驗收〉每一項在乾淨 checkout 上成立，輸出與截圖路徑貼在 `## Comments`。
- [x] 黑箱表的 QIM 欄已補上並分類。
- [x] `docs/archive/M1.md` 有 M1 全部 session 紀錄與偏差；progress 只留 M2；里程碑表與 plan §7 已更新；同一個 commit。
- [x] 封存後 `cargo xtask check` 綠燈。

## Comments

- 2026-11-04 協調者補充（開工時）：M1 開工的 commit 是 `1cb1e17`；`bench/personal` 用軟連結接主 checkout 的個人集，baseline 已同步到 main `56fb4d3`；黑箱只在網頁重跑，不用 iPhone 或模擬器；下面清單裡標「協調者轉入」的五條由協調者指定列入。
- 2026-11-04 里程碑 code-review（`1cb1e17...56fb4d3`，Standards 軸讀 `CLAUDE.md`、plan §0–§1、`GLOSSARY.md`、spec〈共同規則〉，Spec 軸讀 spec 與票 01–23；diff 3.6 萬行，兩軸各分兩半，共四個審查）：沒有高嚴重度的發現。已處理（`4394683`）：
  - 預設佈局的刪除、Enter 標籤是「⌫」「↵」，讀屏會唸符號；給文字的無障礙標籤「刪除」「換行」。先補會紅的測試 `glyph_labelled_edit_keys_have_spoken_accessibility_labels_on_every_page`，網頁讀回 `aria-label` 成立。
  - 詞庫與語言模型資料包的 NOTICE 寫字頻是各語料 "added together"、bigram 剪到 "seen at least 2 times"，漏了語料權重；改成寫出各語料權重（由 `CORPORA` 產生），測試斷言 `hkcancor 64`。兩份 NOTICE 共用的前處理段落收成一個常數。
  - `Session::text` 沒經 `with_radicals`、判斷組字沒用 `composing()`，改成和其他公開方法一致。
  - `qim-bench` 的 `PACK` 與 `PACKS` 只差一字，改名 `QUICK_PACK`；網頁兩個值不同的 `LONG_PRESS_MS` 改名 `CANDIDATE_HINT_PRESS_MS`（500）與 `KEY_LONG_PRESS_MS`（400）；兩處過時的 doc comment。
  - plan §1.4 補 `strsim`（票 02 留下的）；plan §4.3 註明 KN 的 `n1`、`n2`、`D` 在乘權重後的次數上算；`docs/development.md`〈網頁〉寫清楚 `cargo xtask web` 何時失敗（原寫「有一個不存在就失敗」，實際只查目錄）。
- 2026-11-04 沒處理的發現（含票 01–23 Comments 留下的與協調者轉入的），依去處：
  - M2（`qim-ffi` 最小版與量測）：
    - `qim-wasm` 放了邏輯：`perform` 把動作轉成 `Key` 並切頁、`WebLayout` 記版本與目前頁、`WebSession.composing` 另存組字狀態；plan §1.2 說綁定層不放邏輯，做 `qim-ffi` 前先收進 `qim-layout` 與 `Session`（code-review、票 20）。
    - 加一種鍵要改 core、wasm 13 個方法、`keys.ts`、`main.ts` 四處；設計 FFI 介面時改成一個 `handle(kind, value)`。wasm 的 `JsError` 轉換重複約 8 次，一起收。
    - `UserModel` 每個方法都帶 `scheme`；分段有 `(String, String)`、`Segment`、`PathSegment` 三種寫法；`Pack` 只指方案資料包，和 plan §0「資料包 = `Pack`」不符（改名 `SchemePack`）。都在 FFI 介面定型前收。
    - 只有 `handle` 會先套用 `set_options` 後的選項，`select`、`focus_segment`、`text` 用舊的候選列；網頁每次 `refresh` 看不出來，殼只呼叫 `set_options` 時會錯。
    - `jsonschema` 讓 wasm 由 0.42 MB 變 4.6 MB，鍵盤擴充要不要略過 Schema 在量記憶體時定（票 19）。
    - 瀏覽器（Chromium）裡的 WebAssembly 延遲沒重跑，spec 的 50 ms 只有 Node 的數字（p95 51 µs，遠低於預算）；延遲只量字母鍵，選開頭一段時從頭重建 lattice 那一鍵沒算進 p95。M2 在手機上量真實延遲時一起量（票 05）。
  - M3（智慧候選）：
    - 選開頭一段後，剩下輸入碼的第一個字用 unigram、不接選定段的最後一字；plan §2.3 說下一段看部分路徑的最後一個字，性質測試鎖住了現狀。和輸入框上下文同一個機制，做 `set_context` 時一起改（票 21）。
    - brief §7 的「學習」總開關 M1 沒有票做，網頁只有無痕（協調者轉入）。
    - 使用者在網頁試打並按「回報」的案例，由之後的票加進 `fixed.tsv`（協調者轉入，不擋本票）。
    - 繁體模式的候選含 HKSCS 收的簡體字（黑箱 T17：`ls` 门 第 9），brief §1.3 定「繁體模式不混簡體字」沒做到；過濾還是排到最後要使用者決定，詞庫擴充時一起做。粗口字 𨳊 在 `an` 第 3，不當詞怎麼處理一起定。
    - Common Voice 因「說」「記」同碼 `yu` 多錯的題（票 17b，協調者轉入），語言模型重訓時看。KN 在乘權重的次數上算、口語幾乎不平滑（plan §4.3 已註明），同時看。
    - 重排「第 5 個提前」會把規則 3、4 排在第 2 位的使用者詞或英文擠到第 3；plan §2.1、§2.4 沒寫先後，真的重排模型進來時定。
    - 規則 1 把觸發詞釘在首選時，規則 2「選一次不成首選」會多移一次（讀程式推論、未實測）；規則 3、4 與「分段選出、詞庫沒有的整串」沒有測試（票 13、14）。排位重做時補測試再修。
    - `--set beam=0` 或 `NaN` 繞過方案檔的驗證，驗證搬到 `qim_core::Ranking` 讓兩邊共用（code-review）。
    - 資料包清單在 `Outcome`、`verify::Report`、`PackBytes`、`main.rs` 平行散開，`pipeline::run` 的建包形狀重複六次；加 manifest 時收成 `BuiltPack` 與 `write_pack`。六種資料包的 `to_bytes` / `from_bytes` 同形，一起用 trait 收（票 06、11、15、16、22）。
    - 個人集表頭、語料目錄、`dev.tsv` 路徑在 `qim-databuild` 與 `qim-bench` 各寫一份，`xtask` 加測試比對兩邊（code-review；票 04 認為依賴規則不許，但 `xtask` 不受限）。
    - 第二個方案或語言進來時：詞庫路徑寫死（票 06）、英文資料包用速成字母表過濾（票 15）。
    - 口語語料不足、要不要找新來源，問使用者（票 12）。
    - `main.ts` 631 行，網頁再長功能時把候選列與面板抽成模組（code-review）。
  - M4：容錯與空間模型（黑箱 T2）；記住 email 與欄位類型（T5）；引號配對與標點選單（票 10）；學習資料匯入的合併規則（票 14，同步）；`Key::Text` 與 `Weighted`（票 20）。
  - M5：`Warning` 的 variant / page / key 收成型別、`LayoutError::Broken` 拆開（票 19）；`swipe` 只警告還沒作用（票 19）；預設佈局「。」「，」沒有長按清單（黑箱 T13），使用者可在編輯器自己加。
  - M6 起平台殼：殼自己做按鍵對照與字根提示排版，重複太多再放進 `Snapshot`（票 22）；AltGr 組字中打不出字元（票 23）；Home / End 吃下不動作（票 23）；iOS 預設佈局放不放地球鍵（M7，票 19）；跨 App 持久化與密碼欄換鍵盤（黑箱 T14、T15）。
  - CI 接上時（建遠端要先問使用者）：`bench --check` 要不要進 `check` 一直沒定（票 02、04、07、11；現在一次約 1 分鐘，不進 `--fast`）；Node 的延遲腳本沒有單元測試（票 05）；粵語維基 XML dump 停用前換取法（票 03，plan §8 已列）；`grid.sh` 靠表格欄號解析（票 12），報表欄位改時一起改。
  - 等使用者：個人句子檔 `sentences.txt`（票 04）；簡體模式下選去重的同形字會學到排前面的繁體字（票 16）。
  - 名詞表：「語言模型」「學習資料」「標點」（`Punct`）沒進 `GLOSSARY.md`（票 10、11、14），下次 `/grill-with-docs` 時補。
  - 不修，選定的語義或代價：
    - Gboard 真機 email 學習不補測（使用者決定，協調者轉入）；iPhone 版 Gboard 不測（使用者決定，票 09）；票 08、09 缺的截圖與 T4、T7、T5c 不補，要重接裝置。
    - 簡體模式逐段轉換的已知限制（plan §2.2，協調者轉入）。
    - 子字串洩漏 222 句不加檢查，維基集只防退步（票 01）；共用快取殘留的 `.part` 不自動清（票 03）。
    - `Composition::unselected_input` / `unselected_radicals` 只有測試在用，但它們是給桌面殼的公開介面、性質測試靠它們表達「其餘重算」，不刪（code-review）。
    - 來源 id 留在 `pipeline.rs`，和各自的檔名常數相鄰，不搬 `sources.rs`（code-review）。
    - `Action::name` 這類手寫名字沒有和 serde 的往返測試：格式改名時 Schema 測試會先紅（code-review）。
    - `commit_all_then` 與 `select_at` 的重複（票 10、13）；`a2ba5b6` 兩件事同一 commit，不改歷史（票 12）；其餘型別判斷題見票 07、14、16、17、21。
- 2026-11-04 重驗在乾淨 checkout：`git worktree add --detach` 到 session 暫存目錄，`build`、`target`、`web/node_modules`、`bench/personal` 都不存在；照 `docs/development.md`〈安裝〉做 `npm ci`、`playwright install chromium`、`cargo xtask data`（共用快取，55 秒）；`bench/personal` 用 `ln -s` 接主 checkout 的個人集。先在 `56fb4d3` 全跑一次，code-review 的修正後在 `4394683` 重跑 `data --verify`、`check`、`bench --check`，數字相同：
  - `cargo xtask check`（`4394683`）：`==> fmt` … `==> unit`、`==> e2e`、`99 passed (16.0s)`、`check 通過`，exit 0。
  - `cargo xtask data --verify`：連打字集 17611 字、擴展區 57052 字，全部 `[通過]`、`verify 通過`。測試句原樣出現在 train：四份語料都是 0 句；詞的碼由字的速成碼組成：詞條 275860 個都符合；bigram 資料包 2,986,098 位元組（上限 8,000,000）；建兩次 sha256 相同：速成 `ae3bc207ce7a…`、詞庫 `20853183b2cc…`、bigram `68b18859fd32…`、字根提示 `9e755488f431…`、簡繁 `1c5ab2aa21e6…`、英文 `1afd47942b69…`，四份語料也各建兩次相同。
  - `cargo xtask bench --check`：`bench 通過`。固定題 25/25，含 `a 日`、`anr 門口`、`mir 一台`、4 句自擬粵語口語句與 4 題粵語用字。各集合分開列，答錯 / 修正：yue-wiki 1134 / 4526、zh-wiki 1137 / 4655、common-voice-yue 1044 / 4106、hkcancor 661 / 2596、個人集 1090 / 1446（加權首選正確 87.16%），都等於 baseline、沒有翻盤。
  - 每鍵延遲（Apple M6，同一輪）：原生 p50 29.08 µs、p95 39.46 µs，第 20 碼起 p95 41.38 µs；WebAssembly（Node v26.12.0）p50 38.13 µs、p95 51.12 µs，第 20 碼起 p95 54.58 µs（2574 個輸入長到 20 碼）。原生 ≤ 10 ms、WebAssembly ≤ 50 ms 都成立。
  - Playwright（`56fb4d3`，腳本只在暫存區、不進 repo，27 項 passed）。實體鍵盤：`a` 首選「日」、`anr`「門口」、`mir`「一台」，空白上屏同字；`bc` 選第 2 位「貝」兩次後「貝」成首選；`the` 第 2 位是 `the`；簡體 `anr` 首選「门口」、上屏「门口」；`anrkbof` 分段「日弓口｜大月｜人火」，← ← 到中間段後候選等於單打 `kb`，選第 2 位「冇」後分段「門口｜冇｜人火」，空白上屏「門口冇係」，等於門口＋冇＋單打 `of` 的首選。觸控（Pixel 7）：點 a、n、r 首選「門口」、空白上屏；長按 `a` 清單只有「a」、放開上屏 a；橫向（839×412）點出「門口」；系統深色時 `key-a` 底色 `rgb(60, 64, 67)`、字 `rgb(227, 227, 227)`（淺色底 `rgb(255, 255, 255)`）；符號頁 → 常用頁打出「唔該👍」、回字母頁打「日」；重疊佈局點重疊處組字「弓」（上層 n）。截圖 `.scratch/m1/evidence/` 的 `24-rime-a.png`、`24-rime-anr.png`、`24-rime-mir.png`、`24-learn-bc.png`、`24-english-the.png`、`24-simplified-anr.png`、`24-segment-focus.png`、`24-segment-rechosen.png`、`24-touch-anr.png`、`24-touch-commit.png`、`24-long-press-a.png`、`24-landscape-anr.png`、`24-dark.png`、`24-phrases.png`、`24-overlap-tap.png`。
  - `4394683` 只改了預設佈局的無障礙標籤這一項網頁行為：在乾淨 checkout 讀回刪除鍵、Enter 鍵的 `aria-label` 是「刪除」「換行」，字母鍵仍是「日，a」。
- 2026-11-04 黑箱：T1–T18 在 QIM 網頁重跑，寫進 `docs/research/blackbox-baseline.md` 的〈環境：QIM〉、〈首選對照〉的 QIM 欄與〈結果：QIM 與競品並列〉，照 §4 分三類。缺功能：容錯（T2，M4）、email 與欄位（T5，M3、M4）、上下文與聯想（T6、T8，M3）、「。」「，」長按（T13，M5）。行為不同但合理：選兩次成首選與衰減（T3）、自動造詞（T4）、英文排第 2（T7）、Enter 上屏輸入碼與 Shift＋字母（T18）、刪除不撤銷選定段（T1）、無痕代替密碼欄（T15）。行為更差：繁體模式混 HKSCS 簡體字、𨳊 排太前（T17，M3）。〈給 QIM 的取捨〉兩格照實測更正：簡繁那格原寫「繁體模式只出繁體字形」，符號頁那格原寫「票 20 決定」。截圖 `.scratch/m1/evidence/24-qim-*.png`。
- 2026-11-04 封存：M1 的 38 列 session 紀錄與 78 條偏差移到 `docs/archive/M1.md`；里程碑表 M1 連到封存檔、M2 進行中，拆票前讀加上本票 Comments；plan 開頭的狀態行、§7 表 M1 完成、M2 進行中，§7.2 指向 spec、封存檔與本票。封存後 `cargo xtask check` 全套綠燈（見 commit 前的輸出）。
