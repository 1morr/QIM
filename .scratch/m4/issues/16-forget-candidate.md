# 16: 撤銷學習

Status: ready-for-agent
Blocked by: 12
讀: brief §4.5；plan §2.1、§2.4、§3.1；spec〈細節〉（撤銷學習）；`docs/research/cyime.md` §4；spec〈共同規則〉

## What to build

學錯的候選能清掉：

- 核心 API 清掉一個候選的學習紀錄：詞庫原有的詞回到原位，自動造詞不再出，再選可以重新學。不做永久屏蔽（CyIME 的永久屏蔽被使用者評為偏離預期）。手動加詞與常用語的處理（建議只清計數、不刪詞）由票定。
- 網頁桌面用 Shift+Delete（Rime 慣例）；作用在哪個候選（首選或高亮的）由票定。觸控的手勢跟 M5 重做候選列一起定。
- 刪除在同步時怎麼傳到別台，由票 17 處理。

## 驗收

- [ ] 核心測試：學成首選的詞清掉後回到不學習時的位置；自動造詞清掉後不再出；再選兩次又成首選。
- [ ] Playwright：Shift+Delete 後讀回候選回到原位，debug 面板的學習加分歸零。
- [ ] plan §2.1、§2.4 寫明；`cargo xtask check` 綠燈。
