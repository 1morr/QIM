# 12: 個人 bigram 與學習總開關

Status: ready-for-agent
Blocked by: 03, 04
讀: plan §2.3（`personal`）、§2.4（學習、匯出格式）、§5.2（`--learning`）；brief §4.5、§7；`docs/research/libime-user-model.md`；`docs/research/userdict-text-formats.md`；spec〈範圍〉〈共同規則〉

## What to build

學過的接續習慣影響排序：選過「前一個字詞 → 這一段」，下次同樣的前文下這一段加分。另補上 brief §7 的學習總開關。

- 個人 bigram（plan §2.3）：上屏時記（前一個字詞 → 這一段），只看文字、跨方案共用；前一個字詞來自同一次組字的前一段或 `set_context` 的前文。打分照 libime 的內插做法併進 `personal`，只加不減，參數用 `bench --learning` 調。
- 學習總開關：`Options` 加一項（預設開），關掉時不寫任何學習資料、已學的照用（和無痕不同：無痕連前文也不用）；網頁設定區加開關。
- 匯出格式升 `# qim-userdict v2`（使用者 2026-11-11 決定）：加個人 bigram 的列；`import` 收 v1 與 v2，v1 沒有 bigram。寫回 plan §2.4。

## 驗收

- [ ] 單元測試：選過「X → Y」後，前文是 X 時 Y 的排名上升；學習開關關掉時 `UserModel` 不變、已學的仍生效。
- [ ] `bench --learning` 學習開的修正次數比改之前少（含「有前文」集合），貼對照；`bench --check` 不變差。
- [ ] `export` → `import` 等價（性質測試）；一份 v1 的檔仍匯入得了（測試用固定檔）。
- [ ] Playwright：關掉學習後選兩次同一候選，它不成首選；截圖。
- [ ] plan §2.3、§2.4 已更新；`cargo xtask check` 綠燈。

## Comments
