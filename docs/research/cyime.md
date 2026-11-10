# CyIME 查證紀錄

查證日期 2026-10-31，對象 [Cyletix/CyIME](https://github.com/Cyletix/CyIME) commit `31ff80e84405`。只讀 README、外掛設計文件、模型評估文件與少數原始碼，沒有編譯或安裝。以下數字都是該專案自己量的，沒有重跑。

## 1. 專案概況

- Android 輸入法，Kotlin + librime（JNI），GPL-3.0，所以 QIM 只取做法、不抄程式碼（ADR 0006）。
- 衍生自 [Xime](https://github.com/ximeiorg/Xime)（同為 GPL-3.0）；repo 建於 2026-09-22，109 stars，`app/src/main/java` 約 612 個 Kotlin 檔。
- 完整版安裝包含離線模型約 490 MB，輕量版不含模型（`docs/1.15.17.md`）。

## 2. 外掛系統

### 2.1 形態

- 外掛是 Lua 腳本（luaj），打包成 `.xipk`（zip，≤10 MB），由擴充商店安裝；另有 APK 形態給需要 SDK 的語音辨識。repo 內有 14 個：AI 回覆、AI 翻譯、AI 改寫、三種線上語音、顏文字、表情包、打字統計、WebDAV 備份、兩種剪貼簿同步、Windows 同步、快捷發送範例。
- 能力由 manifest 的 `capabilities` 宣告：`emoji`、`speech`、`tool`、`clipboard_sync`、`events`、`candidate_transform`、`clipboard_read`。`prediction` 只預留，沒有實作（`docs/plugin-system-audit.md` §2）。
- 模型和輸入方案**不是**外掛：模型走另一套目錄與下載器（`CandidateModelCatalog.kt`，每個檔案有 URL、sha256、大小與版本，舊版標 `retired` 讓使用者能刪），輸入方案走 Rime 方案檔。引擎、鍵盤和候選主流程也不能插拔。

### 2.2 值得取的規則

- 未宣告的能力不建通道、不呼叫，也沒有開銷（`docs/plugin-capability-registry.md` §3.3）。
- 執行模型只分兩種：同步（有硬超時）和不等回應；是否進按鍵路徑是能力的明確屬性。
- 唯一進按鍵路徑的 `candidate_transform`：硬超時 15 ms，超時回退原候選；連續 3 次超時，本次工作階段熔斷；敏感欄位不發請求；只取第一個外掛，九宮格不支援。外掛在引擎算完後才改列表，不進引擎計分（同上 §4.1、§5）。
- 網路三重門：manifest 宣告網域、使用者授權、請求時白名單檢查（`NetworkPolicy.kt`）。
- 密鑰類設定落盤用 AES-GCM（`docs/plugin-system-audit.md` §3）。

### 2.3 它自己記下的問題

- 新增一個能力要動 6 處，`type` 與 `capabilities` 語意重疊，有 5 個 Lua adapter（`docs/plugin-capability-registry.md` §1）。收斂計畫的第 3、4 階段尚未開始。
- 選中文字和面板輸入沒有宣告要求，只有剪貼簿同步有完整的「宣告、啟動前校驗」（`docs/plugin-system-audit.md` §3）。
- `PluginCrashHandler` 找不到肇事外掛，隔離實際靠超時後丟棄執行緒（同上 §7）。

## 3. 模型結果（九宮格拼音）

2026-11-11 補查評估集、剪枝、神經重排停用的對照與 CyLM，並更正本節幾處（`docs/development/` 已不在 HEAD、5–7 ms 是電腦均值等）：見 `neural-rerank.md` §2、§3。

九宮格一個數字對應多個字母，和速成同樣是碼少、重碼重，但它是拼音，數字不能直接套到速成。

整段首選（`docs/development/t9-ngram-handover-20261010.md` §7）：

| 測試集 | 原算法 | 詞級 bigram | 詞級 trigram |
| --- | --- | --- | --- |
| 真實聊天 130 句 | 54 | 92 | 97 |
| 語法結構 374 句 | 241 | 287 | 292 |
| 生成日常 171 句 | 122 | 151 | 154 |
| 日常 120 句 | 104 | 109 | 113 |

- 模型大小：trigram 緊湊格式 203 MB，剪枝版 105 MB，整句首選約低 3 個百分點（`CandidateModelCatalog.kt`）。可選下載，不內建於輕量版。每鍵均值約 5–7 ms，有記憶體映射（`t9_ngram_model.h`，同文件 §3、§7）。
- 神經整句重排（UER RoBERTa L4，INT8，33.8 MB）已停用，理由原文：「实测对首选提升很小，却拖慢打字并在停顿后改动候选」（`CandidateModelCatalog.kt`）。
- 前文：後半句接上前半句當上文，373→410/488（調參集）、419→436/495（測試集）。
- 使用者學習：線性插值 `(1−λ)·P底座 + λ·P用戶`，λ=0.05，底座唯讀（同文件 §7b）。另有個人語料訓練的模型，在沒見過的個人句子上 25.13%→30.16%，通用短信集持平；個人集首選只有 27–33%，明顯低於公開集（`docs/development/t9-personal-model-eval-20261018.md`）。
- 剩餘錯誤：同碼歧義（他／她、早／晚）、的／得、句尾語氣詞、個人詞彙。語料覆蓋不足時，模型本身就會判錯（「能烧水的」）。
- 語料：LCCC-base 為 MIT；OpenSubtitles 未寫明授權，使用者自己接受了風險，QIM 不能直接比照。
- 測試集不大，且曾有測試句混進訓練語料造成洩漏（個人模型評估的 daily 集），作者已自行標記無效。

## 4. 其他做法

- 實體鍵盤：`service/Hardware*.kt`（候選列跟隨游標、方向鍵選字、語言切換快捷鍵、按鍵音），並有對應的 Android 測試（`androidTest/.../ui/Hardware*Test.kt`）。
- 懸浮鍵盤：README 記載可拖曳、半透明、圓角；1.15.6 可拖角調大小，測試見 `androidTest/.../ui/*Floating*Test.kt`。
- 模型記憶體：`ModelRuntime.kt` 分 HOT、WARM、COLD 三級，引用計數歸零釋放，系統記憶體壓力時釋放非常駐模型。
- 長按候選原為永久屏蔽，使用者評為「完全偏離預期」，改成「忘掉學習」：詞庫原有的詞回到原位（`t9-ngram-handover` §7c）。

## 5. 未查證

- `plugins/` 各外掛的實際程式碼、APK 外掛形態、擴充商店協定。
- 實體鍵盤與懸浮鍵盤的實作細節：只看了檔名和部分程式碼，M8 開工時再讀。

## 6. QIM 不照搬外掛系統

QIM 維持原架構：方案檔與資料包已能替換方案和模型，`rerank_request` / `apply_rerank` 已能替換重排模型，其他服務等有第二種實作時才抽介面。腳本外掛列入待評估，網頁版驗收後再定，理由：

- 每鍵預算與「同一把尺」不容許第三方程式碼進入排序，基準也驗不到實際的外掛組合。
- App Review 2.5.2 限制下載會改變功能的程式碼，4.7 對外掛另有責任要求（brief §12.4）；Lua 商店是否合規未查證。
- 輸入法外掛看得到使用者打的字，與 brief §8 的硬限制衝突。
- 五個平台加網頁都要帶腳本執行環境，而目前只有一位使用者。

若日後要做，只開放不在按鍵路徑上的能力，並沿用 §2.2 的規則。
