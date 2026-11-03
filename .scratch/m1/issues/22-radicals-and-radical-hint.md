# 22: 字根顯示與字根提示

Status: ready-for-agent
Blocked by: 07
讀: plan §2.5（`[radicals]`、`radical_hint`）、§4.1 步驟 5；brief §4.2（預設五代、可改三代）、§4.3（組字中顯示字根、快捷鍵查字根提示）；ADR 0003；spec〈共同規則〉

## What to build

組字中顯示已打的字根（打 `anr` 顯示「日弓口」）；用快捷鍵查目前候選的倉頡碼字根提示，預設五代、可改三代。

- 組字的字根來自方案檔 `[radicals]`，經 `Snapshot.composition` 給殼。
- 字根提示資料包（plan §4.1 步驟 5）：字 → 倉頡三代 / 五代碼，只給提示用，不進 lattice（ADR 0003）；`LICENSE` / `NOTICE` 同 libcangjie。
- 查詢介面放核心或資料包讀取層（守 plan §1.3），殼只顯示。
- 快捷鍵與觸控上的叫出方式在本票定，參考黑箱基線與 macOS 速成（已做完的話），寫回 brief §4.3。

## 驗收

- [ ] Playwright：打 `anr`，讀回組字顯示「日弓口」。
- [ ] Playwright：按快捷鍵，讀回首選的字根提示是五代碼；切三代後讀回三代碼。
- [ ] `--verify` 抽查幾個字的三代 / 五代提示；建兩次 sha256 相同。
- [ ] brief §4.3 寫明快捷鍵。

## Comments
