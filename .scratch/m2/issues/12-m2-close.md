# 12: M2 收尾

Status: done
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11
讀: spec〈驗收〉；`docs/progress.md`〈寫法與封存〉；plan §7、§8；`docs/development.md`〈全部檢查〉；`docs/research/ios-feasibility.md`；票 01–11 的 `## Comments`

## What to build

M2 的驗收全部重驗並留下證據，四個問題的結論寫回 brief 與 plan，然後封存 M2、M3 標進行中。

- 里程碑 code-review：`/code-review <M2 開工時的 commit>`，Spec 軸讀 spec 與票 01–11，Standards 軸讀 `CLAUDE.md`。小問題修掉；沒修的與各票 Comments 沒處理的發現，整理成清單寫在本票 `## Comments`，每條註明留到哪個里程碑或不修的理由。
- 結論寫回：brief §9（記憶體目標從暫定改成實測值，註明只量了 iPhone 18 Pro Max）、§11（免費佈建那條結案）、§12.3；plan §7.8 的前置與模型上限、§8 各相關風險列的狀態。
- 在乾淨 checkout（`git worktree add --detach` 到暫存目錄，照 `docs/development.md` 從零安裝）跑 spec〈驗收〉能在 Mac 上重跑的每一項；iPhone 上的項目用乾淨 checkout 建出的擴充至少重打一次「門口」。
- 封存（〈寫法與封存〉，同一個 commit）：M2 的 session 紀錄與偏差移到 `docs/archive/M2.md`；里程碑表 M2 連到封存檔、M3 進行中；plan 開頭狀態行與 §7 表更新，§7.3 指向 spec、封存檔與本票；本 session 的紀錄標 `M3`，下一步寫「新 session 下 `/to-tickets docs/plan.md 拆 M3 的票`」。

## 驗收

- [x] 里程碑 code-review 跑完；沒處理的發現清單寫在 `## Comments`，每條有去處或理由。
- [x] spec〈驗收〉每一項成立，輸出與截圖路徑貼在 `## Comments`。
- [x] brief §9、§11、§12.3 與 plan §7.8、§8 已寫回。
- [x] `docs/archive/M2.md` 有 M2 全部 session 紀錄與偏差；progress 只留 M3；里程碑表與 plan §7 已更新；同一個 commit。
- [x] 封存後 `cargo xtask check` 綠燈。

## Comments

- 2026-11-10 開工：M2 開工的 commit 是 `17727d9`（spec 與票）；本票從 `581b7f7` 起。`bench/personal` 用軟連結接主 checkout 的個人集。
- 2026-11-10 里程碑 code-review（`17727d9...581b7f7`，diff 144 檔、+8910／−893；Standards 軸讀 `CLAUDE.md`、plan §0–§2.1、§6、`GLOSSARY.md`、spec〈共同規則〉、`docs/development.md`，Spec 軸讀 spec 與票 01–11；兩軸各分產品碼與實驗／文件兩半，共四個審查）：沒有實作錯誤，plan §1.2–§1.4 的表、`RULES` 與各 crate 的依賴一致。已處理（`39d103a`）：
  - `docs/research/ios-feasibility.md`〈小型神經模型〉的結論還是票 11 的「23M 兩種都放得進、f32 剩約 18 MB」，被票 09 推翻卻沒改（兩軸都指出）：改成 23M 只能 q8、f32 放不下，合計改用六份資料包的 79 MB；〈Mac 上的資料包記憶體〉用 Mac 推估的 106 MB 註明 iPhone 實測值。
  - 過時的 doc comment：`Key::parse` 只寫網頁（Swift 也用）；`Output::ToggleAscii` 寫死「先上屏」（實際依 `ShiftSwitch`）。`qim-ffi` 的錯誤轉換 `error` 改名 `core_error`，對齊 `qim-wasm` 的 `js_error`。
  - `docs/research/platform-frameworks.md` 的 `os_proc_available_memory` 還寫「未查證」，補實測與連結；`GLOSSARY.md`〈資料包〉補英文詞表與字根提示；六處〈接上核心〉改成 README 的實際標題〈接上核心（票 08）〉。
- 2026-11-10 沒處理的發現（含票 01–11 Comments 留下的），依去處：
  - M3（智慧候選）：
    - brief §8 的「游標前 200 字」在 iOS 常拿不到（備忘錄最多 300 字、Safari 100 字，測試文字只讀到一兩句）；要不要自己記住這次編輯上屏過的字補上下文，M3 定（票 10，plan §7.8）。
    - Foundation Models 停頓後第一個請求 12 次裡 4 次要 6–15 s，`prewarm()` 擋不住、原因沒查出；M3 若用它當停鍵層，模型階梯實驗要量這個尾巴（票 10）。
  - M6 起（`qim-ffi` 完整版、桌面殼）：
    - FFI 還沒匯出 `set_options`；加的時候讓綁定的 `set_options` 同時重列並回快照，去掉「待生效選項」狀態（票 01 → 07，plan §2.1）。
    - `qim-ffi` 與 `qim-wasm` 各寫一份 `Engine::new` 加 `with_simplifier`、`with_english` 的組裝；加第六種資料包時在核心給收 `Option` 的建構式（票 08）。
    - `qim-ffi` 沒有 `pageSize()`，註解寫死 `page × 9`；Swift 殼要分頁時補上，與 `qim-wasm` 一致（code-review）。
    - `UserModel` 暫時跟著各 `Session`，換輸入框就是新的學習資料；學習資料存檔與共用由需要的票拆（票 08，plan §2.1）。`RadicalHintPack` 只能解開，查詢介面留給要顯示字根提示的票（票 09）。
  - M7（iOS）：
    - 鍵盤擴充略過 Schema 驗證的實作（`qim-layout` 加 feature，票 09 已決定，plan §1.3）。
    - `qim_layout::Keyboard` 把換鍵盤併成 `Output::Modifier`，殼分不出「這下要換鍵盤」，得自己再查鍵的動作；plan §2.6 認可現狀，寫 iOS 殼時重看要不要讓 `Output` 帶出來（code-review）。
    - 低階機的記憶體上限、每鍵延遲沒量，只有 iPhone 18 Pro Max（brief §9、plan §7.8、§8）。
    - 實驗的 Swift（`KeyboardViewController.render()` 的探針條件、各探針重抄 `onUpdate`／`displayLines`、字串前綴表示結果、`Probe` 職責混合）不沿用到產品殼（票 06、09、10）。
  - 不修，選定的語義或代價：
    - `qim-wasm` 的 `inputChar` 是 `press` 以外的第二個按鍵入口（Spec）：它照輸入碼重打（`Key::for_input`：分隔記號送標點鍵），只給延遲量測用；改走 `press` 要把這個判斷搬到 JS，反而讓綁定外多一份邏輯。
    - 票 01 守 `text` 的測試修之前就綠（守門用，找不到會紅的情境，票 01 已說明）。
    - `web/bench/latency.ts` 沒有單元測試（票 05 轉來）：型別由 `tsc -p bench` 守，行為由 `qim-bench` 比對 WebAssembly 與原生量到的輸入數守（票 07 加的，不同時基準失敗）。
    - `ShiftSwitch::from_str` 的錯誤是 `String`、`Key::parse` 是 `KeyError`：同 `Device`、`CangjieVersion` 的既有慣例（票 03）。
    - `pack-memory`、`neural-cost` 的 README 沒寫 Xcode 版本：純 Rust、只用系統的連結器，Rust 版本與 macOS 版本已寫（code-review）。
    - 同一個數字抄在 ios-feasibility、brief §12.3、plan、README 各處（spec〈共同規則〉要的分工）；`swiftc` 的連結參數在 `docs/development.md` 與 `ffi-swift/run.sh` 各一份（文件給人看、腳本給機器跑）。
    - 票 01–11 Comments 列的其餘判斷題（`for_scheme` 五處、`refresh` 與 `apply_pending_options` 相似、`options_pending` 命名、`CandidateSpan.len`、中文錯誤訊息、`Output::kind` 與 `Action::name`、JS 的 `Layout` 名、`Timing` 平行陣列、選段規則與模型檔名多處重複、`xtask/src/ffi.rs` 沒測試、`Mutex` 中毒照用、`by_frequency` 峰值沒單獨量、README 不能寫含自己的 commit）照各票的理由不改。
- 2026-11-10 重驗在乾淨 checkout：`git worktree add --detach` 到 session 暫存目錄（`581b7f7`），`build`、`target`、`web/node_modules`、`bench/personal` 都不存在；照 `docs/development.md`〈安裝〉做 `npm ci`、`playwright install chromium`（wasm-bindgen 0.2.130 已裝）、`cargo xtask data`（共用快取）、`cargo xtask check`；`bench/personal` 用 `ln -s` 接主 checkout 的個人集。spec〈驗收〉逐項：
  - 四個問題：數字都在 `docs/research/ios-feasibility.md`，摘進 brief §9、§11、§12.3（本票寫回 §9、§11，§12.3 由各票寫好、本票核對），都註明 iPhone 18 Pro Max、iOS 27.2。Mac 上能重跑的兩個在乾淨 checkout 重量，和研究文件一致：`pack-memory`（`RUNS=1`）不驗證 footprint 95.80 MB、Schema 驗證 98.36 MB、執行檔 0.67 / 3.76 MB、單獨建 `HashSet` +17.27 MB；`ffi-swift` 與 Chromium 延遲見下。
  - iPhone 打 `anr`：乾淨 checkout 照 ios-keyboard README〈建置與安裝〉建（`Local.xcconfig` 填這台 Mac 的 Personal Team、前綴 `com.personal.qim`，`core.sh`、`xcodebuild` `BUILD SUCCEEDED`、`devicectl` 安裝），備忘錄切到 QIM 實驗鍵盤按「載入」（六份資料包後 footprint 79.28 MB、131 ms），按 a、n、r 讀回候選列「anr 1.門口 2.日名 3.日阿 …」，空白後備忘錄讀回「門口」。截圖 `.scratch/m2/evidence/12-clean-anr.png`、`12-clean-commit.png`（候選列那行被鍵蓋住，首選以讀回的文字為準）。手機上留下一則「門口」的備忘錄，沒刪。
  - `qim-ffi`：`cargo xtask check` 的 `deps` 綠燈；`scripts/experiments/ffi-swift/run.sh` 的 demo 印「按 r：輸入碼 anr，首選 門口」「按空白：上屏 門口」，exit 0。這次的 Swift p95 122.7 µs（票 07 是 58 µs），同時有別的編譯在跑，只當「遠低於 50 ms」的確認。
  - 兩個決定寫回 plan：資料包格式不改、鍵盤擴充略過 Schema（§1.3、§1.4、§4、§8）、M3 模型峰值增量 ≤ 60 MB（§7.8），票 09 已寫；本票補 §7.8 前置、§8 免費佈建與延遲兩列的狀態。
  - 清理票之後：`qim-wasm` 只轉型別（Spec 審查讀過確認）；`set_options` 由 `simplified.rs`、`english.rs` 的測試守住。`cargo xtask bench --check`：`bench 通過`、固定題 84/84、五個集合都沒翻盤，答錯／修正 yue-wiki 1131 / 4467、zh-wiki 1126 / 4505、common-voice-yue 1032 / 4043、hkcancor 663 / 2588、個人集 1081 / 1433，逐格等於 baseline 與票 01 在 `17727d9` 量的「修之前」（M1 封存後的補票改過詞庫，所以和票 24 的數字不同；`bench/baseline.json` M2 沒動）。每鍵延遲原生 p95 52.88 µs、WebAssembly（Node）58.96 µs，選開頭一段 0.81 / 0.94 ms。Chromium（`compare.mjs`）p95 60 µs、選開頭一段 0.96 ms。
  - `cargo xtask data --verify`：`verify 通過`，速成 sha256 `ae3bc207ce7a…`、字根提示 `9e755488f431…`、簡繁 `1c5ab2aa21e6…`、英文 `1afd47942b69…`，詞庫 `08bc8adbe5d3…`、bigram `9e393a0a457d…`（M1 封存後的補票改了詞庫與語料），建兩次相同。
  - GitHub Actions：`gh run list` 最近三次（票 08、11、09 合併後）都 success，最新是 run 42835522049。
  - `cargo xtask check`（乾淨 checkout，`581b7f7`）：`==> fmt` … `==> deps`、`==> docs`、`==> wasm`、`==> tsc`、`==> biome`、`==> unit`、`==> e2e`、`147 passed (31.2s)`、`check 通過`，exit 0。
- 2026-11-10 封存：M2 的 25 列 session 紀錄與 26 條偏差移到 `docs/archive/M2.md`；里程碑表 M2 連到封存檔、M3 進行中（拆票前讀加上本票 Comments 與 ios-feasibility）；plan 開頭的狀態行、§7 表 M2 完成、M3 進行中，§7.3 指向 spec、封存檔、研究文件與本票。
