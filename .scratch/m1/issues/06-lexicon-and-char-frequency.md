# 06: 詞庫與字頻

Status: ready-for-agent
Blocked by: 03
讀: plan §2.5（`[packs]`）、§4 開頭、§4.1 步驟 4、§4.2、§4.5；brief §5.1、§5.2；ADR 0003；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 1」那條）

## What to build

速成資料包多了詞庫與字頻：網頁打 `anr` 整串查得到「門口」，單字照字頻排。連打（票 07）之前，詞庫先用整串輸入碼查，讓詞從資料一路通到網頁。

- rime-essay `054920d` 釘進 `sources.toml`；最低詞頻門檻初值 100；丟掉含連打字集以外字的詞；每個字的速成碼展開成詞的碼，多碼字的組合數設上限（上限在本票定）。
- 詞庫是獨立資料包（plan §2.5 的 `lexicon = "quick-lexicon"`），`LICENSE` 寫 LGPL-3.0 全文、`NOTICE` 列來源；字頻與詞頻放同一包（plan §4.2）。
- 字序改成字頻 → `kGradeLevel` → Big5（plan §4.1）；`a` 仍是「日」。
- `Session` 查候選時，整串輸入碼同時查碼表與詞庫；詞與字怎麼合排在本票定（暫時規則，票 07 換成 §2.3 打分），寫進 `## Comments`。
- `--verify` 加「詞的碼都由字的速成碼組成」，並報詞數、被門檻與字集濾掉的數量。
- `anr` 首選若已是「門口」，就加進 `fixed.tsv`（spec〈共同規則〉：通過的票加）。
- bench 前後對照。

## M0 留下的

票 09 Comments「M1 票 1」：`PACKED` 四個 `&str` 的元組、`license_file` / `license_statement` / `license_text` 三個 `Option` 靠建置時擋組合、`verify.rs` 解碼失敗另組 `Report`、年級表 `BTreeMap<char, u8>` 與字頻收成一個排序依據。第二個進資料包的來源就是本票，一起收；做了哪些、沒做的理由寫進 `## Comments`。

## 驗收

- [ ] 詞庫資料包有 `LICENSE`、`NOTICE`；`cargo xtask data --verify` 綠燈，含新檢查與詞數報告（貼輸出）。
- [ ] Playwright：打 `anr` 讀回候選含「門口」；打 `a` 首選仍是「日」。
- [ ] 字序單元測試：字頻高的排前；沒字頻時退回年級、再退回 Big5。
- [ ] 同樣的來源建兩次 sha256 相同。
- [ ] bench 前後對照貼在 `## Comments`。
- [ ] 〈M0 留下的〉每條有結果。

## Comments
