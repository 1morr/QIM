# 詞庫裡和常見寫法不同的字

回答的問題（M1 票 17c）：rime-essay 用 OpenCC 標準寫法（「羣組」「觀衆」「喫飯」），哪些字和常見寫法不同、各影響幾個詞、兩種寫法同不同碼。結論寫在票 17c 的 Comments 與 plan §4.2、§4.3。

## 跑法

先跑過一次 `cargo xtask data`（只讀快取，不下載）。

```sh
cargo run --release -q -p qim-databuild --example lexicon-forms        # t2tw 換掉的字與 t2hk 反查
HK=1 cargo run --release -q -p qim-databuild --example lexicon-forms   # 另列 t2hk 換成不同碼香港寫法的字
GAPS=1 cargo run --release -q -p qim-databuild --example lexicon-forms # 建詞庫實際換掉、兩字不同碼的字（票 17d）
```

每列：來源、字、常見寫法、是否已在字形對照表、兩字的速成碼、是否同碼、詞庫會收的詞裡含它的詞數與詞頻和、例。「t2hk 反查」列的「常見」欄是 OpenCC 標準寫法（香港寫法反過來的那個字），不是建議換成的字。換 rime-essay 或 OpenCC 版本時要重跑。

`GAPS=1` 回答的是票 17d 的問題：essay 與口語詞表的詞（詞頻達門檻）經建置同一套轉換（`t2hk`、對照表、並存寫法、反向錯字改回）之後，哪些字被換成不同碼的字、打原字的碼就查不到那些詞。每列：原字、換成、兩字的碼、處理（雙向、反向錯字改回、`—` 沒處理）、詞數、詞頻和、例。`—` 的字要嘛照 `t2hk` 換是使用者決定的（票 17c 的表），要嘛該列成並存寫法；換 rime-essay 或 OpenCC 版本時重跑，看有沒有新的。
