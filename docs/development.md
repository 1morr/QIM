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
| `cargo xtask data [--verify]` | 建資料包到 `build/packs/`，見〈資料建置〉 |
| `cargo xtask bench [--check \| --update-baseline]` | 首選字基準，見〈首選字基準〉 |

## 資料建置

```sh
cargo xtask data            # 建資料包
cargo xtask data --verify   # 建資料包並跑 plan §4.5 的檢查，有一項失敗就 exit 非零
```

`cargo xtask data` 用 release 編 `qim-databuild` 並在 repo 根目錄執行，步驟見 plan §4：

- 來源列在 `data/sources.toml`：每個來源釘 `version`（git 來源填 commit，其他填上游版本號）與每個檔的 sha256。下載到 `.cache/sources/<來源>/<sha256 前 16 碼>/<檔案路徑>`；已有快取就不連網。快取或下載的檔 sha256 不符時建置失敗、不重新下載：確認來源後刪掉那個檔重跑，或更新 `sources.toml`。換來源版本時改 version、url 與 sha256，快取路徑跟著換。
- 進資料包的來源要寫授權，二選一，都寫或都不寫時建置失敗：
  - `license_file` 加 `license_statement`：照抄上游檔裡的授權聲明，建置時核對它確實在 `license_file` 裡（空白與換行不計），再寫進資料包的 `LICENSE`（libcangjie）。
  - `license_text`：repo 裡的授權全文檔，整份寫進 `LICENSE`。上游授權檔沒有版本化網址、會改內容時用，例如 Unicode 的 `license.txt` 每年改版權年份，釘 sha256 會讓乾淨建置過期失敗；全文放 `data/licenses/`（Unihan 的 `kGradeLevel` 進速成資料包，用 `data/licenses/Unicode-3.0.txt`）。
- 方案檔在 `data/schemes/`，由 `qim-databuild` 編進資料包；`qim-core` 不讀 TOML。
- 產物：`build/packs/<方案 id>/` 下的 `<方案 id>.pack`、`LICENSE`、`NOTICE`。資料包格式定義在 `crates/qim-core/src/pack.rs`，改格式時加 `FORMAT_VERSION`。
- 語料（plan §4.3）：粵語維基 dump 經抽內文、`s2hk`、切句、切分，寫出 `bench/corpus/<名>.tsv`、`bench/corpus/ATTRIBUTION.md`（進 git）與 `build/corpus/<名>/train.txt`、`dev.txt`（一行一句，不進 git）。第一次要下載約 130 MB 的 dump；之後 `cargo xtask data` 約 30 秒，`--verify` 把語料再建一次比 sha256，約 1 分鐘（Apple M6，release）。

`--verify` 的報告列出連打字集字數、擴展區字數、速成碼個數、資料包 sha256、Unihan 交叉比對的摘要、各語料的句數，再逐項列 `[通過]` 或 `[失敗]`，最後一行 `verify 通過` 或 `verify 失敗`。檢查定義在 `crates/qim-databuild/src/verify.rs`。

### 語料

`bench/corpus/<名>.tsv` 是首選字基準的題目，由建置產生、不要手改。UTF-8，沒有欄名，一行一句：

```text
<逐字速成碼，空格分隔><TAB><句子>
```

例如 `an r⇥門口`（`⇥` 是 tab）。輸入碼是把各字的碼接起來；分開寫是讓基準知道每個字用了幾個鍵。轉碼由 `qim_databuild::transcode::Transcoder` 做：每個字取第一個五代碼的速成碼，沒有五代碼才取三代碼，連打字集以外的字轉碼失敗。

`--verify` 對每個語料多三項檢查：基準句與 train、dev 每句都轉得了碼（基準句的碼也要等於轉碼結果）、基準句沒有一句原樣出現在 train、建兩次 sha256 相同。切句與取樣規則見 plan §4.3，抽 wikitext 內文的規則在 `crates/qim-databuild/src/wikitext.rs`。

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

`.cache/` 與 `build/` 不進 git。

## 首選字基準

改核心或資料包時必跑（plan §5.2），不在 `cargo xtask check` 裡。先跑過 `cargo xtask data`：基準讀 `build/packs/quick/quick.pack`，沒有就失敗。

```sh
cargo xtask bench                     # 印結果
cargo xtask bench --check             # 固定題全過、各集合答錯數 ≤ baseline，否則 exit 非零
cargo xtask bench --update-baseline   # 用這次的答錯數改寫 bench/baseline.json
```

`cargo xtask bench` 用 release 編 `qim-bench` 並在 repo 根目錄執行。每題用新的 `Session` 與空白的 `UserModel`，經 `Session::handle` 逐鍵打完輸入碼，取當下的第一個候選當首選；沒有候選時首選是空字串。

| 檔案 | 格式 |
| --- | --- |
| `bench/fixed.tsv` | 固定必過題，一行 `輸入碼<TAB>期望首選<TAB>來源`；空行與 `#` 開頭的行略過。案例通過的那張票才加，不先加會紅的題 |
| `bench/corpus/<名>.tsv` | 一個檔一個集合，集合名是檔名；格式見〈資料建置〉的〈語料〉 |
| `bench/baseline.json` | `{"<集合名>": {"wrong": <答錯數>}}`，固定題不記。只用 `--update-baseline` 改，同一個 commit 說明原因 |

結果是一個集合一列的 Markdown 表，可以直接貼進票的 `## Comments` 或 commit body 做前後對照：

- 整句首選正確：首選與期望完全相同的題數與比例；答錯 = 題數 − 整句首選正確。
- 字正確率：1 − Σ 編輯距離 ÷ Σ 期望字數；編輯距離以字計（Levenshtein），每題最多算到該題的字數。
- `baseline 答錯`：跑之前 `bench/baseline.json` 的值，沒有檔或沒有該集合時是 `—`。

`--check` 的違規每條一行，最後一行 `bench 通過` 或 `bench 失敗`：固定題沒過（附期望與實際首選）、集合答錯數高於 baseline、集合沒有 baseline（新語料要先 `--update-baseline`）、baseline 的集合這次沒跑到。邏輯在 `crates/qim-bench/`。

## 網頁

`cargo xtask web` 做兩件事，邏輯在 `xtask/src/web.rs`：

1. 用 release 編 `qim-wasm` 到 `wasm32-unknown-unknown`，`wasm-bindgen --target web` 產生到 `web/src/wasm/`，`web/src/main.ts` 直接 import。
2. 把 `build/packs/` 整個複製到 `web/public/packs/`（先清掉舊的），網頁 fetch `/packs/quick/quick.pack`。`build/packs/` 不存在時失敗，先跑 `cargo xtask data`。

兩個目錄都不進 git。改了 `qim-core` 或 `qim-wasm` 要重跑才看得到；`check` 的 `wasm` 步驟只做第 1 步，`e2e` 步驟先做第 2 步再跑 Playwright。

Playwright 的測試在 `web/e2e/`，截圖存在 `web/test-results/`（每次覆寫、不進 git）。

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
| `tsc` | `web/` 的 `tsc --noEmit`（`npm run typecheck`） | 跑 |
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
