# 20: 觸控鍵盤功能

Status: done
Blocked by: 19
讀: plan §2.6、§3.1、§5.3；brief §4.4、§4.6；spec〈共同規則〉

## What to build

觸控模式像一個能用的手機鍵盤：長按字母彈出英文字母、第一排長按另含數字；有符號頁（打完不跳回，按「ABC」回字母頁）；有一個自訂頁面示範多頁；按鍵放大預覽；深色模式；轉橫向用橫向的佈局版本。

- 預設 Gboard 佈局由 `qim-databuild` 算出直向與橫向兩個版本，淺色與深色都有顏色；通過票 19 的嚴格檢查（含「有直向與橫向」）。
- 字母鍵主標籤是字根、角標是字母（brief §4.4）；長按清單照佈局的 `longPress`。
- 符號頁：打完留在符號頁（brief §4.4）；自訂頁面示範 `switchPage` 與 `text` 動作。
- 按鍵放大預覽；深色跟隨 `prefers-color-scheme`，網頁另有手動切換。
- 長按、預覽、切頁的狀態在網頁，判定仍經 `qim-layout`。

## 票 19 留下的

- `crates/qim-databuild/tests/gboard.rs` 的 `has_portrait_and_landscape` 標了 `#[ignore]`：加橫向版本時拿掉 ignore。
- 綁定（`crates/qim-wasm/src/lib.rs` 的 `Session.tap`）把 `text`、`switchPage`、`nextKeyboard`、`toggleAscii` 都當 `Key::Other`：組字中被吃掉、沒組字時網頁照 `tap` 處理（`text` 插入文字，其餘不動作）。接上符號頁時要決定組字中按文字鍵的行為（例如先上屏組字再輸入，照票 10 的標點）並寫 Playwright。
- 綁定只送淺色的 `fill`、`labelColor`，`WebLayout` 只在載入時選一次佈局版本；深色與轉向時重選版本在本票。鍵盤底色不在佈局格式裡（plan §2.6），深色時由網頁的 CSS 換。
- `web/src/style.css` 的 `padding-bottom` 照畫布比例 390 × 216 寫死（票 18 Comments），做橫向時改成由佈局算。

## 驗收

- [x] Playwright（行動裝置 viewport）：點按鍵打出「門口」；長按字母鍵讀回彈出的英文字母，選了上屏該字母；長按第一排讀回含數字。
- [x] Playwright：切到符號頁打兩個符號，讀回仍在符號頁；按「ABC」回字母頁；切到自訂頁面並輸入。
- [x] Playwright：轉成橫向 viewport 後仍打得出「門口」；切深色讀回按鍵顏色是深色版本。
- [x] 內建佈局通過嚴格檢查（含直向與橫向）。

## Comments

- 2026-11-04 預設佈局（`crates/qim-databuild/src/gboard.rs`）：直向（畫布 390 × 216）與橫向（844 × 160，鍵變寬、排變矮）用同一份排列，只換畫布、排高與縫。三頁：字母（最下排 `?123`、`，`、空白、`。`、Enter）、符號（數字、常用符號、中文標點；左下「ABC」，左邊「常用」）、常用（自訂頁面的示範：10 個表情與 7 句粵語常用語，示範 `switchPage` 與多字的 `text`）。切頁鍵與刪除、Enter 同一個深一階的底色。
- 2026-11-04 長按：字母鍵長按是 `text`（英文字母直接上屏、不進組字，plan §2.6 例子就是這樣寫），第一排另含數字，字母在前（角標是字母，預設跟著角標）。網頁按住 400 ms 出清單、橫向滑動換選項、放開做選到的；靠左半的鍵清單往右展開、右半往左，免得超出畫面。
- 2026-11-04 組字中按文字鍵（票 19 留下的）：核心加 `Session::text`，和票 10 的標點相同先上屏整個組字再接文字，文字照原樣不轉全形；沒組字時核心直接上屏，網頁不再自己插入文字。沒用 `Key::Text(String)`：`Key` 是 `Copy`，各處以值比對，加帶 `String` 的變體要動到所有用法；M4 的 `Weighted` 屆時再一起考慮。plan §2.1、§2.6 已補。
- 2026-11-04 切頁：狀態在綁定的 `WebLayout`（`tap` / `longPress` 收 `&mut Layout`），核心收到 `Key::Other`，組字中吃下、組字留著，所以「組字中切符號頁再按符號」= 先上屏組字再接符號。`nextKeyboard` 網頁沒有對應，也當 `Key::Other`；`toggleAscii` 轉 `Key::ToggleAscii`（票 23 接）。
- 2026-11-04 深色與轉向（票 19 留下的）：綁定把淺色與深色都給網頁，網頁用 CSS `light-dark()`，`color-scheme` 跟 `prefers-color-scheme`，「深色」開關在根元素寫 `data-theme` 蓋過；鍵盤底色、預覽與長按清單也用 `light-dark()`。轉向時 `Layout.select` 重選佈局版本、有同 id 的頁就留在那頁；鍵盤寬度取 `min(100%, 45vh × 畫布寬高比)`，`body` 的 `padding-bottom` 改由 `ResizeObserver` 量鍵盤實際高度（不再寫死 390 × 216）。
- 2026-11-04 動作改在放開時做（原本按下就做）：要分辨點一下與長按；預覽在按下時顯示。快打時前一指還沒放開就按下一鍵，前一鍵先當點一下做掉（Playwright 雙向驗過：改回「不理第二指」時紅，打出 `ar` 而非 `anr`）；長按清單開著時不理新的手指。
- 2026-11-04 Playwright 寫測試時抓到兩個釋放順序的錯：沒組字時按切頁鍵，重畫先釋放了舊頁的鍵、`hostDefault` 才讀它（`null pointer passed to rust`）；組字中按切頁鍵，`refresh` 拿新頁的鍵對舊頁的元素。`touch-features.spec.ts` 每個測試結尾檢查沒有 page error，兩個都先紅再修。
- 2026-11-04 亂按：Pixel 7 viewport，暫時的種子腳本（沒進 repo）4 個種子 × 150 步，隨機點鍵、長按並滑動、兩指交疊、按住時轉向、切深色、關開觸控鍵盤，三頁都走到；沒有 page error 或 console 錯誤，每次放開後預覽與長按清單都收起。第一版的亂數產生器（模 2³¹ 的 LCG）低位元太差，永遠點不到切頁鍵，換成 mulberry32 後才真的走到三頁。
- 2026-11-04 驗證：`cargo xtask check` 通過（e2e `42 passed`）。`cargo xtask bench --check` exit 0：資料包 quick `f9499e401df5`、quick-lexicon `fe79d8a13d48`、zh-hant-bigram `088a93199bd2`，各集合答錯數、修正次數都和 baseline 相同，沒有翻盤，固定題 11/11。`Session::text` 是新方法，基準走的 `handle` 沒變，前後一樣是預期的結果。每鍵 p95：原生 50.46 µs、WebAssembly 59.79 µs，比票 19 的 18.92 / 23.92 µs 高。這次沒動 `handle`，並行的 worktree 同時在跑建置，多半是機器負載；只報不擋。截圖：`.scratch/m1/evidence/20-*.png`（長按 a、長按 q 選 1、符號頁、常用頁、放大預覽、深色、橫向）。
- 2026-11-04 code-review（`25834f0` 起）已處理：
  - 轉向留在同頁的規則移到 `qim-layout`（`LayoutVariant::page_index_or_initial`），綁定不放這條規則。
  - 「要不要放大預覽」移到 `qim-layout`（`Key::shows_preview`），也照 `composing` 判斷。
  - 核心沒吃下長按選的動作時，網頁原本照鍵的 `tap` 做。改成綁定在 `Snapshot` 帶上做了的 `action` 與 `letter`，`LayoutKey` 拿掉 `tap` / `letter`。補了自訂佈局「長按選刪除」的 Playwright，先紅（打出 `！！a`）再修。
  - `toggleAscii` 改回 `Key::Other`，票 23 再接、補測試。
  - 改名：`left` / `top` 改成 `ofWidth` / `ofHeight`，`after` 改成 `record_composing`。重畫判斷收成 `followPage`。
  - `touch.ts` 開頭的註解改正，寫明目前頁記在綁定。
  - 深色測試加 Enter 的淺色斷言，分得出佈局顏色和 CSS 預設底色。
- 2026-11-04 code-review 沒處理：
  - 綁定裡 `Action` 轉 `Key`：`qim-layout` 不能依賴 `qim-core`（plan §1.3），只能放在綁定。M2 的 `qim-ffi` 要用時，再決定要不要抽成共用。
  - 淺深兩組顏色攤平成四個欄位：wasm-bindgen 的 getter 不好回傳巢狀物件，網頁也只把它們接成 `light-dark()`。
  - 放開才做動作、兩指交疊：票沒寫，但分辨點一下與長按一定會帶出這兩點。已寫進 plan §2.6，平台殼照做。
- 2026-11-04 合併 main（票 13，`0736054`）：沒有文字衝突，但有一處語意衝突：票 13 讓 `commit_all_then` 多收 `&mut UserModel`（標點先上屏組字時照樣學），`Session::text` 編不過。改成把 `user` 傳下去，觸控文字鍵和標點一樣學。`tests/learning.rs` 補「文字鍵與標點學到的 `UserModel` 相同且不是空的」，雙向驗過：改成學進丟掉的 `UserModel` 時會紅。合併後 `cargo xtask data` 通過（資料包 sha256 不變：quick `f9499e401df5`、quick-lexicon `fe79d8a13d48`、zh-hant-bigram `088a93199bd2`）；`bench --check` exit 0，各集合沒有翻盤，固定題 11/11；`cargo xtask check` 通過（e2e `44 passed`）。合併後觸控亂按：4 個種子 × 150 步，加切換無痕，三頁都走到，沒有錯誤。
