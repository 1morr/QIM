# 11: 小型神經模型在鍵盤擴充裡的開銷（candle）

Status: ready-for-agent
Blocked by: 06
讀: plan §2.7、§7.8、§8（小型神經模型那列）；ADR 0005；brief §9；spec〈共同規則〉；`docs/research/qingjian.md`；`docs/research/ranking-techniques.md`

## What to build

用隨機權重的兩個模型量鍵盤擴充跑神經模型的成本：Gboard 級約 1.4M 參數、青簡級約 23M 參數。只量開銷，不訓練、不量正確率（M3）。推論引擎先試青簡用的 candle；candle 在 iOS 能不能跑本身就是要回答的問題。

- 模型形狀照兩個參照的公開描述（研究文件有的照寫，沒有的取合理近似並註明）；輸入長度取 M3 的用法（重排前 6 條路徑、上下文 0 / 50 / 200 字）。
- `scripts/experiments/neural-cost/`：不加進 workspace 的 Rust crate，先在 Mac 量，再交叉編譯到 `aarch64-apple-ios`，接進票 06 的擴充量。
- 指標：模型檔大小、載入時間、單次推論延遲 p50 / p95、擴充常駐記憶體增量；Mac 與 iPhone 各一份。
- candle 在 iOS 編不過或跑不動時，記下原因並試一個替代（查證後選，例如 Core ML 轉換），或停下問使用者。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈小型神經模型〉一節；放不進票 09 的上限時，照 plan §8 寫回 brief §1.5 的候選做法。

## 驗收

- [ ] 兩個模型在 Mac 與 iPhone 上各有四項指標，附機型、iOS 版本、candle 版本；或明確的「不能」與原因。
- [ ] README 能照著重量。
- [ ] `cargo xtask check` 綠燈。
