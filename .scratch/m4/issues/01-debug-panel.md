# 01: debug 面板

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §2.1（`Session`、`Snapshot`、`rerank_request`）、§2.3、§2.7、§3.1；spec〈範圍〉的第一期前置、〈共同規則〉

## What to build

網頁加一個可開關、預設關的 debug 面板，打字時看得到核心為什麼這樣排（使用者 2026-11-12 要求；之後各票的測試都用它讀回原因）：

- 殼給核心的前文，與核心是否「不知道前文」；無痕時註明不讀。
- 輸入碼的切段；首選的路徑分數與學習加分。
- 每個候選的來源：詞庫、使用者詞、個人 bigram、英文、記住的字串（票 14 換成英文使用者詞時面板跟著改）、emoji、重排插入。
- 停鍵重排的要求（前文、各路徑）與結果（各路徑分數、選了哪條、毫秒、套用或作廢）。

資料由只給網頁用的查詢給（例如 `Session::inspect()`，名字由票定），不進 `Snapshot`；`qim-ffi` 不用加。重排的模型分數與毫秒在網頁那邊記。面板關著時不呼叫這個查詢，每鍵不多花時間。

## 驗收

- [ ] Playwright：預設關；打開後讀得到前文的三種狀態（有前文、不知道前文、無痕不讀）、切段、首選的路徑分數與學習加分。
- [ ] Playwright：候選來源每一種都至少讀回一例。
- [ ] Playwright：停鍵後讀得到重排的要求與結果；用 `?rerank-delay` 讀到作廢。
- [ ] `Snapshot` 沒加欄位；面板關著時不呼叫查詢（測試守）。
- [ ] plan §3.1 加一段說明面板；實際操作亂按一輪、截圖存 `.scratch/m4/evidence/`；`cargo xtask check` 綠燈。
