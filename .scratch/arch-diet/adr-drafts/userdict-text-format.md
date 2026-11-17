# 學習資料匯出成帶版本的 TSV，每列有種類欄、未知種類略過

> 草稿（arch-diet 票 02 從 plan〈核心〉〈學習與排位〉〈匯出格式〉搬來），使用者確認後由票 06 編號寫進 `docs/adr/`。欄位與解析規則寫在 `crates/qim-core/src/user/format.rs`，`crates/qim-core/tests/user_dict.rs` 逐條測；版本與相容規則在 `ARCHITECTURE.md`〈學習資料的格式〉。這裡記的是格式為什麼長這樣。

學習資料（`UserModel::export`）是對外格式：換機、備份、同步都靠它（brief〈版本與更新〉），寫成純文字：UTF-8、Tab 分隔、`\n` 換行，第一行 `# qim-userdict v<版本>`，接著檔頭的 metadata 行 `#@/key<Tab>value`，再來每列第一欄是種類（`entry`、`bigram`、`english`、`deleted`）。

參考 librime userdb 文字檔（第一行註解、`#@/key<Tab>value` 的 metadata 含 `/tick` 與裝置的 `/user_id`、每列記衰減值 `d` 與最後 tick `t`）與 libime 使用者詞庫（手動詞與 `[Auto]` 段分開，這裡收成「來源」欄），不同處與理由：

- **第一行帶格式版本**（兩者都沒有）：改格式時讀得出是哪一版，舊版一直收。
- **統計用固定欄**，不用 librime 的 `c= d= t=`：要多記整串上屏的次數與 tick（排位規則 2），升版本時本來就要改 parser。
- **多個方案放同一檔**。
- **每列有明確的種類欄，未知種類略過並警告、已知種類嚴格**：librime 靠 parser 略過壞列與未知的 `key=value` 相容（`docs/research/userdict-text-formats.md` §1.8）；QIM 讓新版本加的種類在舊版讀得進其餘的列，但已知種類壞了一列就整份不收，收一半再合併等於丟資料。
- **metadata 只能寫在檔頭**：裝置表與各列引用的裝置要先讀到，解析才不看行的位置；librime 的文字檔也是 metadata 在前。
- **各台分開的次數與存雜湊的墓碑**取代 librime 負的 `c`（同步時不收斂）與 libime 的 `[Delete]`（ADR 0007）。
- **日期只到日**、不帶時區與時刻：一年時限只要日，也少洩漏打字的時刻。LatinIME 每個條目存最後使用的時間戳（`docs/research/learning-forgetting-sync.md` §2.2–§2.3），librime 沒有日曆時間只有 tick。

查證在 `docs/research/userdict-text-formats.md`。

## Considered Options

- 照抄 librime 的 `c= d= t=`：能直接讀 Rime 的檔，但整串上屏的次數與 tick 放不進去，各台分開的次數也放不進去。

## Consequences

- 加列的種類或改欄位升版本；新版本加的種類舊版略過並警告，所以只加種類時舊版仍讀得進。
- v1、v2 與 M4 第一期的 v3 草稿（依欄數認）都要一直收，parser 帶著這些分支。草稿時代的匯入是取代：某個瀏覽器曾經取代匯入過別台的檔時，兩邊的同一批次數記在各自的裝置，同步後算兩次（只影響草稿時代就有的次數）。
