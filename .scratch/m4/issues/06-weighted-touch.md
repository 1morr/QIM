# 06: `Key::Weighted` 與觸控空間模型

Status: done
Blocked by: 01
讀: plan §2.1（按鍵的入口、觸控機率）、§2.6；spec〈細節〉〈空間模型〉；brief §4.8；`docs/research/spatial-correction-swipe.md`（空間模型的段落）；spec〈共同規則〉

## What to build

觸控的每次點擊變成各字母的機率，供之後的容錯用：

- `Key` 加 `Weighted`（一組「字母, 機率」），`Key::parse` 收得下；綁定照「種類 + 值」的入口傳。
- `qim-layout` 依點擊位置與按鍵幾何算機率：點擊到每個鍵形狀的距離（旋轉、多邊形、橢圓只是距離函式不同），套 2D Gaussian，σ 跟該鍵的大小成比例並有手指精度的下限（VelociTap、FFitts），單位用實體尺寸；重疊的鍵各自算，最上層加成。AOSP 的 `ProximityInfo` 假設矩形與共用尺度，不搬。參數的初值附來源，之後由容錯的票在 dev 調。
- 網頁觸控的字母鍵送 `Weighted`。容錯進核心之前，核心只用機率最高的字母，打字行為不變；debug 面板顯示前幾名的機率。
- 核心不知道佈局（plan §1.3）。

## 驗收

- [x] `qim-layout` 性質測試：點在鍵中心時該鍵機率最高；離得越遠機率越低；旋轉、縮放後結果一致；重疊時上層加成；機率和為 1，只回前 N 個（N 由票定）。
- [x] `Key::parse` 收得下 `Weighted`，壞格式回錯誤。
- [x] 現有觸控 e2e 照綠；Playwright 點兩鍵中間，debug 面板讀到兩個字母的機率；截圖。
- [x] plan §2.1「觸控機率」改成實際的介面；`cargo xtask check` 綠燈。

## Comments

2026-11-12 實作（`3f7678f` 起）：

- 介面：核心 `Key::Weighted(Weights)`（`crates/qim-core/src/weights.rs`），建構時檢查字母、不重複、正的有限數，正規化成和為 1、由高到低（穩定排序）；`handle` 開頭換成機率最高的字母，這組機率記到下一個鍵或 `text` 為止，給 `Inspection::touch_weights`。`Key` 因此不再是 `Copy`，既有測試補 `.clone()`。`Key::parse("weighted", "a:0.62,s:0.38")` 的文字格式沒有成熟產品可抄（librime 的 `KeyEvent` 文字表示沒有機率），照現有「種類 + 值」的慣例取最簡單可讀的寫法；網頁的觸控經綁定直接傳型別，不走這串字。
- 空間模型（`crates/qim-layout/src/spatial.rs`，`SpatialModel::default` 附來源）：σa 1.5 mm（FFitts，圓形目標）、k 橫 0.21／縱 0.12（Fowler 2015 的 σx 1.97、σy 1.88 mm 照 FFitts 扣掉 σa²，除以 6.16、9.42 mm 的鍵）、剪枝半徑 7.99 mm（Fowler）、N = 5（剪枝半徑內通常 3–6 個鍵）、上層加成 2（**沒有來源**，自訂初值，容錯的票在 dev 調）。Fowler 的 σ 是到鍵中心的距離，這裡套在到觸控範圍的距離（鍵內是 0），分布比 Fowler 寬；code-review 指出最初 k 0.14 的推導和 σa 重複計算，改成現在的拆法。
- 點中的鍵一定最高（協調者要求，本票就守住）：最高的不是點中鍵的字母時，把它提到和最高的一樣、排第一（平手的值，留著「兩個一樣可能」給容錯），而不是把加成調大到必勝：加成調大會讓所有點擊都偏向點中鍵、壓扁空間模型。性質測試：鍵大小不一的頁上任何點、小鍵與大鍵之間靠小鍵那側（原本大鍵贏，先寫成紅燈）；Gboard 格子與鍵中心另斷言「嚴格大於」，證明一樣大的鍵上是 Gaussian 自己勝出、沒靠提升。`Keyboard::tap` 收到的 `index` 和點擊位置判定不同時照沒給位置，打的字照按下的鍵。
- 只算 `tap` 是字母的鍵：Shift 開著的字母鍵（輸入文字）與功能鍵不算；點在空白鍵附近時空白的可能性不在 `Weighted` 裡，容錯的票要時再加。同字母多鍵機率相加。
- 對外看得到的變化：觸控字母鍵的 `Snapshot.action` 從 `letter` 改成 `weighted`（`character` 照舊是會打的字母），`host.ts` 與 `main.ts` 的 `afterHostKey` 一起改。網頁量不到螢幕密度，CSS px 當 1/160 吋（Android dp；iOS pt 是 1/163 吋）。
- `bench --check` 前後：各集合答錯、修正數與 baseline 完全相同、沒有翻盤，固定題 84/84；每鍵延遲原生空白 p95 73 µs、選開頭一段 1.30 ms，學過 186 µs／2.13 ms，都在上限內（行為不變，lattice 沒動，不必跑 `model-ladder`）。
- 驗證：`cargo xtask check` 綠（e2e 236 passed，含新的 `web/e2e/touch-weighted.spec.ts` 兩條）。截圖 `.scratch/m4/evidence/06-weighted-touch.png` 是從 Playwright 輸出目錄手動複製的，e2e 不直接寫 evidence（`e5e4686` 為了不改寫封存的證據）。亂按一輪：一次性腳本在 Pixel 7 viewport 隨機點鍵盤 300 下（縫隙、快速連按、開關面板與數字列），126 次觸控字母鍵的機率和都是 1、第一名都等於點中的鍵、沒有頁面錯誤。
- `cargo xtask check` 跑了四次，兩次在 `qim-bench` 的 `tests/run.rs` 紅：`check_blocks_native_latency_only_and_update_baseline_ignores_latency` 與 `check_passes_when_counts_equal_baseline` 在並行測試負載下把小資料包的每鍵 p95 量成 1.45–1.50 ms（> 1 ms 上限），單獨重跑也偶爾紅，最後一次整個 check 綠。本票沒動 `qim-bench`，`bench --check` 的真實延遲是 73 µs：用小資料包量真實延遲的測試在機器忙時會抖，與本票無關（票 05 加的閘門），留給收尾票：那幾條測試應該用假的計時，不量真實延遲。
- code-review 未處理（判斷題）：`handle_key` 的 `Key::Weighted` 分支照註解到不了，沒改成 `unreachable!()`（寧可吃下也不在網頁 panic）；網頁 `host.ts` 與 `main.ts` 兩處的 `"letter" | "weighted"` 判斷沒有收成一處（兩處、各一行）；`Key::parse` 收不下時只回籠統的 `KeyError`、不說是哪一種錯（同其他種類的鍵）；`Weights::new` 與 `letter_weights` 各做一次正規化是依賴規則（核心不依賴 `qim-layout`）逼出來的。

Session 紀錄：`Key::Weighted` 與 `qim-layout` 的空間模型（到形狀的距離加 2D Gaussian、點中的鍵一定最高）；網頁觸控送機率、面板顯示；bench 前後相同
