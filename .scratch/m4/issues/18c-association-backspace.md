# 18c: 有聯想時第一下退格就刪字

Status: ready-for-agent
Blocked by: 18
讀: plan §2.1 聯想那條（「Esc 與退格只收起聯想」）、§2.4〈退格撤銷〉；brief §12.2 librime-predict 與退格撤銷、§12.4 黑箱 T11；票 15 的 Comments；spec〈共同規則〉

## What to build

期中檢查插入的修復票（使用者 2026-11-13 試用後決定）。現在上屏後出聯想時，第一下退格只收起聯想、吃下不交還（照 librime-predict 與 Gboard 黑箱 T11），第二下才刪字。改成：有聯想時退格交還宿主刪字，聯想同時收起。這偏離 librime-predict 與 Gboard 的慣例，是使用者的決定，plan 與 brief 照改並註明。

- Esc 照舊只收起聯想、吃下不交還；只改退格。
- 和〈退格撤銷〉的關係（使用者 2026-11-13 決定）：這下退格是上屏後第一個交還宿主的鍵，照〈退格撤銷〉撤銷這次上屏的學習，和沒有聯想時一致；殼的 3 秒計時照舊。實作時發現行不通就停下回報。plan §2.4「只收起聯想的退格不算夾在中間」那句跟著改。
- 改 plan §2.1、§2.4，brief 裡描述 QIM 行為的地方（〈查證過的事實〉裡 librime-predict、Gboard 的事實不改，只在 QIM 的取捨處註明偏離）。

## 驗收

- [ ] 核心測試（先紅）：上屏後有聯想時按退格，`Snapshot` 交還宿主（宿主刪字）、聯想收起；Esc 照舊只收起；這下退格撤銷這次上屏的學習（匯出比對）。
- [ ] Playwright（桌面，觸控鍵盤的刪除鍵也驗）：上屏後出聯想，按一下退格，輸入框少一個字、聯想列不見；截圖存 `.scratch/m4/evidence/`。
- [ ] plan §2.1、§2.4 與 brief 的 QIM 行為已改；`bench --check` 不變；`cargo xtask check` 綠燈。

## Comments
