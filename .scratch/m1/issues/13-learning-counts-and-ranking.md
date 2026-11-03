# 13: 學習：計數與排位

Status: ready-for-agent
Blocked by: 04, 07
讀: plan §1.3（`SystemTime` 禁用）、§2.1（`UserModel`、`Options`、`set_options`）、§2.3（`personal`）、§2.4、§5.1、§5.2（學習模式）；brief §4.5、§12.2（衰減）；`docs/research/open-source-imes.md` §5；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 5」那條）

## What to build

候選依使用次數前移：同一個碼選同一個非首選兩次，它就成首選；學過的詞在更長的連打裡也加分。無痕模式下什麼都不記。`bench --learning` 量得出學習讓修正次數下降。

- 開工先讀 libime 怎麼把使用者資料併入 lattice（`UserLanguageModel`、`HistoryBigram`，GPL 只取做法），摘要記到 `docs/research/`。
- 學習計數以（方案, 輸入碼, 候選文字）為鍵，記上屏次數、衰減值、最後 tick；tick 由 `UserModel` 自己累加（plan §2.4），衰減參考 librime `formula_d`（時間常數 200）。上屏時整串與路徑每一段各記一次。
- 排位規則 2：上屏次數 ≥ 2 且距最後上屏 ≤ 10,000 次 → 首選，多個取最近；規則 1、3 在票 14，規則 4 在票 15，本票把「依序套用規則」的骨架做好。
- `personal`：個人機率由衰減值換算（公式本票定，寫回 plan §2.3），`w` 初值 0.2，只加不減。
- `Options { incognito }` 與 `Session::set_options`（〈M0 留下的〉）；無痕時不寫任何學習資料。網頁加無痕開關。
- 分方案：用第二份測試方案（同碼不同字）驗證計數互不影響（plan §2.4）。
- `cargo xtask bench --learning`：照固定順序打一個集合，模擬選字並讓 `UserModel` 學習，和同順序不學習時比修正次數（本來列在首選字基準，移到本票，見 progress 偏差）。其他模式仍用空白的 `UserModel`。
- 本票的學習只在記憶體；存檔與匯出匯入在票 14。

## M0 留下的

票 09 Comments「M1 票 5」：`Options` 沒有簽章用到它，plan §2.1 的 `set_options` 跟著第一個真的選項一起加；本票的 `incognito` 就是第一個。

## 驗收

- [ ] `docs/research/` 有 libime 使用者模型的摘要；`personal` 的換算公式寫回 plan §2.3。
- [ ] 核心測試：同碼選同一個非首選兩次後成首選，兩次之間隔 500 次上屏也成立。
- [ ] 核心測試：學過「門口」後，一串含 `anr` 且原本首選不含「門口」的連打，首選含「門口」。
- [ ] 性質測試：無痕模式下 `UserModel` 不變；兩個方案的計數互不影響。
- [ ] `bench --learning` 在口語集與個人集上，學習開的修正次數低於學習關（貼輸出）。
- [ ] Playwright：同一個碼選同一個非首選兩次，讀回它成首選；開無痕後重做，讀回不變。
- [ ] 〈M0 留下的〉已處理。

## Comments
