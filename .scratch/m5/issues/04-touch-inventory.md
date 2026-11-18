# 04: 盤點一：Gboard 的工具列、面板與候選列

Status: ready-for-agent
Blocked by: None
讀: brief §4.4、§7；`docs/research/gboard-english-2026-11-15.md`（上次 Gboard 實測的寫法）；`docs/research/facts-products.md`（M5、M47–M49）；spec〈範圍〉

## What to build

M5 要把網頁觸控鍵盤做到平台樣板，先把 Gboard 弄清楚。這張盤工具列、各面板與候選列：

- 工具列：每個入口（設定、emoji、剪貼簿、文字編輯、單手模式、浮動鍵盤、調整大小、翻譯、語音、主題、搜尋、校對或 AI 寫作等，實際有哪些照實測），怎麼自訂順序、收在「更多」裡的有哪些。
- 各面板：emoji（分類、最近用過、搜尋、膚色、Emoji Kitchen）、貼圖、GIF；剪貼簿（歷史、釘選、自動清除的時限、截圖、刪除）；文字編輯（方向鍵、選取、全選、剪下、複製、貼上）；翻譯、單手、浮動、調整大小的畫面與操作；其他實測到的面板。
- 候選列：組字中與沒組字時各出什麼；複製後的剪貼簿建議（點了貼上、文字裡的網址等另外拆成一個候選，還拆出哪些種類）；打字時的 emoji 建議；英文下一個詞；Android 自動填入的建議；展開與左右滑。
- 瀏覽器讀寫剪貼簿的限制（Clipboard API 的權限，Android Chrome 與 iOS Safari 的差異），查證寫成事實，決定網頁上剪貼簿建議做得到哪裡。

共同做法（04、04b、04c 一樣；使用者 2026-11-18 定：完整對照 Gboard、所有功能都盤，包括平台才做得到的）：

- 用 `orca emulator-android` 在 Android 模擬器的 Gboard 逐項實測，環境（Gboard 版本、Android 版本、宿主 App）與量法照 `docs/research/gboard-english-2026-11-15.md` 的寫法記在檔頭；每個觀察附截圖（`.scratch/m5/evidence/`）或讀回的畫面文字。
- 每一項記：Gboard 怎麼做（入口、操作、預設值）、QIM 現在有沒有（有的寫在哪）、網頁做不做得到、建議（第一期／第二期／平台才做／不做）。建議只是建議，04d 彙整後由使用者選。
- 外部事實（版本相關的行為、平台 API 限制）補進 `docs/research/facts-products.md` 或 `facts-platforms.md`，給編號、附來源。
- 只盤不做：不改程式、不改 spec 的範圍（04d 統一寫回）。

寫到 `docs/research/gboard-touch-panels-<日期>.md`。

## 驗收

- [ ] 研究檔寫完，上面每一類都有實測的觀察與截圖或讀回的畫面文字。
- [ ] 每一項都有「QIM 現在有沒有、網頁做不做得到、建議」三欄。
- [ ] 瀏覽器剪貼簿的限制寫成有編號的事實。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
