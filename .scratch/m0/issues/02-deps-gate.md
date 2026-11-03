# 02: `deps` 閘門

Status: done
Blocked by: 01
讀: plan §1.3、§6；spec〈共同規則〉

## What to build

有人讓 `qim-core` 依賴綁定 crate、加了表外的依賴，或在 `qim-core` / `qim-layout` 裡用 `std::fs` 這類 I/O，`cargo xtask check` 就會紅，並說出違反 plan §1.3 的哪一條。

- `cargo xtask check` 加 `deps` 步驟：讀 `cargo metadata`，把每個 workspace 成員的依賴和 plan §1.3 的表比對；表上的 crate 還沒建立時跳過。
- workspace 成員不在 §1.3 表上就紅（逼人先改表）；`xtask` 自己是開發工具，例外。
- 只檢查一般與 build 依賴；dev 依賴不受白名單限制（`proptest` 是 dev），但仍不能是表上禁止的 workspace crate。
- 掃 `qim-core`、`qim-layout` 的原始碼，找 §1.3 的禁用路徑：`std::fs`、`std::net`、`std::thread`、`std::time::SystemTime`。要抓到 `use std::fs`、`std::fs::read(..)`、`use std::{fs, io}` 這類寫法；註解與字串裡出現這些字不算違規。
- 白名單放在哪裡由本票決定（xtask 原始碼或一個資料檔），plan §1.3 寫明位置，並註明改表時兩邊同一 commit 改。
- plan §1.3 的「M0 票 01 接上」改成「M0 票 02 接上」。

## 驗收

- [x] 目前的 repo 上 `deps` 綠燈。
- [x] 測試在暫存副本造出下列違規，各自讓 `deps` 紅：`qim-core` 依賴 `qim-wasm`（或任何表上禁止的 crate）、`qim-core` 加表外的一般依賴、`use std::fs`、`std::fs::read(..)`、`use std::{fs, io}`、加入不在表上的 workspace 成員。
- [x] 測試在暫存副本做無關改動不紅：改格式與命名、註解或字串裡寫 `std::fs`、加 dev 依賴 `proptest`。
- [x] plan §1.3 寫明白名單的位置。

## Comments

- 2026-11-03 白名單放 `xtask/src/deps.rs` 的 `RULES`，不另開資料檔：少一種格式要解析。plan 表與 `RULES` 的「可以依賴」欄由 `plan_table_matches_coded_rules` 比對，附雙向變異。plan 原本已寫「M0 票 02 接上」，不用改。
- 2026-11-03 plan 表要具體名字才能當白名單：`qim-wasm` / `qim-ffi` 拆兩列，綁定工具寫死成 `wasm-bindgen` / `uniffi`；`qim-databuild` / `qim-bench` 的外部 crate 不限。workspace crate 不論哪種依賴都只能是欄內列的，比票寫的「不能是表上禁止的」嚴：表只列允許的。
- 2026-11-03 掃描用 `proc-macro2` 斷詞而不用 `syn`：巨集呼叫裡的路徑 `syn` 不當路徑，斷詞一樣掃到。`SystemTime` 以名字判斷，抓得到 `use std::time; time::SystemTime`。只掃 `src/`（含 `#[cfg(test)]`）；`build.rs`、`tests/`、`benches/`、`examples/` 不編進函式庫。
- 2026-11-03 驗收輸出：`cargo xtask check` 印 `check 通過`；真 repo 造 `pub use std::{fs, io};` 與 `serde_json` 依賴後 `check --fast` 印下列兩行與 `check 失敗：deps`、exit 1，還原後綠燈：
  - `qim-core 的一般依賴 serde_json 不在 plan §1.3 的白名單（serde、postcard）：要加先改 plan §1.3 與 xtask/src/deps.rs 的 RULES`
  - `crates/qim-core/src/lib.rs:4 用了 std::fs：plan §1.3 規定 qim-core 不可以用 std::fs、std::net、std::thread、std::time::SystemTime`
- 2026-11-03 code-review 沒處理的發現：
  - 把 `std` 本身改名（`use std as s`、`extern crate std as s`）後的路徑抓不到；plan §1.3 已註明。`std::os::unix::net` 這類表外的 I/O 路徑也不擋，要擋先改表。
  - plan 表的「不可以」欄與 `FORBIDDEN_PATHS`、`no_io` 沒有同步測試，只比對「可以依賴」欄。
  - 違規以字串表示，測試比對訊息片段；改措辭要連帶改測試。`deps.rs` 的斷詞掃描可以之後拆成子模組。
  - xtask 的新依賴（`cargo_metadata` 0.23.1、`proc-macro2` 1.0.107、dev `tempfile` 3.27.0）沒列進 plan §1.4：那張表只列產品的主要依賴。
