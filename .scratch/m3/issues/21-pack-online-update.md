# 21: 資料包線上更新

Status: done
Blocked by: 20
讀: plan §4（manifest 在 §4.6）；brief §1.5、§5.3、§8；票 20 的 Comments；spec〈範圍〉〈共同規則〉

## What to build

網頁可以檢查有沒有新版資料包、下載、核對 sha256 後換上；可以關掉。

- 更新來源是 GitHub Releases（`1morr/QIM`，使用者 2026-11-11 決定）：定 release 上放什麼（一份索引列出各資料包的 manifest 與下載網址），寫回 plan §4。
- M3 用本機靜態伺服器扮演 release 跑完整流程；`xtask` 或腳本產生那份索引。真的在 GitHub 建 release 前先問使用者，不同意就只留本機流程並註明。
- 核心打字不依賴網路（brief §8）：更新在背景，失敗或斷網時照用舊的；設定區有開關。

## 驗收

- [x] Playwright：本機伺服器放一份新版資料包，網頁檢查到、下載、換上，讀回候選變了；sha256 不符時拒絕並保留舊版；關掉開關時不連線（攔請求確認）。
- [x] 斷網（攔掉請求）時打字照常。
- [x] plan §4、`docs/development.md` 已更新；`cargo xtask check` 綠燈。

## Comments

### release 上放什麼（2026-11-11，M3 票 21）

格式寫在 plan §4.7。release 上每份資料包四個 asset：`<id>.pack`、`<id>.manifest.json`、`<id>.LICENSE`、`<id>.NOTICE`（票 20 留的撞名問題，用 id 前綴解；不打包成 zip），加一份更新索引 `index.json`：`{"index_version": 1, "packs": [manifest 欄位攤平 + "url"]}`。慣例照 `docs/research/pack-manifest.md`：外部索引記 sha256，同 Cargo index、Homebrew、Flatpak；一項原樣就是一份合格的 manifest。`url` 相對於索引的網址，所以同一份索引放本機伺服器或 GitHub 都能用。

- 格式與規則放核心（`pack_index.rs` 的 `PackIndex`）：讀索引、`updates`（裝著的、同 id 同種類、版本較大、核心讀得了）、`check_successor`（給建置端）。網頁經 `qim-wasm` 的 `packUpdates`，平台殼之後用同一份。
- 票 20 留的「sha256 變了而 version 沒變」：`cargo xtask pack-release --previous <上一版 index.json>` 擋，版本變小也擋。不給 `--previous` 時不比，因為第一次發佈沒有上一版；這一步還是要人記得做（`docs/development.md` 的發佈步驟有寫）。
- 來源是 tag `packs` 的 release，不用 `releases/latest`：App 的 release（brief §10 的 APK）也會發在同一個 repo，`latest` 會被它搶走（Spec 審查的發現）。
- 沒在 GitHub 建 release，也沒對 GitHub 做任何寫入：使用者不在場，協調者只授權推 `main`。M3 只驗本機流程，plan §4.7 與 `development.md` 都寫了「建 release 前先問使用者」。

### 網頁

- GitHub 的 release asset 不帶 CORS 標頭：2026-11-11 用 curl 帶 `Origin` 實測，github.com 的 302 和 release-assets.githubusercontent.com 的 200 都沒有 `Access-Control-Allow-Origin`，已補進 brief §12.4。所以網頁向自己的 `/pack-updates/` 要，`web/vite.config.ts` 用 Node 的 `fetch` 轉發，同 plan §3.1 雲端 API 的做法。只收單一段的檔名，不是開放代理。`vite build` 出的靜態頁沒有線上更新。
- 「線上更新」開關預設開（brief §5.3），存 `localStorage`。開著時每次開網頁在背景檢查一次，另有「檢查更新」鍵；關掉就一個請求都不送，鍵也停用。背景那次沒新版時不寫狀態列，免得蓋掉使用者剛做的事的訊息。
- 安裝走票 20 的 `PackLibrary` 那條路，拆成 `import`（改用匯入的那份）與 `update`（不改每種用哪份）兩個入口，共用 `install`（核心核對、核心收下才寫 IndexedDB）。下載的來源欄寫「線上更新」，`PackEntry.builtin` 換成 `origin`。舊的 IndexedDB 紀錄沒有 `origin`，當成匯入的：這是持久化格式，不 migration。
- 「從檔案匯入」也認 `<id>.manifest.json`：從 release 下載的檔可以直接匯入（brief §5.3「與下載走同一條安裝流程」）。
- 逾時：索引 15 秒、每份資料包 120 秒。用 `setTimeout` 加 `AbortController`，不用 `AbortSignal.timeout`，e2e 才能用假時鐘快轉。

### e2e

- Playwright 多起一個本機靜態伺服器（`web/e2e/static-server.ts`，埠是 e2e 的埠加 10000），放 `web/e2e/fixtures/pack-release/`。這個目錄由 `qim-databuild` 的 `tests/release.rs` 用 `release::write` 寫出，就是 `cargo xtask pack-release` 的輸出。最初是在 `web_fixture.rs` 手寫索引，Spec 審查指出「xtask 產生 → 本機伺服器 → 網頁」沒有串起來驗，改成現在這樣。
- 更新的是另一份 id 的 `e2e-update-lexicon`：先從檔案匯入 1.0.0（`anr` 是「舊版詞庫」），伺服器上放 1.1.0（「新版詞庫」）。不拿內建的 `quick-lexicon` 來測，原因是靜態伺服器整套 e2e 共用，放新版內建詞庫的話，並行跑的其他測試也會被更新。
- `pack-update.spec.ts` 共 7 個測試：背景更新、按鍵更新且不改用沒在用的、sha256 不符拒絕（在用的與沒在用的都測）、關掉不連線（攔請求確認一個都沒有）、斷網、不回應與逾時、下載中刪掉不裝回來。截圖在 `.scratch/m3/evidence/21-*.png`。
- 變異驗證，以下改動都會讓測試變紅：關掉開關後照樣檢查、下載的不呼叫 `verify()`（只有沒在用的那份會紅：在用的交給核心時會再核對一次）、逾時調到很大、fixture 改一個位元組、說明文字不換行的舊 CSS。

### 亂按一輪

在下載拖慢 1 秒的情況下試了這些：組字中更新完成、下載中刪掉那份、連切開關 6 次、連按「檢查更新」、切簡體、快速打字、觸控模式、手機寬度。console 沒有錯誤。找到兩個問題：
- 下載中刪掉那份，下載完它又被裝回來。先寫成會紅的 e2e，再改成下載完、裝之前再確認那份還在、版本也沒變（`PackLibrary.update` 的 `from`）。
- 手機寬度下資料包區的說明文字被切掉。這是票 20 就有的：整區一起橫捲，說明被撐成表格那麼寬。改成只有表格橫捲，`packs.spec.ts` 的手機寬度測試補了斷言。

### 首選字基準（`cargo xtask bench --check`）

核心只加了更新索引，並把 manifest 的格式核對拆成 `check_format`，排序沒改。各集合的答錯、修正數都和 baseline 相同，全部集合沒有翻盤（所以每一題的結果都相同，依長度分組也逐格相同），`--check` exit 0。這個 worktree 沒有個人集，跳過。審查後的修正動到 `check_format`，又重跑了一次。沒有動到學習，所以沒跑 `--learning`。

| 集合 | 答錯 | baseline 答錯 | 修正 | baseline 修正 |
| --- | ---: | ---: | ---: | ---: |
| yue-wiki | 1123 | 1123 | 4295 | 4295 |
| zh-wiki | 1126 | 1126 | 4381 | 4381 |
| common-voice-yue | 1030 | 1030 | 3908 | 3908 |
| hkcancor | 650 | 650 | 2478 | 2478 |
| 有前文四集（給／不給）合計 | 1780／1793 | 1780／1793 | 6802／6868 | 6802／6868 |

每鍵延遲（WebAssembly）：全部輸入 p50 40 µs、p95 58 µs。

### code-review（從 `0082eb0`，Standards 與 Spec 兩軸）

已處理：
- 名詞表補「線上更新」，「使用中的資料包」改成含下載的那種；plan §0 補 `PackOrigin`、`PackIndex`、`update`。
- `check_format` 原本回傳兩個同型別版本的 tuple，改成具名的 `Versions`。
- 「下載期間那份還在不在」的判斷原本在 `updatePacks` 裡伸進 `library.list()`，搬進 `PackLibrary.update`。`install` 不再拿 `origin` 當行為開關，改由 `import`、`update` 各自決定改不改每種用哪份。
- `qim-wasm` 的 `PackUpdate` 直接包 `IndexEntry`，不另外複製欄位。
- `qim-databuild` 的 `main` 改成一個 match。`build/packs` 改用 `pipeline.rs` 公開的 `PACKS_DIR`，不另寫一份。
- Spec 軸：fixture 改由 `release::write` 寫出；來源改用 tag `packs`；補逾時（不回應時「檢查更新」鍵會一直停用）。

沒處理，留著：
- 下載的新版存在 IndexedDB、取代同 id 的內建那份；之後內建的版本加大時，舊的下載仍然蓋過它（plan §4.7〈已知限制〉）。網頁的內建版本只在開發者重建資料包時才會變，而「列出時比版本」要在純 TS 的 `pack-choice.ts` 比 semver，或再開一個綁定。留給 M6 平台殼的資料包管理一起定（收尾票確認去處）。
- 兩份 `fetchOk`（`packs.ts` 叫人先跑 `cargo xtask web`，`pack-update.ts` 改成有逾時的 `fetchWithin`）錯誤訊息不同，不合併。`vite.config.ts` 與 `e2e/static-server.ts` 都有「單一段檔名」的檢查：一個是網頁的轉發，一個是測試用的伺服器，不為一條 regex 讓它們共用模組。`release.rs` 的 `"LICENSE"`、`"NOTICE"` 和 `packing.rs` 各寫一次。
- `--previous` 要人記得給：發佈步驟寫在 `development.md`。M3 沒有自動發佈的流程，沒地方強制。
