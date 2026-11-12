# 06b: Gboard 格子上點中的鍵與鄰鍵機率平手

Status: done
Blocked by: None (can start immediately)
讀: 票 06 的 Comments（「點中的鍵一定最高」那條）；票 05b 的 Comments（兩個 proptest 種子）；plan §2.6；spec〈細節〉〈空間模型〉

## What to build

協調者插入的修復票（2026-11-12）。`crates/qim-layout/tests/spatial.rs` 的 proptest `on_a_gboard_grid_the_most_likely_letter_is_the_hit_key`（票 06）偶爾抽到失效點，紅在 `w[0].1 > w[1].1`：

- `(376.8759717002673, 146.41283071299227)` 時 `[('l', 0.5), ('m', 0.5)]`；另一次縮到 `(366.0247247808735, 138.6298953419659)`。
- 種子（放進 `crates/qim-layout/tests/spatial.proptest-regressions` 就能重現）：
  - `cc bfd391913d686817b95a7ad7ef167cf94fd7e9f6f2f2a9ec78f057d2931aef5c`
  - `cc 36099777f516688467458ef0caa4416f343fb0ee31cd511e6b5ea1db6e572fbd`
- 票 05b 的 session 跑 `cargo xtask check` 五次紅一次；抽中後 proptest 寫 regression 檔，之後每次都紅。

要做的：

- 先分清是產品還是測試的問題：票 06 定的是 Gboard 等大格子上「點中的鍵嚴格最高、靠 Gaussian 自己勝出」，大小不一時才「提到平手第一」。查清楚這兩個點為什麼平手（例如點在鍵邊界或角上、`l` 與 `m` 不相鄰卻等機率、加成沒套到、命中判定與機率用的座標不一致），決定是產品要修（點中的鍵應嚴格最高），還是測試的前提要改（邊界上的平手是合理的、斷言要排除），理由記在 Comments。
- 先紅：把兩個種子的 regression 檔 commit 進 repo，確認在修之前是紅的。
- 修後綠；regression 檔留在 repo，之後每次都重驗這兩個點。

## 驗收

- [x] regression 檔（兩個種子）已 commit，修前紅的輸出貼在 Comments；寫明是產品還是測試的問題與理由。
- [x] 修後 `cargo test -p qim-layout` 綠；proptest 案例數提高（例如 `PROPTEST_CASES=10000`）連跑一次也綠，輸出貼在 Comments。
- [x] （不適用：產品沒改）若改了產品行為，`cargo xtask bench --check` 與 Playwright 觸控測試照舊綠；plan §2.6 若有對應描述同步改。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-12 修前紅（`9ed32d4` 加上 regression 檔，regression 檔在 `d79f05e` 單獨 commit）：`cargo test -p qim-layout --test spatial on_a_gboard_grid` → `Test failed: [('l', 0.5), ('m', 0.5)] at crates/qim-layout/tests/spatial.rs:285. minimal failing input: x = 366.0247247808735, y = 138.6298953419659`、`successes: 0`、`0 passed; 1 failed`。
- 2026-11-12 原因：兩點都在所有鍵外面，舊測試格子第三排 `m` 右邊、`l` 下面的空白（離最近的鍵 ≥ 31 單位、約 5 mm）。`Page::hit` 在鍵外照直線距離取最近的鍵（`l`），空間模型的縱向 σ（1.71 mm）比橫向（1.89 mm）窄，算出來 `m` 較高；票 06 的「點中的鍵一定最高」照設計把 `l` 提到平手、排第一。用 0.25 步距掃舊格子：880 個平手點全在鍵外、全在第三排兩端，鍵內 0 個。
- 2026-11-12 判定是**測試的前提**，產品不改：票 06 寫的「一樣大的鍵上嚴格最高、靠 Gaussian 自己勝出」只在鍵內與鍵間的窄縫成立；鍵外遠處點擊判定（直線距離）與 Gaussian（各軸 σ 不同）可能選不同的鍵，這時平手正是票 06 定的行為，也留給容錯。把點擊判定改成照 Gaussian 選最近的鍵會改到所有佈局的打字行為，不值得。舊測試格子漏了真實 `letters_page` 第三排兩端各 1.5 鍵寬的 Shift 與刪除，這兩點在真實直向頁點中的是刪除鍵。
- 2026-11-12 修法（只動 `crates/qim-layout/tests/spatial.rs`）：測試格子補 Shift、刪除；點中的不是字母鍵時不看（`return Ok(())` 而不是 `prop_assume!`：約 10% 的點落在這兩鍵，`PROPTEST_CASES=10000` 會累積約 1000 次 reject，貼近 proptest 預設上限 1024）；新增固定測試 `far_off_the_keys_the_hit_key_is_lifted_to_a_tie_with_the_gaussian_winner`，在只有字母鍵的格子上把兩個種子的點各斷言成 `l`、`m` 平手、`l` 排第一，並斷言完整格子上它們點中刪除鍵。regression 檔留在 repo，種子現在點中刪除鍵；兩點的實際行為由固定測試每次驗。變異：拿掉補的兩鍵，proptest（種子）與固定測試都紅。
- 2026-11-12 修後：`cargo test -p qim-layout` 綠（11 個測試執行檔共 74 passed）；`PROPTEST_CASES=10000 cargo test -p qim-layout` 綠（`tests/spatial.rs`：`12 passed; 0 failed … finished in 1.29s`，其餘 10 個執行檔也都 ok）。`cargo xtask check` 通過（e2e `237 passed`、`2 skipped`；這個 worktree 先跑了 `npm ci` 與 `cargo xtask data`）。產品沒改，`bench --check` 與 Playwright 觸控測試不必重跑，plan §2.6 沒有對應的描述要改。
- 2026-11-12 code-review（`9ed32d4` 起）的 Spec 軸在真實佈局上掃描：**橫向**字母頁（含數字列版本）在網頁的 `mm_per_unit` 0.15875 下有約 1900–2300 個鍵外平手點（第二排兩端內縮的空白，例如 `(35.25, 80.75)` 的 `q`／`a`），直向頁在 `mm` 0.25 時也有 43–162 個；所有設定下鍵內平手 0 個、第一名不是點中的鍵 0 個。機制同上，是設計內的行為，固定測試的註解已寫明；容錯的票（09、10）調 σ 時可以留意鍵外平手的比例。
- 2026-11-12 code-review 已處理：功能鍵改用新的 `key()` 建構，不再先做成字母 `x` 再改；檔頭 doc 補上「點中字母鍵時」與鍵外平手；格子的 `8.5`、縮排註明照 `letters_page`；第二個種子也進固定測試。
- 2026-11-12 code-review 沒處理：改用 `prop_assume!`（理由見上）；測試格子與真實 `gboard.rs` 的尺寸差（unit 39 對 38.6、排高 54 對 56、沒有 padding、圓角 6 對 5.5）不改，`qim-layout` 的測試照依賴規則不能用 `qim-databuild`，Spec 軸已在真實直向幾何上確認結論不變。

