# 11: 英文殘項：觸控鍵位接英文修正、無痕與換行後的句首大寫

Status: ready-for-agent
Blocked by: 03
讀: brief §4.7、§4.8；ADR 0008；`ARCHITECTURE.md`〈核心對殼〉；`.scratch/m4/issues/25-autocorrect.md`、`24-english-mode.md` 的 Comments

## What to build

M4 留下的英文模式問題：

- 觸控點擊的按鍵機率接進英文修正（現在英文容錯不看鍵位鄰近，M4 票 25）。要改的接點（開工對程式碼補正，使用者 2026-11-18 同意）：`Session::handle` 在英文模式保留 `Weighted` 的各字母機率（現在換成最高的那個）；`TypedRun` 存每個字母的機率；修正的扣分（`Edit::Typo`）和鄰鍵機率合起來；`inspect` 與 debug 面板加一欄（票 02 的慣例）；實體鍵盤（`Physical`）不接。接上後用 `scripts/experiments/autocorrect-threshold/` 重量自動修正的 4 倍門檻（`AUTOCORRECT_MARGIN`），量法與結果寫在 Comments，門檻要改先問使用者。
- 無痕時殼不給前文，剛聚焦時不知道游標前有沒有字、換行後不句首自動大寫（M4 票 24；Enter 經 `forget_context` 把句首狀態設成不知道）：加一個新介面讓殼告訴核心「游標在行首或空欄」而不給前文內容，照成熟產品的做法設計並寫在票裡（核心對殼的介面，`ARCHITECTURE.md`〈核心對殼〉同步）。`tests/english_mode.rs` 斷言「不知道前文時不大寫」的幾條改成只管「沒告知行首」的情況。

## 驗收

- [ ] 觸控打偏一格的英文字（測試給定點擊位置）修正成鄰鍵的詞；實體鍵盤的行為不變。
- [ ] 4 倍門檻重量的數字寫在 Comments；`cargo xtask bench --check` 沒有翻盤。
- [ ] 無痕時剛聚焦與換行後打英文會句首大寫（開著句首大寫時），Playwright 讀回。
- [ ] 動到網頁的部分照 spec〈共同規則〉亂按一輪，找到的問題先寫成會紅的 Playwright 測試再修。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
