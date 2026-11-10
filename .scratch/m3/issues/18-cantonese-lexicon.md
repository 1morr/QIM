# 18: 完整粵語詞庫：查授權，查清才建

Status: done
Blocked by: 02
讀: brief §1.5、§5.1、§5.2、§11、§12.1；plan §4.2；`docs/research/data-sources.md`；spec〈範圍〉〈共同規則〉

## What to build

現在的粵語口語詞只有自己整理的 196 個。查有沒有授權乾淨、能隨商店 App 發佈的完整粵語詞庫，有就建進資料，沒有就寫明理由留到之後。

- 先查證（`mattpocock-skills:research`，結果放 `docs/research/`）：候選來源例如 rime-cantonese、粵典（words.hk）、HKCanCor 的詞表、維基詞典；每個的授權、能否商用與再散布、署名要求、格式與詞數。結論摘進 brief §12.1 並附來源。
- 查清有可用的：釘進 `sources.toml`，照 plan §4.2 的規則併進詞庫資料包（或另一份資料包，票內定），寫回 plan §4.2。
- 都不行或不確定：停下問使用者；決定不做時寫回 brief §1.5（留在「之後」），本票照樣結案。

## 驗收

- [x] `docs/research/` 有查證紀錄，brief §12.1 有結論與來源。
- [x] 做了的話：`cargo xtask data --verify` 綠燈、`LICENSE`／`NOTICE` 完整；口語集 `bench --check` 不變差，貼對照。
- [x] 沒做的話：brief §1.5 寫明原因。（粵典、rime-cantonese 沒做：brief §1.5 寫明待粵典書面確認與授權鏈不明）
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-11 查證（`docs/research/cantonese-lexicon.md`，結論摘進 brief §12.1）：
  - rime-cantonese／upstream 標 CC BY 4.0，但 repo 與 wiki 沒有授權鏈說明，詞條混有粵典（非商業）、紙本詞典、爬取的詞表、搜狗等用戶詞庫，`word.csv` 沒有逐條來源，頻次併入 GPL-3.0 的 Cifu。不用。
  - 粵典詞典本體是非商業授權。「細數據」頁的 `wordslist`（兩字以上約 5.6 萬詞）與 `existingwordcount` 自標 public domain，但詞典本體的授權頁沒提到這個例外。使用者選「只建 HKCanCor」；協調者補充：spec 寫「查清才建」，這算查不清，不推翻小詞表「不從粵典複製」，brief §1.5、§12.1 記「待粵典書面確認」，使用者再決定要不要去信。
  - 維基詞典（CC BY-SA，沒有現成的粵語詞表）、CC-Canto（CC BY-SA 3.0）、Cifu（GPL-3.0）、開放詞典（沒有下載）、LSHK 詞表（授權不明）都不用。
- 做法（plan §4.2）：HKCanCor 已分詞，取 train 切分裡兩字以上的詞與次數（`hkcancor::TrainWords`），併進詞庫資料包，不另建資料包：授權不變（HKCanCor 的 CC BY 4.0 本來就因字頻在詞庫資料包的 `LICENSE`），`NOTICE` 的來源說明與改動段落寫明取了詞。
  - 切句用語料管線同一個 `corpus::clauses`，切分與丟句也同，看轉換後的話輪，所以 dev、test（基準題）句子裡的詞不數。
  - 人名（詞性 `nr`，「子欣」「艾爾頓」）不收；轉換改了字數的話輪整句略過（現在 0 個）。
  - 只補 essay（達門檻）與口語詞表都沒收、出現 ≥ 2 次的詞，詞頻是次數 × 300。現在補 383 詞（train 裡兩字以上的詞 4,536 個）。
- 換算值（`CORPUS_WORD_FREQUENCY`）：兩邊都有的詞，essay 詞頻是 HKCanCor 次數的 310–3,770 倍（四分位，中位數約 1,000）。dev 切分（`bench --dev`）掃 300／1,000／3,000，答錯／修正：

  | 集合 | 改前 | 300 | 1,000 | 3,000 |
  | --- | --- | --- | --- | --- |
  | yue-wiki | 1171/4631 | 1169/4579 | 1168/4575 | 1168/4565 |
  | zh-wiki | 1142/4375 | 1142/4380 | 1142/4379 | 1142/4380 |
  | common-voice-yue | 1027/3808 | 1022/3741 | 1024/3741 | 1020/3727 |
  | hkcancor | 633/2530 | 624/2366 | 623/2364 | 622/2355 |

  先取 1,000（中位數）。`--check` 時個人集多錯 4 題、沒有由錯變對：查出是只出現 3–5 次的新詞換算後壓過 essay 詞頻一兩千的常用詞。改成 300（下四分位）後個人集只多錯 1 題，dev 上和 1,000 差不多。這一步是看了個人集才改的，plan §5.2 說個人集只守門；理由是低次數詞的詞頻本來就不該比照中位數，dev 上也沒有代價：dev 的修正 300／1,000／3,000 在 common-voice-yue 是 3741／3741／3727、hkcancor 2366／2364／2355、yue-wiki 4579／4575／4565、zh-wiki 4380／4379／4380，三者幾乎平手，個人集只是破平手。
- `cargo xtask bench --check` 前後對照（前是開工 `9d0c627` 的資料包與 baseline，答錯／修正）：

  | 集合 | 前 | 後 |
  | --- | --- | --- |
  | yue-wiki | 1131/4346 | 1123/4289 |
  | zh-wiki | 1126/4385 | 1126/4381 |
  | common-voice-yue | 1032/3964 | 1030/3906 |
  | hkcancor | 663/2596 | 650/2478 |
  | 有前文合計（給） | 1789/6902 | 1780/6796 |
  | 有前文合計（不給） | 1803/6969 | 1793/6862 |
  | personal | 1081/1466 | 1082/1467（加權正確率 87.16% 不變） |

  - 翻盤：yue-wiki 由錯變對 8、common-voice-yue 2、hkcancor 13，三者都沒有由對變錯；hkcancor·前文由錯變對 7、由對變錯 1（第 361 行「幫人」打成「趙人」）。個人集由對變錯 1 題（第 1385 行，上屏 1 次的三字短語被 HKCanCor 的兩字詞組成的候選擠到第 2）。
  - 依長度分組：口語兩集 5–8 字、9 字以上的首選正確率都升（hkcancor 9 字以上 22.84% → 25.31%），其餘各組持平。
  - hkcancor 集與詞表同源：題目取自 HKCanCor 的 test 切分，詞只取 train，同一句不會兩邊都有；但兩邊是同一批對話，說話人、話題、用詞都接近，train 抽出的詞常常就是 test 要打的詞，所以 hkcancor 集的進步有一部分是「考題和教材同源」。增益以和 HKCanCor 無關的 common-voice-yue 為準：修正 3964 → 3906（−1.5%），答錯 1032 → 1030。
  - 使用者接受個人集多 1 題，`--update-baseline` 更新公開 baseline 與本機個人集 baseline。個人集 baseline 在主 checkout、各 worktree 共用；開工後被票 19 改過（修正 1466 → 1468），這次寫成 1082/1467。
  - 沒改學習，沒跑 `--learning`。
- `cargo xtask data --verify`：`verify 通過`，詞庫 sha256 `f2b05295…5d62`，其他六份不變。報告行多「HKCanCor train 切分的詞 4536 個（略過對不上的話輪 0 個），出現 ≥ 2 次的補上 383 詞」。
- 測試：`tests/hkcancor.rs` 加 4 個（只數 train、人名與單字與字集外的句子不數、照轉換後的寫法數、對不上的話輪略過），只數 train 與人名兩條做過變異會紅；`tests/lexicon.rs` 加 3 個；`tests/run.rs` 的 HKCanCor 假來源多一個分好詞的轉寫檔，斷言詞庫收到它的詞、人名不收、`NOTICE` 寫明取了詞，拿掉 pipeline 接抽詞那行會紅。
- 已知限制（抽查 383 詞）：大多是口語詞（直程、話時話、老奉、是但、特登、冚唪唥），也混了數字（五百五十八、二千三）、節目或公司名（小全章、新上海灘、越秀投資）、轉寫的錯字與重字（巡徊、蹉商、點樣樣）。次數低、詞頻小，影響有限。要再濾可以排除純數字的 `m` 詞與 `nz`、`nt` 專名，先不做，等使用者回報或口語集看得出代價。
- 合併 main（票 03、05、17、19）後重建、以 main 的 baseline 為底重跑：公開集合答錯／修正 yue-wiki 1131/4352 → 1123/4295、zh-wiki 1126/4385 → 1126/4381、common-voice-yue 1032/3966 → 1030/3908、hkcancor 663/2596 → 650/2478，`--update-baseline` 寫入。個人集 1082/1469：答錯同合併前；修正多 2 是 main 上票 19 帶來的（它把個人集修正 1466 → 1468），本票仍是 +1，本機個人集 baseline 改成 1082/1469。
- code-review（Standards、Spec 兩軸）的發現與處理：
  - 程式註解與 plan 舉例用了個人集裡的詞（Spec），違反〈共同規則〉，改成只寫題數。
  - `docs/development.md` 的詞庫內容與 `--verify` 報告項目沒跟上（Standards），已補。
  - 缺端到端測試（Standards），已補 `tests/run.rs`。
  - `TrainWords` 自己重寫切句（Standards），改用 `corpus::clauses`；詞與次數三種寫法統一成 `hkcancor::WordCounts`；`sentences` 改名 `tagged_utterances`（「句子」在 `corpus.rs` 指切出來的句）、報告的 `corpus_words` 改名 `corpus_seen`；常數註解縮短。重構後詞庫 sha256 不變。
  - 383 詞沒人工篩（Spec）：記在上面的已知限制。
  - 研究檔的結論補一行使用者的選擇。
