# 10: 鍵盤擴充裡的 Foundation Models 與上下文長度

Status: ready-for-agent
Blocked by: 06
讀: brief §8、§12.3；plan §2.7、§7.8；ADR 0005；spec〈共同規則〉；`docs/research/platform-frameworks.md` §3.2

## What to build

在票 06 的擴充裡回答兩件事：Foundation Models 能不能在鍵盤擴充內用；`textDocumentProxy.documentContextBeforeInput` 實際讀得到多少字。

- Foundation Models：先查官方文件的可用條件（裝置、Apple Intelligence 設定、擴充內是否允許）並補進 brief。擴充裡加一個鍵，送一段固定提示（例如給 10 個候選要它回序號，plan §7.8 的格式），記下能否使用、`availability` 回什麼、首次載入與單次回應的延遲、擴充的常駐記憶體變化。手機沒開 Apple Intelligence 時停下請使用者打開。
- 上下文長度：在備忘錄與另一個宿主（例如 Safari 的文字欄位）貼入長文、游標放在不同位置，擴充讀 `documentContextBeforeInput` 的長度並寫 log 或顯示；看它在換行、段落處是否截斷。
- 結果寫進 `docs/research/ios-feasibility.md` 的〈Foundation Models〉〈上下文長度〉兩節，結論摘進 brief §12.3。

## 驗收

- [ ] Foundation Models：明確的能或不能；能的話有延遲與記憶體數字，附機型、iOS 版本與 Apple Intelligence 狀態。
- [ ] 上下文：至少兩個宿主、多個游標位置的實測字數；截圖 `.scratch/m2/evidence/10-*.png`。
- [ ] brief §12.3 更新並附來源。
- [ ] `cargo xtask check` 綠燈。
