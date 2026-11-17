# 01: 建 ARCHITECTURE.md 的骨架，搬 plan §0、§1

Status: done
Blocked by: 無（原本等 flow-sync 的 CHANGELOG 票；2026-11-18 已做完）
讀: `.scratch/arch-diet/spec.md`；plan §0、§1 全文

## What to build

建根目錄的 `ARCHITECTURE.md`：檔頭的寫法照 project-init 的範本（只寫現況、節不編號、上限 15,000 字），節是〈執行形態〉〈模組地圖〉〈依賴規則〉〈跨模組的資料與契約〉〈主要依賴〉〈測試策略〉〈風險與回寫點〉。把 plan §0〈工程慣例〉與 §1〈架構總覽〉歸位：crate 的分工與依賴規則留下；`deps` 步驟讀的表（`xtask/src/deps.rs` 的測試會讀 plan §1.3）跟著搬，測試改讀新位置。

## 驗收

- [x] `ARCHITECTURE.md` 存在，七個節都有標題；§0、§1 的內容已歸位，plan 這兩節只剩標題與一行指向。
- [x] `cargo test -p xtask` 綠燈（讀依賴規則表的測試改讀 `ARCHITECTURE.md`）。
- [x] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 對照表（plan §0、§1 → 新位置或刪掉的理由）

| plan 的段落 | 去處 |
| --- | --- |
| §0「Rust edition 2024，toolchain 用 stable…」 | 刪：edition 由 `Cargo.toml`（`edition = "2024"`）、toolchain 版本由 `rust-toolchain.toml` 說明；〈主要依賴〉開頭指向它 |
| §0「核心的新功能與修 bug 走 tdd；一個 session 一張票」 | 不搬：`AGENTS.md`〈Skills〉〈每個 session 的流程〉已有同樣的規則；AGENTS.md 不是程式檔，列在下面給使用者 |
| §0「程式碼識別符用英文，對應 GLOSSARY.md 的詞」與對照表 | `ARCHITECTURE.md`〈模組地圖〉底下的〈識別符〉，原樣 |
| §1.1「核心是 Rust 函式庫…C ABI」 | 〈執行形態〉第一句，補上 `qim-wasm`、`qim-ffi` |
| §1.1「Windows 依成熟做法…到 M10 再定」 | 〈執行形態〉，改寫成「未定，傾向…，比較見 roadmap M10」 |
| §1.1 mermaid 圖 | 〈執行形態〉，照 repo 現況改（見下面「給使用者」第 3 條） |
| §1.2 目錄表各列 | 〈模組地圖〉的表；改動見下面「給使用者」第 3 條 |
| §1.2「建置產物不進 git：…」 | 刪：`.gitignore`（`/target/`、`/build/`、`/web/public/packs/`、`/web/public/layouts/`）說明 |
| §1.2「下載的來源放在 repo 外的使用者快取目錄（`QIM_CACHE_DIR`）…各 worktree 共用」 | 刪：`crates/qim-databuild/src/sources.rs`（`CACHE_ENV`）說明；操作說明本來就在 `docs/development.md`〈資料建置〉 |
| §1.3 依賴規則表 | 〈依賴規則〉的表，拿掉票號與 plan 節號；`xtask/src/deps.rs` 的測試改讀這裡 |
| §1.3「強制：`cargo xtask check` 的 `deps` 步驟（M0 票 02 接上）」 | 〈依賴規則〉，拿掉票號 |
| §1.3「白名單在 `RULES`…其他寫法測試會紅」 | 〈依賴規則〉第一條 |
| §1.3「讀 `cargo metadata`：一般與 build 依賴…」 | 〈依賴規則〉第二條；「讀 `cargo metadata`」是實作，由 `xtask/src/deps.rs` 的 `run` 說明 |
| §1.3「workspace 成員不在上表就紅…表上的 crate 還沒建立時跳過」 | 前兩句是〈依賴規則〉第三條；「還沒建立時跳過」刪：`xtask/src/deps.rs` 的 `run` 只走 workspace 成員 |
| §1.3「掃 `qim-core`、`qim-layout`、`qim-neural` 的 `src/`…」 | 掃描範圍與「整合測試可以讀檔」是〈依賴規則〉第四條；斷詞、`{..}` 與 `*` 展開、`SystemTime` 出現就算：刪，`xtask/src/deps.rs` 的測試 `std_paths_in_comments_and_strings_are_green`、`nested_groups_globs_and_submodules_are_red`、`system_time_through_imported_module_is_red` 說明；已知漏洞（`use std as s`）：刪，`xtask/src/deps.rs` 的 `forbidden_uses` 文件註解寫著 |
| §1.3「測試做雙向變異…」 | 刪：`xtask/src/deps.rs` 的 `drift_between_architecture_table_and_rules_is_caught`、`drift_in_forbidden_column_is_caught`、`unrelated_edits_*` 與用 `copy_repo` 的紅綠測試說明 |
| §1.3「鍵盤擴充不做 Schema 驗證…」 | roadmap〈平台里程碑（M7–M11）〉的 M8，併進原本「鍵盤擴充讀佈局時略過 Schema 驗證」那句 |
| §1.4「版本為寫本檔時的 stable，開工時…更新本表」 | 版本：刪，`rust-toolchain.toml`、`Cargo.lock`、`web/package-lock.json` 說明；「加依賴取當時的 stable、在表加一列」留在〈主要依賴〉開頭 |
| §1.4 主要依賴表 | 〈主要依賴〉的表；版本欄刪（理由同上），票號與 plan 節號拿掉；`serde_json` 列的「iOS 鍵盤擴充不帶 `jsonschema`」跟著搬到 roadmap M8；`rustc-hash` 列的「票 08 的實驗也用它」是歷史，刪，實驗的依賴在 `scripts/experiments/*/Cargo.lock` |

### 給使用者決定（spec 共同規則 4、5；照 spec 先留著，交給票 06 的使用者試一起看）

1. 〈識別符〉表約 3,300 字，佔 `ARCHITECTURE.md` 三分之一，範本沒有這一節。建議搬進 `GLOSSARY.md` 各詞條（`GLOSSARY.md` 不在 spec 准動的檔裡，改它也要先給使用者看），空出的字給票 02–05。不決定的話，票 02–05 的契約只剩約 5,000 字可用。
2. §0「核心走 tdd、一個 session 一張票」與 `AGENTS.md` 重複，沒搬。AGENTS.md 不是 spec 規定的「測試或程式檔」，要使用者確認可以這樣刪。
3. plan 與 repo 對不上，`ARCHITECTURE.md` 照 repo 現況寫（`AGENTS.md`〈事實與查證〉：現況以 repo 為準）：`web/editor/` 不存在，拿掉了（位置已寫在 roadmap〈M6 佈局編輯器〉；`AGENTS.md`〈Skills〉還提到它）；`data/` 沒有佈局檔，預設佈局是 `qim-databuild` 算出來的，表與圖都拿掉了「佈局檔」；`qim-neural` 原本不在目錄表與圖裡，補上；`qim-layout` 補「觸控空間模型與實體鍵盤的鄰鍵先驗」（照它的 `lib.rs`）；`platforms/` 寫成「還沒建立」。
4. 鍵盤擴充不做 Schema 驗證的打算搬到 roadmap M8；roadmap 不在 spec 列的四個去處裡，但這段只和還沒開工的 M8 有關（票 03 對 §3 也是這樣搬）。
5. 〈主要依賴〉的「理由」欄原樣留著，沒拆成 ADR：範本要「選它的 ADR」，但新寫 ADR 要先給使用者看草稿。

### Code review

兩軸都看過：Standards 建議的錯誤訊息常數、`ARCHITECTURE.md` 的「未定」寫法、`AGENTS.md`〈文件〉那一行、plan〈風險與回寫點〉指向 §1.3 的舊句都已改；Spec 指出的「加依賴取 stable」那句已留在〈主要依賴〉、`docs/development.md` 的 `deps` 列補上 `qim-neural`。其餘發現是上面「給使用者決定」的五條。plan 其他節裡指向 §1.3、§1.4 的引用（點過去是一行「已搬到」）留給票 06。

- Session 紀錄：2026-11-18 建 `ARCHITECTURE.md` 七節骨架，plan §0、§1 歸位，`deps` 測試改讀新檔，指向已搬節的程式註解與文件引用跟著改
- 決定：〈識別符〉表原樣放進 `ARCHITECTURE.md`〈模組地圖〉，沒搬進 `GLOSSARY.md`（spec 範圍外），列給使用者（Agent 定）
- 決定：〈主要依賴〉保留理由欄、不拆 ADR，因為新 ADR 要先給使用者看草稿（Agent 定）
- 決定：鍵盤擴充不做 Schema 驗證的打算搬到 roadmap M8，不留在 `ARCHITECTURE.md`（Agent 定）
- 決定：模組地圖與流程圖照 repo 現況：補 `qim-neural`，拿掉 `web/editor/` 與 `data/` 的佈局檔（Agent 定）
- 決定：程式註解引 `architecture〈節名〉`；`deps` 的錯誤訊息用常數 `RULES_DOC` 寫 `ARCHITECTURE.md〈依賴規則〉`（Agent 定）
- 決定：只改指向已搬節（§0、§1）的引用，plan 其他節的引用留給票 06（Agent 定）
- 變更：無
