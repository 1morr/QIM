# 10: 階梯：Foundation Models 與雲端 LLM

Status: ready-for-agent
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

- [ ] README 有模型版本、日期、提示全文、結果表、費用；缺 key 時跳過並註明。
- [ ] Foundation Models 的首次請求尾巴有數字。
- [ ] `cargo xtask check` 綠燈。

## Comments
