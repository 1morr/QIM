# 15: 自動記住 email 這類字串

Status: ready-for-agent
Blocked by: 05, 12
讀: plan §2.1（`text`、英文直通、臨時英文、上下文）、§2.4；brief §1.5、§4.7、§8；`docs/research/blackbox-baseline.md`（T5）；spec〈範圍〉〈共同規則〉

## What to build

使用者打過一次的 email 地址（與票內決定的其他字串，例如網址），之後打開頭幾個字母就出現在候選裡，選了整串上屏。

- 先查成熟做法（Gboard、iOS 的 email 建議、SwiftKey）再定：從哪裡記（臨時英文上屏的、`text` 動作的、前文裡出現的）、打什麼會觸發（字母開頭的前綴、要幾個字母）、候選排在哪、會不會擋到英文候選與中文首選。決定寫回 plan §2.1、§2.4，註明採用誰的慣例。
- 英文直通時鍵交還宿主、核心看不到，這點要在設計裡交代。
- 無痕與學習關掉時不記；存在使用者資料裡、可刪；要動匯出格式時照 spec〈共同規則〉加進 v2。

## 驗收

- [ ] 單元測試：記過 `ivan.test@example.com` 後，打票內定的觸發前綴時它在候選裡、選了上屏整串；無痕時不記。
- [ ] Playwright：打一次 email、再打前綴、選它，讀回輸入框；截圖。
- [ ] 黑箱 T5 在 QIM 網頁重跑，補進 `docs/research/blackbox-baseline.md`。
- [ ] `bench --check` 不變差；實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments
