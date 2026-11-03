# 02: `deps` 閘門

Status: ready-for-agent
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

- [ ] 目前的 repo 上 `deps` 綠燈。
- [ ] 測試在暫存副本造出下列違規，各自讓 `deps` 紅：`qim-core` 依賴 `qim-wasm`（或任何表上禁止的 crate）、`qim-core` 加表外的一般依賴、`use std::fs`、`std::fs::read(..)`、`use std::{fs, io}`、加入不在表上的 workspace 成員。
- [ ] 測試在暫存副本做無關改動不紅：改格式與命名、註解或字串裡寫 `std::fs`、加 dev 依賴 `proptest`。
- [ ] plan §1.3 寫明白名單的位置。
