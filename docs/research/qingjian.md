# 青簡查證紀錄（宣傳影片 + 原始碼）

查證日期 2026-11-03，對象 [qingjian-team/qingjian](https://github.com/qingjian-team/qingjian) commit `97dde301126b`（2026-11-03），與上次紀錄的 `3d5fd2d`（`open-source-imes.md` §3）相比改了 264 個檔案。另看了作者在抖音發的宣傳影片（2 分 54 秒，無語音，只有畫面與配樂），影片以每秒一幀讀取。只讀原始碼與文件，沒有編譯、安裝或重跑評測；數字都是該專案自己量的。

GPL-3.0-or-later，QIM 只取做法、不抄程式碼（ADR 0006）。

## 1. 影片內容

時間是影片秒數，畫面文字保留簡體原文。多數示範畫面底部註明「候选由青简引擎实际计算，候选窗由青简渲染器绘制」，並標出當時開的是知意或通变模型。

| 時間 | 標題 | 示範 |
|---|---|---|
| 0–8 | （無） | 全拼 `zhe'shi'yi'kuan'zai'ben'di'yun'xing'de'shu'ru'fa` 上屏「这是一款在本地运行的输入法」；標語「36 个字母，一次打对。」 |
| 11–23 | （聊天） | 仿微信整句輸入；`ku` 出 emoji 與 `(T_T)`；候選旁有英文釋義 |
| 24–27 | 只敲声母 | `w'x'm'y'g'x'd'd'n` → 我想买一个新的电脑 |
| 28–34 | 中英混输，不用切换 | `wo'zai'yong` → 我在用ChatGPT；`jin'tian'de'me'e'ting'qu'xiao'le` → 今天的Meeting取消了（開通变） |
| 35–41 | 敲错了，它看得出来 | 漏鍵 `shurfa`、兩鍵對調 `woxiagnqu`、錯鍵 `nihooma` |
| 42–43 | n / l 分不清也行 | 「九条模糊音，按需勾选」，示範的是 z/zh：`zong'guo` → 中国 |
| 44–49 | 九套双拼 | 同一個「你好」：自然码 / 微软 / 搜狗 `nihk`、小浪 `nihs`、首道 / 拼音加加 `nihd` |
| 50–56 | 五笔 86；五笔拼音混输 | `gant` 开发、`khlg` 中国、`trwu` 我们；混輸「第 5 个字母起自动只剩拼音」 |
| 57–60 | 辅码筛词 | `shiyan;nz` 把「实验」排到第一，候選旁顯示輔碼 `[nz]` |
| 61–67 | 忘了英文怎么说？敲拼音 | 英文信件中敲 `fa'piao`，Ctrl+1 上屏候選旁的譯詞 invoice，「不用查字典，也不用联网」 |
| 68–79 | 换一身皮肤，也能自己做 | 多款皮膚，含角色插圖、橫排與直排 |
| 80–85 | 边打字，边学词 | 譯詞可選英、日、西、越、德，「全部本地」 |
| 86–90 | 那些顺手的小事 | `v1+2*3`→7、`rq` 日期、`v12345` 大寫金額、`erfenzhiyi`→½、`sheshidu`→℃、`umumu`→林（拆字）、`vuuid`、`vts` |
| 90–93 | 九宫格；大千注音；繁体输出 | `64426`→你好；`su3cl3`（限 Windows、Linux、Android）；`ruanjian`→軟件 |
| 94–95 | 英文模式 | 拼錯也給對的詞，`recieve` → receive |
| 96–99 | 所有设置，一个窗口 | 全拼加九套雙拼、模糊音、拼寫糾錯、輔碼、輸入統計（標「样例数据」）、模型按需下載：知意 70 MB、含章·通变 43 MB |
| 100–104 | （平台） | Windows 10 / 11、macOS 13 及以上、Linux Fcitx · IBus、Android 预览版；「一个 Rust 核心，各平台一层壳」 |
| 105–115 | 越用越懂你 | 第一次 `shiyan` 的「实验」排第二，按 2 選；第二次就排第一；「学习记录只存在你自己的电脑上」 |
| 118–129 | 知意 | 「排序模型 · 读懂上文，给每条候选打分，挑出最合适的一句」。上文「…就酱吧，」時 `zaicixianggeweizhiqian` 原第 5 位「再次向各位致歉」（100%）升為首選；`wodefeipaoyaobeixiaozhale` 原第 3 位「我的肺泡要被笑炸了」（79.0%）升為首選。註腳：「知意模型由知意输入法作者训练并授权，演示含三元模型」 |
| 130–141 | 通变 | 「P2C 模型 · 青简自研 · 不看上文，从拼音直接逐字造出整句」；不開模型只拼得出「今天的么哦厅取消了」。註腳：「六个字以上的长句它排第一，短句与纠错时由词图兜底」 |
| 142–149 | 整句命中率 | 見下表 |
| 150–154 | 断网，照样打 | 「词库、整句、模型都在本机」 |
| 155–158 | 隐私优先 | 没有账号、没有统计上报、密码框不记录；AI 服務預設關閉，開啟後資料直送使用者自填的服務商；檢查更新只讀版本列表、不帶輸入內容、可關閉 |
| 159–160 | （資源） | 不開模型時記憶體約 21 MB（Windows 實測）；知意 +142 MB、通变 +33 MB |
| 166–174 | （結尾） | 「全部免费，全部开源」；qingjian.app；GPL-3.0-or-later；「青简 · 好好输入」 |

整句命中率（影片 142–149 秒）：

| | 不开模型 | 通变 | 知意 |
|---|---|---|---|
| 首选 | 54.1% | 59.1% | 75.0% |
| 前 5 | 60.7% | 75.3% | 83.6% |
| 前 9（首页） | 60.7% | 75.3% | 83.6% |

影片說明：語料新爬自 B 站、知乎日报、V2EX、IT 之家等 8 個站點，共 1 萬句，以免「开卷」；表中是 5070 句開發集（參與過調參，帶上文、冷啟動），另 4930 句鎖定未開封，留作最終測試；知意含三元模型。

## 2. 影片宣稱與原始碼對照

路徑相對青簡 repo 根目錄，行號以 `97dde30` 為準。

| 影片宣稱 | 原始碼 | 出處 |
|---|---|---|
| 「36 个字母，一次打对」 | 程式與文件都沒有這句。開頭示範句的全拼正好 36 個字母（本機計數）。倉庫裡的「一次打对」是統計指標：上屏前沒按退格的比例 | `docs/user/learning/statistics.md:18` |
| 知意把原第 5 位升為首選 | 屬實。停鍵後在前 5 / 9 / 18 條（省電 / 均衡 / 準確）中挑最高分移到第一 | `crates/qingjian-core/src/engine/choice/mod.rs:209-224`、`choice/tier.rs` |
| 通变：六字以上長句排第一，短句與糾錯由詞圖兜底 | 屬實，門檻是 6 個音節，且拼音要乾淨（無殘缺音節、全覆蓋、沒糾錯） | `docs/notes/crate-notes.md:271-276`、`engine/query/generating.rs` |
| 知意由知意輸入法作者訓練並授權 | 權重自 2026-10-20 起為 Apache-2.0，不隨包；repo 內沒有授權原文 | `crates/qingjian-neural/src/zhiyi/mod.rs:4` |
| 知意 70 MB、通变 43 MB | 知意 70 MB 是 Windows 的 int8 ONNX，macOS 版約 154 MB（safetensors，載入時量化成 Q8_0）；通变 43 MB | `apps/settings/src/jobs/model_download/pack.rs`、`zhiyi/weights.rs:9-10` |
| 「演示含三元模型」 | `zh.ngram` 79 MB，LGPL-2.1-or-later，不隨包，只有 Windows 讀、要使用者手動放進資料目錄 | `crate-notes.md:345-351` |
| 命中率：知意首選 75.0%、通变 59.1% | 對得上作者筆記的開發集數字；75.0% 是加上 `zh.ngram` 後的成績。不開模型 54.1%、前 5 那一列的數字沒查到 | `crate-notes.md:269,352`、`docs/notes/model-identities.md:8` |
| 1 萬句、8 站、5070 / 4930 切分 | 沒查到：評測資料在被 `.gitignore` 的 `data/`，外人無法重跑 | — |
| 不開模型約 21 MB；知意 +142、通变 +33 MB | 沒查到量測文件。筆記只有通变量化後常駐 +112 → +36 MB、0.3.2「开机少占二十多 MB」 | `crate-notes.md:234`、`CHANGELOG.md` 0.3.2 |
| 九條模糊音、九套雙拼 | 屬實：z/zh、c/ch、s/sh、n/l、f/h、l/r、an/ang、en/eng、in/ing，預設全關；雙拼可自製 | `docs/user/input/fuzzy-and-shuangpin.md:30-34` |
| 第 5 個字母起只剩拼音 | 屬實：五筆最長 4 碼 | `docs/user/input/wubi.md:46,50` |
| 譯詞「全部本地」 | 屬實，譯詞表隨包約 30 MB，是離線用 LLM 或機器翻譯生成的 | `assets/glossary/README.md` |
| 沒有統計上報 | 屬實：對外連線只有檢查更新、使用者開的 AI 服務與語音、使用者觸發的模型下載、區域網路剪貼簿同步 | 見 §8 |
| 密碼框不記錄 | 各平台都有判斷（§8），但輸入日誌在一般欄位預設開啟 | `crates/qingjian-platform/src/config/general/mod.rs:220-223,295` |

## 3. 架構與平台

- 核心是 `qingjian-core`（`Engine` 為唯一門面），加上 `qingjian-dictionary`、`qingjian-format`（`.qj` mmap 容器）、`qingjian-lm`、`qingjian-neural`（candle）、`qingjian-learning`、`qingjian-translate`。`qingjian-platform` 不呼叫系統 API，放 TOML 設定與殼和 Server 之間的協定型別（`crates/qingjian-platform/Cargo.toml:3`）。
- 沒有統一的 FFI：沒有 uniffi、C ABI 或 WASM（整個 repo grep 零命中）。每個殼自己接：
  - macOS：同程序，Rust 經 `objc2-input-method-kit` 直接呼叫 IMK。
  - Windows：TSF DLL 只轉發按鍵、寫入文件；引擎在 `qingjian-server.exe`，走命名管道，訊息是長度前綴 JSON（`apps/windows/README.md:12-39`）。管道放行 Everyone、AppContainer 與 Low 完整性（`apps/windows/server/src/ipc/pipe.rs:37`）。
  - Linux：Rust Server 加 Unix socket（0700 目錄、0600 socket、`SO_PEERCRED` 核對使用者）；前端 Fcitx5、Fcitx4 是 C++，IBus（0.3.2 新增）是 Rust（`apps/linux/server/src/ipc/mod.rs:38,69`）。
  - Android：Kotlin `InputMethodService` 加 JNI，自繪鍵盤，標為預覽版（`docs/user/platforms/android.md:4`）。
  - iOS、網頁：沒有，iOS 仍在路線圖未勾選項（`docs/plan/roadmap.md:176`）。
- 候選窗由 `qingjian-render`（主題格式、場景樹）加 `bamboo-slip`（tiny-skia、cosmic-text、taffy、resvg 的自繪引擎）畫成位圖，各平台只貼圖（`crates/qingjian-render/src/lib.rs:1-8`）。Linux 的候選面板仍由 Fcitx 畫。
- 新增 crate（相對 `3d5fd2d`）：`qingjian-clipboard`、`qingjian-clipboard-sync`、`qingjian-code`、`bamboo-slip-i18n`（設定程式七種介面語言）。

## 4. 模型與排序

2026-11-11 以 `729ecba` 補查訓練、重排、前綴快取與評測，並更正本節與 §5 的幾處（整句才乘 0.5、知微 23M／36M 等）：見 `neural-rerank.md` §1、§3。

### 4.1 三個模型

| | 含章·通变 | 知意 | 含章·知微 |
|---|---|---|---|
| 作用 | P2C：拼音直接逐字生成整句 | 讀游標前文，給前幾條候選打分、挑首選 | 讀前文重排整句 |
| 架構 | 字級 GPT-2 式 decoder，檢查點 `small-15000`；參數量沒查到 | 知意輸入法的 1.4V2-Long-L12：mmBERT-small 留 12 層 + 兩層選詞頭 | 字級模型 |
| 推論 | candle；CPU 上線性層 int8、詞嵌入 f16；專屬 rayon 池最多 4 線程 | macOS candle（Q8_0）；Windows ONNX Runtime 1.24.4，CPU 2 線程 | — |
| 授權 | `.qjm` 標 GPL-3.0-or-later，語料署名中文維基、LCCC、IndustryCorpus2 | 權重 Apache-2.0 | — |
| 狀態 | 可選下載 | 可選下載，設定標「推荐」 | 2026-10-27 下線：開發集首選 58.0 對知意 75.0 |

出處：`docs/notes/model-identities.md:7-8`、`crate-notes.md:231-256`、`zhiyi/mod.rs:1-4`、`tools/release/pack-model.sh:21,48`。0.3.2 起本地模型預設關閉，「不开模型最快、最省电」（`CHANGELOG.md`）。

下載：設定程式由使用者觸發，同時向 Gitee 與 GitHub 的 `qingjian-team/qingjian-models` Release 取 manifest，先回來的先用，逐塊與整包核對 SHA-256（`apps/settings/src/jobs/model_download/fetch.rs:17-20`）。

### 4.2 停鍵後改首選

- 兩個模型都在停鍵後非同步跑：防抖 macOS 80 ms（`apps/macos/src/host/model/rescore_monitor.rs:11`）、Windows 30 ms，輪詢 20 ms，模型交結果時另有 wake 回調不等輪詢。作者推算 Windows 停鍵到首選換好，中位數由 154 ms 降到 84 ms，120 ms 內換好的比例由 3% 升到 93%（`crate-notes.md:252-256`）。
- 知意要過兩輪後台：先出字分佈（中位約 15 ms），再選詞打分（約 23 ms）。總分 = 模型 log softmax + 語言模型分 + 位置先驗，後兩項各乘 0.5（掃過 0.25–1，0.5 最好）。位置先驗取自 1.26 萬次上屏位置的分佈，第 1 位 −0.07、第 2 位 −3.44（`engine/choice/mod.rs:57-62`、`crate-notes.md:326-329`）。
- 防止干擾只做了兩件事：使用者翻過頁或動過高亮就不換（`apps/macos/src/host/model/mod.rs:202-205`）；每個請求帶序號，只認最新的（`docs/notes/neural-rescoring.md:82-85`）。
- 沒處理「首選剛換掉就按空白」：作者在 FreeBSD 端到端測 `firefoxliulanqi`，修好「只認最新」後 9 遍仍有 1 遍按空白時整句還沒換上來，上屏成「勠」（`docs/plan/todo.md:44-45`）。
- 知意不參與的候選：英文候選另外比；糾錯產生的備選不進重排，否則 `suran` 的糾錯備選「虽然」會被頂到使用者打對的「肃然」前面（`engine/choice/mod.rs:117-125`）。

### 4.3 詞圖與學習

- 詞級排序 8 條規則；同輸入串選過的次數排在上下文分前面，所以選過一次就贏過沒選過的，前提是前面幾條（第一層、覆蓋字母、是否簡拼、末音節完整、聲調代價）相同（`crates/qingjian-core/src/ranking/scored.rs:92-100`）。
- 學習加分 0.5 × ln(1 + min(次數, 20))；不封頂時選過 173 次的「的」把 `haode` 拉成「号的」（`ranking/mod.rs:32-40`）。
- 衰減：`count × 2^(−天數/90)`，只影響排序權重，記錄一年後才清（`crate-notes.md:158-162`）。全拼與雙拼的選擇互相承認，單字母簡拼不泛化。
- 0.3.2 起候選分段：整句、詞最多 6 個、首音節單字前 9 個。6 的依據：凍結日誌裡選多字詞的有 99.8% 落在前 6 位（`ranking/mod.rs:49-62`）。

### 4.4 作者記下的教訓（`docs/notes/neural-rescoring.md`）

- 神經分不寫回 `Conversion::score`，只決定名次：P2C 分數比靜態分高十幾二十 nat，寫回會讓 `woxiangxuexirust` 被糾成「我想学习如斯」（:102-122）。
- 重排架構本身是瓶頸：P2C 自由生成首選 51.9%，對前 6 條重排 42.3%；兩者互補（只有生成對 16.5%、只有重排對 6.9%）。通变改成「生成為主」就是這個結論。
- 門檻、加大模型（small 對 base）都沒用（:27-28,59）。
- 回放日誌偏袒基線：日誌裡「選了」的整句多半是當時按空白接受的基線輸出，整句要在冷啟動的乾淨集量（:32-38）。

## 5. 評測

- 工具：`qingjian-cli --eval-text`、`--replay`、`--eval-cold`、`--eval-details`（`apps/cli/src/eval/`）。
- 集合：凍結回放日誌 4867 條、`data/eval/sentences.tsv` 8322 句，另有 social、devA、devC、項目文件、新聞等開發集；`data/` 不入庫，樣本數與來源網站都查不到。
- 加三元模型後六個開發集都漲：devA 63.5 → 65.6、項目文件 40.8 → 46.2、新聞 74.1 → 76.6（`crate-notes.md:349`）；社區開發集首選 74.7 → 75.0，CLI 查詢 95 分位 209 → 242 ms。
- 效能預算：每鍵候選 10 ms、啟動 1 秒內；`qingjian-cli --typing` 逐鍵計時。沒有自動化效能回歸（`docs/notes/performance.md:3-13`）。

## 6. 輸入功能與資料

- **拼寫糾錯**：整段一處編輯（相鄰換位、QWERTY 鄰鍵替換、多敲、少敲，限 4–24 個小寫字母）加音節級變體入詞圖。代價：換位、替換 5.0，多敲、少敲 5.5，整段門檻 5.0，換位再減 1.0；個人敲錯表折扣 `min(ln(1+接受次數), 3.0)`（`crates/qingjian-core/src/correction/costs.rs:36-46,73-77`）。非鄰鍵的錯歸模糊音管。每個音節都完整時照原樣讀，糾正只當備選；雙拼與形碼不糾錯（`engine/correcting.rs:13-46`）。
- **輔碼**：預設 `;` 觸發，筆畫碼（橫 h、豎 s、撇 p、點捺 n、折 z），單字取前四筆加末筆、詞取每字首筆；可匯入 Rime `.dict.yaml`（`docs/user/input/aux-code.md`）。筆畫源資料是 CNS11643 全字庫筆順資料（取 OFL-1.1）加大陸序覆蓋表（`tools/dict-convert/src/pack.rs:20`）。
- **倉頡、速成：沒有**。方案只有全拼、雙拼、九宮格、大千注音（macOS 暫無）、五筆 86；repo 裡「仓颉」只出現在詞庫與譯詞的詞條中。
- **繁體**：詞庫、讀音、語言模型都是簡體，只在候選出 Core 時用 `ferrous-opencc` 0.4.0 轉字形（s2tw、s2hk、s2t），不換用語；作者承認一對多會錯（后/後、发/髮），台灣詞庫只是計畫（`crates/qingjian-core/src/traditional/converter.rs:1-15`、`docs/user/input/traditional.md:22,39`）。
- **譯詞**：英、日、西、越、德，一次一種，合計約 30 MB。英、日、德用 DeepSeek、越用 gpt-5.5 離線批量生成（`tools/gloss-gen`）；西語是 Azure Translator，已知 13% 名詞仍是複數。德語冠詞對照 Wiktionary 快照校驗，一致率 98.4%。上屏譯詞：macOS ⌥+數字、Windows Ctrl+數字、Linux Alt+數字（`assets/glossary/README.md`、`docs/user/learning/translation.md`）。
- **英文糾錯**：至少 4 個字母，對 9.4 萬詞的詞表逐詞查一處編輯，按個人選擇次數再按詞頻排（`crates/qingjian-core/src/english/mod.rs`）。詞表來自 SCOWL、CSpell，詞頻取自 wordfreq。
- **拆字**：kfcd/chaizi 簡體版（CC BY 3.0），約 1.44 萬行編進 Core（`assets/chaizi/README.md:9`）。
- **emoji**：CLDR annotations（Unicode License v3）；口語叫法與顏文字是自編小表，未標授權。
- **詞庫**：基礎約 13.3 萬條（2.5 MB，`詞\t音節\t詞頻`）＋ 11 本領域補充詞庫約 13.3 萬條。來源：通用規範漢字表、現代漢語常用詞表、THUOCL、jieba（MIT），讀音取自 Unihan、多音字詞由 LLM 標，詞頻統計自中文維基與 LCCC。雾凇 rime-ice 因 GPL-3.0-only 不相容已移除（`docs/design/landscape.md:33,70`）。五筆碼表取自 rime-wubi86-jidian（Apache-2.0）。

## 7. 皮膚

- 一個主題一個目錄：`theme.json`（JSONC）加 `images/`；支援 PNG、九宮格切邊的 SVG、過渡與循環動畫；可 `extends` 另一個主題、按淺色 / 深色給變數（`docs/design/theme.md:5-14`）。最小例子：

```json
{
  "extends": "qingjian",
  "schema": 1,
  "meta": { "id": "system-blue", "name": "系统蓝", "author": "青简", "license": "GPL-3.0-or-later" },
  "variables": { "accent": { "light": "#007aff", "dark": "#0a84ff" }, "hl_text": "#ffffff" }
}
```

- 內建 8 個主題，其中 4 個帶角色圖；角色圖權利不隨 GPL 授出（`crates/qingjian-render/themes/bocchi/README.md`）。只在 macOS 與 Windows 生效。

## 8. 隱私與網路

密碼框判斷：

| 平台 | 判斷 | 出處 |
|---|---|---|
| macOS | `IsSecureEventInputEnabled()`；呼叫點只看到禁雲端聯想、禁取上下文、禁選區翻譯，是否也停學習沒查到 | `apps/macos/src/imk/secure_input.rs:5-11` |
| Windows | compartment `KEYBOARD_DISABLED` 或 `EMPTYCONTEXT` 非零就放行按鍵；註明 `IS_PASSWORD` 不可靠 | `apps/windows/tsf/src/com/context.rs:1-17` |
| Fcitx5 | `Password` / `Disable` 不組字；`Sensitive` 不讀周邊文字 | `apps/linux/fcitx5/src/qingjian.cpp:204-206,345` |
| IBus | purpose 為 PASSWORD 或 PIN；hint `PRIVATE` 視為私密 | `apps/linux/ibus/src/ibus/engine.rs:31-35` |
| Android | 密碼、URI、信箱欄位直通；`IME_FLAG_NO_PERSONALIZED_LEARNING` 不學習、不記統計、不上雲 | `apps/android/.../service/input/EditorKind.kt:6-23` |

對外連線（grep `https?://`、`wss?://` 與 HTTP crate）：

1. 檢查更新：每 24 小時 `GET https://qingjian.app/releases.json` 與 `.sig`，只帶 `User-Agent: qingjian/<版本>`；驗 ed25519，只提示不安裝；`[update] check` 可關（`crates/qingjian-update/src/index/fetch.rs:7-22`）。
2. 雲端 LLM：`[predict] enabled = false` 預設關，預設端點 DeepSeek，使用者自帶金鑰（`crates/qingjian-predict/src/config.rs:62`）。
3. 語音：火山引擎豆包流式 ASR，只有 Android 用（`crates/qingjian-voice/src/volcengine/config.rs:25`）。
4. 模型下載：使用者觸發（§4.1）。使用者文件〈什么时候联网〉沒列這項（`docs/user/help/privacy.md:9`）。
5. 剪貼簿同步（新）：預設關，只走區域網路。6 位碼經 SPAKE2 配對，換出的密鑰當 Noise `XXpsk3_25519_ChaChaPoly_BLAKE2s` 的 PSK；mDNS 發現，組播被擋時逐一掃同一 /24 的 53319 埠（`crates/qingjian-clipboard-sync/src/channel/mod.rs:1-42`）。

本機資料：

- 輸入統計 `usage-hourly.tsv` 按小時存字數、鍵數、首選數、退格數，不存字；沒找到單獨關閉的開關。
- 輸入日誌 `input-log.jsonl` 記按鍵、看到的候選與上屏內容，供離線回歸評測與個人模型用，**預設開**，`[general] input_log` 可關（`crates/qingjian-platform/src/config/general/mod.rs:220-223,295`）。
- 生詞本存譯詞與例句，可清空。

## 9. 發佈與測試

- 版本：最新 0.3.2（2026-10-30）；各殼 Cargo.toml 為 `0.3.3-dev`，workspace 版本仍是 `0.1.1`。CHANGELOG 一版一節，標題「版本 · 日期 · 渠道」，手寫、面向使用者；CI 從它產生 GitHub Release 與官網的 `releases.json`。
- 簽章：macOS Developer ID 加公證；Android 簽名；Windows 無程式碼簽章，所以 uiAccess 起不來，UWP 宿主裡候選窗可能被蓋（`docs/notes/release.md`）。
- 更新索引用 ed25519 簽名，公鑰是列表以便輪換。
- 授權：整體 GPL-3.0-or-later；`bamboo-slip`、`bamboo-slip-i18n`、`qingjian-render` 另有商業授權（render 的商業授權不含 `themes/`）。不簽 CLA，改成向這三個 crate 貢獻即同意以 MIT-0 授權（`crates/bamboo-slip/LICENSE.md:3-9`、`docs/contributing.md:110`）。
- 測試：`#[test]` 約 1688 個（grep 計數）；Linux 有真 Server 對 Fcitx 的端到端測試（`apps/linux/fcitx5/tests/real.cpp`）；macOS 靠手動與 `osascript`；Android 沒有測試目錄；設定檔有歷代版本的回歸測試（`crates/qingjian-platform/tests/config_history`）。

## 10. 對 QIM

### 10.1 和 QIM 決定的差異

- **停鍵模型能不能改首選**：ADR 0005 規定停鍵層與雲端層最高到第 2 位。青簡讓知意與通变直接改首選，靠縮短延遲與「翻過頁就不換」降低干擾，仍留下「按空白時還沒換好」的已知問題（§4.2）。這是 ADR 0005 理由的實例，不是反例；但它也說明首選命中的大部分增益（54.1 → 75.0，作者自量）來自讓模型動首選。M2 模型階梯實驗量停鍵層時，應同時量「只到第 2 位」與「可改首選」兩種接法的首選率和空白鍵競態。
- **核心綁定**：ADR 0002 選 uniffi 加 C ABI 加 WASM；青簡每個殼各接各的（同程序 objc2、命名管道 JSON、Unix socket、JNI），沒有網頁版。
- **繁體**：青簡是簡體核心出口轉字形，QIM 是繁體碼表；倉頡、速成青簡完全沒有，無可借的碼表或資料。

### 10.2 可取的做法

- 神經分只決定名次，不寫回分數（已在 plan §2.7）。
- 請求帶序號、只認最新；模型交結果用 wake 回調通知殼，不靠輪詢。
- 糾錯「預設相信使用者」：原樣每段都讀得通就不糾，糾正只當備選；鄰鍵替換與非鄰鍵分開，後者交給模糊規則。
- 學習加分取對數並封頂；同碼選過的優先於語言模型分。
- 模型檔位只調兩個直觀參數（比幾條、看多長前文）。
- 「首選選中率」與「正確句在第一頁」分開量；整句評測用冷啟動乾淨集，不用回放日誌。
- 更新索引從第一版就簽名，公鑰做成列表。
- 設定檔歷代版本回歸測試。

### 10.3 要避開的坑

- 評測資料不入庫，對外數字無法重現（影片的 54.1%、21 MB 都查不到出處）。QIM 對外宣稱的數字應附可重跑的量法。
- 輸入日誌預設開、內含上屏內容；QIM 若要記，應預設關或在首次啟用時明示。
- Windows 無簽章時 uiAccess 失效；命名管道對同機所有程序開放。
- 青簡的詞庫、譯詞表等加工檔隨 repo 為 GPL；要用其上游資料（THUOCL、jieba、CLDR、chaizi、CNS11643 筆順）就從上游取，不拷青簡的加工檔。

## 11. 舊紀錄的更正

`open-source-imes.md` §3 qingjian（`3d5fd2d`）以下幾點已過時，以本檔為準：

- 「P2C 自由生成未進產品」：已是通变，生成為主，6 音節以上排首選。
- 沒提知意：它是目前推薦的模型，作用在首選。
- 「停鍵 80 ms」：只剩 macOS，Windows 30 ms。
- 「56 MB fp16、Metal 首次編譯 12 秒」：12 秒是早期數字，2026-09-12 裝機實測 102 ms（`docs/plan/todo.md:79`）；模型改為可選下載、預設關閉，知微已下線。
- `bamboo-slip` 雙授權：已查清，見 §9。

## 12. 未查證

- 影片的 1 萬句評測集、5070 / 4930 切分、不開模型 54.1% 與前 5 那一列。
- 21 MB、+142 MB、+33 MB 的量測方法。
- 通变的參數量；知意的授權原文（只有 repo 內 Apache-2.0 的一句話）。
- macOS 密碼框是否也停學習與輸入日誌。
- 行動端（Android）能否跑這兩個模型。
