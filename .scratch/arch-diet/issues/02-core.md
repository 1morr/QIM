# 02: plan §2〈核心〉歸位

Status: ready-for-agent
Blocked by: 01
讀: `.scratch/arch-diet/spec.md`；plan §2 全文（約 52,000 字，分小節讀）；`crates/qim-core/` 的公開介面

## What to build

§2 是最大的一節，大部分在逐條描述核心的行為（帶函式名）。留下：對殼的介面契約（`Session`、`Snapshot`、`Key`、`Options` 這一層殼要守什麼）、學習資料與匯出格式的版本與相容規則、排位規則的順序。其餘的行為描述，找到說明它的測試或程式檔後刪掉。這張票預期刪掉最多，對照表要逐小節列。

## 驗收

- [ ] `ARCHITECTURE.md`〈跨模組的資料與契約〉有核心對殼的契約與匯出格式的版本規則。
- [ ] §2 每個小節在對照表裡都有去處；刪掉的每一段都指得到一個測試或程式檔。
- [ ] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
