# 進度

工作單位是票（`.scratch/<里程碑>/issues/NN-slug.md`），狀態在票的 `Status:` 行。這裡只記各里程碑的閱讀指引、進行中里程碑的 session 紀錄、偏差與決定。

## 寫法與封存

- session 紀錄一列、偏差一條，整行各不超過 200 字元；細節在票的 `## Comments`、brief、plan 或 ADR，這裡只放指向它們的一句話。
- session 紀錄的「票」欄與偏差每條的開頭都標里程碑（`M0 01`、`2026-10-30 M0 票 01：…`），封存時靠它分。
- 里程碑驗收時，把它的 session 紀錄與偏差整段移到 `docs/archive/<里程碑>.md`，里程碑表的備註換成那個檔的連結。同一個 commit 把下一個里程碑標成進行中，封存這個 session 的紀錄標下一個里程碑。本檔只留進行中的里程碑。
- 里程碑的閱讀範圍、票序等細節寫在 `.scratch/<里程碑>/spec.md`，里程碑表的備註只寫一句狀態。spec 與票留在原處、不搬：它們是歷史快照，別處用路徑引用。
- `cargo xtask check` 的 `docs` 步驟強制 plan §6 列的規則：`##` 段、session 紀錄一整張表、長度、里程碑標記、最後一列的下一步、`.scratch/` 的 spec。其餘靠收尾自查：細節不寫進本檔、何時封存、備註只寫一句、spec 與票不搬。

## 里程碑與閱讀指引

| 里程碑 | 票目錄 | 拆票前讀 | 備註 |
| --- | --- | --- | --- |
| M0 | `.scratch/m0/issues/` | brief §3、§5、§6；plan §0、§1、§4.1、§4.5、§6；ADR 0001–0003；`.scratch/m0/spec.md` | 完成，[已封存](archive/M0.md) |
| M1 | `.scratch/m1/issues/` | brief §1.5、§4、§5、§9；plan §2–§5、§7.2；ADR 0004–0006；`.scratch/m0/issues/09-m0-close.md` 的 Comments（留給 M1 各票的發現） | 進行中，24 張票，票序見 `.scratch/m1/spec.md` |

## Session 紀錄

| 日期 | 票 | 結果 | 下一步 |
| --- | --- | --- | --- |
| 2026-11-04 | M1 封存 M0 | 票 09：里程碑 code-review、乾淨 checkout 重驗 spec〈驗收〉全過；M0 封存到 `docs/archive/M0.md`，M1 標進行中；細節見票 09 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |
| 2026-11-04 | M1 plan | M0 review 後依使用者同意改 plan §5.2、§7.2：語料管線與基準先行、拆 bigram 與佈局、學習模式、個人集改用 Rime 學習資料；brief §9、§12.2 同步 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |
| 2026-11-04 | M1 plan | 收到使用者小狼毫的 `quick5.userdb`，轉成 `bench/personal/rime-userdb.txt`（本機）；查出 Rime 記的是倉頡全碼而非按鍵，plan §5.2 改由自己轉碼 | 新 session 下 `/to-tickets docs/plan.md 拆 M1 的票` |
| 2026-11-04 | M1 拆票 | 寫 `.scratch/m1/spec.md`，依使用者同意拆成 24 張票（票序在 spec）；plan §7.2 縮成連結，plan、brief 裡的「M1〈名稱〉」改成票號 | 新 session 下 `/implement .scratch/m1/issues/01-corpus-yue-wiki.md` |
| 2026-11-04 | M1 01 | 粵語維基 → `bench/corpus/yue-wiki.tsv`（2,000 句）與 train / dev；`--verify` 加轉碼、洩漏、決定性三項，全過；細節見票 01 | 新 session 下 `/implement .scratch/m1/issues/02-bench-v0.md` |
| 2026-11-04 | M1 02 | `qim-bench`、`cargo xtask bench`；M0 baseline：fixed 1/1、yue-wiki 答錯 2000/2000；細節見票 02 | 新 session 下 `/implement .scratch/m1/issues/03-corpus-zh-wiki-and-colloquial.md` |
| 2026-11-04 | M1 03 | 中文維基、HKCanCor、Common Voice 進基準與訓練切分，字頻 `chars.tsv`；下載快取移到 repo 外；細節見票 03 | 新 session 下 `/implement .scratch/m1/issues/04-bench-corrections-and-personal.md` |
| 2026-11-04 | M1 04 | 基準加修正次數、KSPC、跳動、依長度分組、翻盤清單與個人集（`cargo xtask data` 轉碼，只報數字）；細節見票 04 | 新 session 下 `/implement .scratch/m1/issues/05-bench-latency.md` |
| 2026-11-04 | M1 05 | `cargo xtask bench` 報原生與 WebAssembly（Node）每鍵 p50 / p95，含第 20 碼起一組，只報不擋；M0 核心 p95 < 2 µs；細節見票 05 | 新 session 下 `/implement .scratch/m1/issues/06-lexicon-and-char-frequency.md` |
| 2026-11-04 | M1 06 | rime-essay 詞庫資料包（25 萬詞）與語料字頻，`Engine` 依字頻排字序、整串查詞庫；`anr` 首選「門口」進 `fixed.tsv`，各集合修正/句約減半；細節見票 06 | 新 session 下 `/implement .scratch/m1/issues/07-continuous.md` |
| 2026-11-04 | M1 07 | lattice、Viterbi + beam、unigram（β 2、γ 0）、選開頭一段後其餘重算；`anr`、`mir` 首選都對，維基答錯少 1/4 以上，口語修正次數變多；細節見票 07 | 新 session 下 `/implement .scratch/m1/issues/08-blackbox-apple.md` |
| 2026-11-04 | M1 08 | macOS、iOS 內建速成 T1–T18 寫進 `docs/research/blackbox-baseline.md`；brief §1.3 更新，5 題進 `fixed.tsv`；細節見票 08 | 新 session 下 `/implement .scratch/m1/issues/09-blackbox-gboard.md` |
| 2026-11-04 | M1 09 | Gboard 速成（Android）T1–T18 寫進黑箱基線；iPhone 版未測；brief §1.3 更新，3 題進 `fixed.tsv`；細節見票 09 | 新 session 下 `/implement .scratch/m1/issues/10-composition-key-residue.md` |
| 2026-11-04 | M1 10 | 組字中每個鍵都吃下：Enter 上屏輸入碼，標點、大寫先上屏組字，`z`、← →、`Key::Other` 不動作；標點全形；`toKey` 單元測試；細節見票 10 | 新 session 下 `/implement .scratch/m1/issues/13-learning-counts-and-ranking.md`（11、18 另在並行） |
| 2026-11-04 | M1 11 | 字級 KN bigram 資料包 2.9 MB（`--verify` 查大小與決定性），接進打分；各集合答錯都降，維基答錯少 1/10–1/4，口語少 1/8；細節見票 11 | 新 session 下 `/implement .scratch/m1/issues/12-bigram-tuning-and-report.md` |
| 2026-11-04 | M1 18 | `qim-layout`（點擊判定）、Gboard 直向佈局由 `qim-databuild` 算出、網頁觸控模式點出「門口」；plan §1.3「不可以」欄加同步測試；細節見票 18 | 新 session 下 `/implement .scratch/m1/issues/19-layout-format-final.md` |
| 2026-11-04 | M1 19 | `qim-layout/1` 定稿：JSON Schema、兩級驗證、版本選法、旋轉與圓角的點擊判定與性質測試；重疊佈局在 Rust 與網頁都點出上層鍵；細節見票 19 | 新 session 下 `/implement .scratch/m1/issues/20-touch-keyboard-features.md` |
| 2026-11-04 | M1 22 | 組字顯示字根；`cangjie-hint` 資料包與 `--verify` 抽查；組字中 Tab、觸控長按候選列出字根提示；細節見票 22 | 新 session 下 `/implement .scratch/m1/issues/23-panel-keys-and-rerank-stub.md`（等 20、21） |

## 偏差與決定

- 2026-11-04 M1 plan：M1 改成語料管線 → 首選字基準 → 詞庫 → 連打…（plan §7.2）：原票 1 的字頻要用票 3 的語料，順序矛盾；先建基準，每張票才有前後數字可比。
- 2026-11-04 M1 plan：bigram 拆成訓練、調參兩項，佈局拆成格式、觸控鍵盤兩項；M1 各項改用名稱稱呼，票號拆票時定。
- 2026-11-04 M1 plan：組字中按鍵殘留（`z`、Enter、標點時 `handled` 為假但組字還在）由票 09 的去處 M1 票 8、9 提前到〈連打〉，因為連打後組字變長、更常碰到。
- 2026-11-04 M1 plan：個人集由「≥ 100 句真實句子」改為使用者 Windows 小狼毫的 Rime 學習資料（使用者提供），句子改選填；內容只在本機、不送雲端（plan §5.2）。
- 2026-11-04 M1 plan：基準加依長度分組、翻盤清單、`--learning` 學習模式（plan §5.2）；`fixed.tsv` 起始只放已過的 `a 日`，`anr`、`mir` 在通過時加入，`--check` 才不會一開始就紅。
- 2026-11-04 M1 plan：個人集的輸入碼改由 `qim-databuild` 轉碼，不用 Rime 記下的碼：實測那是詞典的倉頡全碼（99.5%），不是使用者按的鍵，看不出打不打全碼（brief §12.2）。
- 2026-11-04 M1 拆票：plan §7.2 的 14 項拆成 24 張票：語料管線、學習各 2 張，基準 3 張，黑箱分 Apple 與 Gboard，英文與簡繁分開，〈整合〉拆成字根、面板與重排、收尾 3 張。
- 2026-11-04 M1 拆票：`bench --learning` 由首選字基準移到票 13（學習）：沒有學習前，開關學習的結果相同，量不出差別。
- 2026-11-04 M1 拆票：組字中按鍵殘留由〈連打〉拆成票 10；`Left` / `Right` 在票 21 之前先吃下不動作，「`handled` 為假時組字是空的」才成立。
- 2026-11-04 M1 拆票：佈局由「格式 → 觸控鍵盤」改成 tracer（票 18）→ 格式定稿（19）→ 觸控功能（20），第一張就從資料打通到網頁。
- 2026-11-04 M1 拆票：票 06 先讓詞庫用整串輸入碼查，網頁 `anr` 就查得到「門口」；`anr 門口` 在哪張票通過就由哪張加進 `fixed.tsv`。
- 2026-11-04 M1 01：維基語料改用 pages-articles 自己抽內文：CirrusSearch dump 搬到 `cirrus_search_index/` 且只留約 5 週，釘了很快下載不到（plan §4.3、brief §12.1）。
- 2026-11-04 M1 01：「只留連打字集內的字」定為含字集外字元的句子整句丟掉，不拿掉個別字，免得拼出原文沒有的相鄰字（plan §4.3）。
- 2026-11-04 M1 02：`bench --check` 對沒有 baseline 的集合、baseline 有但沒跑到的集合也紅（plan §5.2），新增語料的票要同 commit 跑 `--update-baseline`。
- 2026-11-04 M1 03：中文維基改取 archive.org 的 `zhwiki-20220501` 第一分片：dumps.wikimedia.org 對大站只留約 7 個月，釘了很快下載不到（brief §12.1）。
- 2026-11-04 M1 03：Tatoeba 不加：下載檔每週覆蓋、無版本與快照，又要逐句署名；口語改用 HKCanCor 與 Common Voice 的 CC0 句子檔（brief §5.2、plan §8）。
- 2026-11-04 M1 03：下載快取由 `.cache/sources/` 改到 repo 外的使用者快取目錄（`QIM_CACHE_DIR` 可改），各 worktree 共用（`docs/development.md`）。
- 2026-11-04 M1 04：個人集由 `cargo xtask data` 轉成本機的 `bench/personal/rime-userdb.tsv` 再給基準讀，`qim-bench` 不依賴 `qim-databuild`（plan §1.3、§5.2）。
- 2026-11-04 M1 04：修正次數的模擬卡住時按 Esc（算一次修正）改逐字打，逐字也打不出時 `--check` 紅（plan §5.2、`GLOSSARY.md`）。
- 2026-11-04 M1 05：WebAssembly 延遲用 Node 跑網頁綁定，不用瀏覽器：Chromium 計時粗化到 5 µs（隔離後），量不出現在的核心（`scripts/experiments/wasm-latency-browser/`）。
- 2026-11-04 M1 05：長輸入不另造題，取基準句子中組字第 20 碼起的按鍵（plan §5.2、`docs/development.md`）。
- 2026-11-04 M1 06：字頻放詞庫資料包，`Engine` 載入時依字頻重排字序；速成資料包的碼表照舊排年級 → Big5，不含語料衍生的資料（plan §4.1）。
- 2026-11-04 M1 06：詞庫資料包的 `LICENSE` 除了 LGPL-3.0，另寫四份語料的 CC 授權全文，因為字頻來自語料（plan §4、brief §5.2、§11）。
- 2026-11-04 M1 06：`sources.toml` 的授權三個欄位收成 `license_text` 一個 enum，多一種 `upstream_file`（`docs/development.md`）。
- 2026-11-04 M1 06：`anr 門口` 由票 06 加進 `fixed.tsv`，查詢搬進 `Engine` 也提前在票 06 做完，票 07 不再做（plan §5.2）。
- 2026-11-04 M1 07：`mir 一台` 由票 07 加進 `fixed.tsv`：unigram 連打已選對；票 11 改成換上 bigram 後仍要過（plan §5.2、§8）。
- 2026-11-04 M1 07：空白改成選全部候選的第一個；沒有路徑蓋滿整串時第一個是開頭一段，按空白只固定它、不上屏（brief §4.3 的延伸，票 07 Comments）。
- 2026-11-04 M1 07：口語兩集修正次數高於 baseline（連打錯一處就要選開頭一段），`--update-baseline` 只更新 `bench/baseline.json`，原因在票 07 Comments。
- 2026-11-04 M1 09：iPhone 版 Gboard 沒測：使用者決定不在 iPhone 安裝，比照 plan §8 的備案只測 Android，有沒有速成記為未確認（票 09 Comments）。
- 2026-11-04 M1 09 後（使用者決定）：會改寫打出的字或候選位置的行為都給開關，英文自動修正、英文句首自動大寫預設關，英文候選、容錯、聯想、雲端預設開（brief §4.7、§4.8、§7；plan §7.4、§7.5）。
- 2026-11-04 M1 10：Enter 有選定分段時上屏「選定的字詞＋其餘輸入碼」，照 librime `CommitRawInput`；標點、大寫先上屏組字，不學 macOS 的英文串（brief §4.3）。
- 2026-11-04 M1 10：`Key` 加 `Punct`、`Other`；亂按找到組字中 Tab、Delete、方向鍵改到輸入框，網頁沒對應的鍵都轉成 `Other`，組字中吃下（plan §2.1）。
- 2026-11-04 M1 10：網頁加 `node:test` 單元測試與 `check` 的 `unit` 步驟，不加測試框架依賴（`docs/development.md`、plan §1.4）。
- 2026-11-04 M1 10（協調者要求）：e2e 埠由寫死 5179 改成依 `web/` 路徑算出、`QIM_E2E_PORT` 可蓋，並行 worktree 不互搶，仍不接手別人的伺服器（`docs/development.md`）。
- 2026-11-04 M1 11：詞的段改成第一字用 bigram、詞內用詞頻（plan §2.3）：詞內也用字級 bigram 時同碼的詞分不出常用度，個人集答錯 1146 → 1452；票 12 用 dev 複核。
- 2026-11-04 M1 18：`qim-databuild` 可以依賴 `qim-layout`（用它的型別寫出預設佈局到 `build/layouts/`），plan §1.3 拆出 `qim-bench` 一列（plan §1.2、`docs/development.md`）。
- 2026-11-04 M1 19：佈局格式加四向 `swipe`（M5 才有作用）；鍵盤底色不進格式，由殼依淺深色決定；`touchInset` 正數往外擴（plan §2.6）。
- 2026-11-04 M1 19：「切得回字母頁」只算點一下的 `switchPage`，長按、滑動、組字中的動作不算，免得頁面只靠它們回去而困住使用者（plan §2.6）。
- 2026-11-04 M1 19：內建佈局嚴格檢查不含 iOS 地球鍵規則，iOS 預設佈局要不要放地球鍵在 M7 定（plan §5.3）。
- 2026-11-04 M1 19：`jsonschema` 讓網頁 WebAssembly 由 0.42 MB 變 4.6 MB；iOS 鍵盤擴充要不要略過 Schema 在 M2 量記憶體時定（plan §8）。
- 2026-11-04 M1 19（使用者決定）：`touchInset` 改名 `extendedEdges`，與 Delta skin 同名同義（正數往外擴、負數往內縮），格式還沒發佈、不留舊名（plan §2.6）。
- 2026-11-04 M1 22：字根提示用組字中的 Tab 叫出、觸控長按候選，fcitx5 table 是設定開關、沒有快捷鍵（brief §4.3、§4.4、§12.4）。
- 2026-11-04 M1 22：`Composition.radicals` 只含還沒選定的輸入碼的字根，選定的分段顯示字詞（plan §2.1）。
- 2026-11-04 M1 22：字根提示資料包不經 `Engine`，殼用 `RadicalHints::for_scheme` 另外載入；方案多了欄位，`FORMAT_VERSION` 4 → 5（plan §2.5、§4.1）。
