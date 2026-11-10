# 02: 統一命名：`SchemePack`、一種分段型別、`UserModel` 不必每個方法都帶方案

Status: done
Blocked by: 01
讀: plan §0、§2.1、§2.4；`GLOSSARY.md`；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條（命名那條）

## What to build

FFI 介面定型前，把核心公開型別的命名收成一致，之後 Swift / Kotlin 綁定直接看到的就是這套名字。純改名與收型別，不改行為。

- `Pack` 只指方案資料包，和 plan §0「資料包 = `Pack`」不符：改名 `SchemePack`，plan §0 的對照表與 `docs/development.md` 有提到的地方同步。
- 分段現在有 `(String, String)`、`Segment`、`PathSegment` 三種寫法：留一種公開型別（名詞表的「分段」= `Segment`），內部需要不同欄位時用私有型別，不再用 tuple。
- `UserModel` 每個方法都帶 `scheme`：改成建立時就綁定方案，或另想更小的介面；選法寫在 `## Comments`。匯出匯入的 `# qim-userdict v1` 格式不變（對外的持久化格式，要改先問使用者）。
- 網頁跟著改到能編譯、行為不變。

## 驗收

- [x] 核心與綁定的公開介面不再有 `Pack`（單指方案資料包的那個）、分段 tuple、`PathSegment`；`UserModel` 的方法不再逐一帶方案。
- [x] `cargo xtask bench --check` 綠燈且數字不變，不 `--update-baseline`。
- [x] 匯出匯入的往返測試照舊通過，格式字串沒變。
- [x] `cargo xtask check` 綠燈（含 e2e）。

## Comments

- `UserModel` 的選法：不在建立時綁定方案，改成更小的內部介面。`UserModel` 照舊裝全部方案（`# qim-userdict v1` 一個檔放多個方案，殼只存一份），逐一帶 `scheme` 的九個 crate 內方法搬到私有的 `SchemeModel`（一個方案的個人模型），入口處取一次 `UserModel::for_scheme` / `for_scheme_mut`：`Engine::edges_ending`、`Engine::span`、`lattice::candidates`、`Session::learn`、`Session::add_auto_phrase`、`add_word`、`import`。沒記過的方案回一份靜態的空模型，結果同舊版各方法的 `None` / 0 / 空。公開的 `add_word(engine, …)`、`export`、`import` 本來就不帶方案，簽章不變；匯出格式沒動。不選「建立時綁定」：殼得為每個方案各存一份、匯出要拆檔或合檔，`Engine` 也要驗證綁的是同一個方案，介面反而變大。
- 分段：拿掉 lattice 私有的 `PathSegment { start, end, text }` 與 `Session` 的 `(String, String)`（`selected_segments`、`owned_segments`）；lattice 的路徑與候選的各段直接產生 `Segment`（碼與原文字詞，`Segment::new`），學習與自動造詞收 `&[Segment]`。沒另開私有型別：內部只用到 `Segment` 的子集（`code`、`text`），`Session.selected` 本來就是 `Segment`。`tests/properties.rs` 的「字詞, 碼」對照是測試自己由碼表建的判準，不是分段，沒動。
- `Pack` → `SchemePack`；同理 `qim-databuild` 的 `build_pack` → `build_scheme_pack`，`Engine::new` / `from_pack_bytes` 與 wasm `Session` 建構子的參數 `pack` → `scheme_pack`。`Engine::from_pack_bytes` 收三種資料包，名字不含糊，不改。plan §0 對照表、§2.4、`GLOSSARY.md`〈資料包〉同步；`docs/development.md` 沒有提到這些型別。`qim-wasm` 與 `web/` 只改了參數名，網頁不用改。
- bench（`--check`，資料包 sha256 同票 01：速成 `ae3bc207ce7a`）：固定題 84/84、四個語料集合都沒有翻盤，答錯 / 修正與票 01 的「修之後」逐格相同（yue-wiki 1131 / 4467、zh-wiki 1126 / 4505、common-voice-yue 1032 / 4043、hkcancor 663 / 2588）。這個 worktree 沒有 `bench/personal/`，個人集跳過，沒比到。
- code-review（起點 `159bcf0`）已處理：wasm 建構子參數改 `scheme_pack`（兩軸）；`Session::learn` 遇到空分段直接回，不先取 `for_scheme_mut` 建出空的一份方案模型、讓匯出多一行 tick（Spec，目前的呼叫路徑碰不到，照舊版行為補防護）；四處 `Segment { code, text, ..Default }` 收成 `Segment::new`，註明內部分段只帶碼與原文（Standards）；`Count` 的註解拿掉方案（Standards）。
- code-review 未處理：`user.for_scheme(&engine.scheme().id)` 在五處重複（Standards 判斷題），每處只一行，加 `Engine` 的 helper 反而讓 `Engine` 知道個人模型的內部型別，不加；`CandidateSpan.len` 與各段碼長總和重複（Standards，原本 `PathSegment.end` 就是如此），不在本票範圍；`GLOSSARY.md` 的方案資料包沒自成一條（Standards 判斷題），只是資料包的一種，留在同一條。
