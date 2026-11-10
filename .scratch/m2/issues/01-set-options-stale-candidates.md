# 01: 改完選項後，選字與移分段用的是新的候選列

Status: done
Blocked by: None (can start immediately)
讀: plan §2.1、§2.4；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M2」各條（`set_options` 那條）

## What to build

殼只呼叫 `set_options`、沒有再按鍵就直接選字、移到別的分段或打文字時，`Session` 用的是新選項算出的候選列，不是改選項前那一份。現在只有 `handle` 會先套用新選項；網頁每次都 `refresh` 所以看不出來，原生殼會選錯字。

- 走 `mattpocock-skills:tdd`：先寫會紅的測試（例如開著英文候選組字 `the`，`set_options` 關掉英文候選後直接 `select` 第 2 位，或切簡體後直接 `select`），確認紅了再修。
- 修法讓所有公開方法都走同一條「套用待生效選項」的路，不在各方法各補一次。

## 驗收

- [x] 新測試在修之前紅、修之後綠；`select`、`focus_segment`、`text` 三個入口各有一個情境。（`text` 那個修之前就綠，原因見 Comments）
- [x] `cargo xtask bench --check` 綠燈，數字與修之前相同（基準只經 `handle`，不該變）；前後對照貼在 `## Comments`。
- [x] `cargo xtask check` 綠燈。

## Comments

- 做法：`Session` 加私有 `options_pending`（組字中、影響候選的選項和列候選時不同）與 `apply_pending_options`（待生效就 `refresh_candidates`）；`handle`、`select`、`focus_segment`、`text` 開頭都呼叫後者，`handle` 原本自己的判斷刪掉。拿不到 `Engine`、`UserModel` 的 `all_candidates`、`rerank_request` 與 `apply_rerank` 改看 `options_pending`：重列前回空 / `None`，不給舊候選。plan §2.1 簡體模式那條補一句。
- 先紅後綠（把 `session.rs` 換回 HEAD 版跑）：`select` 兩個（簡體切繁體後選第 2 位得「甲」不是「幹」；關英文候選後選第 2 位得 `the`）、`focus_segment` 一個（回到整串仍是簡體列表）、重排一個、展開面板一個，修之前都紅。
- `text` 的情境（`simplified.rs` 的 `text_after_switching_without_refresh_commits_with_the_new_option`）修之前就綠，留作守門：組字中 `text` 只用候選列的首選（有目前分段時 `clear_focus` 本來就依新選項重列），而簡體與英文候選兩個選項都不改首選：英文候選不搶首選、沒有中文候選時不出（plan §2.4 規則 4），簡體去重留第一個；顯示與上屏的字在上屏時才依目前選項轉換。Spec 軸審查獨立確認找不到會紅的情境。仍照票讓它走同一條路，之後 `lists_like` 納入會改首選的選項時才不會漏。
- 票外的決定：重排與展開面板也是同一類錯（審查兩軸都指出面板：殼照舊列表的位置呼叫 `select` 會選到新列表的另一個字），所以重列前 `all_candidates` 回空、`rerank_request` 與 `apply_rerank` 回 `None`。另一種做法是讓 `set_options` 帶 `Engine`、`UserModel` 並直接回快照，整個「待生效」狀態就不存在；那會改 plan §2.1 的介面，留給票 07 設計 FFI 介面時決定。
- bench 前後：修之前在 HEAD（`17727d9`）的乾淨暫時 worktree 跑，修之後在本 worktree 跑；資料包 sha256 相同（速成 `ae3bc207ce7a`），兩次 `--check` 都通過、固定題 84/84、五個集合都沒有翻盤，下表每格前後相同：

  | 集合 | 整句首選正確 | 答錯 | 修正 | KSPC | 跳動/句 |
  | --- | --- | ---: | ---: | ---: | ---: |
  | yue-wiki | 869 → 869 | 1131 → 1131 | 4467 → 4467 | 2.23 → 2.23 | 9.24 → 9.24 |
  | zh-wiki | 874 → 874 | 1126 → 1126 | 4505 → 4505 | 2.22 → 2.22 | 9.84 → 9.84 |
  | common-voice-yue | 617 → 617 | 1032 → 1032 | 4043 → 4043 | 2.30 → 2.30 | 7.99 → 7.99 |
  | hkcancor | 326 → 326 | 663 → 663 | 2588 → 2588 | 2.38 → 2.38 | 6.99 → 6.99 |
  | personal | 3436 → 3436 | 1081 → 1081 | 1433 → 1433 | 2.49 → 2.49 | 1.38 → 1.38 |

- code-review（起點 `17727d9`）已處理：`all_candidates` 重列前回空（兩軸）；`apply_pending_options` 的註解改成列出實際經過的四個方法（Standards）；`apply_rerank` 註解過長的行重排（Standards）。
- code-review 未處理：`refresh` 與 `apply_pending_options` 都是「條件成立就重列」，只差條件，兩處而已不抽（Standards 判斷題）；`options_pending` 改名成 `has_pending_options` 屬可選，不改。
