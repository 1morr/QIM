# 01: 改完選項後，選字與移分段用的是新的候選列

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §2.1、§2.4；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條（`set_options` 那條）

## What to build

殼只呼叫 `set_options`、沒有再按鍵就直接選字、移到別的分段或打文字時，`Session` 用的是新選項算出的候選列，不是改選項前那一份。現在只有 `handle` 會先套用新選項；網頁每次都 `refresh` 所以看不出來，原生殼會選錯字。

- 走 `mattpocock-skills:tdd`：先寫會紅的測試（例如開著英文候選組字 `the`，`set_options` 關掉英文候選後直接 `select` 第 2 位，或切簡體後直接 `select`），確認紅了再修。
- 修法讓所有公開方法都走同一條「套用待生效選項」的路，不在各方法各補一次。

## 驗收

- [ ] 新測試在修之前紅、修之後綠；`select`、`focus_segment`、`text` 三個入口各有一個情境。
- [ ] `cargo xtask bench --check` 綠燈，數字與修之前相同（基準只經 `handle`，不該變）；前後對照貼在 `## Comments`。
- [ ] `cargo xtask check` 綠燈。
