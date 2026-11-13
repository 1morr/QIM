# 04: plan §4〈碼表與資料管線〉歸位

Status: ready-for-agent
Blocked by: 01
讀: `.scratch/arch-diet/spec.md`；plan §4 全文（約 21,000 字）；`docs/development.md`〈資料建置〉；`data/sources.toml`

## What to build

資料包的格式、manifest、來源與授權怎麼對應到資料包，是跨模組的契約，留在 `ARCHITECTURE.md`。怎麼建、怎麼驗、怎麼發佈資料包是操作步驟，搬到 `docs/development.md`（那裡已有的不重複）。各來源的處理細節由 `crates/qim-databuild/` 的程式與測試說明的，刪掉並指出檔案。

## 驗收

- [ ] 資料包格式與 manifest 的契約在 `ARCHITECTURE.md`；建置與發佈步驟在 `docs/development.md`，搬移逐行比對沒丟。
- [ ] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
