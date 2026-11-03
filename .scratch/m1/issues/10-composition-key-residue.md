# 10: 組字中按鍵殘留

Status: ready-for-agent
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

- [ ] 性質測試「`handled` 為假時組字是空的」先紅後綠（commit 歷史或 Comments 可看出先紅）。
- [ ] brief §4.3 寫明標點與大寫字母在組字中的行為與參考了誰。
- [ ] Playwright：組字中按 Enter，讀回輸入框是原樣的輸入碼；按 `z`、標點、大寫、方向鍵，讀回輸入框沒有多出字元、組字符合 brief。
- [ ] `toKey` 有單元測試。
- [ ] 〈M0 留下的〉每條有結果。

## Comments
