# 開源輸入法調查（階段 1）

調查日期 2026-10-30。方法：各 repo `git clone --depth 1`（commit 見〈來源〉），只讀原始碼與官方文件，未執行 repo 內任何程式。「未查證」＝在該 commit 找不到證據，不代表不存在。permalink 格式 `https://github.com/<owner>/<repo>/blob/<commit>/<path>`。librime 本體內部不在本文範圍（另有調查），這裡只寫各前端怎麼用它。

## 1. 摘要

1. **有兩條路線**：(a) 前端 + librime（Squirrel、Weasel、Trime、Hamster、Yuyan、fcitx5-rime 插件）；(b) 自寫核心（afm-ime、qingjian、libime 的 table/pinyin、FlorisBoard 的 Han provider、ASK 的 RadicalDictionary）。QIM 是速成（≤2 碼形碼，一碼多字），(b) 路線的資料結構（碼→候選的 trie / 檔案順序）遠比 librime 的 YAML 方案直接，但要自己扛學習、詞庫與授權。
2. **afm-ime 確實使用 Apple Foundation Models**（`Sources/IMECore/FMReranker.swift`：`import FoundationModels`、`SystemLanguageModel.default`、`LanguageModelSession`），端側執行，macOS 27 + Apple Intelligence 才可用；可選雲端（OpenAI 相容 / Anthropic）。用法是**異步重排與整句預測**，結果只放候選第 2 位，不取代詞庫首選。不使用 librime。
3. **沒有任何開源專案原生做「速成」的完整體驗**。最接近的是 fcitx5-table-extra 的 `quick3/quick5/quick-classic`（碼表 + libime table 引擎）。FlorisBoard、AnySoftKeyboard 只有倉頡 / 嘸蝦米 / 注音。
4. **libime table 引擎是形碼的最佳參考**：碼表 value = 檔案順序 index；使用者詞 index 遞增＝最近使用優先；`OrderPolicy` 三檔 `No/Fast/Freq`；auto phrase MRU；system 唯讀與 user 可寫分檔。學習只調 index、不記次數（`Freq` 檔才用 LM/history 分數）。
5. **鍵盤佈局格式**：沒有任何格式支援非矩形按鍵。表達力排序：FlorisBoard master 的 Unicode LDML keyboard3 XML（每鍵 `width`、`stretch`、`gap`、`flick`、`longPressKeyIds`、依裝置寬度選版面、`import`）≈ Trime YAML（每鍵 click / long_click / swipe_* / composing / popup）> HeliBoard JSON / AOSP XML（`width`、`keyXPos`）> Hamster YAML（`action` + `width` + `swipe[]`，無長按 popup 欄位）> ASK XML > 寫死在程式碼的 fcitx5-android / Yuyan / qingjian。
6. **iOS**：Apple 官方文件明寫自訂鍵盤無網路與共享容器（除非 open access）、密碼欄與電話欄被系統鍵盤取代、無法選字、無麥克風、**無 marked text**；**記憶體上限官方未公開**（KeyboardKit 自稱約 70 MB，社群說 30–60 MB，皆非第一手）。Hamster 的做法：主 App 部署（編譯詞庫）+ App Group 共享 + 擴充只載入已部署成果；但它沒有明寫記憶體策略。
7. **滑動輸入**：可讀且可重用（Apache-2.0）的是 FlorisBoard v0.5.2（首末鍵剪枝 + 長度剪枝 + 形狀 / 位置高斯機率 + 詞頻）與 ASK（角點路徑 + 方向懲罰 + 起末鍵懲罰 + 詞頻）。HeliBoard / OpenBoard 的原生滑動是閉源 `libjni_latinimegoogle.so`，repo 只有空 hook。FlorisBoard master 現在把 glide 整個註解掉（重寫中），要看 v0.5.2。
8. **授權**：Apache-2.0（FlorisBoard、ASK、Simple Keyboard）與 BSD-3（Yuyan 的 Kotlin 層，但引擎閉源且綁包名）可參考甚至重用；**GPL-3.0**（qingjian、Wisdom-Weasel、Trime、Squirrel、Weasel、HeliBoard、OpenBoard、fcitx5-macos）只能讀；**LGPL-2.1**（libime、fcitx5、fcitx5-android）靜態連結進 iOS App 有疑慮（法律推論，未查證）；afm-ime **無 LICENSE 檔**＝保留所有權利；Hamster 為 MIT 但已是 2025-05-13 的最後開源快照，且與 GPL 來源 Squirrel 的關係未逐檔核對。碼表資料（fcitx5-table-extra）授權另有問題，見 fcitx5 本體 + table 實例一節。
9. **LLM 的共通模式**（afm-ime、qingjian、Wisdom-Weasel）：確定性引擎即時出候選，LLM 異步、非阻塞、可丟棄；Wisdom-Weasel 用獨立鍵觸發。qingjian 的教訓：神經分數只能決定同一串按鍵內的名次，不能寫回跨讀法比較的分數。

## 2. 對照表

| 對象 | 架構（核心） | 排序 / 學習 | 佈局格式 | 授權 |
|---|---|---|---|---|
| afm-ime（macOS） | 自寫：mmap `dict.bin` + 音節 DP 切分 + 詞格 Viterbi；IMK | 靜態詞權重 + `UserFreq`（次數，×3 上限）；Apple Foundation Models 異步重排第 2 位 | 不適用 | 無 LICENSE |
| qingjian | Rust Core + 薄殼（IMK / TSF / Fcitx5 / IBus / Android） | 詞圖 Viterbi + bigram/trigram + 個人 n-gram + 牛頓冷卻衰減；可選本地 Transformer 重排；可選雲端聯想 | Kotlin 內宣告式表（`Key.weight`、choices、flick） | GPL-3.0-or-later |
| Wisdom-Weasel | Weasel + LLM provider（反引號觸發） | Rime 不動；LLM 候選附加在後 | 不適用 | GPL-3.0 |
| Hamster（iOS） | librime xcframework；App 部署 + App Group + 擴充載入 | librime 原生（八股文視方案） | YAML `keyboards:`（action / width / swipe） | MIT（快照，之後閉源） |
| Trime（Android） | librime JNI + librime-lua / octagram / predict | librime 原生 | YAML `preset_keyboards`（click / long_click / swipe_* / composing / popup） | GPL-3.0 |
| Squirrel / Weasel | librime 單例 + 每 context 一個 session | librime 原生 | 不適用 | GPL-3.0 |
| libime（fcitx） | TrieDictionary(DATrie) → SegmentGraph → lattice beam → ranking | 檔案順序 + user index 遞增 + kenlm trigram 與 HistoryBigram 插值（w=0.2） | 不適用 | LGPL-2.1+ |
| fcitx5 / chinese-addons | addon：Frontend / InputMethodEngine / UI 分離 | 同 libime | 不適用 | LGPL-2.1+ |
| fcitx5-android | 把 fcitx5 core 編成 .so，JNI 單執行緒 dispatcher | 同 libime | Kotlin 常數 `KeyDef`（percentWidth、Press/LongPress/Swipe、Popup） | LGPL-2.1 |
| Yuyan | 閉源改版 librime `.so` + Kotlin UI | 閉源，未查證 | Kotlin keycode 陣列 + 比例座標 | BSD-3（Kotlin）；引擎閉源且綁包名 |
| fcitx5-macos | IMK ↔ 自訂 Frontend addon | 同 libime | 不適用 | GPL-3.0 |
| FlorisBoard | NlpManager + SuggestionProvider；Han provider = SQLite 前綴查詢 | 形碼只有預存 weight，無學習；Latin 空殼 | 舊：JSON；master：LDML keyboard3 XML | Apache-2.0 |
| AnySoftKeyboard | 語言包 addon APK；RadicalDictionary(CompactTrie) | 詞頻 0..255 + 編輯距離；next-word bigram；形碼無學習 | XML `<Keyboard><Row><Key>` | Apache-2.0 |
| HeliBoard / OpenBoard | AOSP LatinIME：Java + 原生 Patricia trie 詞典 | unigram/bigram + user history 詞典 + 觸點機率 | simple 文字 / JSON / AOSP XML | GPL-3.0（+ Apache 部分） |
| Simple Keyboard | LatinIME 縮減版，無預測 | 無 | AOSP XML | Apache-2.0 |

## 3. 各對象

以下各節為逐對象調查（含原始碼證據與逐字佈局範例）。

### afm-ime（Bemly/afm-ime）

#### 1.0 是否使用 Apple Foundation Models / LLM — 結論：是

證據：
- `Sources/IMECore/FMReranker.swift` 第 2 行 `import FoundationModels`；第 29 行 `SystemLanguageModel.default.availability`；第 56-59 行 `LanguageModelSession(model: SystemLanguageModel.default, instructions: instructions)` + `session.respond(to: prompt)`。
  [permalink](https://github.com/Bemly/afm-ime/blob/a8543a1eac50ffb2e66f00eca1a047ebf3ba15f0/Sources/IMECore/FMReranker.swift#L28-L67)
- `Package.swift`：`platforms: [.macOS("27.0")]`、`swiftLanguageModes: [.v5]`，無第三方依賴。
- README 自述：「端侧 Apple Foundation Models（隐私安全，离线可用）+ 可选云端模型（OpenAI 兼容 / Anthropic）」，「macOS 27+，需要 Xcode 构建」。

模型在哪執行：
- 預設：**端側**，系統內建 `SystemLanguageModel.default`（程式不下載、不打包模型）。
- 可選：**雲端**。經 FoundationModels 的 Provider 協議（`LanguageModel` + `LanguageModelExecutor`，`Sources/IMECore/CloudProvider.swift` 的 `CloudProviderModel`）接 OpenAI 相容（`POST {base}/chat/completions`）或 Anthropic（`POST {base}/v1/messages`）。`FMReranker.respond` 統一出口（第 35-67 行）：雲端啟用且配置完整 → 雲端（15 秒逾時）；失敗 → 回落端側 FM；再失敗 → 回 nil、靜默用純詞庫。
- 隱私影響：雲端模式會把「游標前最多 60 字上文 + 拼音 + 候選」送給使用者填的服務商（`InputController.contextBeforeCaret`，`Sources/AFMInput/InputController.swift` 第 901-910 行取 `suffix(60)`）。

限制：
- OS / 裝置：macOS 27（README 與 Package.swift）；Experiments/FoundationModels-API-Survey.md 第 63、523 行寫明需裝置支援且開啟 Apple Intelligence，否則 availability 為 `.unavailable`（`deviceNotEligible` / `appleIntelligenceNotEnabled` / `modelNotReady`）。ad-hoc 簽名可直接用，無需 entitlement（作者自述已驗證，AGENTS.md 第 126 行附近）。
- 延遲：作者實測暖場 ~0.3-0.36s，「FM 做不了逐鍵級 (<100ms) 的跟手響應」（AGENTS.md「FM 延迟基准」節）。所以**只能異步**，詞庫引擎負責跟手。

#### 1.1 架構：輸入碼 → 分詞 → 候選 → 排序 → 上屏

| 步驟 | 做法 | 檔案 |
|---|---|---|
| 詞庫 | 多源詞庫（雾凇 rime-ice 的 cn_dicts + 萌娘/中文維基等）離線編成二進制 `dict.bin`，mmap 零拷貝載入，README 稱 698.6 萬條（含簡拼派生鍵）、載入 <1ms、單次查詢 0.1-4ms | `Sources/IMECore/DictStore.swift`, `DictFormat.swift`, `Sources/DictCompiler/main.swift` |
| 分詞 | 逆序 DP 枚舉最多 12 路音節切分，尾音節允許不完整 | `Sources/IMECore/PinyinSegmenter.swift` |
| 模糊音 | zh/z ch/c sh/s an/ang en/eng in/ing 查詢期變體展開，精確拼音永遠優先 | `CandidateEngine.swift` `fuzzySyllables`/`fuzzyKeys`（第 253、276 行） |
| 候選 | 多路切分 + mmap 二分前綴查表；簡拼、全拼混輸；詞格 DP（Viterbi）整句組詞 `composeSentence` | `CandidateEngine.swift` 第 34-196 行 |
| 排序 | 三檔：用戶詞庫命中 > 全鍵覆蓋 > 漸進前綴；檔內按分數；分數 × 用戶詞頻乘數（上限 ×3） | `CandidateEngine.swift` 第 176-184 行 |
| 上屏 | IMK `IMKInputController`，行內 marked text 顯示拼音，候選窗為 `NSPanel + NSGlassEffectView` | `Sources/AFMInput/InputController.swift`, `CandidateWindow.swift` |

不使用 librime；README 結構段與 AGENTS.md 都說 vendor/ 只放 rime-ice、squirrel、fcitx5-macos 等「參考實現」（vendor/ 不在 git 內）。

#### 1.2 排序與學習

- 詞頻：詞庫靜態權重；整句 DP 目標為 max 平均 `log(w) + 詞長加成 min(字數−1,3)`（`composeSentence` 註解，`CandidateEngine.swift` 第 190-196 行附近）。
- 使用者學習：`UserFreq`（`Sources/IMECore/UserFreq.swift`）——上屏時記「詞 → 次數」「詞 → 拼音鍵」，存 UserDefaults，上限 5000 條，2 秒防抖保存；同鍵輸入命中的詞進**最高優先檔**（壓過一切詞庫候選）；內置作者詞庫種子 `Data/builtin-user-words.json`（3581 條，隨倉庫散布，個人打字習慣資料，含刪除墓碑機制）。
- 容錯：模糊音、簡拼、半截音節寬鬆匹配。無 n-gram 模型、無時間衰減（**未查證**有無其他學習；程式中只看到 count）。
- **LLM 的角色**（`InputController.scheduleFMRerank`，第 796-873 行）：
  1. 詞庫候選先即時顯示；
  2. 停頓 0.4 秒後（整句預測則 50ms）以 `Task` 異步呼叫模型；
  3. **重排**：prompt = 上文 + 拼音 + 編號候選，系統提示要求「只輸出序號數字」，解析第一個數字（`FMReranker.rerank` 第 70-90 行、`firstIndex` 第 139-147 行）；
  4. **整句預測**：拼音 ≥8 字母或無詞庫候選時，讓模型把整串拼音轉成中文，並用「CJK 字元佔比 ≥50%、長度 ≤60」過濾（`looksLikeChinese`）；
  5. 結果以 ✦ 標記，**只放第 2 位，不頂掉詞庫首選**（防止慣性空格打出突然換掉的詞，`applyAIRerank`/`applyAISentence`）；
  6. 以 `fmGeneration` 與 `raw` 快照判斷過期，過期結果直接丟棄；
  7. 每次新建無狀態 `LanguageModelSession`（作者實測 1ms，避免 transcript 膨脹）。
  另有「翻譯」用途（⌃F 內聯翻譯、設定中心翻譯頁）。

#### 1.3 平台限制與繞法（IMK，皆摘自 AGENTS.md，作者實測；屬作者第一手記錄，非官方文件）

- IMK 預設只投遞 keyDown，flagsChanged 要覆寫 `recognizedEvents` 才收到（AGENTS.md 第 135 行）。
- Electron/Chromium 系應用不轉發修飾鍵事件，Shift 中英切換改用 `CGEventTap`（`ShiftModeMonitor.swift`，listen-only，作者稱無需 TCC 授權；但 ad-hoc 簽名每次 cdhash 變會令「輸入監控」授權失效，需自簽證書，AGENTS.md 第 178-184 行）。
- 非啟用面板（剪貼簿 ⌃V）必須用 `canBecomeKey=false` 的 NonKeyPanel，否則客戶端失焦觸發 `deactivateServer→commitComposition` 把首選強制上屏（AGENTS.md 第 151 行）。
- 「幽靈副本」事故：舊程序在退出延遲窗用同一 connection name 初始化 IMKServer 會劫持客戶端連線（README/AGENTS.md「kb32→kb33」）。

#### 1.4 鍵盤佈局

**不適用**（macOS 實體鍵盤輸入法，無螢幕鍵盤）。

#### 1.5 特色功能

- 剪貼簿歷史 ⌃V（50 條、1 秒輪詢 `NSPasteboard.changeCount`，落盤 `Application Support/AFM拼音/clipboard-history.json`）。
- 內聯翻譯 ⌃F、設定中心（SwiftUI，獨立 helper 進程 `Contents/PlugIns/AFMSettings.app`）。
- 英文：Shift 輕點中英切換；組詞中 Shift+字母=大寫英文字尾（GDP）。
- 液態玻璃候選窗 + Metal 水滴折射選中態（`Metal/DropletLens.metal`）。
- 同步：**未查證**（只看到用戶詞 JSON 匯入匯出）。聯想（next-word prediction）：沒有，只有重排與整句預測。

#### 1.6 授權

倉庫根目錄無 `LICENSE`（`git ls-files | grep -i licen` 無輸出）；README/AGENTS.md 也沒有 license 字樣。**判定：未宣告授權 = 預設保留所有權利，不可複製或再散布其程式碼**。另外它內含/編譯多個第三方詞庫（雾凇 rime-ice 為 GPL-3.0-only 來源之一，qingjian 的 landscape.md 明確如此標註；萌娘百科、中文維基 CC BY-SA、THUOCL MIT 等），成品散布會牽涉各詞庫授權。重用影響：只能借鑑做法，不能搬程式碼或其 `dict.bin`。

#### 1.7 值得 QIM 採用 / 坑

- 採用：「確定性詞庫引擎即時出候選，LLM 異步以 generation 計數 + 輸入快照校驗、結果只插第 2 位」的非阻塞模式；FM → 雲端 → 純詞庫逐級靜默回退；只要求模型輸出序號，解析容錯。
- 坑：macOS 27 + Apple Intelligence 是硬門檻，不能當唯一路徑；雲端模式要明示上文外送；速成/倉頡是 ≤5 碼短碼，重排價值在同碼多字的選字而非整句，需另評估；`UserDefaults` 存詞頻在大詞量下不可擴張（上限 5000）。

---

### qingjian（qingjian-team/qingjian）

> 2026-11-03 已以 `97dde30` 重查，模型、評測與隱私的部分以 `qingjian.md` 為準，過時處見該檔 §11。

#### 2.0 定位

跨平台輸入法，**Rust 自研 Core，不用 librime**（`docs/design/landscape.md`：「青简的『Core + 薄壳』架构与 Rime 相同」，雾凇 GPL 詞庫已移除）。平台殼：macOS IMK、Windows TSF（DLL + Server 進程）、Linux Fcitx5/Fcitx4/IBus、Android InputMethodService（Kotlin 自繪鍵盤 + Rust JNI `.so`）。iOS 在 roadmap 尚未做（`docs/plan/roadmap.md` 第 176 行）。

#### 2.1 架構

來源：`CLAUDE.md` 目錄地圖、`docs/design/core-internals.md`、`docs/notes/crate-notes.md`。

| 步驟 | 位置 |
|---|---|
| 輸入碼解析/分詞 | `crates/qingjian-core/src/parser`；九宮格 `nine_key`（`docs/design/nine-key.md`）；雙拼/注音/形碼 `code_scheme.rs` 等 |
| 詞庫 | `crates/qingjian-dictionary`（TSV 或 `.qj` mmap，按音節位置二分），容器格式 `crates/qingjian-format` |
| 候選 | `crates/qingjian-core/src/engine`（`Engine` 為唯一門面），`ranking/`、`sentence/`（詞圖 Viterbi）、`correction/`（拼寫糾錯） |
| 排序 | `crates/qingjian-core/src/ranking/mod.rs` 檔頭列出 8 條規則（音節數吻合 > 覆蓋字母 > … > 同輸入串選過次數 > 上下文分 = bigram `log P(詞\|上一詞)` + 選擇次數加分 > …），[permalink](https://github.com/qingjian-team/qingjian/blob/3d5fd2df431cea92eba39d08cd94a8a4093bb872/crates/qingjian-core/src/ranking/mod.rs#L1-L40) |
| 上屏 | `Engine::commit`；各殼只負責按鍵/文字客戶端/繪製（候選窗由 `qingjian-render` + `bamboo-slip` 自繪位圖，各平台只貼圖） |

#### 2.2 排序與學習；是否用語言模型

- **有語言模型**：
  - `crates/qingjian-lm`：bigram（`BigramModel`）、trigram（`TrigramModel`，讀「知意」`zh.ngram`）、`MixedModel` 插值。語料：中文維基（CC BY-SA）+ LCCC（MIT）（`docs/design/landscape.md`）。
  - `crates/qingjian-neural`（candle 推理）：字級 Transformer（small 23M / base 36M）打分，以及 P2C（拼音到漢字）整句生成。接法：詞圖 Viterbi 出前 6 條路徑，異步（停鍵 80ms 後送背景執行緒、20ms 輪詢）重排。
  - 評測（`docs/notes/neural-rescoring.md`）：8322 句凍結集上整句首選 37.6%（bigram 基線）→ 41.9%（字級 LM、λ0.75、前文 64 字）；P2C 重排 42.5%，P2C 自由生成首選 51.9% / 前五 73.5% 但延遲 130-200ms 平均、最慢 2.4s，尚未進產品。
  - 一個重要的坑（同檔「2026-09-24 換 P2C 之後」）：P2C 分數 `log P(漢字|拼音)` 只在**同一串按鍵內**可比，不能寫回 `Conversion::score` 用於跨讀法比較（糾錯、中英混輸），否則 `woxiangxuexirust` 會被糾成「我想學習如斯」；改為只決定名次。
- 使用者學習（`crates/qingjian-learning`，`docs/notes/crate-notes.md` 第 142-160 行）：詞頻、用戶詞、個人 n-gram（`user-ngram.tsv`）、個人敲錯表（`user-typos.tsv`）、個人英文詞，三張表帶**牛頓冷卻時間衰減**（`frequency_learner/decay.rs`）。選擇次數加分取對數並封頂（係數 0.5，上限 20 次），避免「的」壓過整詞。
- 容錯：模糊音（九條可設）、拼寫糾錯（整段一處編輯 + 音節級敲錯變體進詞圖當帶代價邊）、簡拼。
- **雲端 LLM 聯想**：`crates/qingjian-predict` 的 `CloudPredictor`（OpenAI 相容），**預設關閉**；開啟後把輸入與附近文字直接送使用者自填的服務商（README「数据与隐私」）。另有 `CloudGlossFiller` 補釋義。
- 語音：`crates/qingjian-voice` 火山引擎豆包流式 ASR（WebSocket + rustls）。

#### 2.3 平台限制與繞法

- Windows：TSF DLL 不能帶 Engine 依賴樹，所以拆成 `tsf`（DLL）+ `server`（Engine + IPC + 自繪候選窗）兩個 package（CLAUDE.md）。
- Android：輸入法服務彈不了權限框，錄音權限用透明 Activity 彈（`docs/design/voice-and-swipe.md`）；密碼/網址/信箱/非文字欄位不過引擎（`docs/notes/android.md`）；導航欄高度每次 `onStartInputView` 重量（`android.md`）；私密輸入看 `IME_FLAG_NO_PERSONALIZED_LEARNING`（瀏覽器無痕會帶）。
- 神經模型：首次 Metal 編譯載入 12 秒，需背景載入 + 預熱（neural-rescoring.md）；候選窗用 Rust 渲染成位圖貼上，emoji 因系統 COLRv1 畫不出，殼在詞位補畫（android.md）。
- 神經模型在行動端：**未查證**（文件只談 macOS Metal / CPU）。

#### 2.4 鍵盤佈局（Android）

格式：**不是資料檔，是 Kotlin 程式內的宣告式表**（`List<List<Key>>`，行 × 鍵，`Key.weight` 為相對寬度）。沒有使用者可編輯的佈局檔；使用者能選的是「布局種類」（拼音 26 鍵 / 九宮格 / 雙拼 / 五筆 86 / 注音大千 / 英文 / 日語羅馬音）與「鍵位排法」（標準 / 左手便捷）。`crates/qingjian-format/src/layout` 是 `.qj` 容器的 section 佈局，**不是鍵盤佈局**（避免誤讀）。
- 鍵形狀/大小/位置：只能調 weight 與整體高度/排法，無 per-key 自訂；鍵面由 `paint/KeyPainter.kt` 自繪。
- 多頁：有——字母頁 / 「123」數字頁 / 「符」分類符號頁 / `!?#` 常用符號頁 / 表情頁，由 `Key.Symbols`、`Key.NumberPage`、`Key.SymbolPage` 切換。
- 長按彈窗：`Key.choices`（字母鍵：大寫 / 角標符號 / 小寫；九宮格數字鍵：`abc2ABC`），滑到哪條抬手上屏。
- 滑動：上滑出角標（`flick_symbols`，位移 ≥18dp）、空格左右滑移動游標、**滑行輸入**（`SwipeDecoder`，形狀距離 + 詞頻先驗，只給 26 鍵拼音與英文，缺省關，`docs/design/voice-and-swipe.md`）。

逐字範例（`apps/android/app/src/main/kotlin/app/qingjian/inputmethod/keyboard/layout/KeyboardLayout.kt` 第 65-76、118-124 行；[連結](https://github.com/qingjian-team/qingjian/blob/3d5fd2df431cea92eba39d08cd94a8a4093bb872/apps/android/app/src/main/kotlin/app/qingjian/inputmethod/keyboard/layout/KeyboardLayout.kt#L65-L124)）：

```kotlin
    fun qwerty(chinese: Boolean, japanese: Boolean = false, fullWidth: Boolean = true): List<List<Key>> = if (japanese) {
        japaneseRows()
    } else {
        listOf(
            letters("qwertyuiop", TOP_HINTS),
            letters("asdfghjkl", if (chinese) MIDDLE_HINTS_ZH else MIDDLE_HINTS_EN),
            listOf(Key.Shift) + letters("zxcvbnm", BOTTOM_HINTS) + Key.Backspace,
            lettersBottom(chinese, fullWidth),
        )
    }
...
    fun nineKey(fullWidth: Boolean = true): List<List<Key>> = listOf(
        listOf(Key.Digit('1', "分词"), Key.Digit('2', "ABC"), Key.Digit('3', "DEF"), Key.Backspace),
        listOf(Key.Digit('4', "GHI"), Key.Digit('5', "JKL"), Key.Digit('6', "MNO"), Key.Clear),
        listOf(Key.Digit('7', "PQRS"), Key.Digit('8', "TUV"), Key.Digit('9', "WXYZ"), Key.Text("0", 1.5f)),
        lettersBottom(chinese = true, fullWidth),
    )
```

鍵型別定義（`.../keyboard/layout/Key.kt` 第 3-29 行）：

```kotlin
sealed class Key(val label: String, val weight: Float = 1f) {
    /** 长按气泡里的项；空的不弹。 */
    open val choices: List<String> get() = emptyList()

    /** 长按不滑直接抬手选哪一项。 */
    open val initialChoice: Int get() = 0

    /** 在键上往上一划上屏的字：同长按不滑直接抬手那一项（字母键是角标，九宫格是数字）；null 是不能上滑。 */
    val flick: String? get() = choices.getOrNull(initialChoice)

    class Letter(
        val char: Char,
        val hint: String = "",
        val initial: String = "",
        val finals: String = "",
        weight: Float = 1f,
        val flickShift: Boolean = false,
    ) : Key(char.toString(), weight) {
```

#### 2.5 特色功能

- 候選旁譯詞（學習語言：英/日/西/越/德，釋義表由 LLM 離線批量生成後隨包，`tools/gloss-gen`；詞彙記錄/統計頁）。這是它的賣點。
- 剪貼簿歷史：Android `ClipboardWatcher` → Rust `qingjian_learning::ClipboardHistory`（`clipboard.jsonl`，可存圖，最多 100-5000 條，敏感內容不記）；常用語：`phrase_book`（crates/qingjian-learning/src/phrase_book）。
- 英文候選/中英混輸（`english` 模組，`english_in_chinese`/`chinese_first`）；表情、符號面板、算式計算、信箱後綴補全。
- 領域詞庫 11 本、可匯入 TSV 與 Rime 檔（android.md「詞庫」）。
- 同步：**未查證**有雲端同步（todo.md 第 138 行把「配置同步、跨平台詞庫」列為待辦，未實作）。

#### 2.6 授權

`LICENSE` 內容為 GNU GPL v3 全文；README 與 `docs/design/landscape.md`「許可策略」宣告 **GPL-3.0-or-later**，專案名稱與 logo 不在授權內；`crates/bamboo-slip`、`bamboo-slip-i18n` 另有 `LICENSE.md`「雙許可」（本次未展開，**未查證具體條款**）。隨包資料各有授權（CC BY-SA 等，見 landscape.md 資料來源表）。
重用影響：QIM 若要搬程式碼必須 GPL；若 QIM 採寬鬆授權，只能參考設計與演算法（文件本身也是 GPL 專案的文字，借用結構不構成程式碼複製，但勿大段複製）。`bamboo-slip` 的雙許可條款需另讀 `crates/bamboo-slip/LICENSE.md` 才能判斷是否可用。

#### 2.7 值得 QIM 採用 / 坑

- 採用：Core + 薄殼，Core 做完整排序鏈，殼只翻譯事件；異步神經重排用「請求序號、只認最新、舊結果仍進快取」解決打字比模型快的競態；神經分只決定名次不寫回分數；時間衰減的個人詞頻；九宮格用「每位置多讀法」重用全拼管線（對 QIM 的 5 碼速成有借鏡：一碼多字母/多部件候選）。
- 坑：自研引擎 = 自行維護詞庫、LM、學習、糾錯全鏈（312 個檔的 core crate）；神經模型體積 56MB fp16、Metal 12 秒首編；評測要用乾淨集（作者踩過「回放日誌偏向基線」的尺子偏差）；GPL。

---

### Wisdom-Weasel（scukeqi/Wisdom-Weasel）

#### 3.1 相對上游 Weasel 加了什麼

README 開頭：「基于 Rime 小狼毫（Weasel）… 增加基于大语言模型（LLM）的智能预测功能」。**只加 LLM 預測候選**，Rime 方案/詞庫/部署流程不變。

新增檔案（皆在 `WeaselServer/`，對比上游 weasel 無此檔）：`LLMProvider.{h,cpp}`、`LlamaCppProvider.cpp`、`HFConstraintProvider.cpp`、`LLMCandidateWindow.{h,cpp}`、`ContextHistory.h`、`MemoryCompressor.{h,cpp}`；另有 `hf_backend/`（Python HTTP 服務）。`RimeWithWeasel/RimeWithWeasel.cpp` 被改動以掛接。

#### 3.2 架構與觸發

- `RimeWithWeasel.cpp` 第 204-260 行：讀 `weasel.yaml` 的 `llm/enabled`、`llm/provider_type`，建立 `OpenAICompatibleProvider` / `LlamaCppProvider` / `HFConstraintProvider`。抽象介面 `LLMProvider::PredictCandidates(context, current_input, max_candidates)`（`WeaselServer/LLMProvider.h` 第 9-27 行）。
- 觸發：**組詞中按反引號「·」鍵**（`RimeWithWeasel.cpp` 第 393-505 行）；不在組詞時反引號照常輸入；**連按兩下清空上下文歷史**。觸發後取當前 preedit 與 `ContextHistory` 的最近詞當 context，呼叫 `_TriggerLLMPrediction`。
- LLM 預測模式內：Esc 退出；空格選第一個候選，索引 = Rime 候選數 + LLM 候選數的連續編號（第 507 行後），所以 LLM 候選接在 Rime 候選之後共用選字鍵。
- 提示詞（`LLMProvider.cpp` 第 303-316 行，OpenAI 相容路徑）：要求「預測接下來最可能出現的 N 個候選詞」，只回候選、空格分隔、由高到低；輸入含上下文與當前輸入。
- 上下文：`ContextHistory` 保留最近 N（預設 50）個詞；超出容量時可異步用另一個 LLM（`llm/memory/`）把最舊一半壓成摘要（`MemoryCompressor`）。
- llama.cpp 本地：GGUF，`n_gpu_layers`、`model_type: Base|Instruct`（README 推薦 Base）。`hf_constraint`：自架 Python 服務，請求體 `{"prompt":…, "pinyin_constraints":[…]}`，做拼音聲母約束解碼（`hf_backend/README.md`）。

#### 3.3 排序/學習/是否用 LLM

- Rime 自身排序不動（librime）。LLM 候選是**附加的、非排序**：不重排 Rime 候選，也不學習（**未查證**有無把選中 LLM 候選回寫使用者詞庫；程式中只看到 ContextHistory.AddText 記錄上屏文字）。
- 這是「聯想/續寫」而非「依拼音重排」，與 afm-ime 的用途不同。

#### 3.4 平台限制與繞法

- 只支援 Windows（README：Windows 8.1~11）。
- 預測是同步還是異步：`PredictCandidates` 簽名為同步回傳 vector；README 無延遲數字；**未查證**其阻塞行為（LLMCandidateWindow.cpp 有獨立視窗，疑為先顯示再填充，未讀完）。
- 修改 `weasel.yaml` 後需重新部署或重啟 Server；llama.cpp 需自備 dll。

#### 3.5 鍵盤佈局

**不適用**（Windows 桌面）。

#### 3.6 授權

`LICENSE.txt` = GPL v3（README 也稱「GPLv3（与 Weasel 一致）」）。重用：GPL，只能讀。

#### 3.7 值得 QIM 採用 / 坑

- 採用：`LLMProvider` 抽象 + 三種後端的設定結構（可借鑑 `api_url / api_key / model / max_tokens / temperature`）；用獨立鍵觸發而非每鍵觸發，避免延遲與費用。
- 坑：觸發鍵佔用反引號；上下文只來自本程序上屏歷史而非應用游標前文；雲端模式外送使用者輸入；README 沒有任何準確度/延遲量測。

---

### Hamster 倉輸入法（imfuxiao/Hamster，iOS）

#### 4.1 librime 如何進入 iOS 鍵盤擴充

- 預編譯 xcframework：`librimeFramework.sh` 從 `imfuxiao/LibrimeKit` release 下載 `Frameworks.tgz`（版本 2.4.2），內含 librime、boost_{atomic,filesystem,regex,system}、glog、leveldb、marisa、opencc、yaml-cpp（`Packages/RimeKit/Package.swift` 第 17-49 行的 `.binaryTarget`）。
- 包裝：`Packages/RimeKit` = ObjC（`Sources/ObjC/irime_api.m`，819 行）呼叫 `rime_api.h` 再橋到 Swift（`Sources/Swift/Rime.swift`，285 行；`Rime.createTraits` 設 `appName="rime.Hamster"`、`sharedDataDir`、`userDataDir`）。
- 目標：App（`Hamster/`）、鍵盤擴充（`HamsterKeyboard/`，另有聲筆 `SbxlmKeyboard/` 使用改版 librime-sbxlm）。兩者共用 App Group `group.dev.fuxiao.app.Hamster`（entitlements）。
- **分工**：`RimeContext.deployment`（編譯方案/詞庫，重活）「**僅可用於主 App 調用**」，先在 App 沙盒目錄 `maintenance: true, fullCheck: true` 部署，再把 `sharedSupport`/`userData` 複製到 App Group（`Packages/HamsterKeyboardKit/Sources/RimeContext/RimeContext.swift` 第 223-343 行）；`RimeContext.start(hasFullAccess:)`「**僅用於鍵盤擴充調用**」（第 197-215 行）只載入已部署成果：
  ```swift
  Rime.shared.start(Rime.createTraits(
    sharedSupportDir: FileManager.appGroupSharedSupportDirectoryURL.path,
    userDataDir: hasFullAccess ? FileManager.appGroupUserDataDirectoryURL.path : FileManager.sandboxUserDataDirectory.path
  ))
  ```
  無「允許完全存取」時用者資料目錄退到擴充自己的沙盒（用者詞庫不與 App 共享）。
- 同檔註解：「提前在部署階段加載 RimeSwitch hotKey，此步驟放在鍵盤啟動階段會減慢啟動速度」。

#### 4.2 記憶體限制與繞法

- 在此 commit 的程式與文件中**沒找到**顯式的記憶體上限數字或 jetsam 對策（grep `memory|內存|jetsam|OOM` 只有 `KeyboardInputViewController.swift` 第 81-82 行被註解掉的 `didReceiveMemoryWarning` 與第 835 行註解）。**結論：未查證**（iOS 鍵盤擴充的實際記憶體上限值需另查官方文件/實測，本次未做）。
- 可從結構推得的繞法（皆有程式證據）：(a) 編譯/部署只在主 App 做，擴充只載入 prebuilt 的 `build/*.bin`；(b) 啟動階段不做重活；(c) 鍵盤啟動後非同步 `await`。仍屬「推論」而非作者明說的記憶體策略。

#### 4.3 排序與學習

librime 原生（`Rime.swift` 暴露 `candidateList()`、`selectCandidate`、`getCandidate(index,count)` 等）；iOS 端另有「拼音候選」`getPinyinCandidates`（RimeContext.swift 第 957 行），拼寫區高度、候選分頁（滑動分頁 `swipePaging`、上下文分頁）為 UI 層。無自帶語言模型；八股文/octagram 取決於方案是否載入（`Rime.createTraits(models:)` 可指定 modules）。

#### 4.4 鍵盤佈局格式

- 定義處：`Resources/SharedSupport/hamster.yaml`（2040 行）頂層鍵：`general`、`toolbar`、`keyboard`、`rime`、`swipe`、`keyboards`。Swift 模型：`Packages/HamsterKeyboardKit/Sources/Customize/Yaml/Models.swift`（`Keyboard`/`Row`/`Key`/`KeySwipe`/`KeyWidth`/`KeyLabel`）。`keyboard.useKeyboardType: custom(name)` 選用自訂鍵盤。
- 自訂鍵位：能。`Key.action`（必填：`character(a)`、`characterMargin(a)`、`keyboardType(classifySymbolic)`、`backspace`、`space`、`shift`、`enter`、`symbol(《》)`、`shortCommand(#重输)`…）、`width`（`available` / `input` / `inputPercentage` / `percentage(0.13)` / `points`，可分直橫屏）、`label`（可含 `loadingText`+`text`）、`processByRIME`、`swipe[]`；`Keyboard` 有 `rowHeight`、`buttonInsets`；`Row` 有 `rowHeight`。
- 形狀：只有寬度與行高/內距，沒有任意位置/不規則形狀。
- 多頁：以 `keyboardType(...)` 切換到其他鍵盤（符號、分類符號、數字九宮格、emoji、自訂 name）。
- 滑動：每鍵 `swipe: [{direction: up|down|left|right, action, processByRIME, display, label}]`，全域 `swipe` 區塊設 `distanceThreshold: 20`、`tangentThreshold: 0.577`、`longPressDelay: 0.3`、`spaceDragSensitivity: 10`，另有預設的 26 鍵上下滑表（`swipe.keyboardSwipe`，第 214-451 行）。
- 長按彈窗（callout）：`HamsterKeyboardKit/Sources/KeyboardKit/Callouts/` 在程式內提供（上游 KeyboardKit 風格），**yaml 的 `Key` 模型沒有長按候選欄位 → 長按彈窗不可由使用者在 yaml 定義**（`Models.swift` 第 184-200 行欄位列表為證）。

逐字範例 1：倉頡自訂鍵盤（`Resources/SharedSupport/hamster.yaml` 第 452-475 行；[連結](https://github.com/imfuxiao/Hamster/blob/65693706d01fc6c19ed6071968e542b0a2ef3f36/Resources/SharedSupport/hamster.yaml#L452-L520)）：

```yaml
keyboards:
  - name: "仓颉"
    rows:
      - keys:
          - action: character(q)
            label: "手"
            swipe:
              - direction: up
                action: character(1)
                display: true
          - action: character(w)
            label: "田"
            swipe:
              - direction: up
                action: character(2)
                display: true
          - action: character(e)
            label: "水"
            swipe:
              - direction: up
                action: character(3)
                display: true
          - action: character(r)
            label: "口"
            swipe:
              - direction: up
                action: character(4)
                display: true
```

逐字範例 2：第三排（含邊距鍵與百分比寬度，同檔第 575-600 行附近；[連結](https://github.com/imfuxiao/Hamster/blob/65693706d01fc6c19ed6071968e542b0a2ef3f36/Resources/SharedSupport/hamster.yaml#L575-L640)）：

```yaml
      - keys:
          - action: keyboardType(classifySymbolic)
            width: percentage(0.13)
          - action: characterMargin(z)
            width: available
          - action: character(z)
            label: "重"
            swipe:
              - direction: up
                action: character(~)
                display: true
          - action: character(x)
            label: "難"
            swipe:
              - direction: up
                action: character(:)
                display: true
```

逐字範例 3：滑動全域參數與 26 鍵滑動表（同檔第 214-240 行；[連結](https://github.com/imfuxiao/Hamster/blob/65693706d01fc6c19ed6071968e542b0a2ef3f36/Resources/SharedSupport/hamster.yaml#L214-L262)）：

```yaml
swipe:
  # 指空格移动多个 point，光标移动一位。所以数字越小越灵敏
  spaceDragSensitivity: 10

  # 划动距离阈值，当划动的距离大于或等于此值时，才会识别为一次划动
  distanceThreshold: 20

  # 此值是正切角度的阈值，用来限制上下左右划动时的角度，如果太大容易误触
  # 默认为 15 度的正切值，即上下左右划的的角度不能超过 15 度
  # tan(10º) ≈ 0.176, tan(15º) ≈ 0.268, tan(30º) ≈ 0.577, tan(45º) = 1, tan(60º) ≈ 1.732
  tangentThreshold: 0.577
  # 长按延迟时间，只要大于或等于此阈值，才会触发长按 Action，如空格划动
  longPressDelay: 0.3
  keyboardSwipe:
    # 中文 26 键滑动设置
    - keyboardType: chinese
      keys:
        - action: character(a)
          swipe:
            - direction: up
              action: character(`)
              processByRIME: true
            - direction: down
              action: character(~)
              processByRIME: true
```

注意：`Packages/HamsterKeyboardKit/Sources/Customize/自定义键盘.md` 只有標題一行，沒有文件。

#### 4.5 特色功能

- 工具列/候選列：`toolbar.*`（高度、拼寫區、候選字型、顯示 comment、滑動分頁）。
- 內建快捷指令 `ShortcutCommand`（簡繁切換、中英切換、選第二/三候選、單手左右、剪下/複製/貼上、`sendKeys` 等，`KeyboardKit/Actions/ShortcutCommand.swift`）。剪貼簿**歷史**：此 commit 的鍵盤層只看到 `UIPasteboard.general` 讀寫（`KeyboardInputViewController+.swift` 第 156-170 行），**未查證**有歷史清單。
- 同步：(1) librime 原生 `sync_user_data`，App 內「RIME 同步」鈕呼叫 `Rime.shared.API().syncUserData()`（`RimeContext.swift` 第 345 行起），`installation.yaml` 需手動加 `installation_id` 與 `sync_dir`，範例指向 iCloud Documents 容器 `iCloud~dev~fuxiao~app~hamsterapp/Documents/sync`（`HamsteriOS/Sources/ViewModel/RIME/RimeViewModel.swift` 第 334-339 行）；(2) 「iCloud」設定：開 `general.enableAppleCloud` 後，部署前先把 iCloud 檔案增量複製到沙盒，亦可手動把沙盒複製到 iCloud（`AppleCloudViewModel.copyFileToiCloud`）；(3) CloudKit 公共資料庫分享「雲端輸入方案」（`HamsteriOS/Sources/CloudKit/CloudKitHelper.swift`，`publicCloudDatabase`）；(4) 內置 `HamsterFileServer`（GCDWebServer）網頁傳檔與 Files 管理。**WebDAV：未查證/此 commit 無**（grep `webdav` 無命中）。
- 英文候選/聯想：取決於 Rime 方案，無自帶。

#### 4.6 授權

`LICENSE.txt` 全文為 **MIT**（Copyright 2025 xiao.fu）。README：「本项目最初采用 GPL-3.0 许可。从 v2.1.0 开始，它变更为 MIT 许可」，並說「目前已經在做商業化功能，在沒有資金的情況下不打算再開源」→ **此倉庫是 2025-05-13 的最後快照，之後閉源**。
風險：(a) README 的第三方列表含 Squirrel（GPL-3.0）；MIT 重授權是否涵蓋 GPL 來源的片段，**我沒有逐檔核對**（未查證），若 QIM 搬程式碼應避開明顯 Squirrel 衍生的部分；(b) `Packages/HamsterKeyboardKit/Sources/KeyboardKit/LICENSE` 為 KeyboardKit 自己的授權（本次未讀，MIT 係 README 所述）；(c) 內含 librime（BSD-3）。重用影響：可複製檔案並保留版權聲明；舊版（<2.1.0）git 歷史為 GPL，不要取舊版。

#### 4.7 值得 QIM 採用 / 坑

- 採用：「主 App 部署 + App Group 共享 + 擴充只載入」分工；鍵盤 YAML 的 `action: 函式式字串 + width: percentage/available + swipe[]` 設計簡潔，適合 QIM 的 26 鍵倉頡（`label` 顯示字根、`swipe up` 出數字就是現成範例）；以 `characterMargin(x)` 做邊距補位。
- 坑：記憶體上限沒有作者文件可循，QIM 若在 iOS 上放 librime 需自行實測；沒有 Full Access 時 userData 退沙盒（詞庫學習不共享）；長按彈窗不可 YAML 配置；專案已閉源，不能期待上游更新。

---

### Trime 同文（osfans/trime，Android）

#### 5.1 librime JNI 整合

- 子模組：`app/src/main/jni/{librime, librime-lua, librime-lua-deps, librime-octagram, librime-predict, snappy}`（`.gitmodules`）；外掛由 `app/src/main/jni/cmake/Rime.cmake` 第 6 行 `RIME_PLUGINS librime-lua librime-octagram librime-predict` 決定，**所以 Trime 內建 Lua 腳本、八股文語言模型（octagram）與 predict 聯想外掛**。
- JNI 層：`app/src/main/jni/librime_jni/rime_jni.cc`（455 行）。`Rime::startup`（第 39-59 行）逐字：
  ```cpp
  RIME_STRUCT(RimeTraits, trime_traits)
  trime_traits.shared_data_dir = sharedDir;
  trime_traits.user_data_dir = userDir;
  trime_traits.log_dir = "";  // set empty log_dir to log to logcat only
  trime_traits.app_name = "rime.trime";
  trime_traits.distribution_name = "Trime";
  trime_traits.distribution_code_name = "trime";
  trime_traits.distribution_version = versionName;

  rime->setup(&trime_traits);
  rime->initialize(&trime_traits);
  rime->set_notification_handler(notificationHandler, GlobalRef->jvm);
  rime->start_maintenance(fullCheck);
  ```
  暴露 JNI：`startupRime`、`deployRimeSchemaFile`、`deployRimeConfigFile`、`syncRimeUserData`、`processRimeKey`、`commitRimeComposition`、`getRimeContext`、`getRimeStatus`、`setRimeOption`、`selectRimeSchema`、`simulateRimeKeySequence`、`getRimeRawInput` 等。
- Kotlin：`core/Rime.kt`（612 行）+ `core/RimeDispatcher.kt`（協程單執行緒派發，所有 librime 呼叫串行化）、`RimeLifecycle.kt`、`RimeMessage.kt`（通知回呼）。輸入法服務 `ime/core/TrimeInputMethodService.kt`。
- 也實作了 librime 的 levers（`levers.cc`：使用者詞庫匯入匯出、方案選單）與 OpenCC（`opencc.cc`）。

#### 5.2 排序與學習

純 librime：translator 詞頻 + 使用者詞庫（userdb）自學；八股文 octagram（`lotem/librime-octagram`，grammar）與 `librime-predict`（聯想）以 plugin 內建，是否啟用由方案 yaml 決定。UserDict：`data/userdict/UserDictManager.kt`（`restoreUserDict`、`importUserDict` 文字檔匯入、快照）。Trime 本身無額外語言模型/LLM。

#### 5.3 平台限制與繞法

- Android 輸入法服務：以 `enter_label_mode` 取 App 的 ActionLabel 決定 Enter 鍵文字（trime.yaml `style.enter_label_mode`）；`keyboard_padding_*`、`keyboard_padding_bottom` 避開曲面屏/全面屏手勢誤觸；`keyboard_height` 鎖定避免切換閃爍。
- librime 檔案目錄由環境變數 `RIME_USER_DATA_DIR`/`RIME_SHARED_DATA_DIR`/`RIME_DISTRIBUTION_VERSION` 傳入 JNI（rime_jni.cc 第 42-44 行）。
- 藍牙實體鍵盤問題與橫屏分割：見 `doc/Keyboard.md`。

#### 5.4 鍵盤佈局格式（`trime.yaml`，主題檔）

- 檔案與結構：`app/src/main/assets/shared/trime.yaml`（1726 行），頂層：`style`（全域尺寸/字型/間距）、`preedit`、`window`（候選窗）、`fallback_colors`、`preset_color_schemes`、`liquid_keyboard`（符號/emoji 液態鍵盤）、`android_keys`、`preset_keys`（具名按鍵）、`preset_keyboards`（鍵盤）。Kotlin 模型：`data/theme/model/TextKeyboard.kt`、`PresetKey.kt`、`LiquidKeyboard.kt`。JSON Schema：`doc/trime-schema.json`。主題可用 `__include`、`patch:`、`importPreset` 繼承。
- 自訂鍵形狀/大小/位置：能。每鍵 `width`（占鍵盤寬百分比）、`height`、`round_corner`、`key_border`、字號與文字偏移、各狀態顏色（`TextKey` 全欄位）；鍵盤層 `width/height/horizontal_gap/vertical_gap/columns/keyboard_height`、橫屏換鍵盤 `landscape_keyboard` 與分割 `landscape_split_percent`、`label_transform: uppercase`。沒有任意座標/不規則形狀，仍是「行流式」佈局（一個扁平 `keys` 陣列自動折行，總寬 100% 換行）。
- 多頁：`Keyboard_symbols`、`Keyboard_number` 等具名動作切換；`ascii_keyboard`/`lock`/`reset_ascii_mode`；液態鍵盤用 `keyboards:` tab 列表（`type: SINGLE|HISTORY|TABS|…`）。
- 長按/滑動：每鍵可設 `click`、`double_click`、`lazy_double_click`、`long_click`、`swipe_up/down/left/right`、`composing`（組詞中的行為）、`has_menu`、`paging`、`combo`、`ascii`（`TextKeyboard.TextKey`，`KeyBehavior` enum 同名）；`popup: [..]` 為長按彈窗項目清單（`TextKey.popup`）。預設鍵：`repeatable`/`sticky`/`slide_cursor`/`slide_delete`/`shift_lock`（`PresetKey.kt`）。

逐字範例 1：預設 40 鍵鍵盤（`trime.yaml` 第 976-1003 行；[連結](https://github.com/osfans/trime/blob/ed54b8d95a802147129b6981ce79bf9b5937b361/app/src/main/assets/shared/trime.yaml#L975-L1030)）：

```yaml
preset_keyboards:
  default:
    name: 預設40鍵
    author: "osfans <waxaca@163.com>"
    ascii_mode: 0
    width: 10
    height: 44
    lock: true #切換程序時記憶鍵盤
    keys:
      - {click: '1', long_click: '!'}
      - {click: '2', long_click: '@'}
      - {click: '3', long_click: '#'}
      - {click: '4', long_click: '$'}
      - {click: '5', long_click: '%'}
      - {click: '6', long_click: '^'}
      - {click: '7', long_click: '&'}
      - {click: '8', long_click: '*'}
      - {click: '9', long_click: '('}
      - {click: '0', long_click: ')'}
      - {click: 'q', long_click: '_'}
      - {click: 'w', long_click: '-'}
      - {click: 'e', long_click: '+'}
```

逐字範例 2：含四向滑動與寬度（`trime.yaml` 第 1129-1143 行與 default 鍵盤末行；[連結](https://github.com/osfans/trime/blob/ed54b8d95a802147129b6981ce79bf9b5937b361/app/src/main/assets/shared/trime.yaml#L1129-L1155)）：

```yaml
    - {click: 'q', long_click: '!', swipe_up: 1}
    - {click: 'w', long_click: '@', swipe_up: 2}
    - {click: 'e', long_click: '#', swipe_up: 3}
    - {click: 'a', long_click: select_all, swipe_left: Left, swipe_right: Right, swipe_up: Up, swipe_down: Down}
    - {click: 's', long_click: '~', swipe_left: Home, swipe_right: End, swipe_up: Page_Up, swipe_down: Page_Down}
    - {click: 'd', long_click: '-', swipe_down: '_'}
    - {click: 'b', long_click: Time, swipe_up: Date}
```

default 鍵盤末行（寬度與 composing 行為，`trime.yaml` 第 1008-1016 行附近）：

```yaml
      - {click: Shift_L}
      - {click: Keyboard_symbols, long_click: Keyboard_number}
      - {click: Mode_switch, long_click: Menu}
      - {click: space, width: 30}
      - {click: "'", long_click: '"'}
      - {click: BackSpace, width: 15}
      - {click: Return, composing: Return1, long_click: CommitComment, width: 15}
```

逐字範例 3：液態鍵盤固定按鍵條（`doc/Keyboard.md` 與 `trime.yaml` 第 686-707 行；[連結](https://github.com/osfans/trime/blob/ed54b8d95a802147129b6981ce79bf9b5937b361/app/src/main/assets/shared/trime.yaml#L686-L707)）：

```yaml
liquid_keyboard:
  author: "tumuyan"
  row: 6              #每屏最多显示多少行按键
  row_land: 5         #横屏每屏最多显示多少行按键
  key_height: 40      #按键高度
  key_height_land: 40 #横屏模式按键高度
  single_width: 60    #single类型的按键宽度
  vertical_gap: 1     #纵向按键间隙
  margin_x: 5         #左右按键间隙的1/2
  fixed_key_bar:  # 固定按键条
    position: bottom  # 摆放位置（在滚动区域的……） top|bottom|left|right
    keys:  # 按键（显示名称为对应的label，不能放太多）
      - liquid_keyboard_exit
      - space1
      - BackSpace
      - Return2
      - clipboard_window
      - liquid_keyboard_switch
```

（注：範例 2 末行 `Return` 的 `composing: Return1` 即「組詞中 Enter 改送 Return1（上屏英文）」的機制，`preset_keys` 內 `Return`/`Return1` 定義在第 860 行起。）

#### 5.5 特色功能

- 候選：`window.*`（候選窗樣式）、`style.comment_position: right|top|overlay`（顯示 Rime comment）、`ime/candidates/`（compact 橫條 + unrolled 展開）、`ime/composition/`（預編輯視圖）、`ime/bar/`（工具列）。
- 剪貼簿：`data/db/ClipboardHelper.kt` 監聽 `ClipboardManager`，Room 資料庫 `DatabaseBean(text, html, type, time, pinned)`，有固定（pinned）；UI 在 `ime/clipboard/`，液態鍵盤用 `clipboard_window` 鍵跳轉。常用語/收藏：`CollectionHelper.kt`（同一 DB）；常用符號歷史：`data/SymbolHistory.kt` + 液態鍵盤 `type: HISTORY`。
- 同步：librime `sync_user_data`（JNI `syncRimeUserData`）；`worker/BackgroundSyncWork.kt`（WorkManager 週期背景同步，`doBackgroundSync` 內 `rime.runOnReady { syncUserData() }`，設定頁有同步間隔、外部儲存模式 `ProfileSettingsFragment.kt`）。WebDAV/iCloud：**未查證，此 commit 無**。
- 英文、Lua 擴充（librime-lua）、八股文、predict 聯想：以 plugin 提供，內容由方案決定。符號鍵盤含 emoji、數學、日韓、拉丁、音標等多個 tab。
- 無滑行輸入（未在原始碼找到 swipe typing）。

#### 5.6 授權

`LICENSE` = GPL v3 全文；各檔有 `SPDX-License-Identifier: GPL-3.0-or-later`（例如 `PresetKey.kt`）。重用：GPL，只能讀；其 YAML 格式（欄位名稱與語意）本身可借鑑，但直接相容 trime 主題需注意主題作者各自授權。

#### 5.7 值得 QIM 採用 / 坑

- 採用：以 `{click, long_click, swipe_*, composing}` 為單鍵行為集的表達式（「組詞中/非組詞」行為分離對輸入法很實用）；`__include` 繼承；`landscape_keyboard` 獨立定義；固定按鍵條。比 Hamster 表達力高，且有 JSON Schema 可做驗證。
- 坑：YAML 欄位多（主題 1726 行 + 4234 行的同文風格）→ 遷移/維護成本高；librime 全呼叫要經單執行緒 Dispatcher；Lua 與 octagram 模型增加 APK 體積。

---

### Squirrel / Weasel 如何呼叫 librime（簡述）

#### 6.1 Squirrel（macOS，InputMethodKit）

- 全域一次：`sources/Main.swift` 第 25 行 `rime_get_api_stdbool()`；第 129-130 行建 `IMKServer(name: InputMethodConnectionName, bundleIdentifier:)`（Info.plist 第 92-93 行 `Squirrel_Connection`）；`SquirrelApplicationDelegate.swift` 第 148-164 行 `set_notification_handler`、`setup(&squirrelTraits)`、`start_maintenance(fullCheck)`；第 90 行 `sync_user_data()`。
- 每個 `IMKInputController` 一個 session：`SquirrelInputController.swift` 第 369 行 `create_session()`，第 396-399 行 `destroy_session`。
- 輸入：`handle(_ event:)` → 把 macOS keyCode/modifier 翻成 X11 keysym（`MacOSKeyCodes.swift`）→ `rimeAPI.process_key(session, keycode, modifiers)`（第 412 行）→ `get_commit` 上屏（`client.insertText`，第 581 行）→ `get_context` 讀 preedit/候選 → `client.setMarkedText`（第 605 行）顯示行內預編輯 → 以 `client.attributes(forCharacterIndex:0, lineHeightRectangle:)`（第 612 行）取游標矩形，`panel.update(...)` 把候選送給 `SquirrelPanel`（非啟用的 NSPanel，自繪候選窗）。
- 特點：選項如 `_linear`/`_vertical`/`inline_preedit`/`inline_candidate` 以 librime option 與前端同步（第 396-420 行）；支援和弦（chord）輸入。倉庫根有 `SKILL.md` 描述架構。
- 授權 GPL-3.0。

#### 6.2 Weasel（Windows，TSF）

- 結構（兩進程）：`WeaselTSF/`（COM DLL，實作 `ITfKeyEventSink`、`ITfCompositionSink`、`ITfThreadMgrEventSink`、`ITfCandidateListUIElement`，`WeaselTSF.h`/`CandidateList.h`）→ named pipe（`WeaselIPC/PipeChannel.cpp` 第 106 行 `CreateNamedPipe`，`WeaselIPCServer/WeaselServerImpl.cpp`）→ `WeaselServer/`（常駐 Server，持有 librime）。`RimeWithWeasel/RimeWithWeasel.cpp` 是 Server 內呼叫 librime 的 handler：第 40 行 `rime_get_api()`、第 113 行 `start_maintenance`、第 175 行 `create_session`、第 274 行 `process_key`、第 798 行 `get_context`。
- 上屏：TSF 的 edit session 寫入 composition range（`WeaselTSF/Composition.cpp` 第 285 行 `SetText`，`TF_ES_ASYNCDONTCARE | TF_ES_READWRITE`）。候選窗：`WeaselUI/`（Direct2D 自繪，`WeaselPanel.cpp`，多種 Layout：水平/垂直/直書/全屏）由 Server 進程繪製；`WeaselDeployer/` 負責部署。
- 授權 GPL-3.0。Wisdom-Weasel 即在 `RimeWithWeasel.cpp` 與 `WeaselServer/` 這兩處掛 LLM。

---


### fcitx 系 / Yuyan / iOS 鍵盤擴充：先看結論

1. **libime 的 table 引擎就是「速成/倉頡這類形碼表」的現成參考實作**，而且 fcitx5-table-extra 直接帶速成（quick3/quick5/quick-classic）與倉頡（cangjie3/5）。碼表文字格式極簡單（見 §1.3）。**排序規則（`TableCandidateCompare`）、使用者詞「index 越大越前」、auto phrase 的 MRU，都值得抄「想法」**，但授權見下。
2. **授權陷阱**：libime = LGPL-2.1-or-later（含 kenlm 子模組，LGPL）；fcitx5-android = LGPL-2.1；fcitx5-macos = **GPL-3**；fcitx5-table-extra 的 `LICENSES/` 只有 GPL-3.0-or-later；Yuyan 的 .so 是**閉源預編譯**且綁包名。iOS app 上架 App Store 若靜態連結 LGPL 程式碼會有麻煩（LGPL 要求使用者能替換函式庫；App Store 動態替換不可行 —— 這點是我的法律推論，非 repo 內容，未查證，需法務確認）。**建議：只抄演算法與格式概念，自己實作核心；碼表資料單獨處理授權（見 §2.4）。**
3. **iOS 鍵盤擴充**：Apple 官方文件明確寫「secure text input 與 phone pad 不能用自訂鍵盤」、「沒 open access 無法網路與共享容器」、「不能選字、無麥克風」。**記憶體上限 Apple 官方文件沒寫**；廠商（KeyboardKit）自稱約 70 MB，社群 30~60 MB 不等 → QIM 核心必須能在 ≲30 MB 內跑（建議 mmap 唯讀碼表）。

---

### libime（https://github.com/fcitx/libime）

#### 1.0 授權
- `LICENSES/LGPL-2.1-or-later.txt`（第一行 `GNU LESSER GENERAL PUBLIC LICENSE Version 2.1`）；`src/libime/table/tablecontext.cpp` 檔頭 `SPDX-License-Identifier: LGPL-2.1-or-later`。
- `REUSE.toml`：隨附資料（`data/lm_sc.*.tar.*`、`data/dict-*`、`data/table-*`）也標 LGPL-2.1-or-later（含 Yuking 2002-2005 著作權）。
- kenlm：`.gitmodules` 把 `https://github.com/kpu/kenlm.git` 掛在 `src/libime/core/kenlm`（淺 clone 未拉子模組，所以沒讀其原始碼）。kenlm 官方 LICENSE 開頭寫 "Most of the code here is licensed under the LGPL"（curl raw.githubusercontent.com/kpu/kenlm/master/LICENSE），另有例外檔案，未逐檔查證。
- `src/libime/core/CMakeLists.txt`：`-DKENLM_MAX_ORDER=3`，即**最多 trigram**。

#### 1.1 整體流程（輸入碼 → 分詞 → lattice → ranking → 上屏）
共同骨架在 `src/libime/core/`：

| 步驟 | 檔案 |
|---|---|
| 輸入緩衝 | `inputbuffer.h/.cpp`（`TableContext`/`PinyinContext` 皆繼承 `InputBuffer`） |
| 分詞圖 | `segmentgraph.h/.cpp`（`SegmentGraph`：節點=字串位置，邊=可能的切分） |
| 詞典匹配 | `dictionary.h`、`triedictionary.h`（`TrieDictionary`：多個 trie，index 0 = system、`UserDict = 1` = 使用者、其後為 extra） |
| lattice + beam | `decoder.cpp`：`buildLattice` → `forwardSearch`（beam）→ `backwardSearch`（nbest）；預設 `beamSizeDefault=20`、`frameSizeDefault=40`（`decoder.h:33-34`） |
| 語言模型 | `languagemodel.cpp`（kenlm `QuantArrayTrieModel`，`languagemodel.cpp:44`）、`userlanguagemodel.cpp`（疊加使用者歷史）、`historybigram.cpp` |
| 候選 | `lattice.h`：`SentenceResult` |
| 預測/聯想 | `prediction.cpp`、`pinyin/pinyinprediction.cpp` |

資料結構：`core/datrie.h/.cpp` = **double-array trie**（`datrie.cpp:8` 註解 "cedar -- C++ implementation of Efficiently-updatable Double ARray trie"），模板 `DATrie<float|int32_t|uint32_t>`，支援 `set/foreach/traverse/hasExactMatch/save/load`。

#### 1.2 Table 引擎（`src/libime/table/`）
- **字典**：`tablebaseddictionary.{h,cpp}`
  - 多個 `DATrie<uint32_t>`：`phraseTrie_`（主碼表）、`singleCharTrie_`/`singleCharConstTrie_`（單字、造詞用）、`promptTrie_`（提示，例如速成/倉頡的 `&a 日` 字根提示）、`userTrie_`、`deletionTrie_`（使用者刪除）、`autoPhraseDict_`（MRU）。
  - 值（`uint32_t`）= **該條目在檔案中的順序 index**（`insertOrUpdateTrie`：`trie.set(entry, index); index += 1;` `tablebaseddictionary.cpp:182-190`）。key = `code + 分隔符 + word`。
  - `PhraseFlag { None, Pinyin, Prompt, ConstructPhrase, User, Auto, Invalid }`（`tablebaseddictionary.h:32`）。
  - 二進位格式 magic：`tableBinaryFormatMagic=0x000fcabe`（`.main.dict`）、`userTableBinaryFormatMagic=0x356fcabe`（`.user.dict`，zstd 壓縮 userTrie+autoPhrase+deletionTrie，`tablebaseddictionary.cpp:58-62, 1005-1020`）、`extraTableBinaryFormatMagic=0x6b0fcabe`。另可 `TableFormat::Text` 存取文字。
  - 其他能力：`matchWords(code, Exact|Prefix)`、`hasMatchingWords`、`reverseLookup`、`generate(word, code)`（依規則造詞碼）、`loadExtra`（額外詞庫）。
- **碼表文字格式**：`src/libime/table/table-text-format.md`（見 §1.3 貼原文）。
- **輸入上下文**：`tablecontext.{h,cpp}`
  - `TableContext::type/erase` → `update()`（`tablecontext.cpp:502`）：建 `SegmentGraph`（`tabledecoder.cpp: graphForCode`，依 auto rule 把整串碼切成各字的碼）→ `decoder_.decode(..., nbest, beam=20, frame=10)`；當輸入長度==maxLength且設了 `autoRuleSet` 時 nbest=5。
  - 候選去重（`shouldReplaceCandidate`）：句子節點少者優先；同為單詞則碼短者優先；再依 OrderPolicy 與 User flag。
  - `select(idx)`、`selected()`、`selectedSentence()`、`candidates()`、`candidateHint(idx)`（顯示剩餘碼或提示）、`autoSelect()`（`AutoSelect/AutoSelectLength/regex`，唯一候選自動上屏）。
- **排序**（`tablecontext.cpp:58-122` `TableCandidateCompare`），依序：
  1. 非 auto 候選排在 auto（造詞/句子）前。
  2. `lShort`：碼長 ≤ `noSortInputLength` 且非拼音者先，**且這群不看頻率，只看 index**。
  3. 若 `sortByCodeLength`，碼短優先。
  4. 同組：`index(lhs) > index(rhs)`。`index()`：User 詞回傳 `+index`（**越晚學的 index 越大，排越前**），一般詞回傳 `-index`（**檔案越前面越前**）。
  5. `OrderPolicy::Freq` 另先比 `score()`（語言模型分數，含 history）再比 index；`No/Fast` 只比 index。
  6. auto 候選之間比 `score()`。
  - 拼音反查候選有 `pinyinPenalty=-0.5`；若第一個是拼音候選，會旋轉成第一個非拼音/auto 候選。
  - 常數：`beamSize=20, frameSize=10`、`TABLE_DEFAULT_MIN_DISTANCE=1.0f`（`table/constants.h`）：與最佳分數差 <1.0 的 lattice 句子才保留。
  - **`OrderPolicy` 三種**（`tableoptions.h:19`）：`No`（純檔案順序）、`Fast`（User 詞置前，不做語言模型）、`Freq`（用語言模型/history 分數）。fcitx5-table-extra 的 quick5 設 `OrderPolicy=No`，cangjie5 設 `OrderPolicy=Freq`（見 §2.4）。
- **學習**：
  - `learn()`（`tablecontext.cpp:740`）：僅當 `tableOptions().learning()`；每個已選片段呼叫 `learnWord`：單詞若 flag 為 None/User → `dict.insert(code, word, PhraseFlag::User)`（重插即更新 index=升級排序）；若為 Auto → 先從 auto 移除再插為 User；多片段連選 → 組成新詞 `dict_.insert(word, User)`（以規則自動生碼）。接著把整句送進 `model.history().addWithCode(...)` 更新 bigram。
  - `learnAutoPhrase(history, hints)`（`:823`）：把**最近上屏的單字序列**（上限 `autoPhraseLength`）的每個後綴試著用造詞規則生碼，若尚不存在於碼表就以 `PhraseFlag::Auto` 放進 `AutoPhraseDict`（MRU，容量 `TABLE_AUTOPHRASE_SIZE=256`，`autophrasedict.h`「A simple MRU based dictionary」）。fcitx 端緩衝上限 10 字（`fcitx5-chinese-addons/im/table/state.cpp:103 limit=10`）。`saveAutoPhraseAfter` 控制 auto 詞命中幾次後升為 user 詞（選項在 `tableoptions.h`；升格邏輯細節未逐行查證）。
- **聯想/提示**：table 引擎本身**沒有**聯想（`src/libime/table` 內無 prediction）；有「提示 `Prompt=`」（字根/拆碼，顯示用）與「拼音反查 `Pinyin=`」。聯想只在 `core/prediction.cpp` / `pinyin/pinyinprediction.cpp`（以 history 與 LM predictionTrie）。
- **容錯**：table 引擎無 fuzzy；有 `MatchingKey`（萬用字元，quick5.conf 設 `*`）、`ExactMatch`、`EndKey`、`CommitRawInput`。

#### 1.3 碼表文字格式（逐字摘自 `src/libime/table/table-text-format.md`）
````
KeyCode=abcdefghijklmnopqrstuvwxy
Length=4
Pinyin=@
[Rule]
e2=p11+p12+p21+p22
e3=p11+p21+p31+p32
a4=p11+p21+p31+n11
[Data]
xycq 统
yfh 计
nnkd 局
[Phrase]
统计局
````
- 中英關鍵字皆可（`键码=`/`码长=`/`规避字符=`/`拼音=`/`提示=`/`构词=`/`[组词规则]`/`[数据]`/`[词组]`）。
- `[Data]` 每行 `<key> <value>`（空白分隔；value 含空白/引號時可加雙引號並以 `\"` `\\` `\n` 跳脫）；`[Data]` 內 `#` **不是**註解。
- 前綴字元：`Pinyin=@` → `@ni 你` 拼音反查；`Prompt=&` → `&a 日` 提示；`ConstructPhrase=^` → 造詞專用碼。
- `[Rule]`：`eN=` 長度恰為 N、`aN=` 長度 ≥ N；每項 `p|n` + 字序 + 碼位（1-9 從前、`z/y/x` 從後）。`[Phrase]` 只能放詞、碼由規則生成，且必須有 rule。
- 工具：`tools/libime_tabledict.cpp`（text ↔ binary 轉換）。

#### 1.4 Pinyin 引擎（`src/libime/pinyin/`）
- **詞典**：`pinyindictionary.{h,cpp}`（繼承 `TrieDictionary`，多子字典、`PinyinDictFlag {NoFlag, FullMatch, Disabled}`）。文字格式 `pinyindictionary-text-format.md`：`<hanzi> <full-pinyin> [weight]`，例 `倪辉 ni'hui 0.0`、`小企鹅 xiao'qi'e`。拼音編碼成 byte 序列（`pinyinencoder.cpp`）後作 trie key。
- **分詞**：`PinyinEncoder::parseUserPinyin / parseUserShuangpin`（`pinyinencoder.h:251-257`）把使用者字串做成 `SegmentGraph`（一個位置可有多種切分；含 fuzzy / 縮寫 / 簡拼）；雙拼靠 `shuangpinprofile.cpp`（`shuangpin-profile-format.md`）。
- **解碼**：`pinyindecoder.cpp`（`PinyinDecoder::createLatticeNodeImpl`，過濾大量未知單字節點）→ core `Decoder`；`pinyinmatchstate.cpp` 保存匹配狀態，`pinyincontext.cpp`（1199 行）是對外上下文（`select`、`selectCandidatesToCursor`、`cancel`、`selectCustom`、`preedit`、`learn()`、`contextWords`）。
- **Fuzzy / 容錯**：`PinyinFuzzyFlag`（`pinyinencoder.h:25-`）：`CommonTypo, V_U, AN_ANG, EN_ENG, IAN_IANG, IN_ING, U_OU, UAN_UANG, C_CH, F_H, L_N, S_SH, Z_ZH, VE_UE, Inner, InnerShort, PartialFinal, PartialSp, AdvancedTypo, Correction(依鍵盤佈局的鄰鍵誤觸，`pinyincorrectionprofile.{h,cpp}`)…`。
- **IME 參數**（`pinyinime.cpp:25-`）：`nbest_=1, beamSize_=20, frameSize_=40, wordCandidateLimit_=15, maxDistance_/minPath_`。
- **學習**：`PinyinContext::learn()`（`pinyincontext.cpp:1089`）→ `model()->history().addWithContext(contextWordsWithPinyin(), newSentence)`；若有任一詞沒有拼音（非拼音詞）就不入 history。自訂詞第一次不入 history。

#### 1.5 語言模型與使用者學習
- **靜態 LM**：kenlm `QuantArrayTrieModel`（`languagemodel.cpp:44`），最大階數 3；arpa 由 `data/CMakeLists.txt` 下載 `lm_sc.arpa-20260629.tar.zst`（`download.fcitx-im.org`）再轉換，另產生 `sc.lm.predict`（`libime_prediction` 工具）給聯想。`StaticLanguageModelFile` 內的 `predictionTrie()` 是 `DATrie<float>`。`State`（`StateSize = 20 + sizeof(void*)`）保存 kenlm state + 前一個 `WordNode*`。
- **HistoryBigram**（`core/historybigram.{h,cpp}`）：三個「池」`poolSize = {128, 8192, 65536}`，越新的池權重越高（幾何衰減，`p = 1/(1+HISTORY_BIGRAM_ALPHA_VALUE)`，`historybigram.cpp:574-590`）；每池記 unigram/bigram 頻率，新句子進第 0 池，滿了往後溢出（`pools_[i].add(popedSentence)`）；支援 `forget(word)`、`useOnlyUnigram`、`WordWithCode`（詞+碼，避免同形異碼混淆）。檔案 magic `0x000fc315`、version 4；二進位。
- **UserLanguageModel::score**（`userlanguagemodel.cpp:~126`）：
  `score = max(lmScore, log10( (1-w)*10^lmScore + w*10^historyScore ))`，`w = DEFAULT_USER_LANGUAGE_MODEL_USER_WEIGHT = 0.2`（`core/constants.h`）。也就是**靜態 LM 與使用者歷史做線性插值，但不會低於純 LM 分數**。
- 所以「詞頻」在 libime 是：碼表檔案順序（靜態）+ user index（時間遞增，近因）+ LM/History 分數（僅 Freq 策略）。

#### 1.6 值得採用 / 坑（對 QIM）
- 採用：(a) 碼表 text 格式極簡、`KeyCode/Length/[Data]/Prompt` 足夠表達速成；(b) 「值 = 檔案順序」讓碼表作者控制預設排序；(c) 使用者詞 index 遞增 = 最近使用優先，一行實作；(d) 兩個檔案（system 唯讀 + user 可寫）分離，便於 mmap 唯讀；(e) 對 history 加權 0.2 的線性插值公式；(f) HistoryBigram 多池衰減。
- 坑：(a) LGPL；(b) kenlm 子模組 + zstd + boost 相依重，iOS 擴充記憶體敏感；(c) DATrie 的 `userTrie` 需整個載入記憶體（未查證 mmap 支援）；(d) 速成是「簡碼最多 2 碼、1 碼 → 多字」，libime 的 table 路徑為通用設計，對速成「一碼 / 二碼 + 通配」之外的需求（萬用鍵 `*`）有 MatchingKey 支援，但**學習只調整 index、不記次數**（只有 Freq 策略才用頻率），QIM 若要真正頻率排序需自行加計數。

---

### fcitx5 本體 + table 實例

#### 2.0 授權
- fcitx5：`LICENSES/LGPL-2.1-or-later.txt`、`Unicode-DFS-2016.txt`。
- fcitx5-chinese-addons：`COPYING` = LGPL-2.1；`LICENSES/` 含 GPL-2.0-or-later 與 LGPL-2.1-or-later；`im/table/engine.cpp` 檔頭 `LGPL-2.1-or-later`。
- fcitx5-table-extra：`LICENSES/GPL-3.0-or-later.txt`（只有這一個）；README 對專利寫了倉頡作者 1982 年放棄、嘸蝦米與鄭碼專利已過期，**但沒有逐表著作權/授權聲明**；表資料來源未查證 → **QIM 使用其資料前要另查來源**。

#### 2.1 架構
- Addon：`src/lib/fcitx/addoninfo.h:25` `AddonCategory { InputMethod, Frontend, Loader, Module, UI }`。每個 addon 有 `.conf`（`Library=`、`Category=`、`Dependencies`），例如 `src/frontend/dbusfrontend/dbusfrontend.conf.in.in`（`Category=Frontend`、依賴 `core`、`dbus`）。`AddonManager` 載入共享庫並依賴順序初始化。
- **前端 / 引擎分離**：`Frontend`（`src/frontend/`：`dbusfrontend`、`waylandim`（text-input/input-method 協定 v1/v2）、`xim`、`ibusfrontend`、`fcitx4frontend`、`virtualinputcontext`）只負責把「客戶端事件」轉成 `InputContext` 呼叫；`InputMethodEngine`（`src/lib/fcitx/inputmethodengine.h`）只實作 `keyEvent / activate / deactivate / reset / filterKey / listInputMethods / virtualKeyboardEvent(5.1.0+)`，不知道前端是誰。`UI`（classicui/kimpanel 等）讀 `InputPanel` 繪圖。
- **InputContext**（`inputcontext.h`）：`capabilityFlags()`（`fcitx-utils/capabilityflags.h`，含 `Preedit, FormattedPreedit, Password, SurroundingText, Digit, Email, Url, Terminal, Sensitive, NoOnScreenKeyboard, ClientSideUI, KeyEventOrderFix…`）、`commitString`、`commitStringWithCursor`、`deleteSurroundingText(offset,size)`、`inputPanel()`（`inputpanel.h`：`setPreedit`/`setClientPreedit`/`setCandidateList`/`setAuxUp/Down`）。
- **候選 API**（`candidatelist.h`）：`CandidateWord`（`select(InputContext*)` 純虛擬 = 每個候選自帶上屏行為）、`CandidateList`（`label/candidate/size/cursorIndex/layoutHint`）；擴充介面用 mixin：`PageableCandidateList`、`CursorMovableCandidateList`、`CursorModifiableCandidateList`、`BulkCandidateList`（「useful for virtual keyboard」，`totalSize/candidateFromAll`）、`ModifiableCandidateList`、`ActionableCandidateList`（長按/右鍵動作）、`TabbedCandidateList`。
- **Surrounding text**：`surroundingtext.h` `SurroundingText { text, cursor, anchor（以字元計）, isValid/invalidate/setText/setCursor/deleteText }`；前端收到客戶端資料後 `surroundingText().setText(...)` + `updateSurroundingText()`（見 `dbusfrontend.cpp:343-353` DBus 方法 `SetSurroundingText "suu"`、`SetSurroundingTextPosition`）；引擎要刪字呼叫 `deleteSurroundingText`，前端回送給客戶端（`dbusfrontend.cpp:204`）。
- **Preedit**：兩種：`preedit`（面板顯示）與 `clientPreedit`（嵌入應用程式）；DBus 前端把 clientPreedit 以 `UpdateFormattedPreedit` 送回（`dbusfrontend.cpp:188-200`）。
- **Qt/GTK IMModule**：屬於 fcitx5-qt / fcitx5-gtk 另外的 repo（本次未 clone，**未查證**）；它們透過 DBus 前端連到 fcitx5。
- **UI 與「虛擬鍵盤」**：5.1.0 起 `InputMethodEngine::virtualKeyboardEvent`、`UIType::OnScreenKeyboard`（`addoninfo.h:29`），顯示核心已為螢幕鍵盤預留 API。

#### 2.2 chinese-addons 的 table 引擎
`im/table/`：`engine.cpp`（addon 入口）、`state.cpp`（每個 InputContext 一個 `TableState`，處理按鍵、換頁、`pushLastCommit` 收集 auto phrase）、`context.cpp`（繼承 `libime::TableContext`，加 `preeditText`）、`candidate.cpp`、`ime.cpp`（載入 `.main.dict` / `.user.dict`）。`cangjie.conf.in.in` 示例（簡體倉頡，用 libime 內建 `cj.main.dict`）。

#### 2.3 fcitx5-table-extra 速成/倉頡實例
檔案：`tables/quick5.txt`、`quick3.txt`、`quick-classic.txt`、`cangjie3.txt`、`cangjie5.txt`、`cangjie-large.txt`（各 `.conf.in`）。quick5.txt 共 68666 行、cangjie5.txt 68667 行。

`tables/quick5.txt` 開頭（逐字，前 40 行）：
````
键码=abcdefghijklmnopqrstuvwxyz
提示=&
码长=2
[数据]
&a 日
&b 月
&c 金
&d 木
&e 水
&f 火
&g 土
&h 竹
&i 戈
&j 十
&k 大
&l 中
&m 一
&n 弓
&o 人
&p 心
&q 手
&r 口
&s 尸
&t 廿
&u 山
&v 女
&w 田
&x 難
&y 卜
&z 重
a 日
a 曰
aa 𣊫
aa 𣊭
aa 𣊺
aa 𣋐
aa 𣇿
aa 𣅩
aa 𣆆
aa 𣇲
````
觀察：`码长=2` = 速成；同碼多字以**行序**決定預設順序（quick5 的 `ab` 行 75 起先列罕用字，`ab 明` 在第 111 行；quick3/quick-classic 的 `ab 明` 在第 50/45 行 → 不同版本表的排序品質不同，這是表資料問題，不是引擎問題）；`&a 日` 是 Prompt 前綴（字根提示，非候選）。

`tables/quick5.conf.in`（逐字）：
````
[InputMethod]
Name=Quick5
Icon=fcitx_quick5
Label=速
LangCode=zh_TW
Addon=table
Configurable=True

[Table]
File=table/quick5.main.dict
OrderPolicy=No
PinyinKey=[
AutoSelect=True
AutoSelectLength=-1
NoMatchAutoSelectLength=0
MatchingKey=*
ExactMatch=False
Hint=False
DisplayCustomHint=True
PageSize=9
CandidateLayoutHint=Vertical

[Table/PrevPage]
0=Up
1=minus

[Table/NextPage]
0=Down
1=equal
````
`tables/cangjie5.conf.in` 同構，差異：`Name=Cangjie5`、`Label=倉`、`OrderPolicy=Freq`、`PinyinKey=\``，其 `.txt` 頭為 `码长=6`（倉頡最長 5 碼 + 1）。cangjie5 `ab 明` 在第 82 行。

#### 2.4 對 QIM 的影響
- 碼表資料授權不明：table-extra 倉庫只放 GPL-3.0-or-later，且碼表內容來源（如香港/台灣開源碼表）**未在 repo 內查到逐表聲明** → 不能直接把這些 `.txt` 打包進 QIM（尤其閉源/ App Store）。需另找來源明確的資料（例如 Rime 的 `rime-quick`、`rime-cangjie`，**本次未查證其授權**）或自行製作。
- 學習 `learn()` 與 `OrderPolicy` 三檔的設計可直接當產品需求參考。

---

### fcitx5-android（https://github.com/fcitx5-android/fcitx5-android）

#### 3.0 授權
- `LICENSE`：`GNU LESSER GENERAL PUBLIC LICENSE Version 2.1`；原始檔頭 `SPDX-License-Identifier: LGPL-2.1-or-later`（例：`TextKeyboard.kt`）。
- 子模組（`.gitmodules`）包含 fcitx5、libime、fcitx5-chinese-addons、fcitx5-lua、fcitx5-rime、fcitx5-hangul…，各自授權不同；rime plugin 有 `plugin/rime/licenses/`（未逐檔讀）。

#### 3.1 整合 native core
- `lib/fcitx5`、`lib/libime`、`lib/fcitx5-chinese-addons` 是 Gradle 模組，各把 `src/main/cpp/<upstream>` 子模組用 CMake/NDK 編進 `.so`。
- `app/src/main/cpp/native-lib.cpp` + `androidfrontend/`（自訂 Frontend addon，單一 InputContext）+ `androidkeyboard/`（自訂簡單英文 InputMethodEngine，含 word hint）+ `androidaddonloader/`（把 addon `.so` 從 APK 的 `nativeLibraryDir` 載入，取代 dlopen 掃描）+ `androidnotification/`。
- Kotlin 端 `core/Fcitx.kt`：`System.loadLibrary("native-lib")`，`external fun startupFcitx / sendKeyToFcitxString / sendKeySymToFcitx / selectCandidate / resetInputContext / repositionCursor / toggleInputMethod / nextInputMethod / saveFcitxState / reloadFcitxConfig…`（`Fcitx.kt:232-291`）；所有呼叫經 `FcitxDispatcher`（單執行緒）序列化。UI 是原生 Android View（非 fcitx 的 UI addon），事件（preedit、candidates、input method 切換）由 native 以 `FcitxEvent` 推回 Kotlin。

#### 3.2 鍵盤佈局格式
**不是資料檔，是 Kotlin 程式碼常數**（沒有 JSON/XML 佈局；`app/src/main/assets/` 只有 README）。使用者無法自訂鍵位佈局，只能調整偏好（高度百分比、popup、swipe 方向、震動音效、主題等，見 `data/prefs/AppPrefs.kt`、`data/theme/`）。

真實範例：`app/src/main/java/org/fcitx/fcitx5/android/input/keyboard/TextKeyboard.kt:29-71`（逐字）：
````kotlin
        val Layout: List<List<KeyDef>> = listOf(
            listOf(
                AlphabetKey("Q", "1"),
                AlphabetKey("W", "2"),
                AlphabetKey("E", "3"),
                AlphabetKey("R", "4"),
                AlphabetKey("T", "5"),
                AlphabetKey("Y", "6"),
                AlphabetKey("U", "7"),
                AlphabetKey("I", "8"),
                AlphabetKey("O", "9"),
                AlphabetKey("P", "0")
            ),
            listOf(
                AlphabetKey("A", "@"),
                AlphabetKey("S", "*"),
                AlphabetKey("D", "+"),
                AlphabetKey("F", "-"),
                AlphabetKey("G", "="),
                AlphabetKey("H", "/"),
                AlphabetKey("J", "#"),
                AlphabetKey("K", "("),
                AlphabetKey("L", ")")
            ),
            listOf(
                CapsKey(),
                AlphabetKey("Z", "'"),
                ...
                BackspaceKey()
            ),
            listOf(
                LayoutSwitchKey("?123", ""),
                CommaKey(0.1f, KeyDef.Appearance.Variant.Alternative),
                LanguageKey(),
                SpaceKey(),
                SymbolKey(".", 0.1f, KeyDef.Appearance.Variant.Alternative),
                ReturnKey()
            )
        )
````
連結：https://github.com/fcitx5-android/fcitx5-android/blob/eb5b50216e70484a009933d9519db76005f1faf3/app/src/main/java/org/fcitx/fcitx5/android/input/keyboard/TextKeyboard.kt

`KeyDef.kt` 的模型（逐項摘要）：`KeyDef(appearance, behaviors: Set<Behavior>, popup: Array<Popup>?)`
- `Appearance`：`Text / AltText / Image / ImageText`，欄位 `percentWidth(預設 0.1f)`, `variant(Normal|AltForeground|Alternative|Accent)`, `border`, `margin`, `viewId`, `soundEffect`。→ **按鍵寬度是列內百分比，高度由鍵盤整體高度偏好決定**；不能每鍵自訂座標。
- `Behavior`：`Press / LongPress / Repeat / Swipe / DoubleTap`，各帶 `KeyAction`。
- `Popup`：`Preview / AltPreview / Keyboard.Preset(label) / Keyboard.Explicit(items) / Menu(items)`。
- 範例 `KeyDefPreset.kt` 的 `AlphabetKey("Q","1")`：`Press(FcitxKeyAction("Q"))` + `Swipe(FcitxKeyAction("1"))`（上滑出符號），popup = `AltPreview` + `Keyboard.Preset(character)`。
- **多頁**：`KeyboardWindow.kt` 管理 `TextKeyboard` / `NumberKeyboard`（`Name="Number"`）/ `PickerWindow`（符號、emoji、顏文字），`LayoutSwitchKey` 切換。
- **長按 popup 資料**：`input/popup/PopupPreset.kt`——`Map<String, Array<String>>`（`"e" to arrayOf("3","E","ê","ë",...)`），註解寫明 based on fcitx5 `src/im/keyboard/longpress.cpp`。
- **手勢**：`CustomGestureView.kt`：`longPressEnabled`、`swipeEnabled`、`swipeRepeatEnabled`、`swipeThresholdX/Y = 24f`；space 可滑動移動游標（`spaceSwipeMoveCursor`）。

#### 3.3 剪貼簿
`data/clipboard/ClipboardManager.kt`：監聽 Android `ClipboardManager.OnPrimaryClipChangedListener`，存進 Room DB（`clipboard/db`），`transformer` 鉤子（`plugin/clipboard-filter` 用 ClearURLs 規則清追蹤參數）；UI 在 `input/clipboard/`。（Android 10+ 只有前景輸入法能讀剪貼簿，這是平台規則，非 repo 內容，未逐條查證。）

#### 3.4 插件機制
- 外掛是**獨立 APK**（`plugin/rime|anthy|chewing|hangul|jyutping|thai|unikey|sayura|clipboard-filter`）。
- 發現：`core/data/DataManager.kt:37` `PLUGIN_INTENT = "${APPLICATION_ID}.plugin.MANIFEST"`，用 `PackageManager.queryIntentActivities` 找到外掛，讀其 `res/xml/plugin.xml`（schema `plugin/pluginSchema.xsd`：`apiVersion, domain, description, hasService`，rime 範例 `<apiVersion>0.1</apiVersion><domain>fcitx5-rime</domain>`），版本必須符合 `PluginDescriptor.pluginAPI`；取其 `applicationInfo.nativeLibraryDir` 供 `androidaddonloader` 載入 addon `.so`，資料（assets）併入 DataManager。
- 優點：核心 APK 小、引擎可分開授權（例如 GPL 引擎放獨立 APK）。

#### 3.5 對 QIM
- 值得採用：`KeyDef`（外觀/行為/popup 分離）模型、`PopupPreset` 資料、單執行緒 dispatcher 包 native core、候選用 `BulkCandidateList` 概念做展開候選。
- 坑：佈局寫死在程式碼，不適合「使用者/社群可換速成佈局」；LGPL；Android 專用 View 無法重用到 iOS。

---

### Yuyan 輸入法

#### 4.1 正確 repo
- App：`https://github.com/gurecn/YuyanIme`（commit 75bd0ae…）；SDK：`https://github.com/gurecn/yuyansdk`（commit c99f9b7…）。使用者給的 `gurecn/Yuyan-IME` 不存在（GitHub API 無回應）。YuyanIme 以子模組 `yuyansdk` 引入（`.gitmodules`），淺 clone 下 `yuyansdk/` 為空，實際程式碼讀自 yuyansdk repo。

#### 4.2 架構
- 引擎：**librime 的客製修改版，閉源預編譯**：`yuyansdk/libs/<abi>/libyuyanime.so`（arm64 約 4.4 MB）；另有 `libSogouShell.so`、`libgpen_handwriter.so`、`libhandwriting.so`、`libhwInterface.so`（手寫用，來源/授權未說明，**未查證**）。assets 含 `rime/build/*.prism.bin|table.bin`（預編譯 Rime 方案：pinyin、t9_pinyin、stroke、english、多種雙拼 prism）。
- JNI 介面：`src/main/java/com/yuyan/inputmethod/core/Rime.kt`：`startupRime / processRimeKey / selectRimeCandidate / getRimeContext / getRimeCommit / getRimeAssociateList / selectRimeAssociate / selectRimeSchema / setRimeOption / replaceRimeKey / clearRimeComposition…`（`:123-168`）。→ 輸入碼→分詞→候選→排序→上屏 **全在 Rime 內**，Kotlin 只拿 context/candidates；聯想用 `getRimeAssociateList`。
- Android 層：`service/ImeService.kt`（InputMethodService）、`keyboard/`（`TextKeyboard`、`HandwritingKeyboard`、`container/QwertyContainer|T9TextContainer|SymbolContainer|ClipBoardContainer…`）。
- 排序/學習：由 Rime（Rime 本身有 table/prism、userdb 學習）；Yuyan 自己的改動在閉源 .so 中，**未查證**。

#### 4.3 佈局格式
程式碼常數，非資料檔：`keyboard/KeyboardData.kt` 用 Android keycode 整數陣列表示每一列，分 `SkbStyleMode.Google/Samsung/Yuyan` 三套風格（逐字摘自 `:8-18`）：
````kotlin
    val layoutQwertyCn: Map<SkbStyleMode, ArrayList<Array<Int>>> = linkedMapOf(
        SkbStyleMode.Google to arrayListOf(
            arrayOf(45, 51, 33, 46, 48, 53, 49, 37, 43, 44),
            arrayOf(29, 47, 32, 34, 35, 36, 38, 39, 40),
            arrayOf(KeyEvent.KEYCODE_SHIFT_LEFT, 54, 52, 31, 50, 30, 42, 41, KeyEvent.KEYCODE_DEL)),
        SkbStyleMode.Samsung to arrayListOf(
            arrayOf(45, 51, 33, 46, 48, 53, 49, 37, 43, 44),
            arrayOf(29, 47, 32, 34, 35, 36, 38, 39, 40),
            arrayOf(75, 54, 52, 31, 50, 30, 42, 41, KeyEvent.KEYCODE_DEL)),
````
另有 `layoutT9Cn`（九宮格）、`layoutStrokeCn`、`layoutLX17Cn`（乱序17）、`layoutHandwritingCn`。`KeyPreset.kt` 為每鍵 `[主標籤, 上滑符號]`（例：`KEYCODE_Q to arrayOf("Q","[")`）。`entity/keyboard/SoftKey.kt` 以**比例座標**（`mLeftF, mTopF, widthF, heightF=0.25f`）計算區域 → 內部可任意位置大小，但沒有外部可編輯格式。手勢：`BaseKeyboardView.kt` 用 `GestureDetector` + `MSG_LONGPRESS`（逾時為偏好 `longPressTimeout`）；`view/popup/` 長按彈出；上滑符號（`KeyboardSymbolSlideUpMod`）。另有單手、懸浮、數字行模式。

#### 4.4 特色
九宮格（T9）、乱序17、手寫（閉源庫）、花漾字、剪貼簿聯想、常用語、懸浮鍵盤。權限最小化（官方 README 聲稱離線、不要網路權限）。

#### 4.5 授權（實際內容）
- 兩個 repo 的 `LICENSE` 皆為 "Copyright (c) 2026, 王莹 … BSD 3-Clause" 全文。
- **但 `yuyansdk/README.md:96-` 〈許可證與商業授權聲明〉另行聲明**：預編譯閉源庫 `libyuyanime.so` **不適用 BSD-3**；試用版內建技術保護，**不得改包名，否則核心驗證失敗**；要改需商業授權；公開原始碼「不包含」對 librime 核心的優化修改。
- 對 QIM：Kotlin 層（BSD-3）可參考；**引擎不可用**。

#### 4.6 值得採用 / 坑
- 採：T9 / 比例座標 / 單手 / 懸浮 / 數字行等「鍵盤形態」的產品想法。
- 坑：引擎黑箱、包名綁定、雙授權；佈局全寫死。

---

### fcitx5-macos（https://github.com/fcitx-contrib/fcitx5-macos）

- 授權：`LICENSE` = **GNU GPL v3**（`GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007`）→ 不可併入非 GPL 專案。
- IMK 接入：`src/server.swift:164` `AppDelegate.server = IMKServer(name: Info["InputMethodConnectionName"], bundleIdentifier:)`；`cmake/MacOSXBundleInfo.plist.in` 設 `InputMethodConnectionName=org.fcitx.inputmethod.Fcitx5_Connection`、`InputMethodServerControllerClass=Fcitx5.FcitxInputController`、`LSBackgroundOnly`、`ComponentInputModeDict/tsInputModeListKey`、`TICapsLockLanguageSwitchCapable`。
- `src/controller.swift`：`FcitxInputController: IMKInputController`——`handle(_ event:client:)`（只認 keyDown/flagsChanged；`recognizedEvents`）→ `processKey` →C++ `process_key(uuid, unicode, mods, code, isRelease, isPassword, surroundingText, cursor, anchor)`，回傳 JSON（`SyncResponse{commit, preedit, caretPos, dummyPreedit, accepted}`）→ `commitAndSetPreeditSync` 呼叫 `IMKTextInput` 的 insertText / setMarkedText。`activateServer`/`deactivateServer` → `focus_in`/`focus_out`；`commitComposition` 對應 `commit_composition`。
- 與 core 橋接：`macosfrontend/macosfrontend.{h,cpp,swift}`——自訂 fcitx5 **Frontend addon** `MacosFrontend`（`keyEvent(ICUUID,...)`），每個 IMK controller 對應一個 `MacosInputContext : InputContext`，覆寫 `commitStringImpl`、`updatePreeditImpl`；Swift ↔ C++ 靠 `module.modulemap`（Swift/C++ interop）。候選窗用 `fcitx5-webview` + `webpanel/`（網頁渲染），不是 IMK 內建 candidates 窗。
- 平台坑（值得記）：`macosfrontend.swift:110-133` 註解說明「沒有 client preedit，Backspace 在 Terminal/iTerm/JetBrains/VSCode 會同時被 IM 與 app 處理…所以強制 dummy preedit（ZWS 或空白）」；secure input（`IsSecureEventInputEnabled`）被 app 濫用，需 `getSecureInputProcessPID` 判斷（`controller.swift:77-99`）；surrounding text 每次按鍵才讀，且在 Cmd/Ctrl 組合時不讀（JetBrains 副作用，`controller.swift:101-120`）。這些是 QIM macOS 殼的現成踩坑清單。
- 專案本身有 `AGENTS.md`（給 AI agent 的建置指令），Swift 6 + C++20 + CMake/Ninja，目標 macOS 13.3+。

---

### iOS 鍵盤擴充（記憶體 / Full Access 限制）

只挑 2 個：**Hamster（imfuxiao/Hamster，使用者說已知、但它是唯一在 iOS 擴充裡跑完整 Rime 的開源專案，故作記憶體與 Full Access 的實證）** 與 **KeyboardKit（框架）**。使用者提的「Hamster 以外」我另搜（GitHub search、WebSearch）沒找到比這兩個更相關的開源中文輸入 iOS 專案；其他結果多為一般英文鍵盤範例（未查證其品質）。

#### 6.1 Apple 官方文件（App Extension Programming Guide: Custom Keyboard）
來源：https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html （存檔文件，經 WebFetch/tavily 取回，下為摘錄）
- 預設無網路、無共享容器：「By default, a keyboard has no network access and cannot share a container with its containing app. To enable these things, set the value of the `RequestsOpenAccess` Boolean key in the `Info.plist` file to `YES`」。（官方文件用語是 *open access*，設定畫面叫「允許完全取用」/ Full Access。）
- 沒 open access：「Keyboard can perform all the normal duties…」、「No shared container with containing app」、「No access to file system apart from keyboard's own container」；有 open access 則可用共享容器、網路、`UIPasteboard`、播放按鍵音（`playInputClick`）、Location/Address Book/Camera Roll（各需授權）、iCloud。
- **Secure text field**：「any secure text input object… the system temporarily replaces your custom keyboard with the system keyboard」；**phone pad**（如 Contacts 電話欄）同樣被系統鍵盤取代。
- 插入文字：「provide text, in the form of an unattributed `NSString` object, at the text insertion point」；用 `textDocumentProxy.insertText / deleteBackward / adjustTextPositionByCharacterOffset:`。
- 不能選字/編輯選單：「it cannot select text… If that app provides an editing menu… the keyboard has no access to it. A custom keyboard cannot offer inline autocorrection controls near the insertion point.」→ **無 marked text（preedit 嵌入）**，QIM 的組字只能畫在鍵盤自己的候選列。
- 無麥克風：「Custom keyboards, like all app extensions in iOS 8.0, have no access to the device microphone, so dictation input is not possible.」
- 系統級：「After a user chooses a custom keyboard, it becomes the keyboard for every app the user opens.」
- **記憶體上限：Apple 該文件沒有寫**（WebFetch 回覆明確指出頁面不含記憶體限制）。

#### 6.2 記憶體數字（非 Apple 官方，標明可信度）
- KeyboardKit 官網 https://keyboardkit.com/about：「memory capped at ~70 MB (depending on device & OS)」（廠商自述，非 Apple）。
- WebSearch 結果（二手）：React Native 舊文件稱 48 MB、DEV Community 稱 ~60 MB、2015 年 StackOverflow 稱 ~30 MB（iOS 8/9）。**結論：官方未公開、依裝置/系統版本而異，設計目標抓 ≲30-40 MB 常駐較穩**（這是我的建議）。超限由 jetsam 直接殺掉擴充，無 crash log（二手說法，未查證）。

#### 6.3 Hamster（https://github.com/imfuxiao/Hamster）
- 授權：`LICENSE.txt` = MIT（Copyright (c) 2025 xiao.fu）；`README.md`：「本项目最初采用 GPL-3.0 许可。从 v2.1.0 开始，它变更为 MIT 许可」，且說明**後續程式碼不再開源**（商業化）→ 本 repo 為快照（pushed_at 2025-05-13）。引擎 librime 為 BSD；用的 KeyboardKit 是當時 MIT 版本（vendored 在 `Packages/HamsterKeyboardKit/Sources/KeyboardKit/`）；README 列 Squirrel 為 GPL-3.0 的參考。
- 架構：`HamsterKeyboard/HamsterKeyboardInputViewController.swift` 只是 `class … : KeyboardInputViewController {}`；`Packages/RimeKit`（librime 封裝，framework 由 LibrimeKit 的 GitHub Action 預編譯，`make framework`）、`Packages/HamsterKeyboardKit/Sources/RimeContext/RimeContext.swift`（Rime 上下文）。
- Full Access：`HamsterKeyboard/Info.plist:15-16` 設 `RequestsOpenAccess = true`；程式註解證實：沒有完全訪問，`UserDefaults.hamster`（App Group 共享）寫入失敗（`RimeContext.swift:39`、`KeyboardInputViewController.swift:847`）；震動回饋需完全訪問（`StandardHapticFeedbackEngine.swift:23`）；剪貼簿用 `UIPasteboard.general`（`KeyboardInputViewController+.swift:156-170`）。
- 記憶體：repo 內 grep「內存/memory/jetsam」無相關說明 → **其記憶體策略未查證**。

#### 6.4 KeyboardKit（https://github.com/KeyboardKit/KeyboardKit）
- 授權：`LICENSE` 開頭 "Closed Source License … KeyboardKit … is closed-source. The Software is free to start using and has commercial pro features that require a valid license"（README badge 也是 closedsource）。→ **QIM 不可依賴**；僅 `Hamster/Packages/HamsterKeyboardKit` 內含的舊 MIT 版本曾公開。
- 價值：官方文件把 Apple 限制整理得清楚（見 §6.2）；`KeyboardContext` 有 `hasFullAccess` 概念（Hamster 的 vendored 版 `KeyboardContext.swift:69`）。

#### 6.5 繞法與設計建議（對 QIM；來源為上列事實 + 我的推論）
1. 核心保持小：唯讀碼表以 mmap，使用者資料另存小檔；不載 kenlm 級語言模型。
2. 不要求 Full Access 也能打字：Full Access 只用於「與主 App 共享學習資料（App Group）、剪貼簿、按鍵震動/音」。Hamster 實證沒有 Full Access 時 App Group UserDefaults 寫入失敗 → QIM 需要降級路徑（鍵盤自己的容器存使用者詞頻）。
3. 密碼欄與電話欄由系統換成原生鍵盤，無法繞過，需要在產品說明中講明。
4. 沒有 marked text → 組字碼顯示在鍵盤候選列上方，上屏用 `insertText`；退格需要自行維護「已上屏字」以支援學習（`documentContextBeforeInput` 可讀前文，但有長度與權限限制，細節未查證）。

---


### FlorisBoard

#### 1.1 版本與改版狀況（先講結論）

- `ROADMAP.md`（master）：0.5 已完成（Snygg v2 主題、minSdk 24→26）；**0.6 才做預測文字／拼字檢查與 Language Pack extension**；另有 **k3lp**：「New keyboard layout engine + file syntax based on the upcoming Unicode Keyboard v3 standard」（開發也在 codeberg.org/k3lp/k3lp）；0.7+ 才「Reimplementation of glide typing with the new layout engine and predictive text core」。最新 tag：v0.5.2、v0.6.0-alpha01、v0.6.0-alpha02（`git ls-remote --tags`）。
- master 有 `ime/keyboard3/`（約 4400 行，`ImeController.kt`、`touch/TouchModel.kt`、`interaction/PointerTracker.kt`、`ui/ImeKeyboardBox.kt`），以 `org.k3lp.*` 函式庫解析 XML 佈局。
- 證據：master 的 `GlideTypingManager.kt` 第 46 行 `private var glideTypingClassifier = Unit //StatisticalGlideTypingClassifier(context)`；`StatisticalGlideTypingClassifier.kt` 第 38 行 `/*` 到第 667 行 ` */` 整個類別被註解；`ime/nlp/han/HanShapeBasedLanguageProvider.kt` 整個 class 在 `/* ... */` 內；`ime/nlp/latin/LatinLanguageProvider.kt` 的 `suggest()` 直接 `return emptyList()`（詞典是 `assets/ime/dict/data.json` 的 `{"the":255,...}` 測試用 JSON）。
- 因此「真實運作的」滑動輸入與 Han provider 以 **v0.5.2（2e82060）** 為準：Han provider 與 Glide 在 v0.5.2 是啟用的（`NlpManager.kt` 第 68-69 行註冊 Latin 與 Han provider；Glide classifier 在 `GlideTypingManager.kt` 第 `private var glideTypingClassifier = StatisticalGlideTypingClassifier(context)`）。v0.5.2 的 Latin provider 一樣是空殼（`suggest` 回 `emptyList()`），所以 **FlorisBoard 穩定版沒有英文單字預測**，但滑動輸入吃的是 `getListOfWords()`/`getFrequencyForWord()`。

#### 1.2 (1) 架構：輸入 → 候選 → 排序 → 上屏（v0.5.2）

- 按鍵 → `ime/text/keyboard/TextKeyboardLayout.kt` 觸控處理 → `KeyboardManager`（`ime/keyboard/`）→ 編輯器 `ime/editor/EditorInstance`（組字／composing）→ `ime/nlp/NlpManager.kt`。
- `NlpManager.suggest(subtype, content)`（`NlpManager.kt` 第 199 行起）：在 coroutine 內先取 emoji 建議（`EmojiSuggestionProvider`），否則呼叫 `getSuggestionProvider(subtype).suggest(... maxCandidateCount = 8 ...)`；以 `reqTime` 保證舊請求不覆蓋新請求；`assembleCandidates()`（第 280 行起）先問 `ClipboardSuggestionProvider`，沒有才用 internalSuggestions；結果顯示在 Smartbar（`ime/smartbar/CandidatesRow.kt`）。
- Provider 介面：`ime/nlp/NlpProviders.kt` 的 `SpellingProvider`、`SuggestionProvider`（`suggest`、`notifySuggestionAccepted/Reverted`、`removeSuggestion`、`getListOfWords`、`getFrequencyForWord`）；`SuggestionCandidate.kt` 有 `isEligibleForAutoCommit`、`confidence`、`secondaryText`。`determineLocalComposing()` 讓 provider 自己決定游標前哪段是「組字中」（Han provider 用它把游標前連續的碼鍵字元視為 composing）。
- 形碼（Han provider，v0.5.2）：`hanShapeBasedKeyCode`（哪些字元算碼）→ SQLite 查詢。實際 SQL（`HanShapeBasedLanguageProvider.kt`，master 註解版同文）：
  `database.query(layout, arrayOf("code","text"), "code LIKE ? || '%'", arrayOf(composingText), "", "", "code ASC, weight DESC", "$maxCandidateCount")`；第一個候選 `isEligibleForAutoCommit = (n == 0)`，`confidence = 0.5` 固定值。
- 語言包 extension：`assets/ime/languagepack/org.florisboard.hanshapebasedbasicpack/extension.json`（`"$": "ime.extension.languagepack"`，`items[].hanShapeBasedKeyCode`）+ `han.sqlite3`。我用唯讀方式看了 schema：表 `cangjielarge`（70863 列）、`boshiamy`（39694 列）、`zhengma`（125290 列），欄位都是 `code VARCHAR, text TEXT, weight DOUBLE`；例：`('&a','日',0.03125)`、`('a','對',0.0454…)`、`('aa','一下',0.0089…)`（zhengma 含詞組）。內建包只有這三種；README 說完整包需另外下載（連結為 TODO）。

#### 1.3 (2) 排序與學習

- Han provider：只有 SQL `ORDER BY code ASC, weight DESC`，`weight` 是檔內預存值；**沒有使用者學習**（`notifySuggestionAccepted` 只 `flogDebug`）；沒有 n-gram、沒有容錯。
- Latin provider：空殼，沒有排序（0.6 才做；`ROADMAP.md`）。
- 滑動輸入排序見 1.6。
- 使用者詞典：v0.5.2 有 `ime/dictionary/UserDictionary.kt`、`DictionaryManager.kt`（Room/系統 UserDictionary 的管理；細節未查證）；`fldic` 格式：**在 master 與 v0.5.2 兩個 checkout 內 `grep -rl fldic` 都沒有命中，查不到**（舊版 0.4 文件曾提過，這裡未查證）。
- 語言模型／LLM：無。

#### 1.4 (3) 平台限制與繞法

- 剪貼簿：`ime/clipboard/ClipboardManager.kt` 實作 `OnPrimaryClipChangedListener`；Android 10+ 背景不能讀剪貼簿，所以只在 IME 有焦點時收到變更。提供 `syncToSystem` / `syncToFloris`（`ClipboardSyncBehavior`）、歷史上限、到期清除（`enforceExpiryDate`，敏感項目另行處理）。資料庫 `clipboard/provider/ClipboardDatabase.kt` 讀 `ClipDescription.EXTRA_IS_SENSITIVE`（API 33+，第 135-136 行）標記 `isSensitive`；`ClipboardFileStorage` + `ClipboardMediaProvider` 處理圖片／URI。
- `FlorisCopyToClipboardActivity.kt`：用透明 Activity 取得前景才能存取系統剪貼簿（Android 10 限制的繞法；用途細節以檔名推測，未逐行查證）。
- secure 欄位：`KeyVariation.PASSWORD` 與 `variation_selector` 讓同一鍵在密碼欄換成不同資料（`KeyData.kt` 第 218 行 `VariationSelector`：default/email/uri/normal/password）。是否在密碼欄關閉學習／預測：此次未逐行查證。
- incognito：`ime/keyboard/IncognitoMode.kt`，`isPrivateSession` 傳進所有 provider 方法。
- Inline autofill：`ime/nlp/NlpInlineAutofill.kt`、`smartbar/InlineSuggestionsUi.kt`。

#### 1.5 (4) 鍵盤佈局格式（逐字範例）

**A. 舊格式 JSON（v0.5.2，與 master 的 assets 同；`app/src/main/assets/ime/keyboard/org.florisboard.layouts/layouts/characters/qwerty.json`）**
連結：https://github.com/florisboard/florisboard/blob/fe1241f4921b3eae923571ff7a3e113a7e10d677/app/src/main/assets/ime/keyboard/org.florisboard.layouts/layouts/characters/qwerty.json

```json
[
  [
    { "$": "auto_text_key", "code":  113, "label": "q" },
    { "$": "auto_text_key", "code":  119, "label": "w" },
    { "$": "auto_text_key", "code":  101, "label": "e" },
    { "$": "auto_text_key", "code":  114, "label": "r" },
    { "$": "auto_text_key", "code":  116, "label": "t" },
    { "$": "auto_text_key", "code":  121, "label": "y" },
    { "$": "auto_text_key", "code":  117, "label": "u" },
    { "$": "auto_text_key", "code":  105, "label": "i" },
    { "$": "auto_text_key", "code":  111, "label": "o" },
    { "$": "auto_text_key", "code":  112, "label": "p" }
  ],
  [
    { "$": "auto_text_key", "code":   97, "label": "a" },
```

popup 內嵌於鍵（`.../layouts/symbols/western.json`）：
```json
    { "code":   37, "label": "%", "popup": {
      "main": { "code": 8240, "label": "‰" },
      "relevant": [
        { "code": 8453, "label": "℅" }
      ]
    } },
    { "code":   38, "label": "&" },
    { "code":   45, "label": "-", "popup": {
      "main": { "code":   95, "label": "_" },
      "relevant": [
        { "code": 8315, "label": "⁻" },
        { "code": 8212, "label": "—" },
```

語系 popup mapping（`.../org.florisboard.localization/popupMappings/en.json`）：
```json
{
  "all": {
    "a": {
      "relevant": [
        { "$": "auto_text_key", "code":  230, "label": "æ" },
        { "$": "auto_text_key", "code":  227, "label": "ã" },
        { "$": "auto_text_key", "code":  229, "label": "å" },
```

extension manifest（`.../org.florisboard.layouts/extension.json`）：
```json
{
  "$": "ime.extension.keyboard",
  "meta": {
    "id": "org.florisboard.layouts",
    "version": "0.1.0",
    "title": "Default layouts",
    "maintainers": [ "patrickgold <patrick@patrickgold.dev>" ],
    "license": "apache-2.0"
  },
  "layouts": {
    "characters": [
      {
        "id": "arabic",
        "label": "Arabic",
        "authors": [ "HeiWiper" ],
        "direction": "rtl",
        "modifier": "org.florisboard.layouts:arabic"
      },
```
（原檔有 `description` 一行，此處省略以縮短。）

key 屬性（`ime/keyboard/KeyData.kt`、`ime/text/keyboard/TextKeyData.kt`，v0.5.2）：`type`（`KeyType.CHARACTER` 預設）、`code`（Unicode codepoint 或 `KeyCode` 負數特殊碼）、`label`、`groupId`（`GROUP_DEFAULT=0`、`GROUP_LEFT=1`、`GROUP_RIGHT=2`、`GROUP_ENTER=3`、`GROUP_KANA=97`，對應 popup mapping 的 `~left`/`~right`/`~enter`/`~kana`）、`popup`（`main` + `relevant`）。多型 `"$"`：`text_key`、`auto_text_key`、`multi_text_key`、選擇器 `case_selector`、`shift_state_selector`、`variation_selector`、`layout_direction_selector`、`char_width_selector`、`kana_selector`。

**能不能自訂：**
- 按鍵寬度：**舊 JSON 不行**。寬度是程式碼決定：`ime/text/keyboard/TextKey.kt` 第 124-170 行依 `computed.code` 設 `flayShrink`/`flayGrow`/`flayWidthFactor`（例：SHIFT/DELETE `1.56f`，SPACE `flayGrow=1.0f`，其餘 `1.00f`；NUMERIC/PHONE `2.68f`）。`TextKeyboard.kt` 第 54-140 行依 grow/shrink 做 flex 排版。**每鍵不同寬度只能改程式碼**（不能由佈局檔指定）。
- 位置／形狀：row 內從左到右排列，矩形；touchBounds 與 visibleBounds 分開（`Key.kt` 第 56、61 行）。不支援任意形狀。
- 多頁／symbols：支援，用多個佈局檔（`characters`、`symbols`、`symbols2`、`numeric`、`numericAdvanced`、`numericRow`、`phone`、`phone2` 與 `*Mod`），由 extension.json 的 `modifier` 與 `KeyboardMode` 切換。
- 長按 popup：支援（`popup.main` + `relevant`，加上語系 mapping 的 `~left` 等群組）。
- 滑動：鍵上滑動手勢是全域設定（`SwipeAction` 列舉含 `MOVE_CURSOR_*`、`DELETE_WORD`、`SHIFT`、`HIDE_KEYBOARD`、`INSERT_SPACE`、`SHOW_INPUT_METHOD_PICKER` 等，`ime/text/gestures/SwipeAction.kt`），不是每鍵屬性。
- 多語言：subtype + 語系 popup mapping + 不同佈局檔。

**B. 新格式（master，k3lp / Unicode LDML keyboard3 XML）** `app/src/main/assets/extensions/org.florisboard.k3.foundation/keyboard/qwertz.xml`（318 行）
連結：https://github.com/florisboard/florisboard/blob/fe1241f4921b3eae923571ff7a3e113a7e10d677/app/src/main/assets/extensions/org.florisboard.k3.foundation/keyboard/qwertz.xml

```xml
<keyboard3 xmlns="https://schemas.unicode.org/cldr/45/keyboard3" conformsTo="45" locale="de">
    <info author="Lars Mühlbauer for k3lp" name="German (QWERTZ)" layout="QWERTZ" indicator="DE" />
    ...
        <key id="base" layerId="base" width="1.5" />
        <key id="bksp" output="@k3:action/backspace" stretch="true" />
        <key id="enter" output="@k3:action/enter" width="1.6" />
        <key id="D" output="D" longPressKeyIds="dollar pound yen euro cent"
             longPressDefaultKeyId="euro" />
    ...
    <flicks>
        <flick id="a-flicks">
            <flickSegment directions="ne s" keyId="b" />
        </flick>
    </flicks>
    <layers formId="touch" minDeviceWidth="1">
        <import path="flex://org.florisboard.k3.foundation/layers-touch.xml" />
        <layer id="base">
            <row keys="q w e r t z u i o p" />
            <row keys="a s d f g h j k l" />
            <row keys="base-to-shift y x c v b n m bksp" />
            <row keys="symbol-1 comma fn-key space period enter" />
        </layer>
        <layer id="symbol-1">
            <row keys="1 2 3 4 5 6 7 8 9 0" />
```
（`...` 為我省略的行；其餘為逐字。）

新格式能力（讀 `ime/keyboard3/` 與上面 XML）：
- 每鍵 `width`（相對單位，如 `1.5`）、`stretch="true"`（填滿剩餘）、`gap="true"`（`keys-implied.xml` 的 `fn-key`）。**有每鍵寬度**。
- `longPressKeyIds` + `longPressDefaultKeyId`（長按 popup）、`<flicks>`（`flickSegment directions="ne s"` 方向滑動輸出；`ime/keyboard3/hint/FlickKeyHintPlacement.kt`）。
- 多 layer（`base`、`shift`、`symbol-1`、`symbol-2`、`numeric`）用 `layerId` 鍵切換；`<layers formId="touch" minDeviceWidth="1">` 依裝置寬度（mm）選不同佈局（`TouchModel.Multiple.selectKeyboard(deviceWidthMm)`）。
- `<import>` 共用片段（`layers-touch.xml`、`keys-implied.xml`、`displays-implied.xml`）。
- `output="\u{0020}"` 這類 LDML 跳脫；`<variables>` 設顯示字串。
- 仍然是 row 內矩形；沒有自訂形狀。
- 狀態：這是進行中的重寫，目前 master 只有 1 個真實 XML 鍵盤（qwertz.xml）；舊 JSON assets 仍在。k3lp 函式庫 `org.k3lp.*` 的授權：未查證。

#### 1.6 (5) 特色功能

- **滑動輸入（glide typing）演算法**（`ime/text/gestures/StatisticalGlideTypingClassifier.kt`，v0.5.2；註解寫明參考 Étienne Desticourt 在 AnySoftKeyboard PR #1870 的文章）：
  1. 取樣：`addGesturePoint` 只在與上一點距離平方 > `(第一個鍵寬 / 4)²` 時加點，`Gesture` 上限 500 點。
  2. 剪枝一（首末鍵）：`Pruner.pruneByExtremities`——建 `HashMap<Pair<首鍵, 末鍵>, List<單字>>`，只取手勢起點／終點「最近 2 個鍵」的 2×2 組合內的單字。
  3. 剪枝二（長度）：`pruneByLength`——理想路徑長度與手勢長度差 < `PRUNING_LENGTH_THRESHOLD(8.42) × 鍵半徑`（鍵半徑＝第一個鍵 min(寬,高)）。
  4. 理想路徑：`Gesture.generateIdealGestures(word)`——連接每個字母的鍵中心；連續相同字母加一個鍵內小方框迴圈（右下、右上、左上、左下），同時保留有無迴圈兩個版本（為了分 pool/poll）。帶重音字母用 NFD 取基底字元找鍵。
  5. 重採樣到 `SAMPLING_POINTS = 200` 點（`resample`，累積誤差補點）。
  6. 形狀距離：兩者各自 `normalizeByBoxSide`（以外接框最長邊正規化並置中），逐點歐氏距離和，套高斯 `SHAPE_STD = 22.08`。
  7. 位置距離：未正規化的兩路徑逐點曼哈頓距離平均，套高斯 `LOCATION_STD(0.5109) × 鍵半徑`。
  8. 頻率：`255f × getFrequencyForWord(word)`（0..255 的詞頻）。
  9. 分數：`confidence = 1 / (shapeProb × locationProb × frequency)`，**越小越好**，排序插入取前 `maxSuggestionCount`（8）；5 筆 LRU 快取避免鬆手時重算。
  `GlideTypingManager.kt`：拖動中每隔 `previewRefreshDelay` 取 1 筆做預覽；`onGlideComplete` 取 8 筆，第一筆 `commitGesture` 直接上屏，其餘送 Smartbar。
- Smartbar：`ime/smartbar/`（`Smartbar.kt`、`CandidatesRow.kt`、`quickaction/` 含 `QuickActionsRow`、`QuickActionsEditorPanel`、`QuickActionsOverflowPanel`；`ExtendedActionsPlacement`、`CandidatesDisplayMode`、`SmartbarLayout`）——快捷動作可編輯排列。
- 剪貼簿管理：歷史、pin、到期、敏感標記、同步開關、圖片（見 1.4）。
- emoji：`ime/media/emoji/`（`EmojiData`、`EmojiSuggestionProvider`、`EmojiHistory`、膚色、`FlorisEmojiCompat`）；emoji 建議當成 `SuggestionProvider` 與主預測並存；roadmap 0.7+ 要重做 emoji 面板與搜尋。
- 主題：Snygg v2；其他（Compose UI、extension 管理）略。

#### 1.7 (6) 授權

- `LICENSE`：Apache License 2.0 全文（第 1-5 行為 Apache License Version 2.0, January 2004）。各檔頭為 Apache-2.0 notice。內建 extension 的 `meta.license` 為 `apache-2.0`。
- 中文完整形碼包：另一授權，`LANGUAGEPACKS-CHINESE.md` 寫第三方授權連結 fcitx5-table-extra 的 `LICENSES/GPL-3.0-or-later.txt`；該包獨立發佈、釋出連結為 TODO，故不屬於 FlorisBoard 本體。
- 對 QIM：可以直接重用 Apache-2.0 的程式碼（保留版權與 NOTICE）；`fcitx5-table-extra` 碼表是 GPL，QIM 如要用其速成表須另評估（不屬本調查範圍；請交給碼表授權調查）。內建 `han.sqlite3` 三個表的資料來源與授權：**未查證**（extension.json 只寫 apache-2.0 與 authors waelwindows、moonbeamcelery）。

#### 1.8 (7) 值得採用 / 坑

採用：
- Provider 介面（`SuggestionProvider` + `determineLocalComposing`）把「哪段是組字」交給輸入法引擎，非常適合形碼類。
- 新 LDML 風格 XML：每鍵 `width`、`stretch`、`gap`、`flick`、`longPressKeyIds`、`minDeviceWidth` 選版面，是這批裡表達力最高的格式；而且對應 Unicode 標準，不是自創。
- 滑動輸入「首末鍵剪枝 + 長度剪枝 + 形狀/位置機率」結構簡單、可移植（純浮點幾何）。
- `ClipDescription.EXTRA_IS_SENSITIVE`（API 33）過濾剪貼簿敏感內容。

坑：
- master 半成品：glide、Han、Latin 都被關（見 1.1），不要以 master 評估功能。
- 舊 JSON 沒有每鍵寬度；寬度寫死在 `TextKey.kt`。
- Han provider 的 `code LIKE 'x%'` 是前綴查詢，沒有「速成＝首碼＋末碼」語意；`confidence` 固定 0.5。
- 詞頻 0..255 且 Latin 空殼，滑動輸入的詞表來源在 v0.5.2 只有 `data.json` 測試詞典（`getListOfWords` 的實際來源未逐行追蹤，未查證）。

---

### AnySoftKeyboard（ASK）

#### 2.1 (1) 架構

- 服務層用 mixin 繼承鏈，檔案在 `ime/app/src/main/java/com/anysoftkeyboard/ime/`：`AnySoftKeyboardBase` → … → `AnySoftKeyboardSuggestions`、`AnySoftKeyboardWithGestureTyping`、`AnySoftKeyboardClipboard`、`AnySoftKeyboardWithQuickText`、`AnySoftKeyboardIncognito`、`AnySoftKeyboardInlineSuggestions`、`AnySoftKeyboardHardware` 等。
- 候選流程（repo 內文件 `ime/app/src/main/java/com/anysoftkeyboard/dictionaries/SUGGESTIONS.md`）：打字 → `WordComposer`（含按鍵座標與鄰近鍵）→ `Suggest.getSuggestions` → `SuggestImpl.getSuggestions`（`SuggestImpl.java` 第 252 行起）：先放入「輸入原字」（優先級 `Integer.MAX_VALUE`），再依序查縮寫（abbreviations）、自動文字（auto-text）、子詞拆分（`mSplitWords`）、`SuggestionsProvider.getSuggestions`（聯絡人／使用者／主詞典），最後把「next-word 建議中以輸入前綴開頭的」插到最前面（註解：「next-word beats any suggestion OTHER than identical typed」），去重後回傳到建議列。
- `SuggestionsProvider` 管理多本 `Dictionary`（Main、User、Contacts、AutoText、Abbreviations、NextWord）。
- 輸入欄位判斷在 `onStartInputView`（見 2.3）。

#### 2.2 (2) 排序與學習；語言包 addon 機制；中文

- 詞頻 0..255（`Dictionary.java` 第 29 行 `MAX_WORD_FREQUENCY = 255`）；`SuggestImpl` 以 `mPriorities` 陣列排序，並用編輯距離（`IMEUtil.editDistance`）與 `commonalityMaxLengthDiff/MaxDistance` 控制容錯（由 `settings_key_auto_pick_suggestion_aggressiveness` 動態設定）。
- 主詞典：原生 JNI 二進位（`ime/dictionaries/jnidictionaryv1`、`jnidictionaryv2` 的 `dictionary.cpp`、`com_anysoftkeyboard_dictionaries_ResourceBinaryDictionary.cpp`）；語言包的原始檔是 `words.xml`，格式逐字如下（`addons/languages/english/pack/dictionary/prebuilt/additionals.xml`）：
```xml
<wordlist>
    <w f="48">Corona</w>
    <w f="48">Coronavirus</w>
    <w f="128">nd</w>
    <w f="30">LLM</w>
    <w f="60">delish</w>
</wordlist>
```
- 使用者學習：`UserDictionary`（SQLite，`ime/app/.../dictionaries/UserDictionary.java`、`sqlite/`）；`tryToLearnNewWord`（`SuggestImpl.java` 第 414 行）；incognito 時不學。
- 下一詞預測：`ime/nextword/`（`NextWordDictionary.java`：`MAX_NEXT_SUGGESTIONS = 8`、`MAX_NEXT_WORD_CONTAINERS = 900`，超過時隨機淘汰一個容器；`NextWordsContainer.markWordAsUsed` 記次數；`NextWordsStorage` 以檔案持久化；另有 C++ `NextWordDictionary.cpp`）。偏好 `settings_key_next_word_suggestion_aggressiveness` 決定 3/5/8 筆與最小使用次數 5/3/1。**是 bigram（前一詞 → 下一詞次數），無語言模型。**
- **語言包 addon 機制**：每個語言是獨立 APK（`addons/languages/<lang>/apk` + `pack`），`AndroidManifest.xml` 宣告 `BroadcastReceiver`，intent action `com.menny.android.anysoftkeyboard.KEYBOARD` 與 `...DICTIONARY`，`<meta-data android:name="com.menny.android.anysoftkeyboard.keyboards" android:resource="@xml/chinese_traditional_keyboards">` 指向 XML 清單（`addons/languages/chinese_traditional/pack/src/main/AndroidManifest.xml`）。主程式用 package manager 掃描這些 receiver 載入。
- **中文：`addons/languages/chinese_traditional`（2026 新增）**——README 寫明三種輸入法：Boshiamy 嘸蝦米（繁／簡／日三版）、**Cangjie 5 倉頡**、Zhuyin 注音（大千）。`grep -ri "速成\|quick"` 整個目錄無命中 → **沒有速成**。
  - 引擎 `ime/dictionaries/src/main/java/com/anysoftkeyboard/dictionaries/RadicalDictionary.java`（1529 行）：資料檔為 `radical_sequence\tcharacter`（tab 分隔，同一碼多行即多候選）；以 `CompactTrie` 存 `radicalTrie`（碼→候選）、`phraseTrie`（首字→後續詞組的下一詞建議）；`getSuggestions` 做**精確匹配**（`getExactMatches`），候選依序給頻率 `MAX_WORD_FREQUENCY, -1, -2…`（保持檔內順序）；`hasPartialMatch(radicals)` 判斷前綴是否還有可能；另有同音查詢 `getHomophones`、字→碼反查 `charToRadical`、字頻排名 `charFrequencyRank`（Cangjie/Zhuyin 用 `splitMultiCodepointCandidates="true"` 並依常用度排序）、使用者可經 SAF 匯入自己的 `liu*.tab` 覆蓋表（`RadicalOverlay`）。
  - 鍵盤宣告（`pack/src/main/res/xml/chinese_traditional_keyboards.xml`）逐字：
```xml
    <Keyboard nameResId="@string/chinese_traditional_cangjie_keyboard"
              iconResId="@drawable/ic_status_chinese_traditional"
              layoutResId="@xml/qwerty_cangjie"
              id="@string/chinese_traditional_cangjie_keyboard_id"
              defaultDictionaryLocale="zh-cangjie"
              description="@string/chinese_traditional_cangjie_keyboard_description"
              index="6" defaultEnabled="false" hasRadicalInput="true" asciiOnlyPopups="true"
              physicalKeyboardMappingResId="@xml/chinese_traditional_physical" />
```
  `hasRadicalInput`、`radicalKeyCharacters`、`radicalCandidateSelectorKeys="vrsfwlcbkj"`（嘸蝦米用 v r s f w l c b k j 選第 2-10 候選）是針對形碼的鍵盤屬性。
  - 資料來源與授權（`NOTICE.md`）：倉頡 5 來自 Unicode Unihan `kCangjie`（Unicode License）；注音 `kMandarin`；詞組 `chinese_phrases` 由 `scripts/build_phrases.py` 自 McBopomofo `phrase.occ`（MIT，原料來自 libtabe `tsi.src`，BSD）產生；OpenCC STCharacters/TSCharacters（Apache-2.0）；嘸蝦米來自社群「開放蝦米」表，因 Boshiamy 公司保有官方碼表著作權而刻意不使用官方表。`NOTICE.md` 宣稱「No GPL, LGPL, AGPL … used」。

#### 2.3 (3) 平台限制與繞法

- 欄位判斷（`SUGGESTIONS.md`「Environment, IME Callbacks, and EditorInfo Flags」）：`TYPE_CLASS_NUMBER/PHONE/DATETIME` 關預測；`TYPE_TEXT_VARIATION_PASSWORD/VISIBLE_PASSWORD/WEB_PASSWORD` 關預測；EMAIL/URI 關自動選字與自動空格；`NO_SUGGESTIONS` 旗標被尊重；`IME_FLAG_NO_PERSONALIZED_LEARNING` 或密碼欄會自動進入 incognito（`AnySoftKeyboardIncognito.java`），停止寫入使用者詞典與 next-word。
- 游標移出目前組字詞外 → `abortCorrectionAndResetPredictionState`（`onUpdateSelection`）。
- 省電模式可強制關閉滑動輸入（`settings_key_power_save_mode_gesture_control`）。
- 剪貼簿：`ime/app/src/main/java/com/anysoftkeyboard/devicespecific/ClipboardV11.java`，記憶體內 `List<CharSequence>`，`MAX_ENTRIES_INDEX = 15`（最多 16 筆）、不持久化（我只看到 `ArrayList`，沒找到寫檔）；`AnySoftKeyboardClipboard` 在「剛同步的剪貼簿項目」`MAX_TIME_TO_SHOW_SYNCED_CLIPBOARD_ENTRY = 15 秒` 內顯示提示。是否過濾 `EXTRA_IS_SENSITIVE`：未查證（grep 無命中）。

#### 2.4 (4) 鍵盤佈局格式（逐字範例）

格式：Android 舊式 `Keyboard` XML 的擴充（`<Keyboard>/<Row>/<Key>`），屬性定義在 `api/src/main/res/values/keyboard_layout_api.xml`：
- `KeyboardLayout`：`android:keyWidth`、`keyHeight`、`horizontalGap`、`verticalGap`、`showPreview`、`autoCap`。
- `KeyboardLayout_Row`：`rowEdgeFlags`、`keyboardMode`。
- `KeyboardLayout_Key`：`codes`、`popupKeyboard`、`popupCharacters`、`keyEdgeFlags`、`isModifier`、`isSticky`、`isRepeatable`、`iconPreview`、`keyOutputText`、`keyLabel`、`keyIcon`、`keyboardMode`、`longPressCode`、`isFunctional`、`isShiftAlways`、`shiftedCodes`、`shiftedKeyLabel`、`shiftedKeyOutputText`、`keyOutputTyping`、`shiftedKeyOutputTyping`、`showPreview`、`keyDynamicEmblem`（none/text/icon）、`hintLabel`、`showInLayout`（always/if_applicable/never）、`tags`、`genders`、`skinTones`、`extra_key_data`。

逐字範例 1（倉頡，`addons/languages/chinese_traditional/pack/src/main/res/xml/qwerty_cangjie.xml`，89 行）：
連結：https://github.com/AnySoftKeyboard/AnySoftKeyboard/blob/8cf81a3a2f73e9914c8613d56390a933486f0c6b/addons/languages/chinese_traditional/pack/src/main/res/xml/qwerty_cangjie.xml
```xml
<Keyboard xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:ask="http://schemas.android.com/apk/res-auto"
    android:keyWidth="10%p"
    android:keyHeight="@integer/key_normal_height"
    ask:autoCap="false">

    <Row>
        <Key android:codes="113" android:keyLabel="手"
             android:popupCharacters="1ｑㄆ" ask:hintLabel="q"
             android:keyEdgeFlags="left"/>
        <Key android:codes="119" android:keyLabel="田"
             android:popupCharacters="2ｗㄊ" ask:hintLabel="w"/>
        <Key android:codes="101" android:keyLabel="水"
             android:popupCharacters="3ｅㄍˇ" ask:hintLabel="e"/>
        <Key android:codes="114" android:keyLabel="口"
             android:popupCharacters="4ｒㄐˋ" ask:hintLabel="r"/>
        <Key android:codes="116" android:keyLabel="廿"
             android:popupCharacters="5ｔㄔ" ask:hintLabel="t"/>
        <Key android:codes="121" android:keyLabel="卜"
             android:popupCharacters="6ｙㄗˊ" ask:hintLabel="y"/>
        <Key android:codes="117" android:keyLabel="山"
             android:popupCharacters="7ｕㄧ˙" ask:hintLabel="u"/>
        <Key android:codes="105" android:keyLabel="戈"
             android:popupCharacters="8ｉㄚ" ask:hintLabel="i"/>
        <Key android:codes="111" android:keyLabel="人"
             android:popupCharacters="9ｏㄞ" ask:hintLabel="o"/>
        <Key android:codes="112" android:keyLabel="心"
             android:popupCharacters="0ｐㄢ" ask:hintLabel="p"
             android:keyEdgeFlags="right"/>
    </Row>
```
範例 2（英文，`addons/languages/english/pack/src/main/res/xml/qwerty.xml`）展示 `horizontalGap` 與 popup：
```xml
    <Row>
        <Key android:codes="113" android:popupCharacters="1" android:keyEdgeFlags="left"/>
        <Key android:codes="119" android:popupCharacters="2ŵω"/>
        <Key android:codes="101" android:popupKeyboard="@xml/popup_qwerty_e"/>
    ...
    <Row>
        <Key android:horizontalGap="5%p" android:codes="97" android:keyLabel="a" android:popupCharacters="àáâǎãāäåæąăαª"
             android:keyEdgeFlags="left"/>
```
此倉頡檔頭註解重點（逐字摘）：key 顯示倉頡字根為主標籤、拉丁字母為角標（`ask:hintLabel`）；`autoCap="false"` 因為單字元標籤會被 shift 改成大寫字母；倉頡沒有 shift 鍵，空出寬度給字根鍵。

能力：
- 按鍵大小／位置：`keyWidth`/`keyHeight`/`horizontalGap`/`verticalGap`，可用 `%p`（父寬百分比）。**每鍵可不同寬度**（`android:keyWidth` 可寫在 `<Key>`）；位置以 row 內順序 + gap。仍是矩形，無自訂形狀。
- 多頁：由多個 `Keyboard` 檔 + `keyboardMode`、key code（模式切換）處理；popup 可為完整子鍵盤 `popupKeyboard`。
- 長按：`popupCharacters`（字串，每字一鍵）或 `popupKeyboard`；`longPressCode` 指定長按輸出。
- 滑動：鍵上無每鍵滑動屬性；手勢在 `AnySoftKeyboardSwipeListener` 全域設定。
- 多語言：每語言一個 addon APK。
- 鍵盤清單／字典清單 XML：`<Keyboards><Keyboard layoutResId=… defaultDictionaryLocale=… physicalKeyboardMappingResId=…/>`、`<Dictionaries><Dictionary radicalDictionaryResId=… locale=…/>`（見 2.2）。

#### 2.5 (5) 特色功能；滑動輸入演算法（`ime/app/src/main/java/com/anysoftkeyboard/gesturetyping/GestureTypingDetector.java`，788 行）

1. 取點：`addPoint(x,y)` 最小點距 `mMinPointDistanceSquared`。
2. **角點抽取**：`getPathCorners` 保留起點、終點與「曲率夠大」的點；`hasEnoughCurvature`：取前後各 `CURVATURE_NEIGHBORHOOD = 1` 個點，夾角 < `CURVATURE_THRESHOLD = 170°` 才算角（字母轉折點）。
3. **詞的路徑預先產生**：`generateCornersInBackground` 在背景把詞典每個單字轉成角點路徑（`mWordsCorners`，`short[]`），依首字母鍵分組 `mWordsByStartKey`（用 Structure-of-Arrays 的 `CompactWordList` 省記憶體：註解說約 32 → 12 bytes/word）。
4. **候選過濾**（`getCandidates`，第 457 行起）：起點鍵不是正確鍵時，若距離平方 > `mStartKeyProximityThresholdSquared`（約 1.5 鍵寬）略過；否則加起點懲罰 `distanceSquared × 0.000667`（`PROXIMITY_PENALTY_FACTOR`）；終點鍵距離平方 > 起點門檻 × `END_DIST_THRESHOLD_MULTIPLIER = 25` 略過，否則加終點懲罰 `× 0.000333`。
5. **路徑距離**（`calculateDistanceBetweenUserPathAndWord`，第 595 行起）：沿著使用者角點，累加到詞路徑「目前或下一個角點」較近者的距離；乘上方向懲罰 `1 + DIRECTION_PENALTY_FACTOR(1.0) × (1 − cosθ)`（同向 ×1、垂直 ×2、反向 ×3）；詞路徑角點數比手勢角點數多就直接排除；超過 fail-fast 門檻（目前第 N 名的分數與 `MINIMUM_DISTANCE_FILTER = 1e6` 取小）提早中止。
6. **分數**：`finalWeight = distance − mFrequencyFactor × 詞頻 + 起點懲罰 + 終點懲罰`，越小越好；插入排序取 `mMaxSuggestions`。
- 其他特色：quick text（`AnySoftKeyboardWithQuickText`、`ime/addons/quick_text`、`addons/quicktexts`）、emoji（`emojis/`）、主題 addon（`addons/themes`、`ime/addons/keyboard_themes`）、聯絡人詞典、標籤搜尋（`AnySoftKeyboardKeyboardTagsSearcher`：輸入 `:` 搜 emoji 標籤）、媒體插入（`AnySoftKeyboardMediaInsertion`）、JS/TS 工具（repo 有 `package.json`，用途未查證）。
- 同步：未見雲端同步（未查證；僅有偏好備份功能未讀）。

#### 2.6 (6) 授權

- 根目錄 `LICENSE`：Apache License 2.0。chinese_traditional 的程式碼同 Apache-2.0，資料各自授權（`NOTICE.md` 表：Unicode License、MIT、BSD、Apache-2.0、開放蝦米「unconditional use」）。
- 對 QIM：Apache-2.0，可重用 `GestureTypingDetector` 的演算法概念與格式設計；`CompactTrie` + `RadicalDictionary` 的「碼→候選精確匹配 + 前綴判斷」可直接參考；McBopomofo `phrase.occ` 是 MIT，可作聯想詞來源（但仍需確認資料授權，詳見該 NOTICE）。

#### 2.7 (7) 值得採用 / 坑

採用：
- 形碼引擎的資料格式 `code<TAB>char`、Trie、`hasPartialMatch`、SAF overlay 讓使用者匯入自己的表——對速成這種「一碼多字、需字頻排序」很貼近。
- 鍵盤屬性 `hasRadicalInput` / `radicalCandidateSelectorKeys`（以字根鍵選第 N 候選）值得參考。
- 滑動輸入的方向懲罰與起末鍵軟懲罰比 Floris 版多了方向資訊。
- 詞組表 `首字 → 後續`（`phraseTrie`）當作聯想詞，簡單有效。

坑：
- 沒有速成：倉頡 5 是完整碼，若要「首碼＋末碼」要自己加。
- 候選排序在 Radical 引擎只是檔案順序／字頻 rank，沒有使用者學習（未見 radical 候選的個人化學習）。
- 剪貼簿只存記憶體 16 筆。
- 手勢 + 形碼是否能共存：`anyActiveDictionarySupportsGestureTyping()` 存在，細節未查證。
- 佈局是 Android `Keyboard` XML，跨平台不可直接用，只能借欄位語意。

---

### HeliBoard 與 OpenBoard（AOSP LatinIME 系）

#### 3.1 (1) 架構

- HeliBoard（`app/src/main/java/helium314/keyboard/`）：`latin/LatinIME.java`（服務）→ `latin/inputlogic/InputLogic.java`（輸入邏輯）→ `latin/WordComposer.java` → `latin/Suggest.kt` → `latin/DictionaryFacilitatorImpl.kt`（多本詞典）→ JNI `app/src/main/jni/`（`com_android_inputmethod_latin_BinaryDictionary.cpp`、`src/suggest/`、`src/dictionary/`）→ 回傳 `SuggestedWords` → `latin/suggestions` 的建議列。
- `Suggest.getSuggestedWords`（`Suggest.kt` 第 60 行）分流：一般輸入 `getSuggestedWordsForNonBatchInput`（第 73 行）、滑動輸入 `getSuggestedWordsForBatchInput`（第 270 行，呼叫 `mDictionaryFacilitator.getSuggestionResults(...)`）。
- 詞典組合（`DictionaryFacilitatorImpl.kt` 第 151-155 行）：`TYPE_MAIN`（內建 `.dict`）、`TYPE_USER`、`TYPE_USER_HISTORY`（個人化開啟時）、`TYPE_CONTACTS`。`app/src/main/assets/dicts/*.dict` 為內建二進位詞典（如 `main_en-US.dict`、`main_de.dict`）；外部詞典在獨立 repo（`assets/dictionaries_in_dict_repo.txt`）。
- 鍵盤幾何 → 原生：`com_android_inputmethod_keyboard_ProximityInfo.cpp` 與 `src/suggest/core/layout/`（`proximity_info.cpp`、`proximity_info_state.cpp`、`touch_position_correction_utils.h`、`additional_proximity_chars.cpp`、`normal_distribution_2d.h`）——每鍵幾何傳進原生做觸點機率與鄰近鍵修正。
- OpenBoard 結構相同（`org.dslul.openboard.inputmethod.*`，XML 鍵盤在 `app/src/main/res/xml/`），最後 commit 2022-12-17，預設分支實質停更；功能以 HeliBoard 為準，不另外詳述。

#### 3.2 (2) 排序與學習

- 原生打字評分參數（`app/src/main/jni/src/suggest/policyimpl/typing/scoring_params.cpp`，逐字摘）：
```cpp
const float ScoringParams::PROXIMITY_COST = 0.0694f;
const float ScoringParams::OMISSION_COST = 0.467f;
const float ScoringParams::INSERTION_COST = 0.7248f;
const float ScoringParams::TRANSPOSITION_COST = 0.5608f;
const float ScoringParams::SUBSTITUTION_COST = 0.3806f;
const float ScoringParams::SPACE_SUBSTITUTION_COST = 0.33f;
const float ScoringParams::DISTANCE_WEIGHT_LANGUAGE = 1.1214f;
const float ScoringParams::COST_FIRST_COMPLETION = 0.4836f;
const float ScoringParams::COST_COMPLETION = 0.00624f;
const float ScoringParams::AUTOCORRECT_OUTPUT_THRESHOLD = 1.0f;
```
  容錯：一次在 Trie（詞典）上做 best-first 搜尋，節點成本＝空間距離（觸點到各鍵的機率）× `DISTANCE_WEIGHT_LENGTH` + 語言機率（unigram/bigram 機率）× `DISTANCE_WEIGHT_LANGUAGE`，加上漏打／多打／換位／替代／相鄰鍵等成本。我只讀了參數表與檔案結構，各成本在 `typing_weighting.cpp` 的精確用法：未逐行查證。
- Kotlin 層後處理（`Suggest.kt`）：自動更正門檻 `mScoreLimitForAutocorrect`；打字詞有效且分數高時比較「第一建議 vs 打字詞」並要求與空前文情境下的分數差 ≥ 20（第 254-256 行）；若第二名分數 > 0.94 × 第一名且長度 > 1，則把第一名分數乘 0.93 重排（`Suggest.kt` 第 521-538 行）；`SUPPRESS_SUGGEST_THRESHOLD` 以下不顯示；下一詞建議只收 `mScore >= 170`（第 551 行）。
- **n-gram**：`NgramContext`（`latin/NgramContext.java`，前文詞串，`BEGINNING_OF_SENTENCE`）；詞典格式有 bigram（見 3.3）。
- **使用者歷史**：`latin/dictionary/history/UserHistoryDictionary.java` + `UserHistoryDictUtils.kt`，是可寫的原生 v4 詞典（`ExpandableBinaryDictionary`），隨輸入累加 unigram/bigram 並隨時間衰減（衰減細節：未逐行查證）；`InputAttributes.mNoLearning` 對應 `IME_FLAG_NO_PERSONALIZED_LEARNING`（第 135 行）。
- 沒有神經語言模型／LLM（未見；僅 n-gram）。

#### 3.3 詞典格式（binary dict）

`app/src/main/java/helium314/keyboard/latin/makedict/FormatSpec.java` 文件註解：檔案頭 `MAGIC_NUMBER = 0x9BC13AFE`（第 153 行）+ 版本 2 bytes；屬性列表（key/value 字串）；之後是 PtNode array（Patricia trie）：每個 PtNode 有旗標（子位址類型、`FLAG_HAS_MULTIPLE_CHARS`、`FLAG_IS_TERMINAL`、`FLAG_HAS_SHORTCUT_TARGETS`、`FLAG_HAS_BIGRAMS`、`FLAG_IS_NOT_A_WORD`、`FLAG_IS_POSSIBLY_OFFENSIVE`）、字元、**詞頻 1 byte（0..255）**、子節點位址、shortcut 列表、**bigram 位址列表（每項帶 4-bit 頻率 + 位址）**。版本常數：`VERSION2 = 2`、`VERSION201`、`VERSION202`、`VERSION402`、`VERSION403`（`VERSION4 = VERSION403`，可寫的使用者詞典）。轉換工具 `XmlDictInputOutput.java`、`CombinedInputOutput.kt`（combined 文字格式）。字元編碼可涵蓋全 Unicode（3-byte 形式）。

#### 3.4 (3) 平台限制與繞法；滑動輸入

- 欄位：`latin/InputAttributes.java` 第 59、96 行 `mIsPasswordField` → `mShouldShowSuggestions = !mIsPasswordField && !flagNoSuggestions`；`mNoLearning`（第 135 行）。
- **滑動輸入原生實作閉源**：`app/src/main/jni/src/suggest/policyimpl/gesture/gesture_suggest_policy_factory.cpp` 只有 `const SuggestPolicy *(*GestureSuggestPolicyFactory::sGestureSuggestFactoryMethod)() = 0;`（空函式指標，開源版沒有實作）。`latin/utils/JniUtils.java`：嘗試載入使用者自行放入 `files/libjni_latinime.so` 的外部函式庫（對 4 種 ABI 檢查 SHA-256 checksum；符合才 `System.load` 並設 `sHaveGestureLib = true`），常數 `JNI_LIB_NAME_GOOGLE = "jni_latinimegoogle"`。`layouts.md` 也寫「If you use an external glide typing library…」。→ HeliBoard 本身 **不含可重用的滑動演算法**；OpenBoard 的 README 也禁止引入 Google 閉源庫（README 第 74 行）。
- 剪貼簿：`latin/ClipboardHistoryManager.kt` + `latin/database/ClipboardDao.kt`（SQLite）+ `ClipboardContentProvider : FileProvider`；歷史持久化。是否過濾敏感項目：未查證。

#### 3.5 (4) 鍵盤佈局格式（逐字範例）

HeliBoard 有兩種自訂格式（`layouts.md`）與 AOSP 原生 XML：

**A. simple 文字格式**（`app/src/main/assets/layouts/main/qwerty.txt`，逐字前 31 行）：每行一個鍵，空行換 row。`layouts.md`：「Key format: [label] [popup keys], all separated by space, e.g. `a 0 + *`」。
```
q
w
e
r
t
y
u
i
o
p

a
s
d
f
g
h
j
k
l

z
x
c
v
b
n
m
```

**B. JSON 格式**（沿用 FlorisBoard 舊格式，lenient parsing，`//` 開頭行當註解；`app/src/main/assets/layouts/main/azerty.json`）
連結：https://github.com/Helium314/HeliBoard/blob/44ddb1ae06203857c490da75f9fa281a4e8b89cd/app/src/main/assets/layouts/main/azerty.json
```json
[
  [
    { "label": "a" },
    { "label": "z" },
    { "label": "e" },
    { "label": "r" },
    { "label": "t" },
    { "label": "y" },
    { "label": "u" },
    { "label": "i" },
    { "label": "o" },
    { "label": "p" }
  ],
  [
    { "label": "q" },
    { "label": "s" },
    { "label": "d" },
    { "label": "f" },
    { "label": "g" },
    { "label": "h" },
    { "label": "j" },
    { "label": "k" },
    { "label": "l" },
    { "label": "m" }
  ],
  [
    { "label": "w" },
    { "label": "x" },
    { "label": "c" },
    { "label": "v" },
    { "label": "b" },
    { "label": "n" },
    { "$": "shift_state_selector",
      "shiftedManual": { "label": "?" },
      "default": { "label": "'" }
    }
  ]
]
```
`layouts.md` 逐條能力：
- key 屬性：`type`（normal/function/space/action/unspecified/placeholder/numeric）、`code`、`codePoints`（`multi_text_key`）、`label`（特殊字串如 `$$$` 本地貨幣、`shift`、`delete`、`space`、`symbol_alpha`、`emoji`、`language_switch`、`action`；`label|text` 可分離顯示與輸出）、`groupId`、`popup`（`main`/`relevant`）、**`width`**（螢幕寬度比例，`-1` 表示填滿剩餘，一排總和 > 1 會等比縮放）、`labelFlags`（位元旗標，見 `res/values/attrs.xml`）。
- 選擇器：`case_selector`、`shift_state_selector`、`variation_selector`（datetime/time/date/password/normal/uri/email）、`keyboard_state_selector`、`layout_direction_selector`；`kana_selector`、`char_width_selector` 不支援。
- 功能鍵可由 `functional` 佈局客製（placeholder 鍵分隔左右群與上下列）；`more_symbols`、`symbols`、`number_row`、`numpad`、`numpad_landscape`、`phone`、`phone_symbols`、`dpad`、`emoji_bottom`、`clipboard_bottom` 等目錄＝多頁。
- 語系資料 `assets/locale_key_texts/<lang>`：`[popup_keys]`、`[labels]`、`[number_row]`、`[extra_keys]`。
- 中文：`assets/layouts/main` 共 82 個檔，**無任何中文佈局**（有 korean*.json；我用 `ls | grep` 未找到 chinese/zh/cangjie/bopomofo/pinyin）。

**C. AOSP XML（OpenBoard 逐字，`app/src/main/res/xml/rows_qwerty.xml`）**
連結：https://github.com/openboard-team/openboard/blob/c3772cd56e770975ea5570db903f93b199de8b32/app/src/main/res/xml/rows_qwerty.xml
```xml
<merge
    xmlns:latin="http://schemas.android.com/apk/res-auto"
>
    <include
        latin:keyboardLayout="@xml/key_styles_common" />
    <include
        latin:keyboardLayout="@xml/row_optional_number_row" />
    <Row
        latin:keyWidth="10%p"
    >
        <include
            latin:keyboardLayout="@xml/rowkeys_qwerty1" />
    </Row>
    <Row
        latin:keyWidth="10%p"
    >
        <include
            latin:keyboardLayout="@xml/rowkeys_qwerty2"
            latin:keyXPos="5%p" />
    </Row>
    <Row
        latin:keyWidth="10%p"
    >
        <Key
            latin:keyStyle="shiftKeyStyle"
            latin:keyWidth="15%p"
            latin:visualInsetsRight="1%p" />
        <include
            latin:keyboardLayout="@xml/rowkeys_qwerty3" />
        <Key
            latin:keyStyle="deleteKeyStyle"
            latin:keyWidth="fillRight"
            latin:visualInsetsLeft="1%p" />
    </Row>
    <include
        latin:keyboardLayout="@xml/row_qwerty4" />
</merge>
```
能力：`keyXPos`（絕對 X 位置）、`keyWidth`（`%p`、`fillRight`）、`visualInsetsLeft/Right`（可見範圍與觸控範圍分離）、`keyStyle`（可重用樣式）、`include`（片段組合）。HeliBoard `res/values/attrs.xml` 的 Key 屬性含 `altCode`、`maxPopupKeysColumn`、`keyActionFlags`、`keySpec`、`keyHintLabel`、`keyLabelFlags`。

**能否自訂**（HeliBoard / OpenBoard）：每鍵寬度：JSON `width`、XML `keyWidth`（可）；位置：XML `keyXPos`；形狀：不可（矩形；`visualInsets` 只調視覺縮進）；多頁：可；長按 popup：可（popup 欄、`!autoColumnOrder!` 等特殊標籤）；每鍵滑動：HeliBoard 有 flick/鍵上滑動？**未查證**（layouts.md 沒寫）；多語言：subtype + `method.xml` 註冊（`KeyboardLayoutSet` extra value）。

#### 3.6 (5) 特色功能

- 剪貼簿歷史、emoji、數字列、numpad/dpad、自訂功能鍵列、使用者詞典匯入、主題；個人化（user history）可關。
- 詞庫管理：外部 `.dict` 檔（`assets/known_dict_hashes.txt` 驗證）、`dictionaries_in_dict_repo.txt` 列可下載詞典、`makedict` 可轉 XML ↔ binary。
- 拼字檢查服務：`latin/spellcheck/`（Android SpellCheckerService）。
- 英文候選與聯想：原生 bigram + user history（見 3.2）。
- 滑動輸入：需外掛閉源函式庫（見 3.4）。

#### 3.7 (6) 授權（實際 LICENSE）

- HeliBoard：`LICENSE` 為 GNU GPL v3；`README.md` 第 114-123 行寫「licensed under GNU General Public License v3.0」，因基於 Apache-2.0 的 AOSP Keyboard，另附 `LICENSE-Apache-2.0`；icon 為 CC BY-SA 4.0。源碼檔頭如 `JniUtils.java`：`SPDX-License-Identifier: Apache-2.0 AND GPL-3.0-only`（含 AOSP 修改過的檔），`ClipboardHistoryManager.kt`：`SPDX-License-Identifier: GPL-3.0-only`（HeliBoard 新增檔）。
- OpenBoard：`LICENSE` 為 GPL v3；README「licensed under GNU General Public License v3.0」。
- **對 QIM 的影響**：直接複製／改寫 HeliBoard/OpenBoard 新增部分的程式碼會使 QIM（含 Android 殼）須整體以 GPL-3.0 釋出，且與 iOS App Store 發佈有已知相容疑慮（這是一般性的 GPL 與 App Store 條款問題，本調查未查證，請交授權調查）。可乾淨取用的只有 AOSP 原始（Apache-2.0）部分，且檔頭需逐檔確認。只參考「佈局格式的欄位設計」（概念）不構成重用程式碼；但 HeliBoard 的 JSON 格式本身來自 FlorisBoard（Apache-2.0），要相容該格式應直接以 FlorisBoard 為準。

#### 3.8 (7) 值得採用 / 坑

採用：
- 原生 Trie 詞典 + 觸點機率 ProximityInfo + n-gram + user history 的整體分層（Main / User / UserHistory / Contacts）值得當英文候選的架構參考。
- `layouts.md` 的 simple 文字格式（一鍵一行、空行換列、`label popup…`）極簡，適合 QIM 的「快速自訂佈局」使用者面向匯入格式（概念參考）。
- `width` 比例 + `-1` 填滿 + 一排總和 > 1 自動縮放，是好用的寬度語意。
- `placeholder` 型別鍵可做間隔，避免專用 gap 欄位。

坑：
- 滑動輸入閉源、需使用者自備外部 .so（含 checksum 檢查），不可作為 QIM 的參考實作。
- GPL-3.0 傳染。
- 沒有中文輸入、組字（composing）是為拉丁文設計的；原生 JNI 詞典不適合形碼。
- OpenBoard 已停更（預設分支最後 commit 2022-12-17）。

---

### Simple Keyboard（rkkr）

- commit a053c4ca3b61182fc407096ff207d579e5471944；LICENSE 為 Apache-2.0。README（`README.md`）明列：Features 有「Adjustable keyboard height、Number row、Swipe space to move pointer、Delete swipe、Custom theme colors、Minimal permissions (only Vibrate)」；「Feature it doesn't have and probably will never have: Emojis, GIFs, Spell checker, Swipe typing」。
- 程式是 AOSP LatinIME 縮減版（`app/src/main/java/rkr/simplekeyboard/inputmethod/latin/` 只有 `LatinIME.java`、`inputlogic/`、`InputView.java` …；沒有 `dictionary`、`suggest` 目錄），佈局是 `app/src/main/res/xml/kbd_*.xml`（AOSP XML 格式，同 3.5 C）。
- 能力與 OpenBoard 相同的 XML 格式；沒有自訂形狀。對 QIM 的價值：只有「極簡鍵盤」的參考，沒有預測／滑動可學。

---


### FlorisBoard、ASK、HeliBoard 未查證清單

- FlorisBoard：popup mapping extension 的完整 manifest；`fldic` 格式（兩個 checkout 均無命中）；`org.k3lp.*` 授權；`han.sqlite3` 資料來源授權；v0.5.2 Latin 詞表如何餵給滑動輸入；密碼欄是否關閉學習。
- ASK：剪貼簿是否過濾 sensitive；偏好備份／同步；`anyActiveDictionarySupportsGestureTyping` 與 radical 詞典的互動。
- HeliBoard：鍵上 flick／滑動屬性；剪貼簿 sensitive 過濾；user history 衰減細節；`typing_weighting.cpp` 各成本精確用法；Google 閉源滑動函式庫的演算法內容（不可得）。
- 以上授權對 App Store 發佈的法律影響。

## 4. 佈局格式比較

逐字範例在上方各對象節（FlorisBoard、ASK、HeliBoard / OpenBoard、Trime、Hamster、qingjian、fcitx5-android、Yuyan）。

| 專案 | 形式 | 每鍵寬度 | 絕對位置 | 形狀 | 多頁 | 長按 popup | 每鍵滑動 | 使用者可編輯檔 |
|---|---|---|---|---|---|---|---|---|
| FlorisBoard master（k3lp / LDML keyboard3 XML） | 資料檔 | `width`、`stretch`、`gap` | 無（row 內順序） | 矩形 | `layer` + `layerId` 鍵；`minDeviceWidth` 依裝置寬度選版面 | `longPressKeyIds` + `longPressDefaultKeyId` | `<flicks>` 方向輸出 | 是（`<import>` 共用片段）；目前僅 1 個真實鍵盤 |
| FlorisBoard v0.5.2（舊 JSON） | 資料檔 | 無（寫死在 `TextKey.kt` 依 keycode） | 無 | 矩形 | 多檔 + `modifier` | `popup.main/relevant` + 語系 popupMappings | 僅全域 `SwipeAction` | 是 |
| HeliBoard JSON / simple 文字 | 資料檔 | `width`（`-1` 填滿，總和 >1 等比縮） | 無 | 矩形 | 多目錄（symbols、numpad…） | `popup` + `locale_key_texts` | 未查證 | 是 |
| AOSP / OpenBoard XML | 資料檔 | `keyWidth`（`%p`、`fillRight`） | `keyXPos` | 矩形；`visualInsets` 縮視覺範圍 | include / 多檔 | `popupCharacters`、`popupKeyboard` | 無 | 編進 APK |
| AnySoftKeyboard XML | 資料檔（語言包 APK） | `keyWidth` 可寫在 `<Key>` | `horizontalGap` | 矩形 | 多檔 + `keyboardMode` | `popupCharacters` / `popupKeyboard` / `longPressCode` | 無（全域手勢） | 編進 addon APK |
| Trime `trime.yaml` | 資料檔 | `width`（%）、`height`、`round_corner` | 無（扁平 `keys` 自動折行） | 圓角矩形 | 具名切換鍵 + 液態鍵盤 | `long_click`、`popup[]` | `swipe_up/down/left/right` | 是，有 JSON Schema |
| Hamster `hamster.yaml` | 資料檔 | `width: percentage/available/points…` | 無 | 矩形 | `keyboardType(...)` 切換 | 程式內建，yaml 無欄位 | `swipe[]` 四向 + `display` 角標 | 是（沒有文件） |
| qingjian Android | Kotlin 表 | `Key.weight` | 無 | 矩形（自繪） | Letter / Number / Symbol 頁 | `choices` | `flick`、空格滑游標、滑行輸入 | 否 |
| fcitx5-android | Kotlin 常數 | `percentWidth` | 無 | 矩形 | Text / Number / Picker | `PopupPreset` 資料 | `Swipe` behavior | 否 |
| Yuyan | Kotlin keycode 陣列 | 比例座標 `SoftKey`（內部可任意） | 內部比例 | 矩形 | 是 | 是 | 上滑符號 | 否 |

要點：(1) 全部是矩形，QIM 若要非矩形必須自訂；(2) 對速成最實用的是「主標籤＝字根、角標＝拉丁字母」（ASK 的 `qwerty_cangjie.xml` 與 Hamster 的倉頡 yaml 都這樣做，ASK 另有 `hasRadicalInput`、`radicalCandidateSelectorKeys`）；(3) Trime 的 `composing`（組詞中該鍵行為不同）是輸入法特有需求，其他格式沒有。

## 5. 值得採用與要避開

### 值得採用

- **核心與殼分離，核心做完整排序鏈，殼只翻譯事件**（qingjian；librime 前端「單例 + 每 context 一個 session」；fcitx5 的 Frontend / Engine / UI 分離）。
- **碼表格式與排序**（libime）：`KeyCode / Length / [Data]` 純文字，編譯成 system 唯讀 + user 可寫兩檔；value＝檔案順序讓表作者控制預設順序；使用者選過的詞 index 遞增＝最近優先；`OrderPolicy` 分 `No / Fast / Freq`。`UserLanguageModel` 的插值公式 `score = max(lm, log10((1-w)·10^lm + w·10^history))`，w=0.2，不低於純 LM。
- **使用者學習**：近因（libime）、次數取對數並封頂避免高頻字壓過整詞、牛頓冷卻時間衰減（qingjian）、HistoryBigram 多池衰減（libime）、刪除墓碑（afm-ime）。
- **LLM 非阻塞模式**（afm-ime）：詞庫候選即時出；停頓後異步呼叫；以 generation 計數 + 輸入快照丟棄過期結果；只放第 2 位；逐級回退（端側 → 雲端 → 純詞庫）；只要求模型輸出序號。雲端外送上文要預設關並明示（qingjian）。
- **iOS**：主 App 部署、App Group 共享、擴充只載入；沒有 Full Access 時降級（Hamster 實證 App Group UserDefaults 會寫入失敗）；核心用 mmap 唯讀碼表保持小記憶體。
- **佈局格式**：以 FlorisBoard LDML keyboard3（每鍵 `width` / `stretch` / `gap` / `flick` / `longPressKeyIds` / `minDeviceWidth` / `import`，且是 Unicode 標準）為骨架，加 Trime 的 `composing` 與 JSON Schema 驗證，加 Hamster 的 `swipe[].display` 角標；HeliBoard 的 simple 文字格式適合做簡易匯入。
- **滑動輸入**：FlorisBoard v0.5.2 或 ASK（Apache-2.0）。對速成需把「詞」換成「碼序列」（推論，未驗證）。
- **剪貼簿**：FlorisBoard（持久化、pin、到期、`EXTRA_IS_SENSITIVE` 過濾、同步開關）。
- **Android 欄位處理**：密碼欄關預測；`IME_FLAG_NO_PERSONALIZED_LEARNING` 進 incognito（ASK、qingjian、HeliBoard 一致）。

### 要避開的坑

- **授權**：GPL 專案只能讀不能搬；LGPL 靜態連進 iOS App 有疑慮；afm-ime 無授權；fcitx5-table-extra 的碼表資料授權不明；Yuyan 引擎閉源且綁包名。
- **FlorisBoard master 是重寫中的半成品**（glide、Han、Latin 都被關）；功能評估看 v0.5.2。
- **LLM 分數跨讀法不可比**：qingjian 踩過，P2C 分數只在同一串按鍵內有意義，只決定名次。
- **iOS 無 marked text、記憶體上限未公開**：組字碼只能畫在鍵盤自己的候選列；需自行實測，目標抓 ≲30–40 MB 常駐（建議，非官方）。
- **佈局寫死在程式碼**（fcitx5-android、Yuyan、qingjian）無法讓社群換速成佈局。
- **libime table 學習只調 index、不記次數**；若要真正頻率排序要自己加計數。
- **UserDefaults 存詞頻不可擴張**（afm-ime 上限 5000 條）。
- **評測偏差**：qingjian 作者踩過「回放日誌偏向基線」；用乾淨集評。
- **IMK 實測坑**（afm-ime / fcitx5-macos）：預設只投遞 keyDown；Electron 不轉發修飾鍵；非啟用面板必須 `canBecomeKey=false`；Terminal / iTerm / VSCode 沒有 client preedit 會雙重處理 Backspace（fcitx5-macos 用 dummy preedit）；secure input 被 app 濫用；ad-hoc 簽名每次 cdhash 變會令輸入監控授權失效。
- **Android 10+ 背景不可讀剪貼簿**：只在 IME 有焦點時監聽；FlorisBoard 另用透明 Activity 取前景。

## 6. 建議補充的功能（成熟輸入法常見，QIM 可考慮）

每項附出處（已在上文查證者）。

| 功能 | 理由 | 出處 |
|---|---|---|
| 反查 / 字根提示（Prompt） | 速成使用者想知道某字怎麼打、候選旁顯示剩餘碼 | libime `Prompt=&`、`candidateHint`；fcitx5-table-extra `quick5.txt` 的 `&a 日` |
| 萬用鍵 `*`（MatchingKey） | 速成忘了第二碼時可用 | fcitx5-table-extra `quick5.conf.in` `MatchingKey=*` |
| 自動造詞（auto phrase，MRU，升格為 user 詞） | 連打常用詞組後自動成詞 | libime `autophrasedict.h`、`TableContext::learnAutoPhrase` |
| 使用者詞刪除 / 墓碑 | 刪掉誤學詞且同步不復活 | libime `deletionTrie_`；afm-ime 刪除墓碑 |
| 時間衰減的個人頻率 | 習慣會變 | qingjian `frequency_learner/decay.rs`；libime HistoryBigram 多池 |
| 聯想（下一詞） | 減少打字 | ASK `NextWordDictionary`（bigram）與 `phraseTrie`；Trime `librime-predict` |
| 敲錯 / 容錯表 | 速成碼短，誤觸鄰鍵代價高 | qingjian `user-typos.tsv`；libime `PinyinFuzzyFlag.Correction`（鄰鍵誤觸，僅拼音）；HeliBoard ProximityInfo |
| 滑動輸入（glide typing） | 手機主流功能 | FlorisBoard v0.5.2、ASK |
| 空格滑動移游標、刪除鍵滑動 | 手機編輯效率 | Hamster `spaceDragSensitivity`；fcitx5-android `spaceSwipeMoveCursor`；Simple Keyboard README |
| 剪貼簿歷史（pin、到期、敏感過濾、可存圖） | 常用 | FlorisBoard、qingjian、Trime（Room DB + pinned） |
| 常用語 / 快捷文字 | 地址、簽名等 | Trime `CollectionHelper`；qingjian `phrase_book`；ASK quick text |
| 無痕 / incognito | 隱私，尊重 `NO_PERSONALIZED_LEARNING` | ASK `AnySoftKeyboardIncognito`；FlorisBoard `IncognitoMode`；HeliBoard `mNoLearning` |
| 簡繁轉換 | 速成使用者多為繁體，需簡體輸出時 | Hamster `ShortcutCommand`（簡繁切換）；Trime 內建 OpenCC（`opencc.cc`） |
| 同步（資料夾式同步目錄 / iCloud） | 換機、多裝置 | Hamster `sync_dir` 指 iCloud Documents；Trime `BackgroundSyncWork`（均為 librime `sync_user_data`） |
| 英文候選與中英混輸 | 常見需求 | qingjian `english` 模組；HeliBoard unigram/bigram |
| 單手 / 左右手 / 懸浮鍵盤 / 數字行 | 大螢幕可用性 | Yuyan README / `keyboard/`；Hamster 單手快捷指令；HeliBoard number row |
| 候選展開面板（BulkCandidateList） | 同碼多字時一次看全部 | fcitx5 `BulkCandidateList`（"useful for virtual keyboard"）；Trime `unrolled` |
| 多頁符號 / emoji（含搜尋） | 基本需求 | Trime liquid_keyboard；ASK 標籤搜尋 `:`；FlorisBoard emoji |
| 外部詞庫 / 碼表匯入（使用者自帶表） | 授權乾淨且彈性 | ASK `RadicalOverlay`（SAF 匯入）；qingjian 可匯入 TSV / Rime 檔；libime `libime_tabledict` 轉換工具 |
| 候選旁譯詞 / 學習語言輔助 | 差異化賣點 | qingjian（釋義表離線 LLM 生成後隨包） |
| 語音輸入 | Android 可用、iOS 擴充不可 | qingjian `qingjian-voice`；Apple 文件：鍵盤擴充無麥克風 |

## 7. 來源

第一手來源（commit 為調查時 HEAD）：

- Bemly/afm-ime a8543a1eac50ffb2e66f00eca1a047ebf3ba15f0（`Sources/IMECore/FMReranker.swift`、`Package.swift`、`AGENTS.md`）
- qingjian-team/qingjian 3d5fd2df431cea92eba39d08cd94a8a4093bb872
- scukeqi/Wisdom-Weasel 64ba2fdd484c3fa1c1a87e88591911c3d31b3eb6
- imfuxiao/Hamster 65693706d01fc6c19ed6071968e542b0a2ef3f36
- osfans/trime ed54b8d95a802147129b6981ce79bf9b5937b361
- rime/squirrel 46962d2efae5be70389c778d0a6c52f669072827；rime/weasel 82b50c9e9b9ee0bd31357db6f25ddc804546c7f7
- fcitx/libime 5be1300ea00e899b3f2c46d1e9f6bca21139cdeb；fcitx/fcitx5 145e981d6377550ffb39ee8de61452910622652c；fcitx/fcitx5-table-extra dbc7154a7f0b9fc04313160ae8066ed4d8cbc446；fcitx/fcitx5-chinese-addons e8d930f402e359cfd76a101ee3e46815d1331d83
- fcitx5-android/fcitx5-android eb5b50216e70484a009933d9519db76005f1faf3；fcitx-contrib/fcitx5-macos 3466612499c149d5002b1aab37b92bb257da1a75
- gurecn/YuyanIme 75bd0aebb7c313858437195539445b82021904d1；gurecn/yuyansdk c99f9b7c451b7b77d379dab7d55fdbfb93b04b12
- KeyboardKit/KeyboardKit b9cb190e0405fa694906a5002423ebe10f0231bf
- florisboard/florisboard master fe1241f4921b3eae923571ff7a3e113a7e10d677；v0.5.2 2e82060251897226c0739b9f52d1d051b02305fb
- AnySoftKeyboard/AnySoftKeyboard 8cf81a3a2f73e9914c8613d56390a933486f0c6b
- Helium314/HeliBoard 44ddb1ae06203857c490da75f9fa281a4e8b89cd；openboard-team/openboard c3772cd56e770975ea5570db903f93b199de8b32；rkkr/simple-keyboard a053c4ca3b61182fc407096ff207d579e5471944
- Apple，App Extension Programming Guide: Custom Keyboard：https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html
- kenlm LICENSE：https://github.com/kpu/kenlm/blob/master/LICENSE

二手說法（已標註非官方）：iOS 鍵盤擴充記憶體 ~70 MB（https://keyboardkit.com/about，廠商自述）；30–60 MB（社群）。

## 8. 全文未查證清單

- iOS 鍵盤擴充記憶體上限的官方數字；Hamster 的記憶體對策。
- Hamster / Trime 的 WebDAV 同步；Hamster 剪貼簿歷史。
- Wisdom-Weasel 是否把選中的 LLM 候選回寫使用者詞庫；其預測是同步還是異步。
- Hamster MIT 重授權是否涵蓋 GPL 來源（Squirrel）片段；qingjian `bamboo-slip` 雙許可條款。
- fcitx5-qt / fcitx5-gtk IMModule（另一個 repo，未 clone）。
- fcitx5-table-extra 各碼表資料的來源與著作權；FlorisBoard `han.sqlite3` 資料授權；k3lp 授權。
- LGPL / GPL 對 App Store 發佈的法律影響（上文為推論）。
- 其餘見各對象節與 FlorisBoard / ASK / HeliBoard 未查證清單。
