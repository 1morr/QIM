# 02: 統一命名：`SchemePack`、一種分段型別、`UserModel` 不必每個方法都帶方案

Status: ready-for-agent
Blocked by: 01
讀: plan §0、§2.1、§2.4；`GLOSSARY.md`；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條（命名那條）

## What to build

FFI 介面定型前，把核心公開型別的命名收成一致，之後 Swift / Kotlin 綁定直接看到的就是這套名字。純改名與收型別，不改行為。

- `Pack` 只指方案資料包，和 plan §0「資料包 = `Pack`」不符：改名 `SchemePack`，plan §0 的對照表與 `docs/development.md` 有提到的地方同步。
- 分段現在有 `(String, String)`、`Segment`、`PathSegment` 三種寫法：留一種公開型別（名詞表的「分段」= `Segment`），內部需要不同欄位時用私有型別，不再用 tuple。
- `UserModel` 每個方法都帶 `scheme`：改成建立時就綁定方案，或另想更小的介面；選法寫在 `## Comments`。匯出匯入的 `# qim-userdict v1` 格式不變（對外的持久化格式，要改先問使用者）。
- 網頁跟著改到能編譯、行為不變。

## 驗收

- [ ] 核心與綁定的公開介面不再有 `Pack`（單指方案資料包的那個）、分段 tuple、`PathSegment`；`UserModel` 的方法不再逐一帶方案。
- [ ] `cargo xtask bench --check` 綠燈且數字不變，不 `--update-baseline`。
- [ ] 匯出匯入的往返測試照舊通過，格式字串沒變。
- [ ] `cargo xtask check` 綠燈（含 e2e）。
