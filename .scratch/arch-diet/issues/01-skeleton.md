# 01: 建 ARCHITECTURE.md 的骨架，搬 plan §0、§1

Status: ready-for-agent
Blocked by: None
讀: `.scratch/arch-diet/spec.md`；plan §0、§1 全文

## What to build

建根目錄的 `ARCHITECTURE.md`：檔頭的寫法照 project-init 的範本（只寫現況、節不編號、上限 15,000 字），節是〈執行形態〉〈模組地圖〉〈依賴規則〉〈跨模組的資料與契約〉〈主要依賴〉〈測試策略〉〈風險與回寫點〉。把 plan §0〈工程慣例〉與 §1〈架構總覽〉歸位：crate 的分工與依賴規則留下；`deps` 步驟讀的表（`xtask/src/deps.rs` 的測試會讀 plan §1.3）跟著搬，測試改讀新位置。

## 驗收

- [ ] `ARCHITECTURE.md` 存在，七個節都有標題；§0、§1 的內容已歸位，plan 這兩節只剩標題與一行指向。
- [ ] `cargo test -p xtask` 綠燈（讀依賴規則表的測試改讀 `ARCHITECTURE.md`）。
- [ ] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [ ] `cargo xtask check --fast` 綠燈。

## Comments
