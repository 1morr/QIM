# 13: 學習：計數與排位

Status: done
Blocked by: 04, 07
讀: plan §1.3（`SystemTime` 禁用）、§2.1（`UserModel`、`Options`、`set_options`）、§2.3（`personal`）、§2.4、§5.1、§5.2（學習模式）；brief §4.5、§12.2（衰減）；`docs/research/open-source-imes.md` §5；spec〈共同規則〉；`.scratch/m0/issues/09-m0-close.md` 的 Comments（「M1 票 5」那條）

## What to build

候選依使用次數前移：同一個碼選同一個非首選兩次，它就成首選；學過的詞在更長的連打裡也加分。無痕模式下什麼都不記。`bench --learning` 量得出學習讓修正次數下降。

- 開工先讀 libime 怎麼把使用者資料併入 lattice（`UserLanguageModel`、`HistoryBigram`，GPL 只取做法），摘要記到 `docs/research/`。
- 學習計數以（方案, 輸入碼, 候選文字）為鍵，記上屏次數、衰減值、最後 tick；tick 由 `UserModel` 自己累加（plan §2.4），衰減參考 librime `formula_d`（時間常數 200）。上屏時整串與路徑每一段各記一次。
- 排位規則 2：上屏次數 ≥ 2 且距最後上屏 ≤ 10,000 次 → 首選，多個取最近；規則 1、3 在票 14，規則 4 在票 15，本票把「依序套用規則」的骨架做好。
- `personal`：個人機率由衰減值換算（公式本票定，寫回 plan §2.3），`w` 初值 0.2，只加不減。
- `Options { incognito }` 與 `Session::set_options`（〈M0 留下的〉）；無痕時不寫任何學習資料。網頁加無痕開關。
- 分方案：用第二份測試方案（同碼不同字）驗證計數互不影響（plan §2.4）。
- `cargo xtask bench --learning`：照固定順序打一個集合，模擬選字並讓 `UserModel` 學習，和同順序不學習時比修正次數（本來列在首選字基準，移到本票，見 progress 偏差）。其他模式仍用空白的 `UserModel`。
- 本票的學習只在記憶體；存檔與匯出匯入在票 14。

## M0 留下的

票 09 Comments「M1 票 5」：`Options` 沒有簽章用到它，plan §2.1 的 `set_options` 跟著第一個真的選項一起加；本票的 `incognito` 就是第一個。

## 驗收

- [x] `docs/research/` 有 libime 使用者模型的摘要；`personal` 的換算公式寫回 plan §2.3。
- [x] 核心測試：同碼選同一個非首選兩次後成首選，兩次之間隔 500 次上屏也成立。
- [x] 核心測試：學過「門口」後，一串含 `anr` 且原本首選不含「門口」的連打，首選含「門口」。
- [x] 性質測試：無痕模式下 `UserModel` 不變；兩個方案的計數互不影響。
- [x] `bench --learning` 在口語集與個人集上，學習開的修正次數低於學習關（貼輸出）。
- [x] Playwright：同一個碼選同一個非首選兩次，讀回它成首選；開無痕後重做，讀回不變。
- [x] 〈M0 留下的〉已處理。

## Comments

- 先紅：`crates/qim-core/tests/learning.rs` 的「選兩次成首選」紅在 `left: "甲"`、`right: "丙"`；「學過的詞在連打裡加分」紅在「首選仍是 明叩有人」；`bench` 的兩個學習模式測試先編不過；`web/e2e/learning.spec.ts` 在舊綁定上紅。都在改完後轉綠。
- 變異驗證：拿掉無痕檢查，`incognito_leaves_the_user_model_unchanged` 紅。讓兩個方案共用同一組計數，`two_schemes_sharing_a_user_model_do_not_affect_each_other` 也紅。這條測試第一版的兩個方案字完全不同，（碼, 字）不會撞，變異沒抓到；改成同碼、字序相反再多一個字，才抓得到。網頁的無痕開關改成不作用時，無痕那條 e2e 紅。
- libime 摘要在 `docs/research/libime-user-model.md`，事實摘進 brief §12.4。libime 實際是 LGPL-2.1-or-later，不是票上寫的 GPL；照 ADR 0006 只取做法。
- `personal` 的公式（plan §2.3）：`P_personal = d_now ÷ D_max`，`D_max = 1/(1−e^(−1/200)) ≈ 200.5`。另試了「實際上屏段數的衰減總和」當分母，`bench --learning` 的修正次數（學習開）如下，四個公開集合都是常數分母較少：

  | 集合 | 學習關 | 常數分母 | 段數總和分母 |
  | --- | ---: | ---: | ---: |
  | yue-wiki | 7471 | 6574 | 6983 |
  | zh-wiki | 5605 | 5529 | 5552 |
  | common-voice-yue | 8980 | 7443 | 7998 |
  | hkcancor | 6200 | 4128 | 4783 |
  | personal | 1596 | 1561 | 1559 |

- 實作時定下的細節，都寫進 plan §2.4：
  - tick 每個方案各自累加，只算學到東西的上屏，無痕與 Enter 原樣上屏不算。
  - 標點、大寫先上屏組字時照樣學；有沒有字的碼原樣上屏時整次不學。
  - 排位規則只移動候選列裡已有的候選。
- `bench --learning` 最後的輸出（code-review 修掉 beam 的問題之後）：

  | 集合 | 題數 | 修正（學習關） | 修正（學習開） | 修正/句（關） | 修正/句（開） |
  | --- | ---: | ---: | ---: | ---: | ---: |
  | yue-wiki | 2000 | 7471 | 6577 | 3.74 | 3.29 |
  | zh-wiki | 2000 | 5605 | 5529 | 2.80 | 2.76 |
  | common-voice-yue | 1638 | 8980 | 7450 | 5.48 | 4.55 |
  | hkcancor | 992 | 6200 | 4128 | 6.25 | 4.16 |
  | personal | 4517 | 1596 | 1561 | 0.35 | 0.35 |

- `cargo xtask bench` 前後：各集合（含個人集）的答錯、修正次數、KSPC、跳動、依長度分組全部相同，沒有翻盤，固定題 11/11。一般模式用空白的 `UserModel` 並開無痕，`personal` 都是 0。每鍵 p95 的差距是雜訊：

  | 環境 | 改前 p95 | 改後 p95 |
  | --- | ---: | ---: |
  | 原生 | 40.62 µs | 21.96 µs |
  | WebAssembly | 24.25 µs | 26.00 µs |

- 亂按一輪用 Playwright MCP 手動操作，沒有找到問題，內容如下：
  - 連打中夾數字選字、全形標點、大寫、`z`、翻頁 `-` `=`。
  - 組字中途切換無痕，切換後焦點回到輸入框，組字還在。
  - 21 碼以上的長輸入。
  - 選三次「曰」之後，`a` 的首選變成「曰」，再按 `a2` 選到「日」。
- 〈M0 留下的〉：`Options` 隨 `incognito` 有了作用，`Session::set_options` 照 plan §2.1 加上。網頁綁定是 `setIncognito`。`simplified`、`english_candidates` 還沒有作用，在票 16、15。
- code-review 已處理：
  - 字頻排在 beam 之外的字，學過也拿不到 `personal`。先寫成會紅的測試 `a_learned_character_beyond_the_beam_also_scores_in_longer_input`，再修：`exact()` 另外收學過的字。
  - 一般模式的基準在同一題內會學習。改成模擬時開無痕。
  - brief 補上 libime 的事實；plan §2.4 重複的句子合併。
  - 幾個改名：`last_tick_if_chosen_twice`、`Count`；`segment_score` 改收 `bonus`；`Personal` 改成泛型，拿掉重複的 `PersonalItems`。
- code-review 未處理的發現：
  - 排位規則的骨架只有「放到還沒排定的最前面」一種放法。規則 3「至少第 2 位」、規則 4「第 2 位」要在票 14、15 擴充。
  - 規則 2 只移動候選列已有的候選。分段選出、詞庫沒有的整串（如 `abcd → XY`）要排進前 `ranking.sentences` 條路徑才會被移成首選，沒有測試。`personal` 通常會把它推進去。
  - （碼, 字詞）在 `Session` 與 `UserModel` 之間用 tuple 傳，沒有另立型別。`select_at` 與 `commit_all_then` 各自組學習用的分段（票 10 Comments 提過兩處要一起看），`select_at` 會改狀態，暫不合併。
  - 程式碼註解寫票號（「規則 1、3 在 M1 票 14」）。repo 原本就這樣寫，沒改。
  - `WebSession` 自己存一份 `Options` 再整份交給 `set_options`，因為 `Session` 沒有 getter。
- 收尾後使用者決定（協調者轉達）：規則 2 只算整串上屏。原本句中各段也計次，連打句中用過兩次的字，單打時就搶到首選。
  - 先紅：`a_character_used_in_sentences_does_not_count_towards_the_twice_chosen_rule` 紅在 `left: "曰"`、`right: "日"`。內容是連打 `ab` 選「曰月」三次後，單打 `a` 的首選應仍是「日」；接著單打 `a` 選「曰」兩次，首選才變「曰」。
  - 存法：`Count` 分兩組欄位。`commits`、`last_commit_tick` 只在整串上屏時更新，給規則 2 的門檻與「最近」。`decayed`、`last_tick` 由整串與句中各段一起更新，給 `personal`。
  - 「最近」也改看整串上屏：`using_a_twice_chosen_candidate_in_a_sentence_does_not_make_it_more_recent` 驗證句中用過不會讓它變近。把它改回看 `last_tick` 時這條會紅。
- merge main（票 11 的字級 bigram、票 18 的佈局）：
  - 靜態機率改成看前一字後，`personal` 跟著搬進 `Engine::edge_score`。邊先存好個人機率與詞內機率，接上前一字時才算加分。
  - 一般模式的分數照 main 的加法順序算（`first + inner`），空白 `UserModel` 時和 main 逐位元相同。第一版改了結合順序，hkcancor 的修正次數因同分換序少了 3，所以改回。
  - `bench/baseline.json` 和 main 相同，沒改；本機的個人集 baseline 因 bigram 變好而 `--update-baseline`（答錯 1146 → 1106、修正 1596 → 1500）。
  - 合併前後的 `bench --learning`（學習關 → 學習開）：

    | 集合 | 合併前（unigram，規則 2 只算整串） | 合併後（bigram） |
    | --- | ---: | ---: |
    | yue-wiki | 7471 → 6590 | 4607 → 4559 |
    | zh-wiki | 5605 → 5531 | 4495 → 4755 |
    | common-voice-yue | 8980 → 7476 | 6236 → 5746 |
    | hkcancor | 6200 → 4121 | 4518 → 3067 |
    | personal | 1596 → 1583 | 1500 → 1497 |

  - 驗收「口語集與個人集學習開較少」仍成立，但 zh-wiki 學習開變差。試了較小的 w，學習開的修正次數如下。w 越小維基越好、hkcancor 越差；口語是主要依據（brief §9），先留 0.2，由票 12 和 β、γ 一起調（plan §2.3）：

    | w | yue-wiki | zh-wiki | common-voice-yue | hkcancor | personal |
    | ---: | ---: | ---: | ---: | ---: | ---: |
    | 0.2 | 4559 | 4755 | 5746 | 3067 | 1497 |
    | 0.05 | 4447 | 4570 | 5757 | 3300 | 1484 |
    | 0.01 | 4519 | 4482 | 6027 | 3796 | 1483 |

