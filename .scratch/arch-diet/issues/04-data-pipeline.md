# 04: plan §4〈碼表與資料管線〉歸位

Status: done
Blocked by: 01
讀: `.scratch/arch-diet/spec.md`；plan §4 全文（約 21,000 字）；`docs/development.md`〈資料建置〉；`data/sources.toml`

## What to build

資料包的格式、manifest、來源與授權怎麼對應到資料包，是跨模組的契約，留在 `ARCHITECTURE.md`。怎麼建、怎麼驗、怎麼發佈資料包是操作步驟，搬到 `docs/development.md`（那裡已有的不重複）。各來源的處理細節由 `crates/qim-databuild/` 的程式與測試說明的，刪掉並指出檔案。

## 驗收

- [x] 資料包格式與 manifest 的契約在 `ARCHITECTURE.md`；建置與發佈步驟在 `docs/development.md`，搬移逐行比對沒丟。
- [x] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 對照表（plan §4 → 新位置或刪掉的理由）

簡稱：DB = `crates/qim-databuild/src/`，DT = `crates/qim-databuild/tests/`，CORE = `crates/qim-core/src/`，ARCH = `ARCHITECTURE.md`〈資料包〉，DEV = `docs/development.md`，草稿 = `.scratch/arch-diet/adr-drafts/<名>.md`。「DEV（已有）」是那句早就在 development.md；「DEV（新）」是這張票搬過去的句子，見下一節的逐句比對。

| plan 的段落 | 去處 |
| --- | --- |
| §4 開頭「`data/sources.toml` 列出每個來源…」 | 來源欄位與 `license_text` 三種：DEV（已有）〈資料建置〉第一、三條；「每份一個目錄、授權檔不共用、各寫用到的來源」：ARCH 第一段 |
| §4 開頭「資料包是 postcard 整份解開…」 | postcard、整份解開：ARCH 第一段；記憶體數字、不做 mmap 與低階機的備案：草稿 pack-format-and-manifest |
| §4.1 步驟 1「取 libcangjie…`346026b`」 | 刪：`data/sources.toml` 的 `[source.libcangjie]` 釘 commit 與 sha256，DB `libcangjie.rs` 讀它 |
| §4.1 步驟 2「只取 Big5 或 HKSCS…」 | 刪：DB `quick.rs`（`continuous`、`extension`），DT `quick.rs` 的 `big5_order_first_then_other_hkscs_then_extension_by_code_point` |
| §4.1 步驟 3「三代與五代碼推導…聯集」 | 刪：DB `quick.rs` 的 `quick_code`，DT `quick.rs` 的推導測試 |
| §4.1 步驟 4「字的排序先用字頻…」 | 排序：刪，DB `quick.rs` 的排序與 doc、CORE `engine.rs` 的 `by_frequency`；「速成資料包不含語料衍生的資料、字頻在詞庫資料包」：ARCH；授權：DEV（已有）〈資料建置〉產物那條 |
| §4.1 步驟 4 子項「簡體字…」 | 判定規則：刪，DB `simplified.rs` 的 doc 與篩選、DT `simplified.rs`、`verify.rs` 的 `NOT_SIMPLIFIED`；「現在 278 字」：刪，`verify.rs` 的報告印「簡體字 N 個」；「Big5 有 59 字…」：草稿 lexicon-and-model-tuning |
| §4.1 步驟 5「另產一份字根提示資料包…」 | 刪：DB `radical_hint.rs` 與 DT `radical_hint.rs`；不在 `Engine`、殼另外載入：DEV（已有）產物那條與 ADR 0003 |
| §4.2「rime-essay…」門檻、字集、取最大詞頻 | 刪：DB `lexicon.rs` 的 `MIN_FREQUENCY`（doc 有理由）與 DT `lexicon.rs` |
| §4.2「rime-essay…」寫法轉換與撞詞合併 | 規則：刪，DB `lexicon.rs` 的 `Forms`、`combined`；14 組、261 組：草稿 lexicon-and-model-tuning |
| §4.2「rime-essay…」並存寫法的收法、`ONE_WAY_DISTINCT` | 刪：DB `common_forms.rs` 的 `spellings`、`ONE_WAY_DISTINCT`、`REVERSED_ONE_WAY`（doc 有例子），DT `common_forms.rs`；「查 essay 的『X着』7 個」：草稿 lexicon-and-model-tuning |
| §4.2「rime-essay…」已知限制「到底綫時」、照字頻打折沒採用 | 草稿 lexicon-and-model-tuning |
| §4.2「rime-essay…」碼的展開最多 8 個 | 刪：DB `lexicon.rs` 的 `MAX_COMBINATIONS`（doc 有 99.4%） |
| §4.2「rime-essay…」獨立資料包、`LICENSE` | 刪：DB `pipeline.rs` 的 `lexicon_about`；DEV（已有）產物那條 |
| §4.2「粵語口語詞表…」 | 自己整理、不複製、三級詞頻：草稿 data-sources-and-licenses；格式與讓建置失敗的條件：刪，DB `lexicon.rs`（`COLLOQUIAL_PATH`、`word_and_frequency`）與 DT `lexicon.rs`；197／196 詞：刪，詞表本身 `data/lexicon/yue-colloquial.tsv`；授權：DB `packing.rs`、DEV（已有） |
| §4.2「HKCanCor 的詞…」 | 取詞規則：刪，DB `hkcancor.rs` 的 `TrainWords`、`lexicon.rs` 的 `CORPUS_MIN_COUNT`、`CORPUS_WORD_FREQUENCY`，DT `hkcancor.rs`、`lexicon.rs`；300 的由來：草稿 lexicon-and-model-tuning；「現在補 383 詞」：刪，`verify.rs` 的詞庫報告；與 hkcancor 集同源：DEV（新）〈首選字基準〉與草稿；rime-cantonese 不用、授權不另加：草稿 data-sources-and-licenses |
| §4.2「粵典的詞頭…」 | 取詞與詞頻：刪，DB `words_hk.rs`、`lexicon.rs` 的 `HEADWORD_FREQUENCY` 與 `by_spelling` 前的註解，DT `words_hk.rs`、`lexicon.rs`；可以缺的內建：`web/src/packs.ts` 的 `OPTIONAL_IDS`；sha256 對不上時略過：DEV（已有）〈資料建置〉第一條、DB `pipeline.rs`；非商業條款、不用 Wayback：草稿 data-sources-and-licenses；掃參數、`existingwordcount` 偏高、HKCanCor 不比並存寫法：草稿 lexicon-and-model-tuning；20,556 詞：刪，`verify.rs` 的粵典報告；`NOTICE` 內容：DB `pipeline.rs` 的 `words_hk_about`；`--lexicon`：DEV（已有）〈首選字基準〉 |
| §4.2「字頻…」 | 刪：DB `lexicon.rs` 的 `char_frequency`、`common_forms.rs` 的共用；DEV（已有）〈語料〉的 `chars.tsv` |
| §4.3「語料：中文維基與粵語維基…」 | 刪：DB `wikitext.rs`（丟掉的標記、`VARIANTS`）；CirrusSearch 不用的理由：`data/sources.toml` 的註解 |
| §4.3「中文維基只取…`zhwiki-20220501`…」 | 刪：`data/sources.toml` 的註解（7 個月、分片、id 範圍、大小） |
| §4.3「口語語料…」 | 刪：DB `corpora.rs` 的 `CORPORA`、`weight`；Tatoeba：`data/sources.toml`；LIHKG、OpenSubtitles：草稿 data-sources-and-licenses |
| §4.3「前處理…」 | 刪：DB `corpus.rs` 的 doc 與 `clauses`、`dropped`，DT `corpus.rs` |
| §4.3「轉香港繁體…」 | `s2hk`／`t2hk`、對照表規則：刪，DB `corpora.rs` 的 `Conversion`、`common_forms.rs` 的 doc 與 `from_opencc`；6.8 萬字：`data/sources.toml`；0.005%：草稿 lexicon-and-model-tuning；常見寫法：brief §4.1；不用釘住的 OpenCC 字典：草稿 data-sources-and-licenses；34 組：刪，`verify.rs` 印「對照表 N 組」；重看差異的實驗：DEV（已有）〈語料〉 |
| §4.3「並存寫法…」與雙向、單向、罕用三個子項 | 規則與清單：刪，DB `common_forms.rs` 的 `TWO_WAY_FORMS`、`ONE_WAY_FORMS`、`BothForms::as_written`、`bigram.rs` 的照抄，DT `common_forms.rs`、`bigram.rs`；「不改核心」、「囪」、10,121 對、103 詞、11 組查過、罕用不動：草稿 lexicon-and-model-tuning |
| §4.3 子項「已知限制…随著」 | 草稿 lexicon-and-model-tuning |
| §4.3「切分…」「每份語料都切…」 | 刪：DB `corpus.rs` 的 `split_of`、`BENCH_SENTENCES`，DT `corpus.rs` 的 `split_is_about_90_5_5…` |
| §4.3「訓練：字級 interpolated Kneser-Ney bigram」與句內、句首子項 | 刪：DB `bigram.rs` 的 doc（公式、句首不計次數），CORE `language_model.rs` 的 `CLAUSE_START`，DT `bigram.rs` |
| §4.3 子項「單一折扣…」 | 公式：刪，DB `bigram.rs`；口語幾乎不平滑、標準 KN 重訓的數字：草稿 lexicon-and-model-tuning；詞級用標準 KN：DB `word_trigram.rs` 的 doc |
| §4.3 子項「剪枝…」 | 門檻與 8 MB：刪，DB `bigram.rs` 的 `MIN_COUNT`、`MAX_BYTES`（doc 有 Gboard 預算）；2.9／4.6 MB、不剪少 1%：草稿 lexicon-and-model-tuning |
| §4.3 子項「資料包存 ARPA 式…」 | 刪：CORE `language_model.rs` 的 `LOG_PROB_STEP` 與 doc |
| §4.3 子項「各語料的次數乘上權重…」 | 權重：刪，DB `corpora.rs` 的 `weight` doc；掃 1–256：草稿 lexicon-and-model-tuning；不寫在方案檔：ARCH |
| §4.3「詞級 trigram…」與「段」子項 | 切段：刪，DB `word_trigram.rs` 的 `CurrentScoring`、`pipeline.rs`；換了就要重建：ARCH、DEV（新）〈線上更新的 release〉 |
| §4.3 子項「機率…」 | 刪：DB `word_trigram.rs`（`discount_of`、α 夾 1 的 doc）、CORE `word_trigram.rs` 的退回式；詞表編號的 0.35 MB：草稿 lexicon-and-model-tuning |
| §4.3 子項「兩份都建…」「iPhone 18 Pro Max…」 | 門檻與 id：刪，DB `word_trigram.rs` 的 `BUDGET`、`LARGE`、`VARIANTS`，`verify.rs` 的 `WordTrigramSize`；大小、iPhone 數字：草稿 lexicon-and-model-tuning；先就地轉換再量：草稿 pack-format-and-manifest |
| §4.3 子項「和預設詞庫綁在一起…」 | 契約：ARCH；同批發：DEV（新）與草稿 pack-online-update；對不上照退回式：CORE `word_trigram.rs` 的 doc；37.8%→45.4%：草稿 lexicon-and-model-tuning；manifest 記詞庫 id：草稿 pack-online-update |
| §4.3 子項「建置：…約 30 秒」 | DEV（已有）〈資料建置〉語料那條 |
| §4.3「授權：語料是 CC BY-SA 4.0…」 | 待決項：brief §11 與草稿 data-sources-and-licenses；`LICENSE` 列哪些：DB `pipeline.rs` 的 `word_trigram_about` |
| §4.4「英文…」 | 刪：DB `english.rs`、CORE `english.rs`、`engine.rs`、DT `english.rs`；不用 experimental：`data/sources.toml`；base64：DEV（已有）、DB `sources.rs` |
| §4.4「emoji…」 | 規則：刪，DB `emoji.rs`（`MAX_KEYWORD_EMOJI`、`fully_qualified`、`is_squared_ideograph`），DT `emoji.rs`；不用 `annotationsDerived/`：`data/sources.toml`；門檻的抽查例子：草稿 lexicon-and-model-tuning；覆蓋不均、補詞表：草稿 data-sources-and-licenses |
| §4.4「簡繁…」 | 刪：DB `opencc.rs` 的 `hk2s`、CORE `simplifier.rs`，DT `opencc.rs`；不用 ferrous-opencc：`data/sources.toml` 與 ARCHITECTURE〈主要依賴〉 |
| §4.5 標題「M0 起每次改資料管線或來源都跑」 | DEV（新）〈資料建置〉 |
| §4.5 每一條（不掉字、鍵長、抽查、詞庫、Unihan、8 MB、粵語用字與並存寫法、簡體字、字根提示、英文、emoji、決定性、粵典、方案指名、manifest） | 刪：`verify.rs` 的 `CheckId` 與各檢查函式（`NoMissingCharacters`、`KeyLength`、`MenKouForAnr`、`simplified_men_kou`、`english_the`、`emoji_after_laugh`、`word_codes`、`LanguageModelSize`、`WordTrigramSize`、`kept_characters`、`common_forms`、`lexicon_forms`、`not_simplified`、`men_after_traditional`、`hint_coverage`、`hint_by_version`、`english_z`、`Deterministic`、`words_hk_lexicon`、`decode`、`manifests`），DT `verify.rs` 的 `report_lists_every_check_of_plan_4_5_except_unihan`；報告內容：DEV（已有）〈資料建置〉 |
| §4.6「每份資料包是一個目錄…」 | ARCH 第一段；研究出處：草稿 pack-format-and-manifest |
| §4.6 JSON 範例 | 刪：CORE `manifest.rs` 的 `Manifest`，`crates/qim-core/tests/manifest.rs` |
| §4.6 欄位表「內容」欄 | 摘要在 ARCH；細節刪：CORE `manifest.rs` 的 `MANIFEST_VERSION`、`PackKind`、`check_format`，`crates/qim-core/tests/manifest.rs` 的 `kind_names_match_the_manifest_json` |
| §4.6 欄位表「採用的慣例」欄 | 草稿 pack-format-and-manifest |
| §4.6「不認得的欄位略過…」 | ARCH；`crates/qim-core/tests/manifest.rs` 的 `unknown_fields_are_ignored_and_missing_ones_rejected`、`malformed_fields_are_rejected` |
| §4.6「載入：殼交一組 manifest…」 | 核對順序、殼的分工、`Engine::open` 的拒絕條件：ARCH，CORE `engine.rs`、`crates/qim-core/tests/engine_open.rs`；字根提示經 `RadicalHintPack::open`、模型經 `ModelPack::open`、不知道種類時用 `Manifest::verify`、`Pack::from_bytes` 不核對：刪，CORE `pack.rs`、`manifest.rs`，`crates/qim-core/tests/manifest.rs` 的 `model_packs_round_trip_and_open_by_manifest`；網頁的路：`crates/qim-wasm/src/lib.rs`、`web/src/packs.ts`；`qim-ffi` 之後改同一條路：草稿 pack-format-and-manifest |
| §4.6「不照研究建議把 manifest 塞進 `.pack` 檔頭…」 | 草稿 pack-format-and-manifest |
| §4.6「使用中的資料包…」 | ARCH；`Engine::new`、`RadicalHints::for_scheme` 的 doc 與 `crates/qim-core/tests/` 的 `lexicon.rs`、`word_trigram.rs` 等；「推翻 M1」是歷史，刪 |
| §4.6「網頁（plan §3.1）：內建的從 `/packs/<id>/` fetch…」 | 行為：刪，`web/src/packs.ts`（`DB_NAME`、`ACTIVE_KEY`、`change`）、`pack-choice.ts`、`main.ts`，`web/e2e/packs.spec.ts`、`storage.spec.ts`；用 IndexedDB 的理由、Firefox：草稿 pack-format-and-manifest |
| §4.6「e2e 匯入的小詞庫…」 | 刪：`crates/qim-core/tests/web_fixture.rs`；重產指令：DEV（已有）〈網頁〉 |
| §4.7 開頭「資料包不等 App 發版…」 | 契約：ARCH；來源的決定、外部索引的慣例：草稿 pack-online-update |
| §4.7 asset 表 | 刪：DB `release.rs` 的 `collect`，DT `release.rs` 的 `release_has_renamed_assets_and_an_index`；DEV（已有）〈線上更新的 release〉；加前綴、不打包 zip 的理由：草稿 pack-online-update |
| §4.7 `index.json` 範例 | 刪：CORE `pack_index.rs` 的 `PackIndex`、`IndexEntry`，`crates/qim-core/tests/pack_index.rs` 的 `index_entries_are_manifests_with_a_url` |
| §4.7「`index_version`…`url` 是相對參照」 | ARCH；RFC 3986 與本機也能用：草稿 pack-online-update |
| §4.7「讀索引…`InvalidIndex`」 | 刪：CORE `pack_index.rs` 的 `from_json`，`crates/qim-core/tests/pack_index.rs` 的 `malformed_index_is_rejected` |
| §4.7「要下載的（`PackIndex::updates`）…」 | ARCH；同 Firefox、不用 `digest`：草稿 pack-online-update |
| §4.7「發新版要加 `version`…」 | 刪：CORE `pack_index.rs` 的 `check_successor`，DT `release.rs`；DEV（已有）〈線上更新的 release〉 |
| §4.7「固定用 tag `packs` 的 release…」 | 步驟與上傳順序：DEV（已有）〈線上更新的 release〉；理由（Neovim、`latest`、先傳資料包）：草稿 pack-online-update，`web/vite.config.ts` 的註解 |
| §4.7「網頁（§3.1）：設定區…線上更新」開關 | 刪：`web/src/pack-update.ts`（`SWITCH_KEY`、`INDEX_TIMEOUT_MS`、`PACK_TIMEOUT_MS`）、`pack-settings.ts`、`packs.ts` 的 `update`，`web/e2e/pack-update.spec.ts` |
| §4.7「網頁讀不了 GitHub 的 release…」 | 轉發與環境變數：刪，`web/vite.config.ts`；DEV（已有）〈網頁〉；curl 實測、平台殼直接讀：草稿 pack-online-update |
| §4.7「e2e 用 Playwright 起的本機靜態伺服器…」 | 刪：`web/e2e/static-server.ts`、DT `release.rs` 的 `web_release_fixture_is_current` |
| §4.7「`pack-release` 收 `build/packs/` 的每一份…」 | 刪：DB `release.rs` 的 `write`；DEV（已有）〈停鍵層的模型〉最後一條 |
| §4.7「還沒在 GitHub 建 release…」 | 現況：DEV（已有）〈線上更新的 release〉；票 21 的來由是歷史，刪 |
| §4.7「同一個 id 的內建版與下載版…」 | 規則：ARCH，`web/src/pack-choice.ts` 的 `listPacks`、CORE `manifest.rs` 的 `is_newer_than`；理由：草稿 pack-online-update |

### 搬到 development.md 的句子（逐句比對）

plan §4 的段落都是一整行、拆給好幾處，`scripts/docs_moved_lines.py` 比不了，改逐句列。上表標「DEV（已有）」的句子搬之前就在 development.md，這張票沒有再抄一份；新加的三句：

| plan 的句子 | development.md 的位置 |
| --- | --- |
| §4.5 標題「M0 起每次改資料管線或來源都跑」 | 〈資料建置〉指令下面第一句「改了資料管線或來源都要跑 `--verify`」 |
| §4.3「換了詞庫、bigram 或 β、γ 要重建」、「更新詞庫或語言模型時詞級 trigram 也要同一批重建、一起發」 | 〈線上更新的 release〉第二段最後一句 |
| §4.2「首選字基準的 hkcancor 集與這份詞表同源…增益以無關的 common-voice-yue 為準」 | 〈首選字基準〉開頭清單最後一條 |

### 給使用者決定（spec 共同規則 4、5；照 spec 先留著，交給票 06 的使用者試一起看）

1. **`ARCHITECTURE.md` 的預算**：協調者給這張票淨增 1,100 字。契約與實作細節分不清的，照規則 4 先放進去的是 `Engine::open` 拒絕的條件；剩下的字放不下另外兩條，沒放：manifest 各欄位的格式（id 的字元、sha256 的寫法、`kind` 的九種，在 `manifest.rs`）、網頁存資料包的規則（先在記憶體換、核心收下才寫進 IndexedDB；核心不收時只用內建的，在 `web/src/packs.ts`）。要不要進 `ARCHITECTURE.md`、換掉哪一條？
2. **plan 與程式碼對不上，照程式碼寫**：(a) plan §4.5 與 development.md 說「`anr` 的『門口』經 `Engine` 查」，`verify.rs` 的 `MenKouForAnr` 其實直接查詞庫（首選由 `bench/fixed.tsv` 擋）；development.md 已照程式碼改：這是文件寫錯程式的行為，不是設計上的兩種選擇，照 `AGENTS.md`〈事實與查證〉「現況以指令為準」與票 01 的先例；要照 spec 規則 5 退回原句也可以。(b) plan §4.5「決定性」列十份資料包，`build_packs` 實際還有停鍵層模型（有粵典時共 11 份）；那段刪了，以 `build_packs` 為準。
3. **程式註解和量到的數字對不上，這張票沒改（不改程式碼行為以外的內容）**：`crates/qim-databuild/src/word_trigram.rs` 的 `LARGE` 註解寫「常駐估計 35 MB」，iPhone 實測是留下 +23.4 MB、峰值 +71.3 MB；`verify.rs` 的 `CheckId::Decodes` 註解漏了詞級 trigram 的方案指名檢查。另外 plan 說「舊核心讀到不認得的種類拒絕載入」，行為來自 serde，沒有專門的測試。要不要開修復票？
4. **刪掉的理由不是程式檔的幾段**：常見寫法引 brief §4.1、語料授權引 brief §11、口語詞表的詞數引詞表本身、幾條來源的理由引 `data/sources.toml` 的註解（程式讀的資料檔，不是程式碼）；「推翻 M1」「票 21 時只授權推 `main`」是歷史，直接刪。spec 只認測試或程式檔，要使用者確認可以。
5. **四份 ADR 草稿**：pack-format-and-manifest、pack-online-update、data-sources-and-licenses、lexicon-and-model-tuning。最後一份是調參的數字與已知限制，不像一個決定；也可以改放 `docs/research/` 的實驗紀錄（spec 沒列這個去處）。
6. **引用留給票 06**：程式註解、`data/sources.toml`、`GLOSSARY.md`、brief、`docs/research/` 裡指向 plan §4 各節的引用沒改（點過去是一行「已搬走」，還連得到）；只改了 development.md 的引用，和 `data/sources.toml` 裡叫人「更新 plan〈詞庫〉的詞數」的那句（那個數字已經不在文件裡）。`crates/qim-databuild/src/lexicon.rs` 的 `CORPUS_WORD_FREQUENCY`、`HEADWORD_FREQUENCY` 註解寫「見 plan〈詞庫〉」，票 06 編號後要改指 ADR。

### Code review

兩軸都看過，已改：`ARCHITECTURE.md` 補上解開後核對 id、照規則 4 放進 `Engine::open` 的拒絕條件（淨增 1,009 字）；對照表 §4.6「載入」補 `RadicalHintPack::open`、`ModelPack::open`、`Manifest::verify`；草稿 pack-format-and-manifest 改指 `ARCHITECTURE.md`〈風險與回寫點〉；草稿 pack-online-update 不再重述詞級 trigram 同批發的規則，改指 `ARCHITECTURE.md`。沒改、留給使用者的：「門口」照程式碼改（上面第 2 題）、刪掉理由不是程式檔的幾段（第 4 題）、tuning 草稿不像一個決定（第 5 題）、`data/sources.toml` 改了一句註解的內容（第 6 題）、草稿裡的 repo 名 `1morr/QIM`（從 plan 原樣搬來，plan 與 development.md 本來就有）。

- Session 紀錄：2026-11-18 plan §4 歸位：資料包契約進 `ARCHITECTURE.md`〈資料包〉、三句操作步驟進 development.md、理由寫成四份 ADR 草稿，其餘指出程式檔後刪掉
- 決定：選型理由與調參數字分成四份 ADR 草稿（資料包格式、線上更新、取材與授權、調參），沒合成一份（Agent 定）
- 決定：只改 development.md 與 `data/sources.toml` 一句的 plan §4 引用，程式註解、GLOSSARY、brief、research 的留給票 06（Agent 定）
- 決定：development.md 說「門口」經 `Engine` 查，照 `verify.rs` 改成直接查詞庫，列給使用者（Agent 定）
- 決定：`ARCHITECTURE.md` 預算內放 `Engine::open` 的拒絕條件，manifest 欄位格式與網頁儲存規則留在程式碼、列給使用者（Agent 定）
- 決定：plan 的「現在 N 字／詞／組」這類建置輸出不搬，以 `verify.rs` 的報告為準（Agent 定）
- 變更：無
