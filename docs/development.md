# 開發

開發指令的單一來源。新增或改動開發指令的票，同輪更新本檔。

## 環境需求

| 工具 | 版本 | 說明 |
| --- | --- | --- |
| rustup | 任一近期版本 | Rust 版本由 `rust-toolchain.toml` 釘住，第一次在 repo 內跑 `cargo` 時 rustup 自動安裝，含 `rustfmt`、`clippy` 與 `wasm32-unknown-unknown` target |
| `wasm-bindgen-cli` | 與 `Cargo.lock` 的 `wasm-bindgen` 相同（0.2.130） | 產生網頁的 JS 綁定；版本不同時 `cargo xtask web` 與 `check` 的 `wasm` 步驟會擋下並印安裝指令 |
| Node.js | 26 LTS（開工時 26.12.0） | 附帶的 `npm` 管理 `web/` 的依賴 |
| git | 任一近期版本 | hook 用 `core.hooksPath`，Windows 上由 Git for Windows 的 `sh` 執行 |

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
| `npm run --prefix web dev` | 網頁開發伺服器；先跑 `cargo xtask web` |
| `npm run --prefix web e2e` | Playwright 實打網頁（自己起開發伺服器，埠 5179）；先跑 `cargo xtask web` |
| `npm run --prefix web build` | 型別檢查後打包網頁到 `web/dist/` |
| `cargo xtask data [--verify]` | 建資料包到 `build/packs/`、預設佈局到 `build/layouts/`，見〈資料建置〉 |
| `cargo xtask bench [--check \| --update-baseline]` | 首選字基準，見〈首選字基準〉 |

## 資料建置

```sh
cargo xtask data            # 建資料包
cargo xtask data --verify   # 建資料包並跑 plan §4.5 的檢查，有一項失敗就 exit 非零
```

`cargo xtask data` 用 release 編 `qim-databuild` 並在 repo 根目錄執行，步驟見 plan §4：

- 來源列在 `data/sources.toml`：每個來源釘 `version`（git 來源填 commit，其他填上游版本號）與每個檔的 sha256。下載到 `<快取>/sources/<來源>/<sha256 前 16 碼>/<檔案路徑>`；已有快取就不連網，下載是邊收邊寫檔，不整份讀進記憶體。快取或下載的檔 sha256 不符時建置失敗、不重新下載：確認來源後刪掉那個檔重跑，或更新 `sources.toml`。換來源版本時改 version、url 與 sha256，快取路徑跟著換。
- `<快取>` 在 repo 外，所有 worktree 共用，新 worktree 不必重下約 360 MB 的來源：環境變數 `QIM_CACHE_DIR` 有設就用它，否則是平台的使用者快取目錄下的 `qim/`（macOS `~/Library/Caches/qim`、Linux `$XDG_CACHE_HOME/qim` 或 `~/.cache/qim`、Windows `%LOCALAPPDATA%\qim`）。路徑含釘住的 sha256，不同 worktree 釘不同版本也不會互相蓋掉。下載中被中斷時會留下 `<檔名>.part<行程 id>`，可以直接刪。
- 資料進資料包的來源要寫 `license_text`，三選一，沒寫時建置失敗：
  - `statement`（`file`、`quote`）：照抄上游檔裡的授權聲明，建置時核對 `quote` 確實在 `file` 裡（空白與換行不計），再寫進資料包的 `LICENSE`（libcangjie）。`file` 要是這個來源 `files` 之一。
  - `upstream_file`：上游釘住 sha256 的授權檔（`files` 之一）整份寫進 `LICENSE`（rime-essay 的 `LICENSE`）。
  - `repo_file`：repo 裡的授權全文檔，整份寫進 `LICENSE`。上游授權檔沒有版本化網址、會改內容時用，例如 Unicode 的 `license.txt` 每年改版權年份，釘 sha256 會讓乾淨建置過期失敗；全文放 `data/licenses/`（Unihan 用 `Unicode-3.0.txt`；語料的字頻進詞庫資料包，用 Creative Commons 的三份全文）。
- 方案檔在 `data/schemes/`，由 `qim-databuild` 編進資料包；`qim-core` 不讀 TOML。
- 產物：`build/packs/<id>/` 下的 `<id>.pack`、`LICENSE`、`NOTICE`，每個資料包只寫它用到的來源（plan §4）。速成資料包的 `<id>` 是方案 id（`quick`）；詞庫資料包的 `<id>` 是方案檔 `[packs]` 的 `lexicon`（`quick-lexicon`），內容是 rime-essay 的詞與詞頻，加上各語料 train 切分相加的字頻（plan §4.2），在語料之後建。兩種資料包的格式都定義在 `crates/qim-core/src/pack.rs`（魔數不同、共用 `FORMAT_VERSION`），改格式時加 `FORMAT_VERSION`。
- 預設佈局：`build/layouts/gboard-quick.json`，Gboard 速成的手機直向，由 `crates/qim-databuild/src/gboard.rs` 依按鍵排列算出（plan §2.6），格式定義在 `crates/qim-layout`。不讀來源、沒有授權檔。
- 語料（plan §4.3）：清單在 `crates/qim-databuild/src/corpora.rs`。維基（粵語 `yue-wiki`、中文 `zh-wiki`）與口語（`hkcancor`、`common-voice-yue`）各自抽出文字，經 `s2hk`、切句、切分，寫出基準檔與 `ATTRIBUTION.md`（進 git：維基在 `bench/corpus/`，口語在 `bench/colloquial/`）與 `build/corpus/<名>/` 下的 `train.txt`、`dev.txt`（一行一句）、`chars.tsv`（字頻，見〈語料〉），後三者不進 git。第一次要下載約 360 MB 的來源；之後 `cargo xtask data` 約 35 秒，`--verify` 把語料再建一次比 sha256，約 75 秒（Apple M6，release）。

`--verify` 的報告列出連打字集字數、擴展區字數、速成碼個數、兩份資料包的 sha256、詞庫的詞數與被門檻和字集濾掉的數量、有字頻的字數、Unihan 交叉比對的摘要、各語料的句數，再逐項列 `[通過]` 或 `[失敗]`，最後一行 `verify 通過` 或 `verify 失敗`。檢查定義在 `crates/qim-databuild/src/verify.rs`；字序（`a` 的首選）與 `anr` 的「門口」經 `qim-core` 的 `Engine` 查，與網頁同一條路。

### 語料

`bench/corpus/<名>.tsv`（維基）與 `bench/colloquial/<名>.tsv`（口語）是首選字基準的題目，由建置產生、不要手改。UTF-8，沒有欄名，一行一句：

```text
<逐字速成碼，空格分隔><TAB><句子>
```

例如 `an r⇥門口`（`⇥` 是 tab）。輸入碼是把各字的碼接起來；分開寫是讓基準知道每個字用了幾個鍵。轉碼由 `qim_databuild::transcode::Transcoder` 做：每個字取第一個五代碼的速成碼，沒有五代碼才取三代碼，連打字集以外的字轉碼失敗。

`build/corpus/<名>/chars.tsv` 是該語料 train 切分的字頻；各語料的次數直接相加後放進詞庫資料包，`Engine` 載入時用它排字序（plan §4.1、§4.2）。UTF-8、tab 分隔、沒有欄名，一行「字<TAB>次數」，次數多的在前、同次數依碼位；dev、test 不算。

`--verify` 對每個語料多三項檢查：基準句與 train、dev 每句都轉得了碼（基準句的碼也要等於轉碼結果）、基準句沒有一句原樣出現在任何一份語料的 train（bigram 用全部的 train 訓練）、建兩次基準檔、切分與字頻的 sha256 相同。切句與取樣規則見 plan §4.3，各來源怎麼抽文字寫在 `data/sources.toml` 的註解，wikitext 的規則在 `crates/qim-databuild/src/wikitext.rs`，HKCanCor 的在 `hkcancor.rs`。

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

`bench/personal/` 整個在 `.gitignore`，內容只在本機：不寫進 repo 的任何檔案、commit、票，終端機只印題數與略過的列數。邏輯在 `crates/qim-databuild/src/personal.rs`。

## 首選字基準

改核心或資料包時必跑（plan §5.2），不在 `cargo xtask check` 裡。基準讀 `build/packs/quick/quick.pack` 與方案指名的 `build/packs/quick-lexicon/quick-lexicon.pack`，沒有就失敗，但不會自己重建：

- 改了 `data/`、`crates/qim-databuild/` 或資料包格式（`crates/qim-core/src/pack.rs`）之後，跑 bench 前先跑 `cargo xtask data`，否則量到的是舊資料包，前後數字一樣、`--check` 也照過。
- 只改核心程式碼不必重建：`cargo xtask bench` 每次都重編核心。
- 結果表格上方印出每份資料包 sha256 的前 12 碼；做前後對照時兩次不同才表示資料包真的換了，也可以和 `cargo xtask data` 印出的完整 sha256 對開頭。

```sh
cargo xtask bench                     # 印結果
cargo xtask bench --check             # 固定題全過、各集合答錯數與修正次數 ≤ baseline、每題打得出，否則 exit 非零
cargo xtask bench --update-baseline   # 用這次的結果改寫 bench/baseline.json（有個人集時連 bench/personal/baseline.json）
```

`cargo xtask bench` 用 release 編 `qim-bench` 並在 repo 根目錄執行。每題打兩次，都用新的 `Session` 與空白的 `UserModel`、只經 `Session::handle`：

1. 逐鍵打完輸入碼、不選字，取當下的第一個候選當首選；沒有候選時首選是空字串。途中的首選拿來算跳動。
2. 模擬使用者選字打出整題，算修正次數與按鍵數：
   - 打完整題的輸入碼，然後反覆：第一頁的首選接得上還沒打出的文字就按空白；否則按數字鍵選這頁接得上的最長候選（一樣長取前面的）；這頁都接不上就翻頁。
   - 上屏後組字清空、題目還沒打完，就把剩下的字的碼再打一次。
   - 卡住（翻完所有頁都接不上）時，還剩兩個字以上就按 Esc 清掉，之後一個字一個字打；一個字一個字打還卡住就是「打不出」。語料的碼都由碼表轉出，每個字一定查得到自己，打不出代表核心壞了，`--check` 會紅。
   - 選字、翻頁、Esc 各算一次修正。固定題不知道每個字用幾個鍵，整題當一個單位。

| 檔案 | 格式 |
| --- | --- |
| `bench/fixed.tsv` | 固定必過題，一行 `輸入碼<TAB>期望首選<TAB>來源`；空行與 `#` 開頭的行略過。案例通過的那張票才加，不先加會紅的題 |
| `bench/corpus/<名>.tsv`、`bench/colloquial/<名>.tsv` | 一個檔一個集合，集合名是檔名，兩個目錄間不能重名；先跑維基、再跑口語。格式見〈資料建置〉的〈語料〉 |
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
- 翻盤清單：和 baseline 的 `wrong_lines` 比，列出由錯變對、由對變錯的題；每個方向最多列 20 題。公開集合附期望與首選，個人集只列行號。
- 固定題通過數與沒過的題。

個人集只印數字與行號（plan §5.2）。沒有 `bench/personal/rime-userdb.txt` 就跳過並註明，`bench/personal/baseline.json` 也不比；`rime-userdb.tsv` 檔頭的 sha256 和現在的 `rime-userdb.txt` 不同（還沒轉，或換了資料沒重跑 `cargo xtask data`）也跳過，提示先跑 `cargo xtask data`。個人集的行號依轉出的順序，換了 `rime-userdb.txt` 後行號會位移，先 `--update-baseline` 再比翻盤。

個人集只放在主 checkout 的 `bench/personal/`。在 git worktree 裡做票時，先用軟連結接上，否則基準照樣綠燈但沒有個人集的數字：

```sh
ln -s <主 checkout>/bench/personal bench/personal
```

`.gitignore` 的 `/bench/personal` 不帶結尾斜線，目錄與軟連結都排除；帶斜線只比對目錄，軟連結會變成未追蹤檔。接上後各 worktree 共用同一份 `rime-userdb.tsv` 與 `baseline.json`：在任一個 worktree 跑 `cargo xtask data` 或 `--update-baseline` 都會改到主 checkout 那份。

### 每鍵延遲

每次 `cargo xtask bench`（不論哪個模式）都在報告最後印每鍵延遲：每次按鍵到 `Snapshot` 回來的 p50 / p95，單位 µs，原生與 WebAssembly 各一次，各分「全部」與「第 20 碼起」兩組：後者只算每個輸入的第 20 鍵起（組字已有 19 碼再按的鍵），看連打變長時延遲會不會跟著長，前 19 鍵算進來會稀釋；「輸入數」是長到 20 碼的輸入有幾個。只報不擋：機器負載會飄，`--check` 不比、baseline 不記。量不了（沒有 Node、綁定編不起來、計時結果和輸入對不上）則是錯誤，整個基準失敗、不印結果，`--check` 也是，免得延遲安靜地缺席。表頭上方註明機器（CPU、作業系統、架構），「環境」欄註明原生的 build profile 與 WebAssembly 的 runtime 版本。

- 量哪些輸入：固定題與各語料集合的每一題（個人集不量，各機器才比得起來），每題從空的組字逐鍵打輸入碼、不選字。不含載入資料包。
- 原生：`qim-bench` 在同一個行程裡，每題用新的 `Session`，只計 `Session::handle` 那一次呼叫。
- WebAssembly：`qim-bench` 用 Node 跑 `web/bench/latency.ts <資料包> [<詞庫資料包>]`，它載入網頁同一份綁定 `web/src/wasm/`，每鍵計 `letter()` 加讀出網頁畫候選要的欄位（輸入碼、候選），每題之間按 Esc（不計時）。`cargo xtask bench` 先以 release 重編綁定（同〈網頁〉第 1 步）再跑基準，所以要有 `wasm-bindgen-cli` 與 Node；直接 `cargo run -p qim-bench` 時綁定不存在就失敗。
- 為什麼用 Node 不用瀏覽器、兩者差多少：`scripts/experiments/wasm-latency-browser/README.md`。
- 百分位數取 nearest-rank（排序後第 ⌈p/100 × n⌉ 個）。macOS 上原生與 Node 的計時解析度約 0.04 µs，現在的核心每鍵多在這個量級，p50 印 0.00 表示比解析度還快。

改核心的票貼前後對照時，在同一台機器、不跑其他重負載時跑；差距小於幾成多半是雜訊，重跑兩三次看是否穩定。

`--check` 的違規每條一行，最後一行 `bench 通過` 或 `bench 失敗`：固定題沒過（附期望與實際首選）、集合答錯數或修正次數高於 baseline、集合有題打不出、集合沒有 baseline（新語料要先 `--update-baseline`）、baseline 的集合這次沒跑到。`--update-baseline` 讀不了舊 baseline（例如格式改了）時直接覆寫。邏輯在 `crates/qim-bench/`。

## 網頁

`cargo xtask web` 做兩件事，邏輯在 `xtask/src/web.rs`：

1. 用 release 編 `qim-wasm` 到 `wasm32-unknown-unknown`，`wasm-bindgen --target web` 產生到 `web/src/wasm/`，`web/src/main.ts` 直接 import。
2. 把 `build/packs/`、`build/layouts/` 整個複製到 `web/public/packs/`、`web/public/layouts/`（先清掉舊的），網頁 fetch `/packs/quick/quick.pack`、`/packs/quick-lexicon/quick-lexicon.pack` 與觸控鍵盤的 `/layouts/gboard-quick.json`。有一個不存在就失敗，先跑 `cargo xtask data`。

這些目錄都不進 git。改了 `qim-core`、`qim-layout` 或 `qim-wasm` 要重跑才看得到；`check` 的 `wasm` 步驟只做第 1 步，`e2e` 步驟先做第 2 步再跑 Playwright。

Playwright 的測試在 `web/e2e/`，截圖存在 `web/test-results/`（每次覆寫、不進 git）。`web/bench/` 是用 Node 跑的基準腳本（〈首選字基準〉的〈每鍵延遲〉），有自己的 `tsconfig.json`（多了 Node 的型別），不進網頁。

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
| `tsc` | `web/` 的 `tsc --noEmit`，再對 `web/bench/` 跑一次（`npm run typecheck`） | 跑 |
| `biome` | `web/` 的 `biome check`（`npm run lint`） | 跑 |
| `e2e` | 複製資料包（〈網頁〉第 2 步）後跑 Playwright（`npm run e2e`）；要先跑過 `cargo xtask data` | 跳過 |

`cargo xtask check --fast` 是 pre-commit hook 跑的快速版，跳過 `test` 與 `e2e`：測試會隨碼表與性質測試變慢，e2e 要起瀏覽器；其他步驟增量執行幾秒內完成。`clippy --all-targets` 仍會編譯測試碼，所以測試編不過也會被擋。

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
