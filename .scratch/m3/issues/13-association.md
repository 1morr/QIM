# 13: 聯想

Status: ready-for-agent
Blocked by: 05, 12
讀: plan §2.1、§2.4；brief §1.3（iOS 速成的聯想）、§1.5、§7；`docs/research/blackbox-baseline.md`（T6、T8）；`GLOSSARY.md`〈聯想〉；spec〈範圍〉〈共同規則〉

## What to build

上屏之後、還沒開始打下一個字時，候選列出現接下來可能要打的詞，選了就上屏、可以一直接下去。

- 先查成熟做法定按鍵行為（iOS / macOS 速成、Microsoft 速成的聯想詞）：數字鍵或點擊選、空白要不要選、Esc 收起、打字母直接開始新的組字；寫進 plan §2.1，註明採用誰的慣例。
- 來源：詞庫裡以剛上屏的字詞結尾開頭的詞、字級 bigram、個人 bigram，用同一把尺排（plan §2.3）；排序規則寫回 plan §2.3 或 §2.4。
- `Options` 加聯想開關（預設開，brief §7）；無痕時不用前文；網頁設定區加開關。

## 驗收

- [ ] 單元測試：上屏「門」後沒有組字時有聯想候選（例如「口」）；選了之後接著出下一輪；打字母時聯想消失、照常組字；關掉開關時沒有聯想。
- [ ] Playwright：實體鍵盤與觸控各打一次「上屏 → 選聯想 → 再選聯想」讀回輸入框文字；截圖。
- [ ] 黑箱 T6、T8 在 QIM 網頁重跑，結果補進 `docs/research/blackbox-baseline.md` 的 QIM 欄。
- [ ] `bench --check` 不變差；實際操作亂按一輪；`cargo xtask check` 綠燈。

## Comments
