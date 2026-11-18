# 手動加詞與常用語分開；匯入 Gboard 字典時快捷鍵預設不當觸發詞

行為由 `crates/qim-core/tests/learning.rs`、`phrases.rs`、`word_list.rs` 說明；Gboard 快捷鍵不當觸發詞的理由也寫在 `crates/qim-core/src/word_list.rs` 的 `Shortcuts`。

- **分工照成熟產品**：手動加詞是 Gboard 個人字典的「字詞」：輸入碼由各字的碼推出，進 lattice、連打裡也打得出。常用語是 iOS／macOS 文字替代的「短語 + 可不填的快捷鍵」加上搜狗、微信輸入法的常用語面板：文字可以長、可含換行，碼是空的、不進 lattice，只靠觸發詞或從殼的清單選（`docs/research/facts-products.md`）。
- **詞表匯入是加進去不是取代**：壞列略過、其餘照加，結果列出略過的行號與原因（同 librime、libime 讀使用者詞庫）；整份讀不了才回錯誤。和匯入學習資料（整份不收）不同。
- **macOS／iOS 文字替代的 plist 不收**：它是短語加快捷鍵，對應常用語不是使用者詞；要收時另開票匯入成常用語。

## Consequences

- 常用語刪掉或換觸發詞時，舊觸發詞學到的計數一起刪，匯出檔不留刪掉的字（墓碑只存雜湊，ADR 0007）。
