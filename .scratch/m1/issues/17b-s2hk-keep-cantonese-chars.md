# 17b: 語料前處理保留粵語用字

Status: ready-for-agent
Blocked by: 17
讀: plan §4.3（前處理）、§5.2；brief §5.2、§9；`docs/development.md`〈語料〉；票 17 的 Comments；spec〈共同規則〉

## What to build

語料前處理的 OpenCC `s2hk` 會把粵語常用字換掉：「晒」→「曬」、「搵」→「揾」。所以「晒」「搵」的字頻是 0，基準題目也只有「曬」「揾」，打「唔該晒」「搵工」時排序吃虧。使用者平常打「晒」「搵」（2026-11-04 票 17 審閱時決定）。

- 先列出 `s2hk` 在四份語料裡改掉的粵語用字：拿原文與轉換後逐字比對，統計被換掉的字與次數，挑出粵語用字（至少「晒」「搵」，其他由統計結果判斷）。清單與數字記進票的 Comments。
- 前處理改成保留這些字：做法（轉換前保護、轉換後還原、或改用別的 OpenCC 設定）在本票定，寫進 plan §4.3 與 `sources.toml` 的註解。原本就是繁體的粵語語料（HKCanCor、Common Voice）是否該跳過 `s2hk` 一併評估。
- 重建語料與資料包，基準題目會跟著變：`--update-baseline` 並在 commit body 寫理由；前後對照照 spec〈共同規則〉。
- `--verify` 加一項檢查：保留清單裡的字在 train 的字頻大於 0。

## 驗收

- [ ] Comments 有 `s2hk` 改掉的字的統計，與保留清單。
- [ ] `cargo xtask data --verify` 綠燈，含新檢查（貼輸出）；建兩次 sha256 相同。
- [ ] 核心或 Playwright：打「唔該晒」「搵工」的碼，首選分別是「唔該晒」「搵工」（或說明為何不是，並列出名次）。
- [ ] bench 前後對照貼在 Comments，`baseline.json` 已更新並寫理由。

## Comments
