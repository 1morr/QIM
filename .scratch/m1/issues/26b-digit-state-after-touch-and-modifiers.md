# 26b: 數字後半形的狀態與 Shift 判定細節

Status: ready-for-agent
Blocked by: 26, 27
讀: 票 25、25b、26 的 Comments；brief §4.3、§4.4、§4.7、§12.2；plan §2.1；spec〈共同規則〉

## What to build

M1 封存後補票的審查（2026-11-05，獨立審查，對照 librime 原始碼）找到以下問題，使用者 2026-11-05 同意照建議修。審查的探測在 `/private/tmp/claude-501/-Users-roxy-orca-projects-QIM/4808186f-7ee6-44a4-9688-748b966c8ba8/scratchpad/review-a/`（`probe/tests/probe.rs` 核心探測、`loneshift.mts` 網頁探測、`src/` 下載的 librime 原始碼），可參考。

1. **觸控切頁或 ⇧ 讓「緊接在數字後」失效**（已證實）：`crates/qim-wasm/src/lib.rs` 把 `Action::SwitchPage`、`Action::Shift`（與 `NextKeyboard`）送成 `Key::Other`；沒有組字時核心不吃，`session.rs` 就當成交還宿主的非數字鍵，`after_digit` 變假，但輸入框沒變。情境：符號頁打「3」→「返回」→ 字母頁句號 → 讀回「3。」，應是「3.」。修：這些只改鍵盤狀態、不送字元的動作不能結束數字（不送核心，或給不動 `after_digit` 的路）。先寫會紅的核心測試與 Playwright。
2. **單按修飾鍵結束數字**（已證實）：網頁上單按 Ctrl / Cmd / Alt 時 `toKey` 回 null，`web/src/main.ts` 約 653 行呼叫 `session.other()`；CapsLock 也轉成 other。「3」、單按 Ctrl、「.」讀回「3。」。Rime 不記沒有字元的鍵。修：`event.key` 是 Control、Meta、Alt、AltGraph、CapsLock、Shift 等修飾鍵時不送核心（組字中的既有行為不能變）。先寫會紅的測試。
3. **單按 Shift 判定在別的鍵放開時不解除**（已證實，對照 librime `ascii_composer.cc` 的 other keys 分支）：`web/src/keys.ts` `LoneShift.up()` 收到非 Shift 的 keyup 不清 `#since`。情境：按住 a → 按下 Shift → 放開 a → 放開 Shift，QIM 切換、librime 不切。修：`up()` 收到非 Shift 的 code 時解除，補單元測試。
4. **文件**：
   - brief §4.7（約 159 行）「觸控鍵盤的句號、逗號鍵送的是全形文字，不經這條規則」已過時（票 27 改成送 `Key::Punct`），改成現況。
   - 「舊版／新版 librime」的說法改正：使用者用的行為就是目前正式版（≤ 1.17.0，2026-06-06），`74bd5dc4`（commit_raw_input）只在未發佈的 master／rolling prerelease。brief §4.3、§12.2、GLOSSARY、plan、票 25b 裡的「舊版」「新版」都改成「目前發佈版」「未發佈的 master」，並補一句風險：librime 1.18 發佈、小狼毫更新後，使用者的小狼毫會變成 master 的行為。
   - `crates/qim-core/tests/continuous.rs` 約 916 行註解「librime 把整串換成按鍵，QIM 留著選了的字」是 master 的行為，發佈版也留著，改註解。
   - brief §4.3 補兩處沒記錄的差異：Rime `commit_text` 是 `ConfirmCurrentSelection`（只確認最後一段，剩下的碼在英文模式變原樣），QIM 上屏整個組字；Rime 的 `is_after_number` 只看 `thru`/`raw` 紀錄，QIM 看任何上屏文字是否以數字結尾（自加的「MP3」這類詞選了之後不同）。
   - progress〈偏差與決定〉記一行，session 紀錄票欄寫「M2（補 M1 26b）」。

## 驗收

- [ ] 核心測試與 Playwright：觸控符號頁「3」→ 返回 → 句號讀回「3.」；⇧ 之後同；實體「3」→ 單按 Ctrl →「.」讀回「3.」；組字中的行為不變。
- [ ] 單元測試：按住別的鍵時按 Shift、先放開別的鍵，不切換。
- [ ] brief、plan、GLOSSARY、票 25b 的文字已改；`cargo xtask check`、`bench --check` 綠燈；/code-review 跑完。
