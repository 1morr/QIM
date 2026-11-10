# 03: 排位規則補測試再修

Status: ready-for-agent
Blocked by: None (can start immediately)
讀: plan §2.3、§2.4；spec〈共同規則〉；`.scratch/m1/issues/24-m1-close.md` Comments 的「M3」各條（排位規則、`beam=0` 兩條）

## What to build

個人 bigram 與聯想都要動排位，先把 M1 留下沒測到的規則守住。

- 規則 1 把觸發詞釘在首選時，規則 2「選一次不成首選」可能會多移一次（M1 讀程式推論、未實測）：先寫出會紅的測試，紅了再修；寫不出就在 Comments 說明並留測試守門。
- 規則 3、4 與「分段選出、詞庫沒有的整串」補測試（M1 票 13、14 沒有）。
- `--set beam=0` 或 `NaN` 繞過方案檔的驗證：驗證搬到 `qim_core::Ranking`，方案檔與 `bench --set` 共用同一處。

## 驗收

- [ ] 規則 1–4 各有測試；規則 1 與 2 的交互有測試，修了的話先紅後綠（Comments 記紅的輸出）。
- [ ] `bench --dev --set beam=0`、`--set beam=NaN` 被拒，錯誤訊息指出是哪個值。
- [ ] `cargo xtask bench --check` 與 `--learning` 前後對照寫在 Comments；`cargo xtask check` 綠燈。

## Comments
