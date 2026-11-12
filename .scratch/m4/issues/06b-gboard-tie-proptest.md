# 06b: Gboard 格子上點中的鍵與鄰鍵機率平手

Status: ready-for-agent
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

- [ ] regression 檔（兩個種子）已 commit，修前紅的輸出貼在 Comments；寫明是產品還是測試的問題與理由。
- [ ] 修後 `cargo test -p qim-layout` 綠；proptest 案例數提高（例如 `PROPTEST_CASES=10000`）連跑一次也綠，輸出貼在 Comments。
- [ ] 若改了產品行為，`cargo xtask bench --check` 與 Playwright 觸控測試照舊綠；plan §2.6 若有對應描述同步改。
- [ ] `cargo xtask check` 綠燈。
