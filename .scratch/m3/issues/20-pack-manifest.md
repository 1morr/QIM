# 20: 資料包 manifest 與設定頁的資料包管理

Status: done
Blocked by: 01, 02
讀: plan §1.2、§4；brief §5.1、§5.3、§10；spec〈範圍〉〈共同規則〉

## What to build

每份資料包帶一份 manifest；網頁設定頁看得到裝了哪些資料包，可切換、刪除，也可從檔案匯入一份。

- manifest：id、種類、版本、最低核心版本、sha256、大小、授權（spec〈範圍〉）。這是對外格式，先看成熟產品（Rime 的 plum / `rime-install`、Gboard 的語言包、Mozc 的字典）定欄位與檔名，註明採用誰的慣例，寫回 plan §4；`qim-databuild` 產生，`--verify` 檢查。
- 核心載入時檢查最低核心版本與 sha256，不合時拒絕並給明確錯誤。
- 網頁設定頁：列出資料包（名稱、版本、大小、授權）、切換（例如換一份詞庫）、刪除、從檔案匯入；網頁存在瀏覽器裡（儲存方式在票內定）。

## 驗收

- [x] 每份資料包有 manifest；改壞 sha256 或要求更新的核心版本時載入被拒（測試）。
- [x] Playwright：從檔案匯入一份資料包、切換、刪除，每步讀回候選有變；截圖。
- [x] plan §4 寫明 manifest 格式；`cargo xtask check` 綠燈。

## Comments

### manifest 格式與採用的慣例（2026-11-11，M3 票 20）

查證在 `docs/research/pack-manifest.md`，格式寫在 plan §4.6。每份資料包目錄多一個 `manifest.json`：`manifest_version`（WebExtension 的同名欄位）、`id`（Cargo `name` 的字元限制）、`kind`、`version`（semver，同 Cargo / npm）、`min_core_version`（單一下限，同 Cargo `rust-version`、Firefox `strict_min_version`）、`size`、`sha256`（小寫 hex，同 Cargo `cksum`、Homebrew、Flatpak）、`license`（SPDX 運算式，同 npm）。不認得的欄位略過。

取捨：
- 研究建議把 manifest 塞進 `.pack` 檔頭、讓單檔就能匯入；沒照做：要改資料包格式、七份 sha256 全換，檔內也放不了自己的 sha256。匯入改成一起選 `manifest.json` 與 `.pack`（`<input multiple>`）。
- `version` 現在七份都是 `1.0.0`，寫在 `pipeline.rs` 的 `build_packs`，靠人在內容改了時加。沒有自動擋：`--verify` 不知道上一版的 sha256。
- `min_core_version` 寫建它的核心版本（`qim-core` 的 crate 版本，現在 0.0.0），比較保守：比較舊、其實也讀得了的核心會被擋。
- `sources.toml` 的 libcangjie `license` 由 `Public Domain` 改成 `LicenseRef-Public-Domain`，manifest 的 `license` 才是合格的 SPDX 運算式。只改了 LICENSE、NOTICE 裡的字，資料包的位元組不變（七份 sha256 和改之前相同）。

### 切換：方案指名的是預設

票要「換一份詞庫」，可是 `Engine::new` 要求詞庫、語言模型剛好是方案指名的 id，`RadicalHints::for_scheme` 也一樣，換不了。改成只核對方案要不要那種資料包、不核對 id（M1 的決定被推翻，記在 progress〈偏差與決定〉）。建出的預設那份是不是方案指名的，改由 `--verify` 的「解得開」那項擋，原本的變異測試（`undecodable_pack_fails_without_panicking` 裡 id 不對的三組）照樣紅。已知的代價：別的方案的詞庫（碼不是速成碼）也接得上，只是打不出東西；要擋的話 manifest 之後可以加一個 `scheme` 欄位。

### 網頁

- 儲存：匯入的存 IndexedDB（`idb-keyval` 6.3.0，plan §1.4），每種用哪份存 `localStorage`。`localStorage` 每個網域只有約 5 MB，而且只能存字串，詞庫一份就 5 MB，所以不用它。
- 和內建同 id 的匯入會取代內建那份，刪掉就回到內建的；內建的不能刪。匯入後那一種就改用它。
- 換資料包時 `Session.setPacks` 只換 `Engine`，學習資料、開關與組字都留著，組字照新的資料包重列。匯入、刪除先在記憶體換，核心收下了才寫進 IndexedDB；換不了就退回原本的樣子。
- `qim-ffi` 沒改，殼還是只交位元組；M6 起平台殼接資料包管理時，改走 `Pack::open`（plan §4.6）。
- 亂按一輪的做法：組字中匯入、連切 10 次、開簡體、Tab 叫字根提示、組字中刪掉正在用的詞庫、觸控模式下連續匯入兩次、重新整理。console 沒有錯誤。有一處起初看起來像 bug：刪掉匯入的詞庫之後，`anr` 還是出「测试词库」。查下來是前面上屏過「测试词库」，它學成了 `anr` 的使用者詞，不是 bug。「組字中切換、刪除，候選列立刻換」補成正式的 e2e。

### 留給票 21

- 資料包目錄有 `manifest.json`、`.pack`、`LICENSE`、`NOTICE` 四個檔。release asset 的名字不能重複，七份的 `LICENSE` 會撞名，所以要選一種放法：每份打包成一個檔，或把 asset 改名、加上 id 前綴。安裝一律走 `PackLibrary` 的「manifest 加位元組」：現在的 `import(files)` 可以拆出一個收 `(manifest, bytes)` 的入口給下載用。
- 檢查「有新版」要比 `version`（semver）。內容改了卻沒加 `version` 的情形，可以在產生索引時擋：拿上一版索引比，sha256 變了而 `version` 沒變就失敗。

### 首選字基準（`cargo xtask bench --check`）

核心改了 `Engine::new` 的核對、加了 manifest，排序沒改。前後各集合的答錯、修正數都和 baseline（`c1ffdea`）相同，所有集合都沒有翻盤，`--check` exit 0，七份資料包的 sha256 也和改之前相同。WebAssembly 每鍵延遲改成經 manifest 載入（`web/bench/latency.ts`），照樣跑完：全部 p50 57 µs、p95 151 µs。沒動到學習，所以沒跑 `--learning`。code-review 的修正做完後重跑了一次，結果還是相同。

依長度分組（整句首選正確率 · 修正/句（題數））：在暫存 worktree 用開工時的 `69b7a2e` 跑了一次，資料包是同一批位元組，和現在逐格相同。

| 集合 | 1 字 | 2 字 | 3–4 字 | 5–8 字 | 9 字以上 |
| --- | ---: | ---: | ---: | ---: | ---: |
| yue-wiki | 0.00% · 2.50（4） | 36.00% · 1.14（50） | 47.25% · 1.05（273） | 49.84% · 1.33（628） | 39.14% · 3.03（1045） |
| zh-wiki | 0.00% · 2.33（3） | 33.33% · 1.57（30） | 47.11% · 1.09（242） | 52.13% · 1.27（587） | 39.02% · 2.92（1138） |
| common-voice-yue | 42.86% · 0.86（7） | 65.96% · 0.40（47） | 51.27% · 1.18（275） | 33.89% · 2.08（661） | 33.08% · 3.40（659） |
| hkcancor | 37.50% · 0.88（8） | 53.49% · 0.53（43） | 42.73% · 1.33（220） | 33.50% · 1.98（394） | 22.84% · 4.61（324） |
| yue-wiki·前文 | 0.00% · 2.33（3） | 37.50% · 0.94（16） | 52.80% · 0.95（125） | 46.99% · 1.44（332） | 37.40% · 3.20（524） |
| yue-wiki·前文·不給 | 0.00% · 2.33（3） | 43.75% · 0.94（16） | 52.00% · 0.96（125） | 46.99% · 1.46（332） | 37.79% · 3.21（524） |
| zh-wiki·前文 | 0.00% · 5.00（1） | 26.32% · 1.63（19） | 50.41% · 1.07（121） | 53.40% · 1.27（294） | 38.58% · 2.88（565） |
| zh-wiki·前文·不給 | 0.00% · 5.00（1） | 31.58% · 1.58（19） | 50.41% · 1.06（121） | 52.72% · 1.32（294） | 38.76% · 2.90（565） |
| common-voice-yue·前文 | 50.00% · 0.50（4） | 50.00% · 0.60（20） | 49.33% · 1.25（75） | 34.72% · 2.14（144） | 33.33% · 2.73（84） |
| common-voice-yue·前文·不給 | 75.00% · 0.25（4） | 50.00% · 0.60（20） | 45.33% · 1.29（75） | 32.64% · 2.20（144） | 33.33% · 2.74（84） |
| hkcancor·前文 | 50.00% · 0.67（6） | 55.17% · 0.52（29） | 47.40% · 1.18（154） | 32.96% · 2.04（270） | 26.36% · 4.37（239） |
| hkcancor·前文·不給 | 50.00% · 0.50（6） | 55.17% · 0.52（29） | 44.81% · 1.23（154） | 31.48% · 2.06（270） | 25.10% · 4.41（239） |


| 集合 | 答錯 | baseline 答錯 | 修正 | baseline 修正 |
| --- | ---: | ---: | ---: | ---: |
| yue-wiki | 1131 | 1131 | 4352 | 4352 |
| zh-wiki | 1126 | 1126 | 4385 | 4385 |
| common-voice-yue | 1032 | 1032 | 3966 | 3966 |
| hkcancor | 663 | 663 | 2596 | 2596 |
| 有前文四集（給／不給）合計 | 1789／1803 | 1789／1803 | 6908／6975 | 6908／6975 |

### code-review（從 `69b7a2e`，Standards 與 Spec 兩軸）

已處理：
- 依種類分派、組 `Engine` 的邏輯原本在 `qim-wasm` 與 `verify.rs` 各寫一份，違反「綁定不放邏輯」（plan §1.3）。改成核心的 `Engine::open(&[(&Manifest, &[u8])])` 與 `Manifest::verify`，並補上同種兩份、給了字根提示、沒有方案資料包時的拒絕測試（`crates/qim-core/tests/engine_open.rs`）。
- 退回邏輯有漏洞：匯入一份和內建同 id、種類卻不同的資料包時，它會取代內建那份；核心不收時退不回去，IndexedDB 裡卻已經存了，重新整理後還會卡在「載入中」。改成三件事：
  - 同 id 不同種類的匯入直接拒絕（`importConflict`）。
  - 匯入、刪除都等核心收下了才寫進 IndexedDB。
  - 載入失敗時這次只用內建的，狀態列說明原因。
  三種情況都補了 e2e。
- sha256 不是 64 位小寫十六進位時，現在報 manifest 不合格式，不再報成「檔案損壞」。
- `PackKind::as_str` 和 serde 的寫法同一份對照寫兩次，補測試讓兩邊對得上。TS 端的種類名稱集中到 `pack-choice.ts`。`main.ts` 原本靠比對「載入中…」字串清狀態列，改成用旗標。plan §0 的識別符表補上 `Manifest`、`PackKind`、`active`。
- 補「切換」那步的截圖（`20-switched.png`）、bench 依長度分組的前後對照，progress.md 也已更新。

沒處理，留著：
- `manifest.rs` 的 `sha256_hex` 和 `qim-databuild/src/sources.rs` 那份重複。databuild 有一堆來源檔要算雜湊，為此讓核心公開一個雜湊函式不值得。
- `pipeline.rs` 的 `"1.0.0"` 七份各寫一次：版本本來就是每份各自的，之後內容改了只加那一份。
- `Manifest` 的 `version`、`min_core_version` 存成字串，載入時才解析：manifest 是讀進來的外部資料，存原字串，錯誤訊息才能照原樣引用。
- `web/bench/latency.ts` 另外寫死 `"manifest.json"`：它是 Node 腳本，不 import 網頁的模組。
