# 10: 階梯：Foundation Models 與雲端 LLM

Status: done
Blocked by: 07
讀: spec〈模型階梯實驗〉〈採用門檻〉〈共同規則〉；plan §2.7；brief §8、§12.3（Foundation Models）、§12.4；`docs/research/open-source-imes.md` §5；`scripts/experiments/ios-keyboard/README.md`（FM 那節）

## What to build

量 LLM 重排的正確率、延遲與費用。

- Apple Foundation Models：在這台 Mac 用 Swift 命令列跑；另量停頓後第一個請求的延遲尾巴（M2 在 iPhone 上 12 次裡 4 次 6–15 s）。
- DeepSeek `deepseek-flash`（關思考模式）與 `deepseek-v4-pro`：開工時重查模型清單與價格（補進 brief §12.4）；經 OpenAI 相容介面。
- 提示照 spec（上下文 + 輸入碼 + 前 10 個候選、只回序號），可試幾種寫法，記下最好的那種給雲端票照搬。
- 付費呼叫前把估計的請求數與費用告訴使用者、等同意；個人集不送，除非使用者這次同意。
- 量指標、延遲 p50 / p95、每千次請求費用，上下文 0、50、200 字各一次；與門檻比較。

## 驗收

- [x] README 有模型版本、日期、提示全文、結果表、費用；缺 key 時跳過並註明。
- [x] Foundation Models 的首次請求尾巴有數字。
- [x] `cargo xtask check` 綠燈。

## Comments

- 2026-11-11 結果（`scripts/experiments/llm-rerank/README.md`）：flash `hint-paths` 口語前 2 名 49.1% → 59.5%（+10.4，翻好 301、翻壞 25），p95 1,083 ms、逾時 0%、每千次 0.43 元，正確率與成本都過；Foundation Models `one-paths` 50.4%（+1.3），連續 p50 333 ms，停頓後第一個請求 14.6% 超過 2 s，沒過。前文 200 字比 50 字只多 0.2–0.3 點，照門檻該改 50 字。採用由票 11 定。
- 提示的選項（使用者經協調者要求在 dev 比「只給候選列第一頁」與「加 lattice 路徑」）：第一頁只有 3 條整句，答案在選項裡的上限 57%，給 9 條路徑 68%；dev 上只給第一頁兩個模型都比基線差（模型常挑開頭幾個字的短詞，頂掉核心的第 2 名），所以另加只給路徑的 `paths`，定案用它。產品要 `apply_rerank` 能插入路徑，票 11 定。
- 付費（使用者同意只跑 flash、約 11 元內）：dev 3,020 次（試送 20、兩段調參各 1,500），test 15,713 次一次（11:52–12:24），照高峰價合計約 7.9 元，12 時後的部分實付半價。個人集沒送雲端（使用者決定），雲端的「個人集不退步」沒量。`deepseek-v4-pro` 沒量：票 07 記「已轉給 V4.1-Flash」，官方文件（2026-11-11）寫轉走的是舊名 `deepseek-v4-flash`，V4 Pro 9 月 14 日後繼續提供；要量 pro 先另問費用（約 flash 的 4.5 倍）。
- test 只跑一次：flash 2026-11-11 11:52–12:24；Foundation Models 11:59–14:18（含個人集，只在本機）。基線在開工 commit 重量，和票 07 相同（主分支在票 07 之後沒改核心與資料包）。
- 開工時 Foundation Models 是 `modelNotReady`（Mac 與 Siri 語言不同），使用者改設定後可用；寫進 brief §12.3。
- 本票沒改核心、排序與資料包，沒有跑 `bench --check` 對照；個人集 baseline 沒動。
- code-review 未處理（判斷題）：`split`／`set`／`line` 三欄在 `build_prompt`、`orders`、`call` 各抄一次（Data Clumps）；`percentile`、`mcnemar_exact` 從 `model-ladder` 的 Rust 照抄成 Python（跨語言，docstring 註明）；命令列函式的參數名 `a`；Swift 的旗標解析遇到壞值靜默用預設；`stats` 的「沒有數字」不檢查序號是否超出範圍（`orders` 才算）。實驗程式，不進產品。
