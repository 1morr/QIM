# 04b: 盤點二：Gboard 的按鍵、手勢與佈局

Status: ready-for-agent
Blocked by: None
讀: brief §4.4、§7；`docs/research/gboard-english-2026-11-15.md`（上次 Gboard 實測的寫法）；`docs/research/facts-products.md`（M5、M47–M49）；spec〈範圍〉；`crates/qim-databuild/src/gboard.rs`、`web/src/style.css`（QIM 照 Gboard 做的現有尺寸與配色）

## What to build

盤 Gboard 的按鍵、手勢與佈局：

- 各頁的佈局：速成、英文、?123、=\<、數字鍵盤、常用或 emoji 頁，以及中文佈局裡其他實測到的頁（例如手寫）；直向與橫向；淺色與深色；和 QIM 現在的內建佈局逐頁比，列出不同的地方。
- 按鍵：按下的彈出預覽、長按的選單與角標、長按延遲、Shift 一次與鎖定、刪除鍵按住連刪、Enter 依欄位換標籤。
- 速成的 `z` 鍵：鍵上標什麼、點了在速成做什麼（M4 票 11），建議要不要把 QIM 的「重」改標「＊」。
- 手勢：滑行輸入（軌跡、選字、大寫）、刪除鍵往左滑刪整個詞、空白鍵左右滑移游標、長按空白、在鍵盤上往下滑收起等，實際有哪些照實測。
- 鍵盤高度改了以後按鍵怎麼變：只改高度還是寬高一起（M5 已照「只改高度」定，實測不同就在 Comments 寫 `等使用者：`）。

共同做法（04、04b、04c 一樣；使用者 2026-11-18 定：完整對照 Gboard、所有功能都盤，包括平台才做得到的）：

- 用 `orca emulator-android` 在 Android 模擬器的 Gboard 逐項實測，環境（Gboard 版本、Android 版本、宿主 App）與量法照 `docs/research/gboard-english-2026-11-15.md` 的寫法記在檔頭；每個觀察附截圖（`.scratch/m5/evidence/`）或讀回的畫面文字。
- 每一項記：Gboard 怎麼做（入口、操作、預設值）、QIM 現在有沒有（有的寫在哪）、網頁做不做得到、建議（第一期／第二期／平台才做／不做）。建議只是建議，04d 彙整後由使用者選。
- 外部事實（版本相關的行為、平台 API 限制）補進 `docs/research/facts-products.md` 或 `facts-platforms.md`，給編號、附來源。
- 只盤不做：不改程式、不改 spec 的範圍（04d 統一寫回）。

寫到 `docs/research/gboard-touch-keys-<日期>.md`。

## 驗收

- [ ] 研究檔寫完，各頁佈局都有截圖並和 QIM 逐頁比過。
- [ ] 按鍵、手勢、`z` 鍵、鍵盤高度各有實測的觀察。
- [ ] 每一項都有「QIM 現在有沒有、網頁做不做得到、建議」三欄。
- [ ] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments
