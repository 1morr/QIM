# 20: 滑動輸入研究與 ADR：和 flick 怎麼分

Status: ready-for-agent
Blocked by: 12
讀: `docs/research/spatial-correction-swipe.md` §5；`docs/research/facts-products.md`；`ARCHITECTURE.md`〈佈局格式〉；ADR 0004

## What to build

先讀 FlorisBoard v0.5.2 與 AnySoftKeyboard 的滑動輸入（GPL 的只取做法），寫 ADR 定：

- 滑動輸入和佈局裡按鍵自己的滑動動作（flick）怎麼分（研究建議照青簡的方向扇形）。
- 解碼的輸入：軌跡經 `qim-layout` 的按鍵幾何轉成什麼給核心，不寫死 QWERTY；核心不知道佈局。
- 英文滑動與中文（形碼）滑動共用到哪一層。

外部事實補進 `facts-products.md`。

## 驗收

- [ ] ADR 寫完並經使用者確認（`AGENTS.md`〈使用者確認〉）。
- [ ] 新事實有編號與來源。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
