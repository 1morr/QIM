# 30: 聯想補強

Status: ready-for-agent
Blocked by: 18, 18b, 18c
讀: plan §2.1（聯想）、§2.4；`.scratch/m3/issues/13-association.md`；`docs/research/blackbox-baseline.md` 的 T8；spec〈共同規則〉

## What to build

M3 票 13 留下、黑箱 T8 的缺功能，先評估再做：

- emoji 聯想、空輸入的句首建議、數字與標點後的量詞聯想。
- 每項查成熟做法、資料來源與誤觸風險，決定做或不做。
- 做的照聯想現有的開關（brief §7，預設開）。

## 驗收

- [ ] 每項做或不做與理由寫在 Comments 與 plan §2.1。
- [ ] 做的各有核心測試與 Playwright 讀回、截圖；`bench --check` 不變；`cargo xtask check` 綠燈。
