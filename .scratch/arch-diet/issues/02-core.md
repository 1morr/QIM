# 02: plan §2〈核心〉歸位

Status: done
Blocked by: 01
讀: `.scratch/arch-diet/spec.md`；plan §2 全文（約 52,000 字，分小節讀）；`crates/qim-core/` 的公開介面

## What to build

§2 是最大的一節，大部分在逐條描述核心的行為（帶函式名）。留下：對殼的介面契約（`Session`、`Snapshot`、`Key`、`Options` 這一層殼要守什麼）、學習資料與匯出格式的版本與相容規則、排位規則的順序。其餘的行為描述，找到說明它的測試或程式檔後刪掉。這張票預期刪掉最多，對照表要逐小節列。

## 驗收

- [x] `ARCHITECTURE.md`〈跨模組的資料與契約〉有核心對殼的契約與匯出格式的版本規則。
- [x] §2 每個小節在對照表裡都有去處；刪掉的每一段都指得到一個測試或程式檔。
- [x] Comments 有對照表：這張票負責的 plan 每一段 → 新位置或刪掉的理由。
- [x] `cargo xtask check --fast` 綠燈。

## Comments

### 對照表（plan §2 → 新位置或刪掉的理由）

縮寫：`T/` 是 `crates/qim-core/tests/`，`S/` 是 `crates/qim-core/src/`，`L/` 是 `crates/qim-layout/`，「ADR NNNN」是這張票寫的草稿，票 06 經使用者同意編成 `docs/adr/` 的 0009–0017（調參的那份改放 `docs/research/`），〈契約〉是 `ARCHITECTURE.md`〈跨模組的資料與契約〉。每列的檔由子代理逐段找、我抽查過主要的幾個；完整的證據表（含測試函式名、缺口）在 `.scratch/arch-diet/02-evidence/`，行號是 `4832a21:docs/plan.md` 的。

| plan 的段落 | 去處 |
| --- | --- |
| §2.1「參考 librime 的 `RimeApi`…殼只負責畫」 | 〈核心對殼〉第一句 |
| §2.1 rust 區塊：`Engine`、`UserModel`、`Session` 三個物件的角色 | 〈核心對殼〉第一條 |
| §2.1 rust 區塊：`Key`、`ShiftKey`、`Key::parse`、`ShiftSwitch`、`FieldKind`、`Options`、`Snapshot` 各欄、`Session` 各方法的簽章 | 刪：`S/session.rs` 的型別與文件註解（子代理逐個比對，簽章與 16 個 `Options` 欄位都相符），`T/session.rs`、`T/key.rs`、`T/inspect.rs`；殼要守的（`handled`、`delete_before`、`set_field`、`set_options` 與 `refresh`、重排）在〈核心對殼〉 |
| §2.1「分隔記號（`qim_core::SEPARATOR`…」 | 刪：`T/separator.rs`、`S/scheme.rs` 的 `SEPARATOR`；觸控「'詞」送同一鍵由 `crates/qim-databuild/tests/gboard.rs` 說明 |
| §2.1「萬用鍵（M4 票 11…」 | 刪：`T/wildcard.rs`、`T/english.rs`、`T/english_user_words.rs`，`S/engine.rs` 的 `wildcard_edges`、`MAX_WILDCARDS` 註解 |
| §2.1「`Letter` 含大寫…」 | 殼攔下只改畫面的鍵、`handled` 為假時先做 `commit` 進〈核心對殼〉；其餘刪：`S/session.rs` 的 `Key` 文件註解、`T/session.rs`、`T/digit_separators.rs`、`T/properties.rs`、`web/test/host.test.ts`；各平台怎麼先上屏再交還 Enter 見「給使用者決定」4 |
| §2.1「`text`：直接輸入一段文字…」 | 刪：`T/continuous.rs`、`T/simplified.rs`、`T/digit_separators.rs`、`T/learning.rs` |
| §2.1「`ToggleAscii`（單按 Shift…」 | 刪：`T/session.rs`、`T/continuous.rs`、`web/test/keys.test.ts`（500 ms）、`L/tests/shift.rs` |
| §2.1「英文模式（ADR 0008…」 | 刪：`T/english_mode.rs`、ADR 0008 |
| §2.1「建議列（`english_mode::suggestions`）…」 | 刪：`T/english_mode.rs`，`S/english_mode.rs` 的常數與 `learned_bonus` 註解（含 helmet、tumors 的數字） |
| §2.1「點建議（`select`）…」 | 刪：`T/english_mode.rs`、`S/english_mode.rs` 的 `takes_picked_space` |
| §2.1「無痕與學習關掉時照樣在記憶體追蹤…」 | 刪：`T/english_mode.rs`、`T/english_user_words.rs`、`S/english_user.rs` 的 `TypedRun` 註解 |
| §2.1「句首自動大寫…」 | 刪：`T/english_mode.rs`、`T/field_kind.rs`、`L/tests/keyboard.rs` |
| §2.1「英文容錯…」 | 刪：`T/english_mode.rs`、`S/english_mode.rs` 的 `CORRECTION_PENALTY`、`AUTOCORRECT_MARGIN` 註解 |
| §2.1「自動修正…」 | 刪：`T/english_mode.rs`、`web/e2e/autocorrect.spec.ts`、`S/english_mode.rs` 的 `autocorrection` 註解 |
| §2.1「不修：去掉詞尾…」 | 刪：`T/english_mode.rs`、`S/english_mode.rs` 的 `AUTOCORRECT_MIN_CHARS` |
| §2.1「修正的對象是分數最高的一處拼錯…」 | 刪：`T/english_mode.rs`、`S/english_mode.rs` 的 `AUTOCORRECT_MARGIN`；Gboard 22 個、每鍵 p95 的數字在 `scripts/experiments/autocorrect-threshold/README.md`，whitelist 在事實 M51 |
| §2.1「還原：剛修正完緊接著退格…」 | 刪：`T/english_mode.rs`、`T/backspace_undo.rs` |
| §2.1「和 Gboard 的差異…」 | 退格就還原：`T/english_mode.rs`；不收 whitelist：`scripts/experiments/autocorrect-threshold/README.md`；沒有復原鈕與下一個詞：roadmap〈M5 觸控鍵盤與設定〉已列（候選列重做、英文下一個詞） |
| §2.1「`delete_before`（ADR 0008…」 | 單位與先刪後上屏進〈核心對殼〉；其餘刪：`web/test/host.test.ts`、`T/english_mode.rs`、`T/smart_punctuation.rs`；各平台的刪法在 `docs/research/facts-platforms.md` P25–P28。「核心一直送 0」已過時，見「矛盾」1 |
| §2.1「英文候選：`Engine::with_english`…」 | 刪：`T/english.rs` |
| §2.1「詞級 trigram（M3 票 11b…」 | 刪：`T/word_trigram.rs`、`S/word_trigram.rs` 的模組註解 |
| §2.1「emoji 候選：`Engine::with_emoji`…」 | 刪：`T/emoji.rs` |
| §2.1「英文使用者詞（M4 票 14…」與「從哪裡學」「怎麼認」「每個詞記打過的次數」「無痕不學」四條 | 刪：`T/english_user_words.rs`、`S/english_user.rs`（`MIN_CHARS`、`MAX_LEN`、`TypedRun`）；成熟做法的查證在 `docs/research/remember-strings.md` |
| §2.1「簡體模式：`Engine::with_simplifier`…」 | 刪：`T/simplified.rs`、`T/english.rs`；`qim-ffi` 的 `set_options` 那句見「矛盾」2 |
| §2.1「原文：候選、分段與上屏都另帶…」 | 刪：`T/simplified.rs`、`T/radical_hint.rs` |
| §2.1「簡體字排後…」 | 刪：`T/simplified_characters.rs` |
| §2.1「聯想（M3 票 13…」與「數字鍵 1–9」「空白不選」「字母收起聯想」「聯想不給重排」「聯想看剛上屏的各段」「聯想的 emoji」六條 | 行為刪：`T/association.rs`、`T/backspace_undo.rs`；照 macOS 速成、不學 Microsoft、退格偏離 librime-predict、emoji 誤觸的代價進ADR 0012 |
| §2.1「不做（M4 票 30 評估）…」 | roadmap〈M5 觸控鍵盤與設定〉原有的「評估空輸入的句首建議…」一條，補上 27–33%、14–26% 與量詞沒有資料、桌面候選窗的理由 |
| §2.1「數字後半形…」 | 刪：`T/digit_separators.rs`、`S/punctuation.rs` 的 `DIGIT_SEPARATORS` |
| §2.1「引號左右輪流…」 | 刪：`T/smart_punctuation.rs`、`S/punctuation.rs` 的 `Quotes` |
| §2.1「連按兩下空白輸出句點…」 | 殼計 1100 ms 呼叫 `expire_double_space` 進〈核心對殼〉；其餘刪：`T/smart_punctuation.rs`、`S/punctuation.rs` 的 `SpaceTail` |
| §2.1「殼負責：鍵碼轉 `Key`…」 | 〈核心對殼〉 |
| §2.1「按鍵的入口（M2 票 03）…」 | `Key::parse` 收「種類 + 值」進〈核心對殼〉；其餘刪：`T/key.rs`、`crates/qim-ffi/src/lib.rs` 開頭註解（uniffi 沒有 `char`）、`web/src/keys.ts` |
| §2.1「`qim-ffi` 的物件（M2 票 08）…」 | `Engine` 一個、各 `Session` 共用進〈核心對殼〉；其餘刪：`crates/qim-ffi/tests/session.rs`、`crates/qim-ffi/src/lib.rs` |
| §2.1「上下文（M3 票 04、05）…」與「`Session` 記一份前文」 | 殼何時給、何時清進〈核心對殼〉；其餘刪：`T/context.rs`、`S/session.rs` 的 `CONTEXT_LIMIT` |
| §2.1「殼沒給前文時…」（含 librime、fcitx5、LatinIME 的來源連結） | ADR 0009；行為由 `T/context.rs` 說明 |
| §2.1「無痕（brief §8…」 | 刪：`T/context.rs`、`T/digit_separators.rs`、`T/inspect.rs`；用滑鼠移游標的代價進ADR 0009 |
| §2.1「組字中修改」與「`Composition.focus`」「改選第 i 段」「移到某一段時空白」三條 | 刪：`T/continuous.rs`、`T/session.rs`、`T/rerank.rs` |
| §2.1「按鍵機率（M4 票 06、07、09c…」 | `qim-layout` 算、殼轉交、核心不知道佈局進〈核心對殼〉；兩種變體由殼選、整個當美式鍵盤的理由進ADR 0011；其餘刪：`T/key.rs`、`T/inspect.rs`、`S/weights.rs`、`L/tests/physical.rs`、`web/test/keys.test.ts` |
| §2.1「觸控鄰鍵讀法（M4 票 09c…」 | 刪：`T/neighbor_readings.rs`、`S/neighbor.rs` 的常數；在候選列的位置在〈候選的排位順序〉第 4 步 |
| §2.1「欄位類型（M4 票 26…」與「`set_field` 先照換欄位」「email、網址、密碼欄切到英文」「使用者在這三種欄位切到英文模式時」「密碼欄」「數字欄」「debug 面板顯示」六條 | `set_field` 進〈核心對殼〉；理由與代價進ADR 0010；行為刪：`T/field_kind.rs`、`L/tests/number_page.rs`、`web/test/field.test.ts`、`S/session.rs` 的 `wants_english` |
| §2.1「重排：`apply_rerank` 只接受目前的 `generation`…」 | 〈核心對殼〉最後一條；ADR 0005 |
| §2.1「以路徑為單位」 | 理由與數字進ADR 0013；行為刪：`T/rerank.rs` |
| §2.1「前文：`Session` 記的前文…」 | 刪：`T/rerank.rs`、`S/session.rs` 的 `RERANK_CONTEXT` 註解（量過 200 字不比 50 字好） |
| §2.1「`order` 是路徑的序號…」 | 順序進〈候選的排位順序〉；其餘刪：`T/rerank.rs` |
| §2.1「不給要求（`None`）…」 | 刪：`T/rerank.rs`、`T/association.rs`、`T/english.rs`；改字時不給的理由進ADR 0013 |
| §2.1「名次真的變了才算候選變動…」 | 刪：`T/rerank.rs`、`T/session.rs` |
| §2.1「停鍵結果快取（§2.7，M4 票 03）…」 | `set_reranker_id` 進〈核心對殼〉；其餘刪：`T/rerank.rs`、`S/session/rerank_cache.rs` |
| §2.1「翻頁不是候選變動…」 | 刪：`T/rerank.rs`、`T/session.rs`；理由進ADR 0013 |
| §2.1「`apply_rerank` 要 `Engine` 才回得出快照…」 | 刪：`S/session.rs` 的 `apply_rerank` 簽章 |
| §2.1「`paths(limit)`（M3 票 07）…」 | 刪：`T/rerank.rs`、`S/session.rs` 的 `paths` |
| §2.2 mermaid 圖 | 刪：`S/lattice.rs` 的模組註解與 `candidates`、`S/engine.rs`；圖與程式碼有出入，見「矛盾」3 |
| §2.2「已知限制：簡繁後處理逐個分段轉換…」 | roadmap〈不排里程碑〉加一條（沒有測試或註解說明，見「給使用者決定」2） |
| §2.3「所有候選…都用同一個函式打分」 | 〈候選的排位順序〉開頭；ADR 0001 |
| §2.3 打分公式的區塊 | 刪：`T/viterbi.rs`（照同一個式子窮舉對照）、`S/engine.rs` 的 `word_edge`、`edge_score_parts`、`S/user.rs` 的 `personal_bonus` |
| §2.3「`P_bigram`：字級 interpolated Kneser-Ney…」 | 刪：`S/language_model.rs` 的模組註解、`T/language_model.rs` |
| §2.3「詞級 trigram（M3 票 11b，§4.3）…」 | 刪：`S/word_trigram.rs`、`T/word_trigram.rs`；雜湊索引的理由在 `S/word_trigram.rs` 的 `WordTrigram` 註解，µs 數字留在 M3 票 11b |
| §2.3「組字開頭的字的『前一字』…」 | 規則刪：`S/engine.rs` 的 `context_char`、`T/context.rs`；略過標點的實驗進ADR 0009 |
| §2.3「詞的段：`P(詞 \| 前一字)`…」 | 刪：`S/engine.rs` 的 `word_edge`；27% 進`docs/research/ranking-parameters.md` |
| §2.3「`β` 鼓勵詞庫裡的詞…」 | 刪：`S/scheme.rs` 的 `Ranking::check`、`T/ranking.rs`、`data/schemes/quick.toml` 的註解；調法進`docs/research/ranking-parameters.md` |
| §2.3「`personal`：學過的段…」與「個人機率」「個人 bigram」「前一個字詞」三條 | 刪：`S/user.rs` 的 `personal_bonus`、`PERSONAL_WEIGHT`、`BIGRAM_WEIGHT`、`max_decayed`，`S/lattice.rs` 的 `last_word`，`T/learning.rs`、`T/personal_bigram.rs`；取捨進`docs/research/ranking-parameters.md` |
| §2.3「分母改用實際上屏的段數…」「詞內在詞頻與字級 bigram 之間內插…」「`personal` 的 w…」 | `docs/research/ranking-parameters.md` |
| §2.3「每個位置保留 beam 寬 20…」 | 刪：`S/lattice.rs` 的 `Lattice`、`S/engine.rs` 的 `exact_code`、`T/viterbi.rs`、`T/learning.rs`；理由進`docs/research/ranking-parameters.md` |
| §2.3「分隔記號（§2.1）是『這裡一定要斷字』…」 | 刪：`T/separator.rs`、`S/engine.rs` 的 `exact`、`splits_at` |
| §2.3「聯想（M3 票 13，§2.1）也用這把尺…」與四種來源、`personal`、依文字另存一份、參考 macOS 五條 | 刪：`S/association.rs`、`S/lexicon.rs` 的 `WordsByText`、`T/association.rs`；不另建資料包與記憶體代價進ADR 0012 |
| §2.3「Unicode 擴展區的字不進 lattice…」 | 刪：`T/continuous.rs`、`S/engine.rs` 的 `edges_ending` 註解 |
| §2.3「候選列：蓋滿整串的字、詞與…」 | 順序進〈候選的排位順序〉第 1 步；其餘刪：`S/lattice.rs` 的 `candidates`、`T/continuous.rs`、`T/simplified_characters.rs`；簡體字不扣分與多段路徑放 3 條的理由進`docs/research/ranking-parameters.md` |
| §2.4「學習計數以『(輸入方案, 輸入碼, 候選文字)』為鍵…」 | 刪：`T/properties.rs`、`T/personal_bigram.rs`、`T/learning.rs`、`S/user.rs` 的 `DECAY_STEP` |
| §2.4「選了觸控鄰鍵讀法時學的是換過的碼」 | 刪：`T/neighbor_readings.rs` |
| §2.4「觸控鄰鍵讀法與停鍵重排都從候選列第 2 格起插…」 | 順序進〈候選的排位順序〉第 4、5 步（程式先插鄰鍵讀法，`apply_rerank` 再把重排的路徑插到它前面）；理由進ADR 0013；`web/e2e/neighbor-readings.spec.ts` |
| §2.4「上屏時記兩種」與「整串上屏」「衰減值」兩條 | 刪：`T/learning.rs`、`S/user.rs` 的 `SchemeModel::learn` 註解 |
| §2.4「排位規則依序套用在 Viterbi 結果上…」與規則 1–5、「排定的候選釘在它的位置」 | 順序進〈候選的排位順序〉第 3 步；細節刪：`S/lattice.rs` 的 `RULES`、`Ranked`，`T/learning.rs`、`T/english.rs`、`T/english_user_words.rs`；規則 5 的誤觸量測在 `S/english_user.rs` 的 `ENGLISH_COMPLETION_POOL` 註解與 `scripts/experiments/remembered-prefix/` |
| §2.4「停鍵重排在排位規則之後套用…」 | 〈候選的排位順序〉第 5 步；使用者定的理由進ADR 0013；`T/rerank.rs` |
| §2.4「emoji 候選（M3 票 16…」 | 〈候選的排位順序〉第 2 步；其餘刪：`T/emoji.rs`、`S/emoji.rs` 的 `EMOJI_LIMIT` 註解 |
| §2.4「使用者詞像詞庫的詞一樣進 lattice…」 | 刪：`S/engine.rs` 的使用者詞邊的註解、`T/learning.rs` |
| §2.4「手動加詞」「觸發詞剛好是整串時」 | 刪：`T/learning.rs`、`S/user.rs` 的 `SchemeUser::triggered` |
| §2.4「常用語」 | 行為刪：`T/phrases.rs`；分工照成熟產品的理由進ADR 0016 |
| §2.4「詞表匯入」與 Rime、Gboard、純文字、壞列略過四條 | 刪：`T/word_list.rs`、`S/word_list.rs`（檔頭註解、`Shortcuts` 的理由）；加進去不取代進ADR 0016 |
| §2.4「macOS／iOS 文字替代的 plist 不收」 | ADR 0016 |
| §2.4「自動造詞」 | 刪：`T/learning.rs` |
| §2.4「分隔記號不進學習資料」 | 刪：`T/separator.rs` |
| §2.4「`Options.incognito` 為真或在密碼欄…」 | 刪：`T/properties.rs`、`T/learning.rs`、`T/field_kind.rs` |
| §2.4「選了聯想只記個人 bigram…」 | 刪：`T/association.rs` |
| §2.4「退格撤銷（M4 票 15…」與「和 librime 不同」 | 刪：`T/backspace_undo.rs`、`S/user/undo.rs` |
| §2.4「時限（協調者 2026-11-12 決定…」 | 殼計 3 秒呼叫 `expire_undo` 進〈核心對殼〉；其餘刪：`web/src/undo-window.ts`、`web/test/undo-window.test.ts` |
| §2.4「撤銷學習（M4 票 16…」與「字詞」「英文候選…」「無痕、學習關掉時也清得掉」「清掉的…各留一個墓碑」四條 | 刪：`T/forget.rs`、`T/english_user_words.rs`、`T/sync_merge.rs`、`S/user.rs` 的 `forget` 註解；不做永久屏蔽在 ADR 0007，CyIME 的評價在 `docs/research/cyime.md` §4 |
| §2.4「桌面殼的按鍵：組字中 Shift+Delete…」 | 刪：`web/e2e/forget.spec.ts`、`web/src/main.ts` 的按鍵註解；清不到第 2 位起的缺口已在 roadmap〈M5 觸控鍵盤與設定〉；fcitx5 的孤例在 `docs/research/facts-products.md` |
| §2.4「今天的日期（M4 票 12…」 | 殼給 `set_today` 進〈核心對殼〉；不寫假日期進ADR 0015；其餘刪：`T/date.rs`、`T/user_dict.rs`、`T/learning.rs` |
| §2.4「學習的時限（M4 票 13…」 | 停用不刪的理由進ADR 0015；行為刪：`S/user.rs` 的 `EXPIRY_DAYS`、`expired`，`T/learning.rs` |
| §2.4「條目上限（M4 票 13…」 | 刪：`S/user.rs` 的 `ENTRY_LIMIT`、`BIGRAM_LIMIT` 註解（含 1.9 萬、2.8 萬、4,600）、`enforce_limits`，`T/learning.rs`；結構照 LatinIME 與英文詞典的已知限制進ADR 0015 |
| §2.4「匯出格式（對外…」 | 版本規則進〈學習資料的格式〉；欄位刪：`S/user/format.rs` |
| §2.4 匯出格式的「第一行 `# qim-userdict v3`…」 | metadata 只在檔頭進〈學習資料的格式〉；其餘刪：`S/user/format.rs` 的 `HEADER`、`DEVICE_KEY` 等常數，`T/user_dict.rs` |
| §2.4 匯出格式的「每列第一欄是種類…」 | 未知種類略過、已知種類嚴格進〈學習資料的格式〉；日期只到日的理由進ADR 0014；`T/user_dict.rs` |
| §2.4 匯出格式的「各台的次數與墓碑」「`entry`」「`bigram`」「`english`」「`deleted`」五條 | 刪：`S/user/format.rs` 的 `COLUMNS`、`BIGRAM_COLUMNS`、`ENGLISH_COLUMNS`、`DELETED_COLUMNS`、`Shape`（子代理逐欄比對，16、10、9、7 欄與欄序都相符），`T/user_dict.rs` |
| §2.4 匯出格式的「M4 票 12–16 的 v3 草稿…」 | 欄位刪：`S/user/format.rs`（依欄數認的分支）；同步後算兩次的代價進ADR 0014 |
| §2.4 匯出格式的「v1、v2（M3 spec…」 | 一直收進〈學習資料的格式〉；欄位刪：`S/user/format.rs`、`T/user_dict.rs` |
| §2.4 匯出格式的「參考：librime userdb 文字檔…」 | ADR 0014 |
| §2.4「匯入：`UserModel::import(text, today)`…」 | `merge`、`replace_learning` 的分工進〈學習資料的格式〉；其餘刪：`S/user/format.rs` 的 `import`、`T/user_dict.rs` |
| §2.4「跨裝置合併（M4 票 17，ADR 0007…」與 G-Counter、墓碑、版本、tick、清空、還原、上限淘汰七條 | 三條律與殼只傳輸進〈學習資料的格式〉；其餘刪：ADR 0007、`S/user/sync.rs`、`T/sync_merge.rs`、`T/properties.rs`；清空與還原的說明另在 `web/index.html` |
| §2.5「方案檔是 TOML，欄位命名參考 Rime…」與 TOML 範例 | 刪：`data/schemes/quick.toml`、`S/scheme.rs`、`crates/qim-databuild/tests/scheme.rs`；範例與現況不同，見「矛盾」4 |
| §2.5「核心只認方案檔與資料包…」與各部分的表 | ADR 0017 |
| §2.5「發佈方式（brief §5.3）…」 | `qim-core` 不讀 TOML 已在 `docs/development.md`〈資料建置〉，要求較新核心時拒絕由 `T/manifest.rs` 說明；App Group、擴充不編譯、手寫方案檔的打算進ADR 0017 |
| §2.5「方案之間怎麼切換由殼負責…」與形碼、音碼兩條 | ADR 0017 |
| §2.6「佈局是一張畫布…」 | 刪：ADR 0004、`crates/qim-databuild/src/gboard.rs` 的模組註解 |
| §2.6「格式 `qim-layout/1` 在 M1 票 19 定稿…」 | 〈佈局格式〉 |
| §2.6「採用的慣例」Delta skin、Trime、FlorisBoard、自己定的四條 | 刪：`L/src/model.rs` 的欄位註解（`extendedEdges` 同名同義等）與 `docs/research/open-source-imes.md` §4；畫布模型的理由在 ADR 0004 |
| §2.6 jsonc 範例 | 刪：`L/schema/qim-layout-1.schema.json`、`L/tests/validate.rs` 的 `every_field_of_the_format_is_read_and_saved_back` |
| §2.6「佈局版本的選法…」 | 刪：`L/src/model.rs` 的 `Layout::select`、`L/tests/validate.rs`、`web/src/display.ts`（600 px） |
| §2.6「動作（`tap`…」 | 刪：`L/src/model.rs` 的 `Action`、`L/tests/format.rs`、`L/tests/punct.rs`、`crates/qim-databuild/tests/gboard.rs`（每個 `text` 鍵的標籤等於它打的字） |
| §2.6「切中英（M4 票 23…」 | 刪：`L/tests/ascii_page.rs`、`L/tests/keyboard.rs` |
| §2.6「數字欄的頁（M4 票 26…」 | 刪：`L/tests/number_page.rs` |
| §2.6「殼上的狀態（M2 票 03）…」 | 刪：`L/src/keyboard.rs`、`L/tests/keyboard.rs`；中英鍵轉成哪個 `Key` 見「矛盾」5 |
| §2.6「`"shift"`（M1 票 27 加…」 | 刪：`L/src/model.rs` 的 `Shift`、`SHIFTED_LETTER_SCALE`，`L/tests/shift.rs`、`crates/qim-databuild/tests/gboard.rs` |
| §2.6「`composing.a11yLabel`…」 | 刪：`L/tests/label.rs` |
| §2.6「殼的觸控行為（網頁在 M1 票 20 先做…」 | 「各平台殼照網頁做」進〈佈局格式〉最後一句；門檻與畫法刪：`web/src/touch.ts`、`web/e2e/touch-features.spec.ts` |
| §2.6「點擊判定在 `qim-layout`…」 | 刪：`L/src/geometry.rs` 的 `Page::hit`、`L/tests/hit.rs`、`L/tests/hit_properties.rs` |
| §2.6「空間模型（M4 票 06…」 | 刪：`L/src/spatial.rs`（`SpatialModel::default` 的初值與來源）、`L/tests/spatial.rs` |
| §2.6「鄰鍵先驗（M4 票 07…」 | 刪：`L/src/physical.rs`、`L/tests/physical.rs`；不拿觸控幾何算的理由進ADR 0011 |
| §2.6「容錯與滑動輸入用按鍵的實際位置…」 | 刪：ADR 0004〈Consequences〉第三條（原文相同）、`L/tests/spatial.rs` |
| §2.6「驗證分兩級…」與錯誤、警告兩條 | 刪：`L/src/validate.rs`、`L/tests/validate.rs` |
| §2.7「候選要跟手…」與三層的表 | 刪：ADR 0005；雲端層不做在 `docs/research/model-ladder.md` |
| §2.7「停鍵層和雲端層都走 `rerank_request`…」 | 〈核心對殼〉最後一條；`T/rerank.rs` |
| §2.7「停鍵結果快取（M4 票 03）放在核心的 `Session`…」與鍵、記的時機、殼的契約、上限四條 | `set_reranker_id` 進〈核心對殼〉；其餘刪：`S/session/rerank_cache.rs`（`RERANK_CACHE_LIMIT` 註解）、`S/session.rs` 的 `set_reranker_id`、`T/rerank.rs`、`T/backspace_undo.rs`；每鍵 µs 數字留在 M4 票 03 |
| §2.7「重排結果只決定名次，不寫回分數…」 | 刪：ADR 0005 第一段（原句） |
| §2.7「`cargo xtask bench` 只跑同步層…」 | 整行搬到 `docs/development.md`〈停鍵層的模型〉（原樣，`grep -n 只跑同步層 docs/development.md` 在第 149 行），原本那條「照 plan §2.7」改成「照下一條」 |
| §2.7「`qim-core` 不能開執行緒…」 | 推論在背景執行緒、`qim-neural` 不碰檔案進〈核心對殼〉與〈依賴規則〉；其餘刪：`crates/qim-neural/src/lib.rs`（`CONTEXT_CHARS`、`Reranker::new`、`rank`）、`crates/qim-neural/examples/latency.rs`；延遲是事實 L7、L8 |
| §2.7「簡體模式下殼給的前文是簡體字…」 | 刪：`S/simplifier.rs` 的 `traditional_chars`、`T/simplified.rs`、`T/context.rs` |
| §2.7「模型的來源：`scripts/neural-rerank/train.py`…」 | 刪：已在 `docs/development.md`〈停鍵層的模型〉 |
| §2.7「各層的模型由 M3 模型階梯定…」 | 刪：`docs/research/model-ladder.md`（採用表與〈停鍵層在新基線重量〉）、`data/neural-rerank/model.toml`（λ） |

### 矛盾（spec 共同規則 5；請使用者確認）

照 `AGENTS.md`〈事實與查證〉「現況以 repo 為準」處理：plan 那段照樣歸位或刪掉，沒改程式。使用者要以 plan 為準的，原文在 `git show 4832a21:docs/plan.md`，改程式另開票。

1. §2.1〈delete_before〉寫「M4 票 21 只加欄位，核心一直送 0」，現在點建議、自動修正、連按兩下空白都送非 0（`T/english_mode.rs`、`T/smart_punctuation.rs`）。這句是過時的歷史，刪了。
2. §2.1〈簡體模式〉寫「`qim-ffi` 加換選項時，綁定的 `set_options` 同時呼叫 `refresh`」，`crates/qim-ffi/src/lib.rs` 還沒有 `set_options`；網頁是每項一個方法（`setSimplified`…）各自回快照。〈核心對殼〉只寫核心的規則（改設定後組字中要 `refresh`），綁定怎麼包留給 `qim-ffi` 加換選項的票。
3. §2.2 的資料流圖缺詞級 trigram 與聯想，圖上「去重」在排位規則之後，程式是在之前（`S/lattice.rs` 的 `candidates`）。圖刪了、沒照改。
4. §2.5 的 TOML 範例 `word_bonus`、`segment_penalty` 是 0.0，`data/schemes/quick.toml` 是 1.0；範例也少了 `[packs] word_trigram`。範例刪了。
5. §2.6〈殼上的狀態〉寫中英鍵轉成 `ToggleAscii(Right)`，`crates/qim-wasm/src/lib.rs` 轉成 `Key::ToggleEnglish`（英文模式，ADR 0008）。
6. 程式註解過時，spec 不准改，列給之後：`S/session.rs` 的 `Session::forget` 註解讀起來像清掉所有前文後面的個人 bigram（實際只清現在前文那一組，`UserModel::forget` 的註解是對的）；`S/user/format.rs` 的 `HEADER` 註解還寫「M4 收尾前還沒定稿」。
7. 小出入，不影響契約：jsonc 範例的 `canvas` 高 216，預設佈局是 224；plan 寫網頁在 `keydown`、`pointerdown` 給今天的日期，`web/src/main.ts` 另在 `blur` 也給；plan 一處寫 librime 的 `installation_id`、一處寫 `#@/user_id`（ADR 0014 用 `/user_id`，同 `S/user/format.rs` 的註解）。

### 給使用者決定（spec 共同規則 4；照 spec 先處理，交給票 06 的使用者試一起看）

1. **ADR 草稿 10 份**（票 06 已編號成 ADR 0009–0017，`ranking-parameters` 改放 `docs/research/`）：`preceding-text-from-shell-then-own-commits`、`ranking-parameters`、`userdict-text-format`、`learning-expiry-deactivates`、`field-kinds`、`association`、`rerank-by-paths-after-rules`、`new-schemes-share-models`、`key-probabilities`、`phrases-and-manual-words`。內容是 plan 的理由原樣整理，沒加新的決定；「Considered Options」只寫 plan 裡提到的另一種做法。要使用者看過才編號。
2. **搬到 roadmap 的兩處**（roadmap 不在 spec 的四個去處裡，同票 01 的做法）：§2.1「不做（M4 票 30 評估）」的資料併進 roadmap M5 原有的那條；§2.2「已知限制：簡繁逐段轉換（乾隆 → 干隆）」沒有測試或註解說明，放進〈不排里程碑〉。
3. **刪掉的理由指到研究筆記或文件、不是測試或程式檔的**（spec 規則 1 只准測試或程式檔）：Gboard 22 個錯字與每鍵 p95（`scripts/experiments/autocorrect-threshold/README.md`）、whitelist（事實 M51）、各平台刪字的 API（事實 P25–P28）、CyIME 的評價（`docs/research/cyime.md`）、fcitx5 Ctrl+7（`docs/research/facts-products.md`）、模型階梯的數字（`docs/research/model-ladder.md`）、ADR 0004、0005、0007 已有原句的段落、`docs/development.md` 已有的段落。這些是量測或查證的結果，研究筆記本來就是它們的家，建議接受。
4. **契約與實作細節分不清、沒放進 `ARCHITECTURE.md` 的**（淨增預算 2,200 字，已用約 1,800）：英文模式 Enter 自動修正時各平台怎麼「先刪字、上屏，再交還 Enter」（Android 先 `commitText` 再送 Enter 的 `KeyEvent`、iOS 先 `insertText` 再 `insertText("\n")`、macOS 先 `insertText` 再讓 `handle` 回 false）。〈核心對殼〉只寫了「先照做 `delete_before`、`commit` 再交還」；平台的做法建議在各平台里程碑開工時寫進它的 spec。
5. **數字只留在票上的**：µs 量測（詞級 trigram 雜湊 31 → 64 → 45 µs、停鍵結果快取 154 → 161 µs）原本就是從票抄來的，沒搬，票留在 `.scratch/m3/`、`.scratch/m4/`。
6. 指向 plan §2 的引用（程式註解約 150 處，如 `plan〈介面〉〈上下文〉`）沒改：plan 的節標題還在、點過去是「已搬到」一行；照票 06 的範圍一次改，免得和票 03–05 平行改同一行註解。

### Code review

兩軸都看過（`git diff 4832a21...HEAD`）。已改：Spec 指出〈核心對殼〉的 1100 ms 計時起點寫錯（是每個按鍵、點擊之前從頭計）、〈候選的排位順序〉鄰鍵讀法與重排的先後寫反、漏了 v3 草稿的相容規則、plan〈資料流〉的「已刪」行沒提搬到 roadmap 的已知限制；Standards 指出 ADR 草稿有幾處 plan 沒寫的利弊或後果（已刪或改回 plan 原句）、`association` 與 `ranking-parameters` 的〈Considered Options〉不是替代做法（併進正文）、`key-probabilities` 與 `rerank-by-paths-after-rules` 補上 plan 有提的替代做法、roadmap 新條目不再指向 plan、證據表從 scratchpad 移進 `.scratch/arch-diet/02-evidence/`、〈矛盾〉改寫成請使用者確認。沒改的：roadmap 兩處與指到研究筆記的刪除理由（上面「給使用者決定」2、3）；`ARCHITECTURE.md` 的觸控門檻和 `web/src/touch.ts` 數字重複，是給其他平台殼的契約，留著；`new-schemes-share-models` 寫的是還沒有的第二個方案，是 plan 原有的打算，交使用者看要不要留。

- Session 紀錄：2026-11-18 plan §2 歸位：契約、排位順序、格式版本寫進 `ARCHITECTURE.md`，理由寫成 10 份 ADR 草稿，其餘逐段指到測試或程式檔
- 決定：取捨的理由依功能拆成 10 份 ADR 草稿，不併成一份（Agent 定）
- 決定：「不做句首建議」的資料併進 roadmap M5 原有那條，簡繁逐段轉換的已知限制放〈不排里程碑〉（Agent 定）
- 決定：plan 和程式碼對不上的 7 處照 repo 現況寫，列給使用者確認（Agent 定）
- 決定：觸控手勢門檻用一句話放進〈佈局格式〉、指向 `web/src/touch.ts`，當給各平台殼的契約（Agent 定）
- 決定：指向 plan §2 的程式註解（約 150 處）留給票 06 一次改，不和票 03–05 平行改同一行（Agent 定）
- 決定：逐段的證據表進 `.scratch/arch-diet/02-evidence/`，給票 06 與使用者查（Agent 定）
- 變更：無
