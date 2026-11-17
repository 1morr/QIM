# 05: plan §5〈測試策略〉、§6〈開發環境與閘門〉歸位

Status: done
Blocked by: 01
讀: `.scratch/arch-diet/spec.md`；plan §5、§6 全文；`docs/development.md`〈全部檢查〉

## What to build

各層測什麼、首選字基準的組成與門檻怎麼定，留在 `ARCHITECTURE.md`〈測試策略〉。指令、環境、閘門怎麼跑搬到 `docs/development.md`（已有的不重複）。

## 驗收

- [x] §5、§6 的內容已歸位；`docs/development.md` 是開發指令的單一來源，沒有和 `ARCHITECTURE.md` 重複的段落。
- [x] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 對照表（plan §5、§6 → 新位置或刪掉的理由）

「dev」指 `docs/development.md`，「ARCH」指 `ARCHITECTURE.md`〈測試策略〉。「已有」是目的地原本就寫著、逐句對過；「補」是這張票加進目的地的句子。

| plan 的段落 | 去處 |
| --- | --- |
| §5.1「單元測試與性質測試（`cargo test`）…」 | ARCH 第一條；「新功能與修 bug 先寫會紅的測試」不搬：`AGENTS.md`〈Skills〉的 tdd 已有（給使用者第 2 條） |
| §5.1「性質測試例：…」 | 刪：`crates/qim-core/tests/properties.rs` 的 `every_candidate_can_be_typed_with_the_beginning_of_the_listed_input`、`incognito_leaves_the_user_model_unchanged`、`exporting_then_importing_gives_an_equal_user_model` 說明 |
| §5.2 表 `bench/fixed.tsv` 列 | 格式與 `'` 鍵：dev〈首選字基準〉檔案表（已有）；組成：ARCH；哪張票加了哪題是歷史，刪（spec 共同規則 3 不留票號）：每題的來源由 `bench/fixed.tsv` 的第三欄說明 |
| §5.2 表 `bench/corpus/` 列 | 格式、轉碼取五代第一碼、由 `cargo xtask data` 產生：dev〈語料〉（已有）；授權刪：`crates/qim-databuild/src/corpora.rs` 寫出 `ATTRIBUTION.md` 的授權（每個語料的 `license_url`）說明 |
| §5.2 表 `bench/colloquial/` 列 | 同上（口語的授權也在 `corpora.rs`）；「開發切分只用來調參」：ARCH「調參只看 dev 切分」、dev〈dev 切分調參〉（已有） |
| §5.2 表「有前文」列 | 取法與格式：dev〈語料〉；兩個集合、都進 baseline：dev 檔案表；每次先 `set_context`：dev〈首選字基準〉第 2 步（都已有） |
| §5.2 表 `typo/`、`touch/` 列 | dev〈語料〉（已有：錯誤模型、Goodman 2002 的雜訊、和容錯模型故意不同、sha256 種子、dev 的同種題） |
| §5.2 表 `rime-userdb.txt` 列 | dev〈個人集〉；`rime_dict_manager -b quick5` 的取得步驟（補） |
| §5.2 表 `rime-userdb.tsv` 列 | dev〈個人集〉與檔案表（已有） |
| §5.2 表 `sentences.txt` 列 | 沒實作，搬到 roadmap〈不排里程碑〉（給使用者第 3 條） |
| §5.2 表兩個 `baseline.json` 列 | dev 檔案表（已有） |
| §5.2 個人集「輸入碼由 `qim-databuild` 轉…」 | dev〈個人集〉（已有「同一個 `Transcoder`」；Rime 的碼為什麼不能用：補） |
| §5.2 個人集「同時報不加權與加權」 | dev 結果表「個人集多一列加權」（已有） |
| §5.2 個人集「偏差：…」 | dev〈首選字基準〉「個人集只印數字」那段（補；給使用者第 1 條） |
| §5.2 個人集「隱私：…」 | ARCH「不進 repo、不送雲端」；dev〈個人集〉（已有不進 repo；雲端那句：補） |
| §5.2「指標：…」 | 各指標：dev 結果表；每鍵延遲（原生與 WebAssembly、Node、第 20 碼起、選開頭一段、空白與學過、萬用鍵、原生點觸控而 WebAssembly 照實體鍵、原生 p95 由 `--check` 擋、WebAssembly 只報）：dev〈每鍵延遲〉（都已有）；ARCH 留「原生每鍵延遲 p95 在上限內」 |
| §5.2「按鍵照殼的做法送…」 | ARCH 一句；細節 dev〈首選字基準〉（已有）；「照實體鍵重打會低估…」與參數由 `xtask/tests/shared_paths.rs` 比對（補） |
| §5.2「打錯碼的集合另報…」 | dev 表下「打錯碼的集合」與檔案表的觸控列（已有）；M4 spec〈容錯採用門檻〉的連結（補）；「原樣讀法關掉鄰鍵讀法、鄰鍵讀法不搶首選」：dev 原本寫錯成「原樣讀法就是預設選項」，照 plan 與程式改回（給使用者第 6 條） |
| §5.2「依長度分組…」 | dev（已有）；理由（補） |
| §5.2「翻盤清單…」 | dev（已有）；理由（補） |
| §5.2「學習模式…」 | dev〈學習模式〉；其他模式用空白的 `UserModel`：dev〈首選字基準〉（都已有） |
| §5.2「修正次數…」 | dev〈首選字基準〉第 2 步與結果表的 KSPC（已有） |
| §5.2「首選跳動…」 | dev 結果表（已有） |
| §5.2「`cargo xtask bench --check`…」 | 門檻：ARCH；完整的違規清單：dev〈學習模式〉後的 `--check` 段（已有） |
| §5.2「更新 baseline 要用 `--update-baseline`…」 | dev 檔案表的 baseline 列（已有） |
| §5.2「dev 切分也有有前文的題…」 | dev〈dev 切分調參〉（已有） |
| §5.2「長前文…」 | 檔案：dev〈資料建置〉的語料條（已有）；同批同序、200 字、不跨換行：刪，`crates/qim-databuild/tests/corpus.rs` 的 `long_context_items_pair_each_context_item_with_up_to_200_chars_before_it` 說明；基準不讀：`crates/qim-bench/src/lib.rs` 的 `context_long_items` 文件註解；截成 0、50、200 字：`scripts/experiments/model-ladder/`（README〈有前文〉與 `src/`） |
| §5.2「口語長前文…」 | 檔案：dev〈資料建置〉（已有）；規則刪：`crates/qim-databuild/tests/corpus.rs` 的 `dialogue_items_take_earlier_turns_of_the_same_conversation_as_context`、`dialogue_context_keeps_the_last_200_chars_and_turns_still_feed_the_other_items`，最多 2,000 句是 `corpus.rs` 的 `BENCH_SENTENCES`；基準不讀：`qim_bench::dialogue_items` 文件註解 |
| §5.2「調參只看 dev 切分…」 | ARCH；dev〈dev 切分調參〉（已有） |
| §5.2「使用者回報的新案例一律加進 `fixed.tsv`…」 | 規則：dev 檔案表 fixed 列（補）；網頁「回報」的行為：刪，`web/e2e/report.spec.ts` 的兩個測試說明（存的格式、首選附在來源欄、重新整理後還在、標點與臨時英文的上屏不能回報） |
| §5.2「防洩漏…」 | ARCH「測試句不得出現在訓練語料」；檢查：dev〈語料〉的 `--verify`（已有），CyIME 出處（補）；個人集不進訓練、不匯入基準的 `UserModel`：dev〈個人集〉（補） |
| §5.3「所有佈局：通過 JSON Schema…雙向」 | ARCH 第一條 |
| §5.3「內建佈局另加嚴格檢查…」 | ARCH 點名 `qim_layout::check_builtin`；檢查項目刪：`crates/qim-layout/src/validate.rs` 的 `check_builtin`、`StrictIssue` 文件註解與 `crates/qim-databuild/tests/gboard.rs` 說明；iOS 地球鍵在 M8 定：roadmap〈平台里程碑（M7–M11）〉的 M8（給使用者第 4 條） |
| §5.3「點擊判定的性質測試…」 | ARCH「性質測試」；項目刪：`crates/qim-layout/tests/hit_properties.rs` 的模組說明；多邊形在 M6：刪，`crates/qim-layout/src/model.rs` 的 `Shape` 文件註解（「繪製與點擊判定在 M6」） |
| §5.4 網頁、原生殼兩條 | ARCH 第二條 |
| §6「M0 票 01–04 交付…」 | 不是一段內容：這句只列交付的票號，每項交付的去處在下面各列 |
| §6「`docs/development.md`：…單一來源」 | dev 開頭第一句（已有） |
| §6「`cargo xtask check`：…」 | dev〈全部檢查〉表（已有；`xtask/src/check.rs` 的測試比對兩邊的步驟） |
| §6「`.githooks/`：…」 | dev〈安裝〉的 `core.hooksPath`、〈全部檢查〉的 `--fast` 段、〈Commit message〉（已有）；雙向測試：刪，`xtask/src/commit_msg.rs` 的測試 |
| §6「文件的閘門…」 | dev〈全部檢查〉的 `docs` 列與段（已有）；守哪些規則、里程碑要有驗收票：刪，`scripts/check_docs.py` 開頭的規則清單 |
| §6「CI…」 | dev〈全部檢查〉的 CI 段（已有跑什麼）；`cargo test` 不讀 `build/packs/`、約 360 MB、`bench --check` 手動跑不進 `check`、推到遠端先問（補）；「約 1 分鐘」沒搬（給使用者第 5 條） |

沒有要寫成 ADR 的段落：§5、§6 是測試的組成與操作，取捨理由（錯誤模型和容錯模型為什麼不同、WebAssembly 為什麼用 Node 量）已在 dev〈語料〉與 `scripts/experiments/` 的 README。

引用跟著改：指向 §5、§6 的 `plan〈首選字基準〉`、`plan〈佈局〉`（§5.3）、`plan〈開發環境與閘門〉`、`plan §5.2`、`plan §6` 改成 dev 的節或 `architecture〈測試策略〉`（程式註解、`bench/fixed.tsv`、`.gitignore`、CI、`AGENTS.md`、brief、`docs/research/model-ladder.md`、兩份實驗 README）；已封存的 `.scratch/m0`–`m4` 不改。

### 給使用者決定（spec 共同規則 4、5；照 spec 先留著，交給票 06 的使用者試一起看）

1. ARCH 這張票只有 450 字的預算（協調者定），〈測試策略〉寫了三條約 400 字。下面這些拿不準是不是「測試策略」，因為預算放進 dev 而不是 ARCH：個人集的偏差（分數可能偏高，要和口語集一起看）、依長度分組與翻盤清單的理由、使用者回報的案例一律加進 `fixed.tsv`、防洩漏的 CyIME 出處、錯誤模型故意和容錯模型不同（原本就在 dev〈語料〉）。建議維持在 dev；要搬進 ARCH 的話約多 300 字。
2. §5.1「新功能與修 bug 先寫會紅的測試」與 `AGENTS.md`〈Skills〉重複，沒搬；同票 01 的第 2 條，AGENTS.md 不是 spec 規定的「測試或程式檔」。
3. `bench/personal/sentences.txt` 從沒做過（M1 票 04 留下、等使用者提供句子），不是現況，搬到 roadmap〈不排里程碑〉；roadmap 不在 spec 列的四個去處裡。
4. iOS 預設佈局要不要放地球鍵在 M8 定，搬到 roadmap M8（同票 01 第 4 條的做法）。
5. 矛盾：plan §6 寫 `bench --check`「約 1 分鐘」，dev〈每鍵延遲〉寫整個 `cargo xtask bench` 約 3 分鐘（加了學過的學習資料之後）。沒搬「約 1 分鐘」，dev 照舊寫 3 分鐘；要不要重量一次再寫，由使用者定。
6. 矛盾：dev〈首選字基準〉原本寫「原樣讀法就是預設選項」，但預設選項開著鄰鍵讀法（`crates/qim-core/src/session.rs` 的 `Options::default`），`score.rs` 的 `literal_options` 是預設再關掉鄰鍵讀法；plan 寫的「原樣讀法關掉鄰鍵讀法、鄰鍵讀法不搶首選」和程式一致。dev 照 plan 與程式改了（`AGENTS.md`〈事實與查證〉：現況以 repo 為準），列給使用者確認。
7. 矛盾：plan §5.3 說內建佈局有 `data/layouts/` 的檔，這個目錄不存在（預設佈局都是 `qim-databuild` 算出來的，同票 01 第 3 條）；〈測試策略〉只寫 `qim_layout::check_builtin`，沒提目錄。

### 給協調者（合併時）

- ARCH 開頭的狀態句：協調者要求先合併 main（票 03）再拿掉「測試策略」，已做。
- `plan〈佈局〉` 在 `crates/qim-layout/src/validate.rs`、`crates/qim-layout/tests/hit_properties.rs`、`crates/qim-databuild/tests/gboard.rs` 指的是 §5.3（嚴格檢查、點擊性質測試），這張票改成 `architecture〈測試策略〉`；`crates/qim-layout/tests/hit.rs`、`spatial.rs`、`validate.rs` 開頭的 `plan〈佈局：畫布模型〉〈佈局〉` 只改了後半（〈佈局：畫布模型〉是票 02 的）。票 02 若把它們當成 §2.6 改，合併時兩邊都留。

### Code review

兩軸都看過。已改：dev〈首選字基準〉開頭指向 ARCH 的「必跑」（ARCH 沒寫這條，拿掉括號）、`AGENTS.md` 的 `bench/` 改指 dev〈首選字基準〉、`web/bench/latency.ts` 改指 dev〈每鍵延遲〉、roadmap M8 那條補出處、對照表的刪除理由改成程式檔（授權在 `corpora.rs`）、dev〈dev 切分調參〉與 `qim-layout` 三個測試檔漏改的 §5 引用、dev「原樣讀法就是預設選項」的錯（給使用者第 6 條）、`data/layouts/` 的矛盾補列（第 7 條）。沒改：ARCH 的狀態句（見「給協調者」）；ARCH 第三條和 dev 有三處同義的短句（門檻、按鍵照殼送、個人集隱私），ARCH 寫規則、dev 寫完整清單與量法，粒度不同，不算重複的段落；dev 的 CI 段「推到遠端仍要先問使用者」照 plan 原句搬，沒刪。

- Session 紀錄：2026-11-18 plan §5、§6 歸位：ARCH〈測試策略〉寫三條（淨增 391 字），其餘補進 `docs/development.md`，指向 §5、§6 的引用跟著改
- 決定：ARCH〈測試策略〉只寫各層測什麼與首選字基準的組成、門檻、紀律；格式、指標、量法與理由放 `docs/development.md`（Agent 定）
- 決定：程式註解指向 §5.2 的改指 dev 的對應節（〈首選字基準〉〈語料〉〈個人集〉〈每鍵延遲〉〈學習模式〉），策略層的改指 `architecture〈測試策略〉`（Agent 定）
- 決定：沒實作的 `sentences.txt` 搬到 roadmap〈不排里程碑〉，iOS 地球鍵搬到 roadmap M8，列給使用者（Agent 定）
- 決定：dev「原樣讀法就是預設選項」照 plan 與程式改成「預設再關掉鄰鍵讀法」，列給使用者確認（Agent 定）
- 變更：無
