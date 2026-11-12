# 開發

開發指令的單一來源。新增或改動開發指令的票，同輪更新本檔。

## 環境需求

| 工具 | 版本 | 說明 |
| --- | --- | --- |
| rustup | 任一近期版本 | Rust 版本由 `rust-toolchain.toml` 釘住，第一次在 repo 內跑 `cargo` 時 rustup 自動安裝，含 `rustfmt`、`clippy` 與 `wasm32-unknown-unknown` target |
| `wasm-bindgen-cli` | 與 `Cargo.lock` 的 `wasm-bindgen` 相同（0.2.130） | 產生網頁的 JS 綁定；版本不同時 `cargo xtask web` 與 `check` 的 `wasm` 步驟會擋下並印安裝指令 |
| Node.js | 26 LTS（開工時 26.12.0） | 附帶的 `npm` 管理 `web/` 的依賴 |
| git | 任一近期版本 | hook 用 `core.hooksPath`，Windows 上由 Git for Windows 的 `sh` 執行 |
| Xcode | 27.1 起（含 `swiftc`） | 只有 Apple 平台的 Swift 綁定要（〈Swift 綁定〉）；`check` 不用 |

## 安裝

在 repo 根目錄：

```sh
npm ci --prefix web                    # 依 web/package-lock.json 裝網頁依賴
(cd web && npx playwright install chromium)   # Playwright 用的瀏覽器
cargo install wasm-bindgen-cli --version 0.2.130 --locked
git config core.hooksPath .githooks    # 啟用 pre-commit 與 commit-msg hook
cargo xtask data                       # 建資料包（第一次要下載來源），e2e 要用
cargo xtask check                      # 確認環境就緒
```

macOS 上 `npm ci` 會提示 `fsevents` 的 install script 未核准：它附預編譯檔，不必核准。

## 建置

| 指令 | 作用 |
| --- | --- |
| `cargo build` | 建置全部 Rust crate |
| `cargo xtask web` | 編 WebAssembly 綁定、複製資料包到網頁，見〈網頁〉 |
| `cargo xtask ffi [--ios]` | 編 `qim-ffi` 的靜態函式庫並產生 Swift 綁定；`--ios` 另包 iPhone 的 xcframework，見〈Swift 綁定〉 |
| `npm run --prefix web dev` | 網頁開發伺服器；先跑 `cargo xtask web` |
| `npm test --prefix web` | 網頁的單元測試（`web/test/` 下的 `*.test.ts`，不必先跑 `cargo xtask web`） |
| `npm run --prefix web e2e` | Playwright 實打網頁，自己起開發伺服器與扮演 release 的本機靜態伺服器（〈網頁〉）；先跑 `cargo xtask web`。埠由 `web/` 的絕對路徑算出（20000–29999），每個 worktree 固定一個、並行不互搶；環境變數 `QIM_E2E_PORT` 可指定。埠被佔用時直接失敗、不接手別人的伺服器（可能是別的 worktree 的程式碼）：關掉佔用的行程，或兩個 worktree 剛好算到同一個埠時設 `QIM_E2E_PORT` |
| `npm run --prefix web build` | 型別檢查後打包網頁到 `web/dist/` |
| `cargo xtask data [--verify]` | 建資料包到 `build/packs/`、預設佈局到 `build/layouts/`，見〈資料建置〉 |
| `cargo xtask pack-release [--previous <index.json>]` | 把 `build/packs/` 整理成線上更新的 release 目錄 `build/pack-release/`，見〈線上更新的 release〉 |
| `cargo xtask neural-import <訓練目錄> --lambda <λ>` | 把訓練好的停鍵層模型核對、量化後匯入 `data/neural-rerank/`，見〈停鍵層的模型〉 |
| `cargo xtask bench [--lexicon <id>] [--check \| --update-baseline \| --learning \| --dev [--set <鍵>=<值>]…]` | 首選字基準，見〈首選字基準〉；`--dev` 調參用，見〈dev 切分調參〉；`--lexicon` 換一份詞庫 |

## 資料建置

```sh
cargo xtask data            # 建資料包
cargo xtask data --verify   # 建資料包並跑 plan §4.5 的檢查，有一項失敗就 exit 非零
```

`cargo xtask data` 用 release 編 `qim-databuild` 並在 repo 根目錄執行，步驟見 plan §4：

- 來源列在 `data/sources.toml`：每個來源釘 `version`（git 來源填 commit，其他填上游版本號）與每個檔的 sha256。下載到 `<快取>/sources/<來源>/<sha256 前 16 碼>/<檔案路徑>`；已有快取就不連網，下載是邊收邊寫檔，不整份讀進記憶體。快取或下載的檔 sha256 不符時建置失敗、不重新下載：確認來源後刪掉那個檔重跑，或更新 `sources.toml`。例外是只進可選資料包的粵典（`[source.words_hk]`，網址沒有版本、詞典一更新 sha256 就對不上）：它 sha256 不符或下載失敗時只略過粵典詞庫 `quick-lexicon-words-hk`、刪掉上次建出的那份，其他資料包照建，stderr 印「警告：略過 quick-lexicon-words-hk：…」與重釘步驟，結束碼仍是 0；`--verify` 的報告也列出略過的那份，不算失敗。網頁沒有這份時照樣載入，選過它的改用預設詞庫。換來源版本時改 version、url 與 sha256，快取路徑跟著換。
- `<快取>` 在 repo 外，所有 worktree 共用，新 worktree 不必重下約 360 MB 的來源：環境變數 `QIM_CACHE_DIR` 有設就用它，否則是平台的使用者快取目錄下的 `qim/`（macOS `~/Library/Caches/qim`、Linux `$XDG_CACHE_HOME/qim` 或 `~/.cache/qim`、Windows `%LOCALAPPDATA%\qim`）。路徑含釘住的 sha256，不同 worktree 釘不同版本也不會互相蓋掉。下載中被中斷時會留下 `<檔名>.part<行程 id>`，可以直接刪。
- 資料進資料包的來源要寫 `license_text`，三選一，沒寫時建置失敗：
  - `statement`（`file`、`quote`）：照抄上游檔裡的授權聲明，建置時核對 `quote` 確實在 `file` 裡（空白與換行不計），再寫進資料包的 `LICENSE`（libcangjie）。`file` 要是這個來源 `files` 之一。
  - `upstream_file`：上游釘住 sha256 的授權檔（`files` 之一）整份寫進 `LICENSE`（rime-essay 的 `LICENSE`）。
  - `repo_file`：repo 裡的授權全文檔，整份寫進 `LICENSE`。上游授權檔沒有版本化網址、會改內容時用，例如 Unicode 的 `license.txt` 每年改版權年份，釘 sha256 會讓乾淨建置過期失敗；全文放 `data/licenses/`（Unihan 用 `Unicode-3.0.txt`；語料的字頻進詞庫資料包、bigram 進語言模型資料包，用 Creative Commons 的三份全文）。
- 方案檔在 `data/schemes/`，由 `qim-databuild` 編進資料包；`qim-core` 不讀 TOML。
- 產物：`build/packs/<id>/` 下的 `<id>.pack`、`manifest.json`（id、種類、版本、最低核心版本、大小、sha256、授權，plan §4.6）、`LICENSE`、`NOTICE`，每個資料包只寫它用到的來源（plan §4）。資料包的內容改了，就在 `pipeline.rs` 的 `build_packs` 加那份的 `version`。速成資料包的 `<id>` 是方案 id（`quick`），碼表的簡體字標記由 OpenCC 的字典判定（plan §4.1，`crates/qim-databuild/src/simplified.rs`），所以授權除了 libcangjie、Unihan 還有 OpenCC；詞庫資料包的 `<id>` 是方案檔 `[packs]` 的 `lexicon`（`quick-lexicon`），內容是 rime-essay 與 repo 裡的粵語口語詞表 `data/lexicon/yue-colloquial.tsv` 的詞與詞頻、HKCanCor train 切分裡的詞（已分詞，`crates/qim-databuild/src/hkcancor.rs` 的 `TrainWords`），加上各語料 train 切分乘上語料權重相加的字頻（plan §4.2），在語料之後建；詞表不經 `sources.toml`，`NOTICE` 記它這次內容的 sha256，授權是 `data/licenses/CC0-1.0.txt`；另一份詞庫 `quick-lexicon-words-hk` 是同樣的詞再補粵典的詞頭（`crates/qim-databuild/src/words_hk.rs`，plan §4.2），可選、不是預設，授權多一段粵典的非商業條款（`data/licenses/words-hk-NCODL-1.0.txt`）；語言模型資料包的 `<id>` 是 `[packs]` 的 `language_model`（`zh-hant-bigram`），是由各語料的 `train.txt` 訓練的字級 bigram（plan §4.3，訓練與剪枝在 `crates/qim-databuild/src/bigram.rs`，語料權重在 `corpora.rs`），也在語料之後建；字根提示資料包的 `<id>` 是 `[packs]` 的 `radical_hint`（`cangjie-hint`），是 libcangjie 每個漢字的倉頡三代 / 五代碼（plan §4.1 步驟 5，`crates/qim-databuild/src/radical_hint.rs`），授權只有 libcangjie，`Engine` 不載入它，由殼另外載入顯示（ADR 0003）；簡繁資料包 `opencc-hk2s` 不由方案指名、各方案共用，是 OpenCC 的 `hk2s` 轉換表（plan §4.4，讀字典的規則在 `crates/qim-databuild/src/opencc.rs`），由 `Engine::with_simplifier` 掛上；英文資料包 `scowl-en-us` 也不由方案指名、各方案共用，是 ESDB release 的 Hunspell en_US 字典展開後速成打得出的詞（plan §4.4，展開與篩選的規則在 `crates/qim-databuild/src/english.rs`），由 `Engine::with_english` 掛上；emoji 資料包 `cldr-emoji-yue-hant` 也不由方案指名、各方案共用，是 CLDR 粵語與繁中標註的「字詞 → emoji」（plan §4.4，規則與關鍵字門檻在 `crates/qim-databuild/src/emoji.rs`，門檻怎麼定見 `scripts/experiments/emoji-keywords/`），由 `Engine::with_emoji` 掛上；停鍵層的模型資料包 `zh-hant-neural-rerank`（種類 `model`）不由方案指名，是 `data/neural-rerank/` 裡採用的 1.4M 神經模型原樣包起來（q8 權重、字表、λ，`crates/qim-databuild/src/neural.rs`，見〈停鍵層的模型〉），授權是訓練語料的，`Engine` 不載入它，由殼交給 `qim-neural`（plan §2.7）。九種資料包的格式都定義在 `crates/qim-core/src/pack.rs`（各自實作 `Pack` trait：魔數不同、共用 `FORMAT_VERSION`），改格式時加 `FORMAT_VERSION`。建哪些資料包只列在 `crates/qim-databuild/src/pipeline.rs` 的 `build_packs`：加一種資料包就在那裡加一項（建法、授權來源、`NOTICE` 的說明），寫檔、印路徑與 sha256、`--verify` 的決定性比對都跟著這份清單走；要它解得開、接得上核心的抽查另加在 `verify.rs`。
- 預設佈局：`build/layouts/gboard-quick.json`（設定的「數字列」打開時改讀 `gboard-quick-number-row.json`，字母頁上方多一排數字），Gboard 速成的手機直向與橫向，各有字母、符號、常用（自訂頁面的示範）三頁，由 `crates/qim-databuild/src/gboard.rs` 依按鍵排列算出（plan §2.6），格式 `qim-layout/1` 的型別與 JSON Schema 在 `crates/qim-layout`（`schema/qim-layout-1.schema.json`）。不讀來源、沒有授權檔。內建佈局的嚴格檢查（plan §5.3）是 `crates/qim-databuild/tests/gboard.rs` 的測試，跟著 `cargo xtask check` 的 `test` 跑；網頁載入佈局時把驗證的警告印在瀏覽器 console。
- 語料（plan §4.3）：清單在 `crates/qim-databuild/src/corpora.rs`。維基（粵語 `yue-wiki`、中文 `zh-wiki`）與口語（`hkcancor`、`common-voice-yue`）各自抽出文字，轉成香港繁體（中文維基 `s2hk`，其餘 `t2hk`）再換回常見寫法（字形對照表，plan §4.3）、切句、切分，寫出基準檔、「有前文」的題（各目錄下的 `context/<名>.tsv`）與 `ATTRIBUTION.md`（進 git：維基在 `bench/corpus/`，口語在 `bench/colloquial/`）與 `build/corpus/<名>/` 下的 `train.txt`、`dev.txt`（一行一句）、`dev.tsv`（dev 切分的調參題，取法與格式同基準檔，見〈首選字基準〉的〈dev 切分調參〉）、`dev-context.tsv`（dev 切分的「有前文」調參題，取法與格式同 `context/` 下的題）、`test-context-long.tsv` 與 `dev-context-long.tsv`（test、dev「有前文」的同一批題，前文換成同一段裡前面最多 200 字，模型階梯用，`scripts/experiments/model-ladder/`）、只有 `hkcancor` 有的 `test-dialogue.tsv` 與 `dev-dialogue.tsv`（口語長前文：test、dev 的句子配同一段對話裡前面的話輪，最多 200 字，模型階梯用）、`chars.tsv`（字頻，見〈語料〉），`build/` 下的都不進 git。第一次要下載約 360 MB 的來源；之後 `cargo xtask data` 約 75 秒（其中詞級 trigram 切段、訓練約 30 秒，峰值記憶體約 5 GB），`--verify` 把語料再建一次、bigram 與詞級 trigram 再訓練一次比 sha256，約 130 秒（Apple M6，release）。

`cargo xtask data` 每次都印各資料包的路徑與 sha256；`--verify` 的報告另列出連打字集字數、擴展區字數、速成碼個數、詞庫的詞數與被門檻和字集濾掉的數量、HKCanCor 補上的詞數與對不上而略過的話輪數、粵典詞庫的詞頭數與補上的詞數、有字頻的字數、bigram 的字數、剪枝前後的相鄰字對數與句首的字數、英文的詞幹數、禁忌詞數、詞形數、打不出而不收的數量與收的詞數、Unihan 交叉比對的摘要、各語料的句數，再逐項列 `[通過]` 或 `[失敗]`，最後一行 `verify 通過` 或 `verify 失敗`。檢查定義在 `crates/qim-databuild/src/verify.rs`；字序（`a` 的首選）、`anr` 的「門口」與繁體模式 `ls` 的「门」排在繁體字之後經 `qim-core` 的 `Engine` 查，簡體模式的「门口」與中文狀態 `the` 的英文候選經 `Session` 打，與網頁同一條路。

### 語料

`bench/corpus/<名>.tsv`（維基）與 `bench/colloquial/<名>.tsv`（口語）是首選字基準的題目，由建置產生、不要手改。UTF-8，沒有欄名，一行一句：

```text
<逐字速成碼，空格分隔><TAB><句子>
```

例如 `an r⇥門口`（`⇥` 是 tab）。同目錄下 `context/<名>.tsv` 是「有前文」的題，一行「前文<TAB>逐字速成碼<TAB>句子」：test 句配上來源同一行裡它前面的兩句與中間的標點（原樣，可含英文、數字，`corpus.rs` 的 `CONTEXT_CLAUSES`），取雜湊最小的 1,000 句（`CONTEXT_SENTENCES`），段落第一句不收。輸入碼是把各字的碼接起來；分開寫是讓基準知道每個字用了幾個鍵。轉碼由 `qim_databuild::transcode::Transcoder` 做：每個字取第一個五代碼的速成碼，沒有五代碼才取三代碼，連打字集以外的字轉碼失敗。

`build/corpus/<名>/chars.tsv` 是該語料 train 切分的字頻；各語料的次數乘上語料權重（和 bigram 同一個，`corpora.rs` 的 `weight`）相加後放進詞庫資料包，`Engine` 載入時用它排字序（plan §4.1、§4.2）。UTF-8、tab 分隔、沒有欄名，一行「字<TAB>次數」，次數多的在前、同次數依碼位；dev、test 不算。

`--verify` 對每個語料多三項檢查：基準句與 train、dev 每句都轉得了碼（基準句與「有前文」的題的碼也要等於轉碼結果，有前文的題要剛好三欄）、基準句與「有前文」的句子沒有一句原樣出現在任何一份語料的 train（bigram 用全部的 train 訓練，有前文的另列一項 `<名> 有前文`）、建兩次基準檔、有前文的題、切分與字頻的 sha256 相同；另有一項看全部語料：前處理保留的粵語用字（`corpora.rs` 的 `CANTONESE_KEPT`）在 train 的字頻都大於 0，字形對照表（`common_forms.rs`）換掉的香港寫法與並存寫法（`TWO_WAY_FORMS`）的另一種寫法在 train 都不出現（確認前處理接上了對照表；表本身由轉換用的 OpenCC 查出，不另驗覆蓋）；詞庫也查一項：沒有對照表換掉的寫法與「着名」這類反向的錯字（`common_forms.rs` 的 `REVERSED_ONE_WAY`；rime-essay 原有的「顯着」建詞庫時併進「顯著」），含雙向並存寫法的詞每種寫法都收、詞頻相同、兩字字頻相同，兩字的 unigram 與詞裡相鄰字每種寫法的 bigram 也相同（確認 bigram 接上了並存寫法），含「着」「臺」「菸」的詞都另收「著」「台」「煙」的寫法（`ONE_WAY_FORMS`；義項不同的 `ONE_WAY_DISTINCT` 除外）；粵典詞庫收了預設詞庫的每個詞條（同碼、同詞頻）、也多了詞。切句與取樣規則見 plan §4.3，各來源怎麼抽文字寫在 `data/sources.toml` 的註解，wikitext 的規則在 `crates/qim-databuild/src/wikitext.rs`，HKCanCor 的在 `hkcancor.rs`。換 OpenCC 或語料版本時，用 `scripts/experiments/s2hk-diff/` 重看轉換換掉了哪些字；換 rime-essay 或 OpenCC 版本時，用 `scripts/experiments/lexicon-forms/` 重看詞庫裡和常見寫法不同的字，`GAPS=1` 看建詞庫實際換成不同碼的字（用法在各自的 README）。

### Unihan 交叉比對

每次建置都讀 Unihan（`Unihan.zip`）的 `kGradeLevel` 排字序；`--verify` 另外拿 `Unihan_DictionaryLikeData.txt` 的 `kCangjie` 與 libcangjie 的倉頡三代碼比對，抓 libcangjie 的錯碼。兩邊都有碼的字才比；Unihan 的碼等於該字任一個三代碼就算相同。差異只寫報表、不影響 `verify` 結果。摘要一行：

```text
Unihan kCangjie：比對 <字數> 字，差異 <字數> 字（其中 <字數> 字等於五代碼），報表 build/reports/unihan-kcangjie.tsv（不影響 verify 結果）
```

報表 `build/reports/unihan-kcangjie.tsv` 每次覆寫。UTF-8、tab 分隔，第一行是欄名，之後每個差異一行，依碼位排序：

| 欄 | 內容 |
| --- | --- |
| `codepoint` | `U+` 加至少 4 位大寫十六進位，同 Unihan |
| `character` | 字 |
| `unihan` | `kCangjie` 的碼，轉成小寫 |
| `libcangjie_cangjie3` | libcangjie 這個字所有列的三代碼，逗號分隔、依字母序 |
| `libcangjie_cangjie5` | 同上，五代碼；沒有就留空。只當參考 |

`unihan` 等於 `libcangjie_cangjie5` 其中之一的列多半是三代、五代的版本差異；要找錯碼先看其餘的列，例如 `awk -F'\t' 'NR>1 && index(","$5",", ","$3",")==0' build/reports/unihan-kcangjie.tsv`。比對在 `crates/qim-databuild/src/unihan.rs`。

`build/` 不進 git。

### 個人集

有 `bench/personal/rime-userdb.txt`（使用者 Rime 的學習資料，brief §12.2 的三欄格式，取得方式見 plan §5.2）時，`cargo xtask data` 取上屏次數 `c ≥ 1` 的詞，用同一個 `Transcoder` 從詞轉出速成碼（不用 Rime 記下的倉頡全碼），寫成 `bench/personal/rime-userdb.tsv`，第一行記 `rime-userdb.txt` 的 sha256；同一個詞出現多次時併成一題、上屏次數相加，含連打字集以外字的詞略過。沒有 `rime-userdb.txt` 時刪掉上次轉出的 `rime-userdb.tsv`。換了 `rime-userdb.txt` 要重跑 `cargo xtask data`。

`bench/personal/` 整個在 `.gitignore`，內容只在本機：不寫進 repo 的任何檔案、commit、票，終端機只印題數與略過的列數。邏輯在 `crates/qim-databuild/src/personal.rs`。個人集的路徑與檔頭、基準目錄與「有前文」的目錄、`build/corpus/<名>/dev.tsv`、`dev-context.tsv`、兩份長前文題與兩份口語長前文題在 `qim-databuild`（寫）與 `qim-bench`（讀）各有一份（plan §1.3 不讓 `qim-bench` 依賴 `qim-databuild`），`xtask/tests/shared_paths.rs` 比對兩邊。

### 線上更新的 release

`cargo xtask pack-release` 讀 `build/packs/` 的每份資料包（先跑 `cargo xtask data`），核對 manifest 對得上資料包檔、有 `LICENSE` 與 `NOTICE`，清空後寫出 `build/pack-release/`：`<id>.pack`、`<id>.manifest.json`、`<id>.LICENSE`、`<id>.NOTICE` 與更新索引 `index.json`（格式見 plan §4.7）。邏輯在 `crates/qim-databuild/src/release.rs`。

發新版前用 `--previous` 給上一版 release 的 `index.json`：同一份資料包內容（sha256）變了、版本卻沒加大，或版本變小時失敗並列出是哪幾份，`build/pack-release/` 不動。版本在 `crates/qim-databuild/src/pipeline.rs` 的 `build_packs` 每份各寫一個。

```sh
gh release download packs --repo 1morr/QIM --pattern index.json --dir /tmp/prev   # 上一版的索引
cargo xtask pack-release --previous /tmp/prev/index.json
```

資料包固定放在 tag `packs` 的 release（不用 `latest`，App 的 release 也在同一個 repo；plan §4.7），網頁與之後的平台殼讀 `releases/download/packs/index.json`。第一次 `gh release create packs build/pack-release/*`；之後換 asset 時先傳資料包、最後傳索引：`gh release upload packs build/pack-release/*.pack build/pack-release/*.manifest.json build/pack-release/*.LICENSE build/pack-release/*.NOTICE --clobber`，再 `gh release upload packs build/pack-release/index.json --clobber`。在 GitHub 建 release 會公開資料包，先問使用者（M3 還沒建過）。

### 停鍵層的模型

停鍵層用 Gboard 級 1.4M 的字級 CIFG-LSTM 重排 lattice 路徑（plan §2.7）。訓練不是決定性的（同設定重訓的分數會變），所以 `cargo xtask data` 不訓練：採用的那份放在 git 的 `data/neural-rerank/`（`model.gguf` 是 q8 權重、`vocab.txt` 是模型用到的字表、`model.toml` 是 λ、字表外的字數與訓練出處），`cargo xtask data` 只把它包成模型資料包，`--verify` 抽查 `qim-neural` 載入得了、排得出名次（`NeuralRerank`）。換模型的步驟：

```sh
cargo xtask data                                     # 語料的 train、dev 切分（build/corpus/）
ln -s <主 checkout>/bench/personal bench/personal    # worktree 裡要剔除個人集的句子時（〈個人集〉）
cd scripts/neural-rerank
uv run --script train.py prepare                     # 字表與 token 檔到 build/neural-rerank/data/，約 15 秒
uv run --script train.py train --model lstm --name lstm-c0.5 --steps 4000 --batch 128 --lr 2e-3 --weight-decay 0.01 --colloquial-frac 0.5   # 約 15 分鐘（Apple M6，MPS）
cd ../..
cargo xtask neural-import build/neural-rerank/lstm-c0.5 --lambda 2   # 核對 reference.json、量化成 q8、寫進 data/neural-rerank/
cargo xtask data                                     # 建模型資料包
```

- `train.py`（PyTorch，PEP 723 的 inline 依賴，`train.py.lock` 釘版本，`uv run --script` 跑）在權重旁寫 `reference.json`：幾組前文與路徑和 PyTorch 算的分數。`neural-import` 用 candle 的 f32 重算，差超過 0.0001 nat 就不寫檔（模型結構或權重名和 `qim-neural` 對不上），過了才量化成 q8（`qim_neural::to_gguf`，同樣的權重得到同樣的位元組）。
- λ 只在 dev 切分上挑（`scripts/experiments/model-ladder` 的 `neural --lambdas`），不看 test 與個人集；換了模型要照 plan §2.7 跑模型階梯報 test 的數字，`cargo xtask bench` 不跑重排。
- `qim-neural` 的單元測試用 `train.py fixture` 產生的極小模型（`crates/qim-neural/tests/fixtures/`）核對 PyTorch 的分數，CI 也跑；本機有 `build/neural-rerank/lstm-c0.5/` 時另比真的模型。改了 `train.py` 的模型結構要重產這組檔。
- 重排一次的延遲：`cargo run --release -p qim-neural --example latency`（9 條路徑 × 18 字、前文 50 字，2 條執行緒）。
- 模型資料包和其他資料包一起進 `cargo xtask pack-release` 的索引（〈線上更新的 release〉）。

## 首選字基準

改核心或資料包時必跑（plan §5.2），不在 `cargo xtask check` 裡。基準讀 `build/packs/quick/quick.pack` 與方案指名的 `build/packs/quick-lexicon/quick-lexicon.pack`、`build/packs/zh-hant-bigram/zh-hant-bigram.pack`、`build/packs/zh-hant-word-trigram/zh-hant-word-trigram.pack`，再掛上英文資料包 `build/packs/scowl-en-us/scowl-en-us.pack` 與 emoji 資料包 `build/packs/cldr-emoji-yue-hant/cldr-emoji-yue-hant.pack`（預設選項開著英文候選與 emoji 候選，同網頁），沒有就失敗，但不會自己重建：

- 改了 `data/`、`crates/qim-databuild/` 或資料包格式（`crates/qim-core/src/pack.rs`）之後，跑 bench 前先跑 `cargo xtask data`，否則量到的是舊資料包，前後數字一樣、`--check` 也照過。
- 只改核心程式碼不必重建：`cargo xtask bench` 每次都重編核心。
- 結果表格上方印出每份資料包 sha256 的前 12 碼；做前後對照時兩次不同才表示資料包真的換了，也可以和 `cargo xtask data` 印出的完整 sha256 對開頭。

```sh
cargo xtask bench                     # 印結果
cargo xtask bench --check             # 固定題全過、各集合答錯數與修正次數 ≤ baseline、每題打得出、原生每鍵延遲 p95 在上限內（〈每鍵延遲〉），否則 exit 非零
cargo xtask bench --update-baseline   # 用這次的結果改寫 bench/baseline.json（有個人集時連 bench/personal/baseline.json）
cargo xtask bench --learning          # 學習模式：比學習開關的修正次數，見〈學習模式〉
cargo xtask bench --lexicon quick-lexicon-words-hk --check   # 換成粵典詞庫，和預設詞庫的 baseline 比
```

`--lexicon <id>` 放在最前面，讀 `build/packs/<id>/<id>.pack` 換掉方案指名的詞庫，其餘資料包不變；和印結果、`--check`、`--dev` 合用（`cargo xtask bench --lexicon <id> --dev --set …`）。baseline 是預設詞庫的數字，所以不能和 `--update-baseline` 合用，`--check` 照樣和它比、看換了詞庫有沒有變差；`--learning` 不收它。

`cargo xtask bench` 用 release 編 `qim-bench` 並在 repo 根目錄執行。每題打兩次，都用新的 `Session` 與空白的 `UserModel`、只經 `Session::handle`；每次上屏後換回空白的 `UserModel`，題內也不學（核心只在上屏時學；不開無痕，無痕連前文也不用）。「有前文」的題給前文時，兩次都先 `Session::set_context`：

1. 逐鍵打完輸入碼、不選字，取當下的第一個候選當首選；沒有候選時首選是空字串。途中的首選拿來算跳動。
2. 模擬使用者選字打出整題，算修正次數與按鍵數：
   - 打完整題的輸入碼，然後反覆：第一頁的首選接得上還沒打出的文字就按空白；否則按數字鍵選這頁接得上的最長候選（一樣長取前面的）；這頁都接不上就翻頁。
   - 上屏後組字清空、題目還沒打完，就把剩下的字的碼再打一次；給前文時先 `set_context`（前文接著這題已上屏的字）。不給前文時核心照樣接自己上屏的字（plan §2.1，M3 票 05），同殼沒給前文的情形。
   - 卡住（翻完所有頁都接不上）時，還剩兩個字以上就按 Esc 清掉，之後一個字一個字打；一個字一個字打還卡住就是「打不出」。語料的碼都由碼表轉出，每個字一定查得到自己，打不出代表核心壞了，`--check` 會紅。
   - 選字、翻頁、Esc 各算一次修正。固定題不知道每個字用幾個鍵，整題當一個單位。

| 檔案 | 格式 |
| --- | --- |
| `bench/fixed.tsv` | 固定必過題，一行 `輸入碼<TAB>期望首選<TAB>來源`；空行與 `#` 開頭的行略過。輸入碼可以含分隔記號 `'`，照實體鍵盤的 `'` 鍵按（「回報」照打的記）。案例通過的那張票才加，不先加會紅的題 |
| `bench/corpus/<名>.tsv`、`bench/colloquial/<名>.tsv` | 一個檔一個集合，集合名是檔名，兩個目錄間不能重名；先跑維基、再跑口語。格式見〈資料建置〉的〈語料〉 |
| `bench/corpus/context/<名>.tsv`、`bench/colloquial/context/<名>.tsv` | 「有前文」的題，一行 `<前文><TAB><逐字速成碼，空格分隔><TAB><句子>`，由 `cargo xtask data` 產生（plan §5.2）。每個檔打兩次：給前文的集合 `<名>·前文`、不給前文的 `<名>·前文·不給`，都進 baseline、都由 `--check` 擋；不量每鍵延遲 |
| `bench/personal/rime-userdb.tsv` | 個人集的題目：第一行 `# rime-userdb.txt sha256 <sha256>` 記轉自哪份資料，之後一行 `<逐字速成碼，空格分隔><TAB><詞><TAB><上屏次數>`，由 `cargo xtask data` 從 `bench/personal/rime-userdb.txt` 轉出（見〈資料建置〉的〈個人集〉）；集合名 `personal`，語料不能用這個名字 |
| `bench/baseline.json` | `{"<集合名>": {"wrong": <答錯數>, "correction_count": <修正次數總和>, "wrong_lines": "<答錯的行號，如 1-3,7>"}}`，固定題不記。只用 `--update-baseline` 改，同一個 commit 說明原因 |
| `bench/personal/baseline.json` | 個人集的 baseline，格式同上，只在本機 |

結果是一個集合一列的 Markdown 表，可以直接貼進票的 `## Comments` 或 commit body 做前後對照：

- 整句首選正確：首選與期望完全相同的題數與比例；答錯 = 題數 − 整句首選正確。
- 字正確率：1 − Σ 編輯距離 ÷ Σ 期望字數；編輯距離以字計（Levenshtein），每題最多算到該題的字數。
- 修正/句：每句平均修正次數。KSPC：模擬的總按鍵數（含輸入碼、空白、數字鍵、翻頁、Esc）÷ 總字數。
- 跳動/句：逐鍵打字時，首選開頭屬於已打完的字的那段（首選比它短就取整個首選）被下一鍵改掉的次數，每句平均。只回報，不擋。
- 修正：修正次數的總和，`--check` 比的是它。`baseline 答錯`、`baseline 修正`：跑之前 baseline 檔的值，沒有檔或沒有該集合時是 `—`。
- 個人集多一列「加權」：每題依上屏次數加權，題數欄是上屏次數總和。

表下面依序是：

- 依長度分組：各集合按期望的字數分成 1、2、3–4、5–8、9 以上，每格「整句首選正確率 · 修正/句（題數）」，不加權。
- 有前文的題：每批題給前文與不給前文的答錯數、修正次數並列，最後一列合計。
- 翻盤清單：和 baseline 的 `wrong_lines` 比，列出由錯變對、由對變錯的題；每個方向最多列 20 題。公開集合附期望與首選，個人集只列行號。
- 固定題通過數與沒過的題。

個人集只印數字與行號（plan §5.2）。沒有 `bench/personal/rime-userdb.txt` 就跳過並註明，`bench/personal/baseline.json` 也不比；`rime-userdb.tsv` 檔頭的 sha256 和現在的 `rime-userdb.txt` 不同（還沒轉，或換了資料沒重跑 `cargo xtask data`）也跳過，提示先跑 `cargo xtask data`。個人集的行號依轉出的順序，換了 `rime-userdb.txt` 後行號會位移，先 `--update-baseline` 再比翻盤。

個人集只放在主 checkout 的 `bench/personal/`。在 git worktree 裡做票時，先用軟連結接上，否則基準照樣綠燈但沒有個人集的數字：

```sh
ln -s <主 checkout>/bench/personal bench/personal
```

`.gitignore` 的 `/bench/personal` 不帶結尾斜線，目錄與軟連結都排除；帶斜線只比對目錄，軟連結會變成未追蹤檔。接上後各 worktree 共用同一份 `rime-userdb.tsv` 與 `baseline.json`：在任一個 worktree 跑 `cargo xtask data` 或 `--update-baseline` 都會改到主 checkout 那份。

### dev 切分調參

β、γ 這類打分參數只用 dev 切分調（plan §2.3、§5.2），不看 test 與個人集：

```sh
cargo xtask bench --dev                                              # 用方案檔的值打各集合的 dev 切分
cargo xtask bench --dev --set word_bonus=1.5 --set segment_penalty=0.5   # 暫時蓋掉方案檔 [ranking] 的值
```

- 題目是 `build/corpus/<名>/dev.tsv`，`<名>` 是 `bench/corpus/`、`bench/colloquial/` 下的每個集合；由 `cargo xtask data` 寫出，取 dev 切分雜湊最小的 2,000 個不重複句，格式同基準檔。沒有就失敗，先跑 `cargo xtask data`。
- 「有前文」的調參題是 `build/corpus/<名>/dev-context.tsv`，取 dev 切分、取法與格式同 `bench/*/context/`；有這個檔的集合在所有集合之後多打 `<名>·前文`、`<名>·前文·不給` 兩個集合，報告並列兩者與合計；沒有就不打。前文規則的實驗：`scripts/experiments/context-rule/`。
- `--set` 可重複，鍵是方案檔 `[ranking]` 的欄位名（`word_bonus`、`segment_penalty`、`beam`、`sentences`）；資料包不變。蓋過的值照方案檔的規則驗（`qim_core::Ranking::check`：`beam` 至少 1，β、γ 是有限的數），不合就失敗、指出是哪個值。語料權重要改 `crates/qim-databuild/src/corpora.rs` 再 `cargo xtask data`。
- 每個集合後面接兩組由它的 dev 句子取出的短輸入（依出現次數加權的列才是照語料打的正確率）：`<名>·單字` 是句子裡每個字單獨打，`<名>·詞` 是句子裡出現、詞庫也有的 2–4 字詞單獨打。dev 都是長句，沒有它們就看不出參數讓兩碼的單字或單打一個詞時選錯。
- 只印結果：固定題、各集合與短輸入的指標表、依長度分組；不比 baseline、不跑個人集、不量每鍵延遲，所以不重編 WebAssembly 綁定，一次約 15 秒（Apple M6）。
- dev 的句子和 bigram 的訓練語料同源，字級 bigram 在 dev 上特別準：票 12 照 dev 調低 β 時個人集（使用者自己的詞）答錯變多，`·詞` 也看不出來。所以 dev 挑出的參數仍要過 `cargo xtask bench --check` 的個人集 baseline。
- 格點搜尋：`scripts/experiments/bigram-tuning/grid.sh "<β 值…>" "<γ 值…>" [<鍵>=<值>…]` 平行跑每一格，一格一列 TSV；票 12 的調參過程與結果在 `.scratch/m1/issues/12-bigram-tuning-and-report.md`。

### 每鍵延遲

每次 `cargo xtask bench`（`--learning` 以外的模式）都在報告最後印每鍵延遲：每次按鍵到 `Snapshot` 回來的 p50 / p95，單位 µs，原生與 WebAssembly 各一次，各分三組：「全部」；「第 20 碼起」只算每個輸入的第 20 鍵起（組字已有 19 碼再按的鍵），看連打變長時延遲會不會跟著長，前 19 鍵算進來會稀釋，「輸入數」是長到 20 碼的輸入有幾個；「選開頭一段」是打完有兩段以上的輸入再按分段數那麼多次 ←（不計時）移到第一段、選它的首選（`select(0)`）的那一次，選定後剩下的輸入碼從頭重建 lattice，是最重的一種鍵（規則在 `qim_bench::lefts_to_first_segment`）。三組再各量兩份學習資料：「空白」是空白的 `UserModel`（同基準其他部分），「學過」是一個 `UserModel` 依序照修正次數的模擬學完各語料集合與「有前文」的題（給前文，同 `--learning` 的學習開）之後（M3 票 12 的量法），計時的輸入就是學過的句子，偏向最壞情況。原生的 p95 由 `--check` 擋，上限在 [M4 spec〈每鍵延遲上限〉](../.scratch/m4/spec.md#每鍵延遲上限)（`qim_bench::p95_limits_ms`），報告的「p95 上限」欄列出、超過的註明；WebAssembly 只報；baseline 都不記。量不了（沒有 Node、綁定編不起來、計時結果和輸入對不上、量選開頭一段的輸入和原生不同）則是錯誤，整個基準失敗、不印結果，`--check` 也是，免得延遲安靜地缺席。表頭上方註明機器（CPU、作業系統、架構），「環境」欄註明原生的 build profile 與 WebAssembly 的 runtime 版本。

- 量哪些輸入：固定題與各語料集合的每一題（個人集不量，各機器才比得起來），每題從空的組字逐鍵打輸入碼、不選字；打完有兩段以上的另量「選開頭一段」那一鍵（見上）。不含載入資料包。
- 原生：`qim-bench` 在同一個行程裡，每題用新的 `Session`，各題共用那份學習資料的一份複本（打字與選開頭一段都不上屏、不學），只計 `Session::handle`（選開頭一段是 `Session::select`）那一次呼叫。
- WebAssembly：`qim-bench` 用 Node 跑 `web/bench/latency.ts <資料包>…`（基準載入的每份資料包，順序不拘；每份旁邊的 `manifest.json` 給種類並經核心核對，plan §4.6），學過的那次另給 `--user-dict <檔>`（`UserModel::export` 的文字寫到暫存檔，腳本 `loadUserDict`、不計入載入），它載入網頁同一份綁定 `web/src/wasm/`，每鍵計 `inputChar()` 加讀出網頁畫候選要的欄位（分段、輸入碼、候選），每題之間按 Esc（不計時）；輸出格式見 `qim_bench::parse_timing`。`cargo xtask bench` 先以 release 重編綁定（同〈網頁〉第 1 步）再跑基準，所以要有 `wasm-bindgen-cli` 與 Node；直接 `cargo run -p qim-bench` 時綁定不存在就失敗。
- 為什麼用 Node 不用瀏覽器、兩者差多少：`scripts/experiments/wasm-latency-browser/README.md`；經 Swift 綁定的同一批輸入：`scripts/experiments/ffi-swift/README.md`。
- 載入（M3 票 11b）：表後一行「載入」，原生是 `qim_bench::load_engine` 解開各資料包加建 `Engine` 的毫秒數（不含讀檔與算 sha256），WebAssembly 是 `latency.ts` 的 `new Session(packs)`（`Engine::open`：核對 manifest 與 sha256、解開、建 `Engine`，同網頁），由輸出第一行 ` | ` 後的數字帶回；只報不擋。載入一行是空白那次的；學過那次另花的時間在學完全部公開集合（約 1 萬句的模擬），整個 `cargo xtask bench` 約 3 分鐘（Apple M6）。
- 百分位數取 nearest-rank（排序後第 ⌈p/100 × n⌉ 個）。macOS 上原生與 Node 的計時解析度約 0.04 µs，現在的核心每鍵多在這個量級，p50 印 0.00 表示比解析度還快。

`--check` 因每鍵延遲紅時（stderr 的「每鍵延遲超過上限」），先在負載低時（關掉其他 build、測試與別的 worktree 的基準）單獨重跑一次 `cargo xtask bench --check`，仍超過才算紅；重跑通過的話在票的 Comments 記兩次的數字。上限只在 spec 寫的條件下改，不因某張票超過而放寬。

改核心的票貼前後對照時，在同一台機器、不跑其他重負載時跑；差距小於幾成多半是雜訊，重跑兩三次看是否穩定。

### 學習模式

`cargo xtask bench --learning` 只印一張表：各語料集合與個人集的修正次數，學習關與學習開（plan §5.2）。「有前文」的題只打給前文的（集合名 `<名>·前文`）。每個集合照檔案的順序，每題照上面第 2 步模擬一次：學習關同一般模式，每題用空白的 `UserModel`、上屏後換回空白的（數字和一般模式的修正次數相同），學習開整個集合共用一個 `UserModel`，上屏的字詞照常學進去，後面的題受前面的影響。只報不擋，不讀也不寫 baseline，不量延遲也不重編 WebAssembly 綁定。個人集同樣只印數字。

`--check` 的違規每條一行，最後一行 `bench 通過` 或 `bench 失敗`：固定題沒過（附期望與實際首選）、集合答錯數或修正次數高於 baseline、集合有題打不出、集合沒有 baseline（新語料要先 `--update-baseline`）、baseline 的集合這次沒跑到。`--update-baseline` 讀不了舊 baseline（例如格式改了）時直接覆寫。邏輯在 `crates/qim-bench/`。

## 網頁

`cargo xtask web` 做兩件事，邏輯在 `xtask/src/web.rs`：

1. 用 release 編 `qim-wasm` 到 `wasm32-unknown-unknown`，`wasm-bindgen --target web` 產生到 `web/src/wasm/`，`web/src/main.ts` 與停鍵層的 Web Worker（`web/src/rerank-worker.ts`）直接 import。`.cargo/config.toml` 給這個 target 開 SIMD128：candle 的 q8 內積有 simd128 的實作，停鍵層模型在瀏覽器裡重排一次快約 3 倍（M3 票 11e）。
2. 把 `build/packs/`、`build/layouts/` 整個複製到 `web/public/packs/`、`web/public/layouts/`（先清掉舊的）。網頁照 `web/src/packs.ts` 的內建清單，每份 fetch `/packs/<id>/manifest.json` 與 `/packs/<id>/<id>.pack`（速成 `quick`、詞庫 `quick-lexicon` 與可選的 `quick-lexicon-words-hk`、語言模型 `zh-hant-bigram`、詞級 trigram `zh-hant-word-trigram` 與可切換的放寬版 `zh-hant-word-trigram-large`、字根提示 `cangjie-hint`、簡繁 `opencc-hk2s`、英文 `scowl-en-us`、emoji `cldr-emoji-yue-hant`、可缺的停鍵層模型 `zh-hant-neural-rerank`），觸控鍵盤 fetch `/layouts/gboard-quick.json`、`/layouts/gboard-quick-number-row.json`。`build/packs/` 或 `build/layouts/` 不存在時這一步失敗；目錄在但少了某個檔（例如舊的建置沒有後來加的資料包或 manifest），要到網頁載入時才失敗。兩種都先跑 `cargo xtask data`。

這些目錄都不進 git。改了 `qim-core`、`qim-layout` 或 `qim-wasm` 要重跑才看得到；`check` 的 `wasm` 步驟只做第 1 步，`e2e` 步驟先做第 2 步再跑 Playwright。

網頁把學習資料（`# qim-userdict v3`，plan §2.4；仍讀得進 v1、v2）存在 `localStorage` 的 `qim.userdict`，每次上屏後覆寫、載入時匯入；讀不進來的搬到 `qim.userdict.broken`。匯出檔頭的裝置 id 在 `qim.device`（第一次載入時產生）；今天的日期取瀏覽器的當地時間，Playwright 用 `page.clock` 換日。回報清單在 `qim.reports`。設定區「資料包」匯入的資料包存在 IndexedDB 的 `qim-packs` 庫，每種用哪份存在 `localStorage` 的 `qim.packs.active`（plan §4.6）。瀏覽器封鎖網站資料或存不下時，網頁只用內建的資料包、存不了的只留到關掉網頁，狀態列會說明（plan §3.1、§4.6）；`e2e/storage.spec.ts` 在網頁的程式跑之前換掉 `indexedDB.open` 與 `window.localStorage` 來試。要從空白開始試打就在開發者工具清掉它們，或按網頁的「清空學習資料」、刪掉匯入的資料包。

線上更新（plan §4.7）：網頁向開發伺服器的 `/pack-updates/<檔名>` 要更新索引與資料包，`web/vite.config.ts` 轉給更新來源，預設 `https://github.com/1morr/QIM/releases/download/packs/`（瀏覽器直接讀不了 GitHub 的 release，沒有 CORS 標頭）；環境變數 `QIM_PACK_UPDATE_URL`（結尾要有 `/`）換成別的來源，例如在本機試：

```sh
cargo xtask pack-release
(cd build/pack-release && python3 -m http.server 8787)
QIM_PACK_UPDATE_URL=http://localhost:8787/ npm run --prefix web dev
```

還沒建 release 時，預設來源回 404，設定區的狀態列會寫「檢查更新失敗」，打字不受影響。開關存在 `localStorage` 的 `qim.packs.update`（關掉是 `off`）；`vite build` 出的靜態頁沒有轉發，也就沒有線上更新。

e2e 的小詞庫在 `web/e2e/fixtures/` 並進 git，Biome 不碰它們：`e2e-lexicon/`（從檔案匯入用）、`e2e-update-lexicon/`（線上更新前先匯入的舊版）、`quick-lexicon-1.0.1/` 與 `quick-lexicon-1.0.2/`（和內建詞庫同 id 的兩版，驗下載版與內建版取較高的）、`pack-release/`（扮演 GitHub release 的 `index.json` 與新版）。`cargo test` 比對它們和現在的格式一致，格式改了會紅，照訊息重產：`pack-release/` 以外的由 `crates/qim-core/tests/web_fixture.rs` 產生（`QIM_UPDATE_FIXTURES=1 cargo test -p qim-core --test web_fixture`），`pack-release/` 由 `crates/qim-databuild/tests/release.rs` 照 `cargo xtask pack-release` 的路寫出（`QIM_UPDATE_FIXTURES=1 cargo test -p qim-databuild --test release`）。Playwright 另起一個本機靜態伺服器（`web/e2e/static-server.ts`）放 `pack-release/`，埠是 e2e 的埠加 10000，開發伺服器的 `QIM_PACK_UPDATE_URL` 指向它；被佔用時同樣直接失敗。手動試匯入時，一起選 `build/packs/<id>/` 裡的 `manifest.json` 與 `.pack` 也行。

停鍵層（plan §3.1）：「重排」選單預設「神經模型」，停鍵後在 Web Worker 裡跑；沒有 `zh-hant-neural-rerank`（或從檔案匯入的模型資料包）時不重排。網址加 `?rerank-delay=<毫秒>` 讓結果回來後再等那麼久才套用，模擬慢的模型（e2e 驗舊結果不套用）。候選列的 `data-rerank-applied`、`-unchanged`、`-stale`、`-none` 是各種結果的次數，`data-rerank-ms` 是模型上次算了幾毫秒；工具列的「debug」打開 debug 面板，看得到每次重排的要求、各路徑的模型總分與結果、候選列是不是照停鍵結果快取排（`debug-rerank-cache` 的 `data-cached`）與模型跑過幾次（`#debug` 的 `data-rerank-requests`），和每個候選的來源、路徑分（plan §3.1）。瀏覽器裡重排一次的延遲：`QIM_RERANK_LATENCY=1 npm run --prefix web e2e -- e2e/rerank-latency.spec.ts`（平常略過；合成 9 條 × 18 字與實打口語集前 200 題，印 p50、p95，只報不擋）。

Playwright 的測試在 `web/e2e/`，截圖存在 `web/test-results/`（每次覆寫、不進 git）。在 Node 跑的檔（Playwright 設定、`e2e/`、單元測試 `test/`）由 `web/tsconfig.node.json` 型別檢查，網頁的 `src/` 看不到 Node 的 API（Vite 範本的 `tsconfig.node.json` 慣例）。`web/bench/` 是用 Node 跑的基準腳本（〈首選字基準〉的〈每鍵延遲〉），有自己的 `tsconfig.json`（多了 Node 的型別），不進網頁。

## Swift 綁定

`cargo xtask ffi [--ios]` 給 Apple 平台的殼用 `qim-ffi`（uniffi，plan §1.1），邏輯在 `xtask/src/ffi.rs`：

1. `cargo build --release -p qim-ffi`：這台機器架構的靜態函式庫 `target/release/libqim_ffi.a`。
2. 用 `qim-ffi` 的 `uniffi-bindgen-swift`（只在 `bindgen` feature 編，uniffi 的 CLI 依賴不進函式庫）從它產生 `build/ffi/swift/` 的 `qim.swift`、`qimFFI.h`、`module.modulemap`（library mode：介面讀自函式庫，不必 UDL）。

Swift 程式把 `qim.swift` 一起編、連結靜態函式庫、指到 modulemap，例如：

```sh
swiftc -O -o build/ffi/bin/qim-swift main.swift build/ffi/swift/qim.swift \
  -Xcc -fmodule-map-file=build/ffi/swift/module.modulemap -I build/ffi/swift -L target/release -lqim_ffi
```

`--ios` 再做一步：編 `aarch64-apple-ios`（iPhone 實機，要先 `rustup target add aarch64-apple-ios`；模擬器沒用到、不編）的靜態函式庫，標頭與 modulemap 產生到 `build/ffi/ios-headers/qimFFI/`，用 `xcodebuild -create-xcframework` 包成 `build/ffi/QimFFI.xcframework`。Xcode 專案把它加進 target 的連結（不內嵌），再把同一份 `qim.swift` 加進原始碼。modulemap 放在以模組名命名的子目錄、用一般的 `module`：Xcode 把標頭複製到建置產物的 `include/`，clang 依模組名找到子目錄；uniffi 的 `--xcframework` 產生的 `framework module` 在靜態函式庫的 xcframework 裡找不到（M2 票 08 實測）。

Swift 端的用法：每份資料包的位元組各自解開成 `SchemePack`、`LexiconPack`、`LanguageModelPack`、`WordTrigramPack`、`SimplifierPack`、`EnglishPack`，交給 `Engine(...)`（每份只能給一個 `Engine`），每個輸入框 `Session(engine:)` 共用同一個 `Engine`（plan §2.1）。字根提示 `RadicalHintPack` 目前只能解開、不交給 `Engine`（M2 票 09 量記憶體用）。範例：Mac 是 `scripts/experiments/ffi-swift/`（打 `anr` 讀回「門口」、量每鍵延遲），iPhone 鍵盤擴充是 `scripts/experiments/ios-keyboard/` 的〈接上核心（票 08）〉。產物都不進 git。`check` 不跑這一步：CI 是 Ubuntu，`qim-ffi` 的 Rust 測試照常在 `cargo test` 裡跑。

## 全部檢查

每張票收尾時照跑，並貼輸出：

```sh
cargo xtask check
```

依序跑下列步驟，第一個失敗就停，最後一行印 `check 失敗：<步驟>`，exit code 非零；全部通過印 `check 通過`。

| 步驟 | 指令 | `--fast` |
| --- | --- | --- |
| `fmt` | `cargo fmt --all --check` | 跑 |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | 跑 |
| `test` | `cargo test --workspace` | 跳過 |
| `deps` | 依賴規則（plan §1.3）：`cargo metadata` 比對白名單，並掃 `qim-core`、`qim-layout` 的禁用 I/O | 跑 |
| `docs` | 文件規則（plan §6）：`docs/progress.md` 的段落、表格、長度、里程碑標記、下一步指令，與 `.scratch/` 的 spec | 跑 |
| `wasm` | 產生網頁的 WebAssembly 綁定（〈網頁〉第 1 步），`tsc` 要讀它的型別 | 跑 |
| `tsc` | `web/` 的 `tsc --noEmit`（`src/`），再對 `tsconfig.node.json`（Playwright 設定、`e2e/`、`test/`）與 `web/bench/` 各跑一次（`npm run typecheck`） | 跑 |
| `biome` | `web/` 的 `biome check`（`npm run lint`） | 跑 |
| `unit` | `web/test/` 的單元測試（`npm test`，Node 內建的 `node --test`，直接跑 TypeScript） | 跑 |
| `e2e` | 複製資料包（〈網頁〉第 2 步）後跑 Playwright（`npm run e2e`）；要先跑過 `cargo xtask data` | 跳過 |

`cargo xtask check --fast` 是 pre-commit hook 跑的快速版，跳過 `test` 與 `e2e`：測試會隨碼表與性質測試變慢，e2e 要起瀏覽器；其他步驟增量執行幾秒內完成。`clippy --all-targets` 仍會編譯測試碼，所以測試編不過也會被擋。

CI（`.github/workflows/ci.yml`）在 push 到 `main` 與每個 PR 時跑 `cargo xtask check --fast` 與 `cargo test --workspace`；`e2e` 與 `bench --check` 要資料包，只在本機跑（plan §6）。

`deps` 失敗時每條違規印一行，說明是哪個 crate、違反 plan §1.3 的哪一條；禁用路徑附 `檔案:行號`。

`docs` 失敗時每條違規印一行 `檔案:行號: 說明`，規則在 `xtask/src/docs.rs`。最常見的是平行的票合併後 `docs/progress.md` 被 union 合併抄出第二個〈Session 紀錄〉段：把第二段新的列移回第一段的表尾、刪掉重複段。

格式自動修正：`cargo fmt --all`、`npm run --prefix web lint -- --write`。

步驟定義在 `xtask/src/check.rs`；加步驟時同輪更新上表，`cargo test` 會比對兩邊的步驟名與 `--fast` 欄。

## Commit message

commit-msg hook 跑 `cargo xtask commit-msg <訊息檔>`，subject（第一個非空白、非 `#` 註解的行）不符下列規則就擋下並逐條說明：

- 格式 `type(scope)!: description`，`(scope)` 與 `!` 選填，冒號後接一個空格。
- type 只能是 `xtask/src/commit_msg.rs` 的 `TYPES`：feat、fix、docs、refactor、test、chore、build、ci、perf、style、revert。
- description 不以大寫字母開頭，結尾不加句號；subject 整行 ≤ 72 字元。

不檢查的情況：

- git 自動產生的 `Merge …`、`Revert "…"`、`Reapply "…"`、`fixup!`、`squash!`、`amend!` 開頭的訊息。
- 正在 `git merge` 時（`MERGE_HEAD` 存在）一律放行：ticket-coordinator 用 `merge: ticket NN slug` 合併票。
- body 與 `git commit -v` 剪刀線之後的 diff。
