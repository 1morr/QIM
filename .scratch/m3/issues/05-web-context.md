# 05: 網頁接上下文

Status: ready-for-agent
Blocked by: 01, 04
讀: plan §2.1（上下文、數字後半形）、§3.1；brief §8、§12.3（iOS 上下文長度）；spec〈範圍〉〈共同規則〉；`.scratch/m2/issues/12-m2-close.md` Comments 的「M3」各條（上下文那條）

## What to build

在網頁的輸入框打字時，候選照游標前的文字變。

- 網頁把輸入框游標前最多 200 字傳給 `set_context`（聚焦、移動游標、上屏之後）；無痕時不傳。
- 數字後半形改看前文（plan §2.1：「M3 有 `set_context` 後改看前文」），`Session` 記最近上屏字的那份狀態要不要留由票定，寫回 plan §2.1。
- 殼沒給前文時要不要用這個輸入框上屏過的字補（iOS 常只讀到一兩句）：先查成熟做法（例如 fcitx5 / libime 在宿主不給 surrounding text 時怎麼辦）再定，寫回 plan §2.1 並附來源。

## 驗收

- [ ] Playwright：輸入框先有一段前文，打同一串碼讀回首選和沒有前文時不同；把游標移到別處後候選跟著變；截圖。
- [ ] Playwright：前文以半形數字結尾時 `.` 上屏半形；無痕時前文不影響候選。
- [ ] 「殼沒給前文」的決定與來源寫在 plan §2.1。
- [ ] 實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments
