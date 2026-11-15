# 20: 譯詞的來源與授權

Status: done
Blocked by: 18
讀: brief §1.5（譯詞）、§5、§11；spec〈細節〉（譯詞）；skill `mattpocock-skills:research`

## What to build

找繁體與粵語對英文的詞表，能做成可選資料包散布：

- CC-CEDICT、CC-Canto 是不是 share-alike、能不能以資料包散布；粵典的英文釋義是非商業條款；其他候選來源。
- 查不到合用的，評估自己用 LLM 離線生成（青簡做法）：成本、生成結果的授權、品質抽查方法。要花錢的先問使用者。

輸出 `docs/research/translation-sources.md`，結論摘進 `docs/research/facts-tables.md` 與 §11。

## 驗收

- [x] 每個來源附授權條款原文的連結與能不能用。
- [x] 結論：譯詞暫不做（使用者定），研究與估計費用留在 translation-sources.md；brief〈事實索引〉更新；票 29 已 wontfix。（驗收依使用者 2026-11-15 決定改寫）

## Comments

- 等使用者：打中文時候選旁要顯示英文譯詞（例「發票」旁的 invoice），譯詞從哪來？(1) 用 LLM 離線生成整份詞庫（約 25 萬詞），Claude Sonnet 5.5 Batch 約 $17、Haiku 5.5 約 $1，要你提供 API 金鑰並盲評 100 詞，輸出可標 CC0；(2) 免費的 CC-CEDICT 加 CC-Canto（CC BY-SA 4.0），但前 2 萬個常用詞只有約 48% 拿得到能直接上屏的譯詞；(3) CC-CEDICT 為主、LLM 補缺，整包仍是 BY-SA。票 20 的建議是 (1) 用 Sonnet。不決定的話票 20 收不了尾（brief 要寫進採用的來源），票 29 譯詞資料包不能開工；協調者的隊列停在票 20。研究在票 20 的分支 `1morr/t20` 的 `docs/research/translation-sources.md`。
- 決定：暫不需要譯詞，移出 M4（roadmap〈不排里程碑〉）；本票的來源研究照常合併、不花錢、brief 不寫採用的來源，票 29 wontfix（使用者定）
- 做到：研究文件 `docs/research/translation-sources.md`（各來源的授權原文連結與能不能用、CC-CEDICT 覆蓋率、LLM 費用與抽查方案）、覆蓋率腳本 `scripts/experiments/gloss-coverage/`、事實 T34–T38；結論段依使用者決定改成暫不採用、不花錢，brief 只改〈事實索引〉的 T 條數。
- 全部檢查：`cargo xtask check` 通過（本 worktree 先 `npm ci`）。
- code-review（`66077e1` 起，Standards／Spec 兩軸）：已處理：Comments 的紀錄與決定行改成行首格式；授權原文連結補齊（CC-Canto、Unihan、Wiktionary、ECDICT、MUSE、DoJ、WordNet 類）；腳本實際挑「第一個化得成短詞的義項」，文件、T36 與 docstring 改成照實寫並標明是上限；T38 只寫查到的禁止條款、附官方連結；費用表補旗艦單價；腳本 docstring 改中文、變數改名、少算一次、缺參數印用法（輸出逐字不變）。未處理：檔名不加日期（內容是日後重開時要接著更新的主題整理，見下面的決定）；`EXTRA_FOLDS` 手抄 Rust 的並存寫法、快取路徑只寫 macOS 預設（一次性實驗腳本，docstring 註明）；腳本印的報表是中文（給人讀的報表，不是 log，同 `llm-rerank`）；T37 的來源只連研究文件（各條原文連結在研究文件 §1，同 T3 前例）。

Session 紀錄：查譯詞來源的授權與 LLM 生成費用，寫成研究文件、事實 T34–T38 與覆蓋率腳本；使用者決定譯詞移出 M4，結論改為暫不採用，票收掉。
決定：研究文件照票上的檔名寫成主題整理 `translation-sources.md`，不加日期：譯詞日後重開時從它接著更新（Agent 定）
決定：覆蓋率實驗的腳本收進 `scripts/experiments/gloss-coverage/`，CC-CEDICT 與 CC-Canto 的下載檔不進 repo，研究文件記檔頭日期與 sha256（Agent 定）
決定：CC-CEDICT 釋義品質由 Agent 一人抽判 100 條，只看量級，在文件與 T36 註明（Agent 定）
決定：結論摘成事實 T34–T38 放 `facts-tables.md`，LLM 條款與定價也放 T（T38），不另開事實檔（Agent 定）
