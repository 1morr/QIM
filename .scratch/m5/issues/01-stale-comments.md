# 01: 修復過時的程式註解，補資料包種類的測試

Status: done
Blocked by: None
讀: `ARCHITECTURE.md`〈資料包〉〈學習資料的格式〉；`docs/research/ios-feasibility.md`〈詞級 trigram 的記憶體〉；`.scratch/arch-diet/issues/06-wrap-up.md` 第八題

## What to build

arch-diet 只准改指向文件的註解，下面幾處內容過時沒改（arch-diet 票 06，使用者 2026-11-18 同意開票；開工對程式碼後補正，使用者 2026-11-18 同意）：

- `Session::forget` 的註解讀起來像清掉所有前文後面的個人 bigram；實際只清「前一個詞 → 這個候選」那一組、沒有前一個詞時不清。照 `user.rs` 的措辭改寫。
- `user/format.rs` 的 `HEADER` 註解還寫「M4 收尾前還沒定稿」，`tests/user_dict.rs` 開頭有同一句；兩處一起改（v3 已在 M4 驗收時定稿），指向 M4 spec 的也改指封存或 `ARCHITECTURE.md`〈學習資料的格式〉。
- `word_trigram.rs` 的 `LARGE` 寫「常駐估計 35 MB」：改引 `docs/research/ios-feasibility.md`〈詞級 trigram 的記憶體〉，寫明 iPhone 實測留下 +23.4 MB、峰值 +71.3 MB，超過 60 MB 所以不採用，預設用預算版。
- `verify.rs` 的 `CheckId::Decodes`：`decode()` 已解開詞級 trigram，缺的是註解沒列詞級 trigram 與 emoji；另外依魔數只解第一份，放寬版 `zh-hant-word-trigram-large` 從沒被解開。補註解，再定要不要加解開放寬版的檢查（定了記 `決定：`；加的話會紅過的測試放 `crates/qim-databuild/tests/verify.rs`）。
- 不認得的資料包種類：單一 manifest 的 `kind` 不認得時拒絕載入（現在靠 serde），補測試鎖住。資料包索引 `PackIndex::from_json` 現在對整份陣列反序列化，只要有一項新種類，舊核心整份索引都讀不了、線上更新整個停掉；改成**略過不認得種類的那一項並警告，其他照讀**（使用者 2026-11-18 同意），測試鎖住。這是對外格式的行為，`ARCHITECTURE.md`〈資料包〉同一個 commit 補一句。

## 驗收

- [x] 上面四處註解都改了；`CheckId::Decodes` 要不要解開放寬版有一行 `決定：`，加了的話有會紅過的測試。
- [x] 單一 manifest 的不認得種類被拒絕，錯誤讀得出是哪一種（測試）。
- [x] 索引裡多一項不認得的種類時，其他資料包照讀、有警告（測試先紅後綠）；`ARCHITECTURE.md`〈資料包〉寫了這條。
- [x] 全部檢查綠燈（`docs/development.md`〈全部檢查〉）。

## Comments

略過的項：`PackIndex::from_json` 回傳 `ParsedIndex { index, warnings }`，`IndexWarning` 是第幾項、id、種類，同 `ImportWarning` 的形狀；wasm `packUpdates` 改回傳 `{ updates, warnings }`。

code-review 未處理：
- `word_trigram.rs` 的 `BUDGET`、`LARGE` 仍寫「票 08」（M3 的票，不是 ADR、事實編號或節標題）；舊寫法，這張票只改 `LARGE` 的記憶體數字。
- `qim-databuild` 的 `pack-release --previous` 把略過的項印到 stderr，沒有測試；行為由 `PackIndex::from_json` 的測試鎖住。
- `crates/qim-core/src/date.rs` 開頭還指向「M4 spec〈細節〉」，不在這張票的清單裡。
- `Session::forget` 的註解沒寫自動造詞裡前後相接的組也一起清，連到 `UserModel::forget` 看。

Session 紀錄：2026-11-20 改四處過時註解；`Decodes` 解開每一份詞級 trigram；更新索引略過不認得種類的項並警告（核心、wasm、網頁 console、pack-release），單一 manifest 的不認得種類補測試
決定：`CheckId::Decodes` 也解開放寬版 `zh-hant-word-trigram-large`（同魔數的每一份），會紅過的測試是 `undecodable_word_trigram_other_than_the_named_one_fails_decoding`（Agent 定）
決定：更新索引的警告由核心回傳（`ParsedIndex` 的 `IndexWarning`），網頁印在 console、`pack-release` 印到 stderr（Agent 定）
決定：只有 `kind` 是字串而核心不認得的項才略過；其他不合格式（少欄位、sha256、id 重複）照舊整份不收（Agent 定）
變更：Changed｜線上更新｜更新來源多了這個版本還不認得的資料包種類時，其他資料包照樣更新（以前整次檢查失敗）
Heads up（不修）：Claude Code 提醒「索引略過不認得的種類只保護這版之後的核心，已發出去的舊版遇到新種類照樣整份讀不了」。查證：repo 沒有任何 release、網頁沒有 service worker（每次載入都拿最新的 wasm），目前沒有在使用者手上的舊版核心，不修；平台殼發版後往索引加新種類前要先確認舊版已換掉。
