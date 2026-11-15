# 23: 觸控的中英鍵與英文鍵盤

Status: ready-for-agent
Blocked by: 18, 18b
讀: brief §4.4；plan §2.6（佈局格式、動作）、§3.1、§5.3；ADR 0004；spec〈共同規則〉

## What to build

觸控現在只有接實體鍵盤才進得了英文（brief §4.4 的已知限制）。加上：

- 預設觸控佈局加中英鍵與 QWERTY 英文字母頁。英文頁與中英鍵做成佈局的頁面與按鍵動作，自訂佈局可以自己決定放不放；格式只加不改，先看能不能沿用現有的 `toggleAscii` 動作。中英鍵的位置照 Gboard 或研究定。
- 先照 `AGENTS.md`〈UI〉出英文頁與中英鍵的設計稿（`docs/design/`），使用者確認後才實作；行為照 Gboard 實測，設計稿管外觀與互動細節。
- 第一版英文頁接英文直通（字母直接上屏），英文模式在票 24 接上；英文狀態的建議列沿用現在的候選列（M5 重做）。

## 驗收

- [ ] 英文頁與中英鍵的設計稿在 `docs/design/`，使用者確認過（回覆記在 Comments）。
- [ ] 內建佈局的嚴格檢查照過（plan §5.3）；格式加的東西有 Schema 與雙向測試。
- [ ] Playwright 手機 viewport：按中英鍵切到英文頁、打 `hello` 上屏、再切回中文頁；英文頁打得出半形的 `? ! :`；截圖。
- [ ] brief §4.4 的已知限制改寫；plan §2.6 更新；`cargo xtask check` 綠燈。

## Comments

- 設計稿 `docs/design/23-touch-english-keyboard.html` 已出（2026-11-15），等使用者確認標籤（5.1）、英文頁數字角標（5.2）與整體；確認前不實作。
- Gboard 的中文佈局沒有中英鍵（事實 M49），中英鍵放在 Gboard「Language switch key」的位置：逗號與空白之間，空白 5 格縮成 4 格。
- 等使用者：設計稿確認（中英鍵標籤、英文頁數字角標、設計稿規格寫進 brief、整體確認），協調者已轉使用者。
- 做到哪：設計稿與 Gboard 實測截圖（`.scratch/m4/evidence/23-*.png`、事實 M49）已 commit；brief、plan、佈局格式與程式都還沒改。
- 剩什麼：使用者確認後，照設計稿加佈局格式欄位（沿用 `toggleAscii`、Trime `ascii_keyboard` 式的換頁欄位，Schema 與雙向測試）、預設佈局加中英鍵與英文三頁、網頁接上、Playwright 驗收截圖、改 brief §4.4 已知限制與 plan §2.6、`cargo xtask check`。
