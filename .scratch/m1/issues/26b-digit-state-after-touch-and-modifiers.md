# 26b: 數字後半形的狀態與 Shift 判定細節

Status: done
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

- [x] 核心測試與 Playwright：觸控符號頁「3」→ 返回 → 句號讀回「3.」；⇧ 之後同；實體「3」→ 單按 Ctrl →「.」讀回「3.」；組字中的行為不變。
- [x] 單元測試：按住別的鍵時按 Shift、先放開別的鍵，不切換。
- [x] brief、plan、GLOSSARY、票 25b 的文字已改；`cargo xtask check`、`bench --check` 綠燈；/code-review 跑完。

## Comments

- 做法：核心加 `Key::Modifier`（只改鍵盤狀態、不送字元的鍵），不是在殼裡「不送核心」：組字中它和 `Other` 走同一支吃下，組字不變；沒有組字（含英文直通）時 `handled` 為假、交還宿主，`Session::handle` 不動 `after_digit`。觸控的切頁、⇧、換鍵盤與網頁單按修飾鍵共用這條路，也讓第 1 項能先寫會紅的核心測試；殼不必自己判斷組字中與否。plan §2.1 的 `Key` 列表與說明跟著改。
- 網頁：`toKey` 對 MDN「Modifier keys」一表的 `event.key`（Control、Meta、Alt、AltGraph、CapsLock、Shift、Fn…）回 `modifier`，放在檢查 `ctrlKey` 之前（單按 Ctrl 的事件本身帶 `ctrlKey`）。Ctrl 按住再按 B 仍是交還瀏覽器的組合鍵、照樣結束數字（票 26 的 e2e 不變）。Shift 的 keydown 在 `main.ts` 照舊不送核心，表裡的 Shift 只有單元測試走到。
- `LoneShift.up()` 收到不是 Shift 的 `code` 就解除，對照 librime 1.17.0 `ascii_composer.cc:60-122`：other keys 分支不分按下放開；放開 Ctrl、Alt 也走修飾鍵那支的解除，網頁一樣解除。
- 先紅後綠：核心兩條（`a_modifier_key_*_does_not_end_the_number`）在加變體、還沒改 `handle` 時紅；e2e 五條（單按 Control / Meta / Alt / CapsLock、觸控「返回」與 ⇧）在 HEAD 的 `keys.ts`、`main.ts`、`qim-wasm` 上紅，組字中單按 Ctrl 那條本來就綠（守既有行為）；`LoneShift` 與 `toKey` 的單元測試在改之前紅。
- 文件：Rime `is_after_number` 的差異寫在 brief §4.7（數字後半形那段），不是票寫的 §4.3：它講的是數字後半形，§4.3 只放 `commit_text` 那條。librime 的事實另對 1.17.0 tag 查過（`ascii_composer.cc:257-258`、`context.cc:168-182`、`punctuator.cc:85`）並補進 §12.2；「1.17.0 是最新正式版、`latest` 是 master 的 rolling prerelease」照 GitHub releases 補來源。票 25b 已完成，只改用詞並在它的 Comments 加一行說明。
- bench 前後：改動前 = `bench/baseline.json`（HEAD）；改動後 `cargo xtask bench --check` 通過，五個集合都沒有翻盤，固定題 71/71（合併 main 之前）；`Key::Modifier` 不影響排序。`cargo xtask data --verify` 通過。
- 亂按（一次性 Playwright 腳本，沒進 repo）：桌面 40 輪各 40 鍵，混字母、數字、`.` `:` `,`、單按 Ctrl / Cmd / Alt / CapsLock、左右 Shift、「按住 a、按 Shift 或修飾鍵、先放開 a」、空白、退格、Enter、Esc、方向鍵、Tab、`-` `=`、`'`、Ctrl+B、Delete，每步確認沒有「沒組字卻有候選」；每輪結尾 Esc、回中文、打 `7`、單按一個修飾鍵、`.`，讀回「7.」。觸控 12 輪各 25 下，在目前頁的鍵裡亂點（含切頁、⇧、數字、標點），每輪結尾點符號頁「3」→ 返回 → ⇧ → 句號，讀回「3.」。兩邊都通過，沒有頁面錯誤與原生對話框，沒找到問題（腳本本身的兩個錯：Tab 移走焦點、選到隱藏的放大預覽，改腳本後重跑）。
- code-review（起點 `7a23944`）已處理：程式碼與測試註解改寫成「librime 1.17.0」，「目前發佈版 / master」的區分只留在文件，免得 1.18 發佈後一起變假；`perform` 走不到的 `Action::Text(_)` 不再對應成 `Modifier`，改成 `unreachable!`；`qim-wasm` 的註解改指 brief §4.7；票 25b 補一行說明。
- code-review 未處理：
  - 第一個 commit（`83b8187`）改了 `Key` 卻到下一個 commit 才改 plan §2.1、brief §4.7，違反「同一 commit 改文件」；已經 commit，重寫歷史要先問，所以只記在這裡。
  - `perform` 把切頁、⇧ 送成 `Modifier` 這一步只有 e2e 測得到：`qim-wasm` 沒有測試框架（要 wasm 執行環境），同其他觸控動作。
  - 「只改鍵盤狀態、不送字元的鍵」的說明在 `session.rs` rustdoc、plan §2.1、brief §4.7 各一份：plan 是介面文件、brief 是需求，各自要讀得懂，不收成一份。
  - `MODIFIERS` 整份照 MDN 表收（含 Hyper、Super、Symbol），有來源可循，不只收會遇到的幾個。
- progress 的票欄寫「M2（補 M1 26b）」：`docs` 閘門只收進行中的里程碑 M2 開頭。
- 合併 main（票 28b，`c42de50`）：衝突只在 brief §4.3、plan §2.1 的相鄰段落，取本票的 Shift／臨時英文／`Key::Modifier` 段與 main 的分隔記號段；28b 只改 lattice 與 engine，和 `Key::Modifier` 不互相影響。合併後 `cargo xtask data --verify` 通過；`bench --check` 通過，五個集合都沒有翻盤，固定題 75/75；`cargo xtask check` 通過（e2e 138 條）。
- 合併後的 code-review（`git diff main...HEAD`）：沒有阻擋問題。已處理：progress〈偏差與決定〉本票的兩條移到 28b 之後，和 session 紀錄同序。未處理：merge commit 照協調者指定用 git 的預設訊息；`c80ae23` 的 subject 只寫 wasm，也改了核心與 e2e 的註解（body 有交代）；`unreachable!` 的訊息是中文，同 repo 裡所有 `expect` / `panic!` 的慣例；票 27 的 Comments 裡「照舊版 librime」是當時合併的歷史紀錄，不在本票改正範圍。
- 合併 main（票 17d，`e42d112`）：沒有衝突，只改詞庫建置與資料，和本票的程式不重疊；progress 本票的列與兩條偏差移回最後。合併後 `cargo xtask data --verify` 通過；`bench --check` 通過，五個集合都沒有翻盤，固定題 84/84；`cargo xtask check` 通過（e2e 144 條）。
