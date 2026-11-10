# 16: emoji 候選

Status: ready-for-agent
Blocked by: 02
讀: plan §2.2、§2.4、§4、§4.5；brief §1.5、§5.1、§7；spec〈範圍〉〈共同規則〉

## What to build

打出「笑」「貓」「多謝」這類詞時，候選裡出現對應的 emoji。

- 資料：CLDR 的繁中（`zh_Hant`）與粵語（`yue`）標註，釘進 `data/sources.toml`（版本、sha256、Unicode License），建成一份 emoji 資料包，加進 `--verify` 的決定性檢查；授權查證補進 brief §12.1。
- 排法先看成熟做法（Rime 的 emoji 濾鏡把 emoji 接在對應候選後面、iOS 速成）再定，寫回 plan §2.4；會改候選位置，所以 `Options` 加開關（預設值在票內定、寫回 brief §7），網頁設定區加開關。
- 簡體模式下仍照繁體原文對標註。

## 驗收

- [ ] 單元測試：打「笑」的碼時候選裡有 😄 這類 emoji、位置照票內的規則；開關關掉時沒有。
- [ ] `cargo xtask data --verify` 綠燈，emoji 資料包有 `LICENSE`、`NOTICE`。
- [ ] Playwright：打一個詞、選 emoji 上屏讀回；截圖。
- [ ] `bench --check` 不變差（emoji 不是正解時不該影響首選）；`cargo xtask check` 綠燈。

## Comments
