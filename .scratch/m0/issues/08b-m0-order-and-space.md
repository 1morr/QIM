# 08b: 「日」當 `a` 的首選、空白一律上屏首選

Status: done
Blocked by: 08
讀: brief §4.1、§4.3、§5.1、§12.1；plan §2.3、§4.1、§4.5；`docs/research/data-sources.md`（libcangjie、字頻與排序兩節）；`GLOSSARY.md`「候選頁」；票 05、07、08 的 `## Comments`；spec〈範圍〉〈共同規則〉〈驗收〉

## What to build

使用者在 M0 收尾時定了兩件事（2026-11-03）：

1. **`a` 的首選要是「日」**。現在照 libcangjie 的 Big5 排序值，「曰」（23060）排在「日」（23061）前面。
2. **空白鍵永遠上屏第一個候選字**，就是第一頁的首選，翻頁後也一樣。這推翻票 08 寫進 brief §4.3 的「翻頁後上屏目前頁的第一個」。

### 字序

- 要找一條通用的字序規則，讓「日」排在「曰」前面；不准只為「日」寫特例。
- libcangjie 只有 Big5 排序值、沒有字頻（`docs/research/data-sources.md`），所以要另找排序依據。先查證可用的來源與授權，再寫成 2–3 個方案，每個方案附：
  - 授權
  - 「日」能不能排在「曰」前
  - 對 `ar` → 「問」的翻頁數有什麼影響
  - 是否提前做了 M1 的字頻工作
- 用 AskUserQuestion 讓使用者選，附建議。方案定了才動手。
- 新的排序依據若是新來源，照 plan §4 釘進 `data/sources.toml`（URL、sha256、授權），授權檔放進 `NOTICE`。
- 擴展區仍排在連打字集之後（plan §2.3）。
- `--verify` 的抽查加「`a` 的首選是『日』」。同樣的來源建兩次，sha256 仍要相同。

### 空白

- `Session` 的空白改回上屏全部候選的第一個，與目前在第幾頁無關。
- brief §4.3 的空白一列改回「上屏首選」，名詞表「候選頁」照新規則改寫。
- progress〈偏差與決定〉記一行，寫明推翻票 08 的那條、由使用者決定。

### 驗收同步

- spec〈驗收〉改回：打 `a` 按空白，讀回「日」。
- 票 07 的 Comments 加一行指向本票。

## 驗收

- [x] 字序方案經使用者選定，方案與理由寫在 `## Comments`；brief / plan 的字序描述與 progress 偏差已同步。
- [x] `cargo xtask data --verify` 綠燈，抽查含「`a` 的首選是『日』」與原有三項；兩次建置 sha256 相同（貼輸出）。
- [x] `qim-core` 單元測試：翻到第 2 頁後按空白，上屏的是第 1 頁的首選；原有空白測試仍綠。
- [x] Playwright：打 `a` 按空白，讀回輸入框為「日」；打 `ar` 仍能找到「問」並用數字鍵上屏；翻頁後按空白，上屏全部候選的第一個；截圖存證。
- [x] brief §4.3、名詞表、spec〈驗收〉、progress 偏差已更新。
- [x] `cargo xtask check` 全套綠燈。

## Comments

- 2026-11-04 字序方案（使用者選 A）。查證與模擬用 libcangjie `346026b` 與 Unihan 18.0.0（已釘），三個方案都讓擴展區留在後面、「日」排在「曰」前：
  - A（選定）Unihan `kGradeLevel`：香港小學第幾年級要認得這個字（1–6，《朗文初級中文詞典》2001，UAX #38）。有年級的字依年級排前面，其餘照 Big5。Unicode-3.0。日 1 年級、曰沒有年級；`ar` 的「問」從第 2 頁第 5 位到第 1 頁第 1 位。不是字頻，不提前做 M1；M1 的字頻排在年級前。缺點是只有 2,625 個連打字集的字有年級。
  - B Unihan `kHanyuPinlu`（《現代漢語頻率詞典》次數，多為簡體字，經 `kSimplifiedVariant` 對到繁體）：日 1276、曰 12，「問」第 1 位；等於提前做一部分 M1 字頻，語料是大陸普通話，簡繁對應有雜訊（閤拿到「合」的次數，排 `ar` 第 2）。
  - C Unicode 碼位取代 Big5：不用新來源；日 U+65E5 < 曰 U+66F0 只是部首剛好在前，「問」第 2 位、排在罕用字「䦡」後。
  - 「有年級的字在 9 個一頁的第一頁」的比例：現況 294/2564、A 2558、B 2207、C 1078；以年級當常用字的定義，對 A 有利。
- 2026-11-04 擴展區內也照年級排：規則對兩區一視同仁，擴展區實際上幾乎沒有字有年級。
- 2026-11-04 Unihan 原本只用來驗證，現在每次建置都讀 `kGradeLevel`、進速成資料包。票寫「授權檔放進 `NOTICE`」，照既有機制改成：授權寫進 `LICENSE`，`NOTICE` 列兩個來源與取用的內容。plan §4「不同來源的授權不混在同一個檔」原意是不同資料包不共用授權檔（票 05），措辭改清楚。
- 2026-11-04 Unicode License V3 要求整份聲明隨副本出現，上游 `https://www.unicode.org/license.txt` 卻沒有版本化網址、每年改版權年份；釘 sha256 下載會讓新 worktree 與舊 commit 的乾淨建置在改版後失敗（使用者要求本票修掉）。改成全文存 `data/licenses/Unicode-3.0.txt`（2026-01-05 版，sha256 `e7a93b00…`），`sources.toml` 加 `license_text` 欄位指向它，建置從 repo 讀、整份寫進 `LICENSE`，不再下載。`license_text` 與 `license_file` 加 `license_statement` 二選一，都寫或都不寫、檔案不存在都讓建置失敗（`tests/run.rs` 三支測試）。
- 2026-11-04 空白改上屏 `Session` 全部候選的第一個，數字鍵仍選目前頁；`commit_at(index)` 改成 `commit(Option<String>)`，兩個鍵各自決定上屏哪個。e2e 的 `a` `2` 改驗第 2 個「曰」。
- 2026-11-04 驗收輸出：
  - `cargo xtask data --verify`：連打字集 17611 字、擴展區 57052 字、速成碼 649 個，八項 `[通過]`（含「抽查：a 的首選是「日」」），`verify 通過`，sha256 `31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f`。
  - `rm -rf build` 後建兩次，`shasum -a 256` 都是上面的值。
  - `cargo xtask check`：各步驟通過，e2e `6 passed`，最後一行 `check 通過`。
  - 截圖：`.scratch/m0/evidence/08b-type-a.png`、`08b-type-a-space.png`、`08b-ar-found.png`、`08b-ar-committed.png`、`08b-space-after-paging.png`、`08b-space-after-paging-committed.png`。
- 2026-11-04 code-review 已處理：progress 偏差、票 07 Comments；`pipeline.rs` 的 `notice()` 原本寫死兩套平行欄位，改成進資料包的來源清單 `PACKED`，`LICENSE` 與 `NOTICE` 都逐一列；`build` 的四元組改 `Build` 結構；與模組同名的變數改 `dictionary_like`；plan §4 授權檔措辭。
- 2026-11-04 授權改存 repo 後的驗收：刪掉快取裡的 `license.txt`、`rm -rf build` 後跑兩次 `cargo xtask data --verify`，都是八項 `[通過]`、sha256 `31974c80fe810e4d0a24537acb6767346e34f40bac338d8361b44f1963e2125f`（授權檔不在資料包本體裡，所以與改前相同），快取沒有再出現 `license.txt`；`LICENSE` 裡的 Unicode 段與 `data/licenses/Unicode-3.0.txt` 逐字相同。
- 2026-11-04 code-review 沒處理的發現：年級表用 `BTreeMap<char, u8>` 從 `build_pack` 傳到 `code_table`，沒有包成型別；只有一個用途，M1 加字頻時再看排序依據要不要收成一個型別。
