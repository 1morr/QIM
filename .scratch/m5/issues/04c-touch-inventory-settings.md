# 04c: 盤點三：Gboard 設定頁的全部項目

Status: ready-for-agent
Blocked by: None
讀: brief §4.4、§7；`docs/research/gboard-english-2026-11-15.md`（上次 Gboard 實測的寫法）；`docs/research/facts-products.md`（M5、M47–M49）；spec〈範圍〉；`web/src/settings.ts`（QIM 現有的開關）

## What to build

盤 Gboard 設定頁的全部項目：

- 每一個設定畫面與分類（語言、偏好設定、主題、文字修正、滑行輸入、語音輸入、字典、搜尋、隱私、進階等，實際有哪些照實測），每一項記名稱、預設值、選項或段數、改了之後鍵盤怎麼變（打字讀回）。
- 鍵盤高度的段數與每段的高度；按鍵音量、震動強度、長按延遲的範圍。
- 和 QIM 現在的設定（設定表 17 列、數字列、深色、字根提示）一項項對：QIM 有而 Gboard 沒有的也列出來。
- 平台才做得到的項目（語音、震動、按鍵音、隱私與分享統計等）照樣列，建議欄寫「平台才做」。

共同做法（04、04b、04c 一樣；使用者 2026-11-18 定：完整對照 Gboard、所有功能都盤，包括平台才做得到的）：

- 用 `orca emulator-android` 在 Android 模擬器的 Gboard 逐項實測，環境（Gboard 版本、Android 版本、宿主 App）與量法照 `docs/research/gboard-english-2026-11-15.md` 的寫法記在檔頭；每個觀察附截圖（`.scratch/m5/evidence/`）或讀回的畫面文字。
- 每一項記：Gboard 怎麼做（入口、操作、預設值）、QIM 現在有沒有（有的寫在哪）、網頁做不做得到、建議（第一期／第二期／平台才做／不做）。建議只是建議，04d 彙整後由使用者選。
- 外部事實（版本相關的行為、平台 API 限制）補進 `docs/research/facts-products.md` 或 `facts-platforms.md`，給編號、附來源。
- 只盤不做：不改程式、不改 spec 的範圍（04d 統一寫回）。

寫到 `docs/research/gboard-touch-settings-<日期>.md`。

## 驗收

- [ ] 研究檔寫完，每個設定畫面都有截圖，每一項有預設值與作用。
- [ ] QIM 現有的每個設定都對到 Gboard 的一項或註明 Gboard 沒有。
- [ ] 每一項都有「QIM 現在有沒有、網頁做不做得到、建議」三欄。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
