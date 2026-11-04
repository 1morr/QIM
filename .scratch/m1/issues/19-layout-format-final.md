# 19: 佈局格式定稿

Status: done
Blocked by: 18
讀: plan §1.4（`serde_json`、`jsonschema`）、§2.6、§5.3；brief §4.6；ADR 0004；`docs/research/open-source-imes.md` §4；brief §12.4（Delta skin）；spec〈共同規則〉

## What to build

佈局格式 `qim-layout/1` 定稿：plan §2.6 的全部欄位都有、有 JSON Schema、驗證分錯誤與警告兩級；矩形、圓角、旋轉與上下層順序的點擊判定正確。做一份兩鍵重疊的測試佈局，在網頁觸控模式點重疊處輸出上層按鍵。這是對外格式（之後改要加版本並提供轉換，ADR 0004），定稿前對照 Delta skin、Trime、FlorisBoard，採用了誰的慣例寫回 plan §2.6。

- 欄位：佈局版本與選法（裝置與方向都符合 → 方向符合 → 第一個版本縮放並警告）、`grid`、`initialPage`、多頁、`rotate`、`shape`（rect 含 `radius`；ellipse、polygon 的欄位留好，實作與點擊判定在 M5，遇到時給警告）、`touchInset`、`label`、`a11yLabel`、`style`（淺色與深色）、`longPress`、`composing`、動作清單（plan §2.6）。
- JSON Schema 放在 repo，`qim-layout` 用 `jsonschema` 驗證。
- 兩級驗證（plan §2.6）：錯誤（檔案壞掉；頁面切不回有字母的頁；iOS 需要切換鍵盤鍵時卻沒有）、警告（重疊、完全被蓋住、超出畫布、缺字母、缺直向或橫向版本）。
- 內建佈局的嚴格檢查（plan §5.3）：票 18 的 Gboard 佈局通過；橫向版本在票 20 才有，嚴格檢查的「有直向與橫向」在本票先寫好測試、標明票 20 補齊。
- 點擊判定的性質測試：上層優先、旋轉後內外判定正確、縮放後結果不變。

## 驗收

- [x] plan §2.6 寫明定稿的欄位與參考了誰；Schema 進 repo。
- [x] 驗證的雙向測試：造出每種錯誤會紅（不能存檔），只有警告不紅。
- [x] 點擊判定的性質測試綠燈（上層、旋轉、縮放）。
- [x] Rust 測試：兩鍵重疊的測試佈局，點重疊處判定為上層按鍵。
- [x] Playwright：載入重疊測試佈局，點重疊處讀回輸出的是上層按鍵。

## Comments

- 2026-11-04 格式（plan §2.6 有完整例子與採用了誰的慣例）：Schema 在 `crates/qim-layout/schema/qim-layout-1.schema.json`（draft 2020-12），型別在 `src/model.rs`，`validate` 先過 Schema 再讀進型別；`format` 不是 `qim-layout/1` 時先講「只讀得懂 qim-layout/1」，不列 Schema 的錯。`initialPage`、`grid`、`rotate`、`shape`、`touchInset`、`a11yLabel`、`style`、`longPress`、`swipe`、`composing` 都可省略；顏色淺深成對、兩個都要寫。
- 2026-11-04 票的欄位清單以外的決定：
  - 加 `swipe`（四向）：改前的 plan §2.6 動作清單就有它，M5 編輯器要設定；先定欄位免得到時升 `qim-layout/2`。有寫時給「還沒有作用」的警告。
  - 不加鍵盤底色：code-review 指出票與 brief §4.6 都沒要（外觀主題是之後的功能），底色由殼依淺深色決定。
  - `touchInset` 正數往外擴（照 Delta skin 的 `extendedEdges`，plan 寫的就是「可擴大的觸控範圍」），負數往內縮。
  - 畫布高：plan 例子由 260 改成 Gboard 佈局實際的 216（票 18 留下的）。
- 2026-11-04 驗證的細節：
  - 「切得回字母頁」只算點一下的 `switchPage`：code-review 指出長按、滑動、組字中的動作不是隨時都在，只靠它們回去的頁會困住使用者；「有字母」也只看 `tap`。只查從 `initialPage` 切得到的頁。
  - iOS 的規則以 `Target { needs_input_mode_switch_key }` 由殼傳入，每頁都要切得到有 `nextKeyboard` 鍵的頁；內建佈局的嚴格檢查固定用預設的 `Target`，iOS 預設佈局要不要放地球鍵在 M7 定（plan §5.3）。
  - 重疊用旋轉後的矩形（分離軸），只碰到邊不算、不計圓角；完全被蓋住用觸控範圍 16 × 16 含貼邊的取樣點，比格距窄的空隙抓不到（所以是警告），觸控範圍縮成 0 的鍵不算。
  - 錯誤級的「檔案壞掉」（`LayoutError::Broken`）只帶訊息字串，測試比對子字串；之後編輯器要逐項標位置時再拆。
- 2026-11-04 點擊判定：旋轉以 `frame` 中心順時針（同 CSS），先把點轉回按鍵自己的座標；左、上邊算在內、右、下邊不算（同票 18），圓角外的角落不算，最近的鍵用到圓角矩形的距離。性質測試在 `tests/hit_properties.rs`（上層優先、同層後者在上、旋轉內外、任意倍數縮放；縮放只比離邊界 1e-6 內結果不變的點）。變異驗證：旋轉方向寫反、拿掉 z 排序各自讓性質測試紅。
- 2026-11-04 綁定與網頁：`new Layout(json, device, orientation)` 驗證後選版本、警告印在 console；網頁以螢幕短邊 600 px 分手機與平板。按鍵照佈局畫旋轉、圓角、淺色顏色、透明度、字級與位移，`aria-label` 用 `a11yLabel`；組字中照 `composing` 換標籤與動作。「是否在組字」由綁定記上一個 Snapshot：`qim-core` 沒有這個查詢，票 11、13 正在並行改核心，不在本票加；規則本身（選哪個標籤與動作）在 `qim-layout`。`text`、`switchPage`、`nextKeyboard`、`toggleAscii` 先當 `Key::Other`，留給票 20（寫在票 20〈票 19 留下的〉）。
- 2026-11-04 重疊測試佈局：`crates/qim-layout/tests/fixtures/overlap.json`，Rust（`tests/overlap.rs`）與 Playwright（`web/e2e/touch-overlap.spec.ts`，用 `page.route` 換掉預設佈局）讀同一份。截圖：`.scratch/m1/evidence/19-overlap-layout.png`、`19-overlap-tap.png`（點重疊處組字 `n`、首選「弓」、空白鍵顯示「選字」）、`19-overlap-monkey.png`。
- 2026-11-04 亂按：Pixel 7 viewport，Gboard 佈局兩個種子、重疊佈局一個種子，各 400 步（隨機點鍵盤含縫、夾實體 Enter、標點、方向鍵、Tab、Delete、Home、End、大寫、Esc，切換觸控鍵盤），加 20 次並發連點：沒有 console 錯誤、鍵盤不移動、每個鍵都有 `aria-label`。沒找到新問題。
- 2026-11-04 `jsonschema`（關掉預設 features）讓網頁 WebAssembly 由 0.42 MB 變 4.6 MB（gzip 1.2 MB），多半是它帶的 regex 與大數；記在 plan §8，M2 量 iOS 鍵盤擴充記憶體時決定要不要讓鍵盤擴充略過 Schema。
- 2026-11-04 bench 前後對照：沒改核心與資料包，`cargo xtask data` 前後兩份資料包的 sha256 相同（quick `b515cedf128b`、quick-lexicon `4403437674d5`），各集合沒有翻盤、固定題 11/11，`--check` exit 0。每鍵 p95：原生 18.92 µs、WebAssembly 23.92 µs。
- 2026-11-04 票 18 留下的「三個測試檔各自組 JSON 外殼」：不抽。各檔現在組的外殼不同（單頁、直橫兩版、有無字母），抽成共用 helper 要加參數，反而看不出每個測試的佈局長怎樣。
- 2026-11-04 code-review（`9dbdb8c` 起）已處理：切頁的路只算 `tap`；拿掉 `background`；觸控範圍為 0 的鍵誤報被蓋住；取樣加密並含貼邊點；Schema 補 `z`、`grid` 的整數範圍；`Device`、`Orientation` 加 `FromStr`，綁定不再自己解析字串；`Shape::corner_radius` 收掉兩處相同的判斷；旋轉換算、錯誤接句（`Report::into_layout`）、`composing` 篩選各收成一處；`select` 移到 `model.rs` 並改收 `When`；`top_down`、`corners` 改 `pub(crate)`；訊息改稱「佈局版本」；補文字鍵的 Playwright 與名字對照 serde 的測試。
- 2026-11-04 code-review 沒處理：
  - `variant`、`page`、`key` 三個欄位在多個 `Warning` 重複（Data Clumps）：等 M5 編輯器要用位置標警告時再收成一個型別。
  - 網頁照 `style`、`rotate`、`radius`、`a11yLabel` 畫鍵算票外：不照畫的話，旋轉或圓角的鍵畫的位置和點擊判定對不上，保留。
- 2026-11-04（使用者決定，經協調者）：`touchInset` 改名 `extendedEdges`，語意不變（正數往外擴、負數往內縮），與 Delta skin 同名同義；格式還沒發佈，舊名不留相容，寫舊名的檔案不符合 Schema（`tests/validate.rs` 有一例）。上面 Comments 提到的 `touchInset` 都指這個欄位。
