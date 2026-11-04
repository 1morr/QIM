# 10: 組字中按鍵殘留

Status: done
Blocked by: 07
讀: plan §2.1（`handled`）、§5.1；brief §4.3；`docs/research/closed-source-imes.md` §4 的 T18；`docs/research/blackbox-baseline.md` 的 T18（票 08、09 做完的話）；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 8、9」那條）

## What to build

組字中按任何鍵都不會把字元漏進輸入框、組字卻還留著。現在按 `z`、Enter、標點、大寫字母或方向鍵，核心回 `handled` 為假，宿主照常輸入那個字元，組字卻沒清；連打之後組字變長，更常碰到。

- 先加性質測試「`handled` 為假時組字是空的」（現在會紅），再定各鍵行為：
  - Enter：把輸入碼原樣上屏（brief §4.3）。
  - `z`：萬用鍵（M4）之前吃下、不動作。
  - 標點與大寫字母：參考 Rime、macOS 速成與黑箱基線 T18（已做完的話），定了寫回 brief §4.3。
  - `Left` / `Right`：組字中修改（票 21）之前吃下、不動作。
- 網頁的鍵碼轉 `Key`（`toKey`）分支變多了，加單元測試工具並補測試（〈M0 留下的〉）。

## M0 留下的

票 09 Comments「M1 票 8、9」：組字中 `z`、Enter、←、→ 交還宿主的行為；`toKey` 的分支在鍵位變多時加單元測試工具再測。

## 驗收

- [x] 性質測試「`handled` 為假時組字是空的」先紅後綠（commit 歷史或 Comments 可看出先紅）。
- [x] brief §4.3 寫明標點與大寫字母在組字中的行為與參考了誰。
- [x] Playwright：組字中按 Enter，讀回輸入框是原樣的輸入碼；按 `z`、標點、大寫、方向鍵，讀回輸入框沒有多出字元、組字符合 brief。
- [x] `toKey` 有單元測試。
- [x] 〈M0 留下的〉每條有結果。

## Comments

- 先紅：性質測試 `a_key_left_to_the_host_leaves_no_composition_behind`（`crates/qim-core/tests/properties.rs`）在改核心前跑，紅在最小反例 `keys = [Letter('a'), Left]`：「Left 交還宿主，組字還在」，`left: "a"`、`right: ""`。改完綠。
- 各鍵的決定（brief §4.3，附 librime `9777cb2` 與 rime-prelude `082425e` 的出處，事實記在 brief §12.2）：
  - Enter 照 librime express_editor 的 `CommitRawInput`：選定的段上屏字詞，其餘上屏輸入碼。brief 原本只寫「輸入碼原樣上屏」，沒講有選定分段時怎麼辦。
  - 標點與大寫字母：先上屏整個組字再接字元。「上屏整個組字」和一直按空白相同，沒有字的碼原樣上屏。標點轉全形，取 Rime `half_shape` 每鍵第一個是全形的選項，不出選單。
  - `z`、← / →、`ToggleAscii`、新的 `Key::Other` 組字中吃下不動作。沒有組字時交還宿主。
- 順手做的：沒有組字時，有全形的標點也上屏全形。brief §4.3 原本就寫「標點預設全形」，M1 沒有別張票負責；核心同一張表，只轉組字中的話前後不一致。
- 亂按一輪用一次性 Playwright 腳本，沒進 repo：
  - 內容是隨機混按 1,500 鍵，鍵包含字母、`z`、大寫、標點、Enter、Esc、各方向鍵、Home、End、Delete、Tab、翻頁、數字；另外 20 輪不等畫面連送 60 鍵。
  - 找到兩個問題：組字中 Tab 移走焦點；Delete、↑ ↓、Home、End 改動輸入框或游標，組字卻還在。
  - 先寫成 `web/e2e/composition-keys.spec.ts` 的「組字中按 Tab、Delete…」測試，紅在 Tab 失焦。之後加 `Key::Other`，網頁沒對應的鍵都轉成它，測試轉綠。
  - 切換模式還沒有，`ToggleAscii` 在票 23。
- 〈M0 留下的〉：
  - `z`、Enter、← / → 已定，都有核心測試與 Playwright。
  - `toKey` 用 Node 內建的 `node:test` 測（`web/test/keys.test.ts`），不加依賴。`cargo xtask check` 多一個 `unit` 步驟，`--fast` 也跑。
- `cargo xtask bench` 前後：各集合（含個人集）的答錯、修正次數、KSPC、跳動、依長度分組全部相同，沒有翻盤，固定題 11/11。基準只送字母、空白、數字，本票沒改這些路徑。每鍵 p95 前後如下，差距是雜訊：

  | 環境 | 改前 p95 | 改後 p95 |
  | --- | ---: | ---: |
  | 原生 | 19.21 µs | 19.88 µs |
  | WebAssembly | 23.79 µs | 27.87 µs |

- code-review 未處理的發現：
  - `commit_all_then` 自己重做「一直按空白」，沒有重用 `select_at`。`select_at` 會改狀態，要重用得先拆開。票 13 學習接上選字時，兩處要一起看。
  - `Key::Punct(char)` 在核心端什麼字元都收。「可見 ASCII」只寫在 doc comment，由殼保證。
  - `Punct`、全形、`Other` 沒進 `GLOSSARY.md`，留給下次 `/grill-with-docs`。
- 留給票 23：
  - Tab、Delete、↑ ↓、Home、End 現在只是組字中吃下，真的鍵位行為在票 23 做。
  - 帶 Ctrl / Cmd / Alt 的鍵照舊交給瀏覽器，所以組字中 Cmd+V 會貼進輸入框而組字還在。
  - 標點選單（Rime 的多選項、macOS 的標點選單）沒做。引號配對屬智慧標點（M4）。

