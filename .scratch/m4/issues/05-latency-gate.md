# 05: 每鍵延遲閘門

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §5.2；`docs/development.md`〈每鍵延遲〉；spec〈每鍵延遲上限〉；`.scratch/m3/issues/12-personal-bigram.md` Comments（學過資料的量法）

## What to build

容錯與萬用鍵會放大 lattice，每鍵延遲要從只報不擋改成會擋：

- `cargo xtask bench --check` 照 spec〈每鍵延遲上限〉擋原生的 p95（全部、第 20 碼起、選開頭一段），WebAssembly 照舊只報。
- 報告另量一組「學過的學習資料」：一個 `UserModel` 依序學完全部公開集合後再計時（M3 票 12 的量法），原生與 WebAssembly 都量，原生照它那一列的上限擋。
- 超過時的處理照 spec（負載低時單獨重跑一次，仍超過才紅），寫進 `docs/development.md`。

## 驗收

- [ ] 判斷函式的雙向測試：量到的數字超過任一格會紅，在上限內不紅。
- [ ] 報告有空白與學過兩組的 p50 / p95（原生、WebAssembly）。
- [ ] plan §5.2 與 `docs/development.md` 改成「原生擋、WebAssembly 只報」，上限連到 spec。
- [ ] Comments 附這次量到的兩組數字；`cargo xtask check` 綠燈。
