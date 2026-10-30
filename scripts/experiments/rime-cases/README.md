# Rime 速成三案例復現

回答的問題：Rime 速成（rime-quick `quick5`）連打 `mir`、`anr` 時首選為何不是「一台」「門口」。結論與根因見 `docs/research/rime.md` §4。

## 跑法

```sh
scripts/experiments/rime-cases/run.sh base  mir anr     # 現行方案
scripts/experiments/rime-cases/run.sh xform anr         # 只留速成碼的反事實
WITH_USERDB=1 scripts/experiments/rime-cases/run.sh base mir   # 帶上本機學習紀錄
```

分段選字學詞（`'mir#i,j' mir`）：候選索引從 0 起算，隨碼表版本與 userdb 而變，先用 `N=200` 列出 `mir` 找到「一」的索引，再列選完後 `ir` 段找「台」的索引。

需要：已安裝鼠鬚管（用它的 SharedSupport 與內建 librime），`brew install librime`（只用來取標頭編 `probe.c`），`~/Library/Rime` 有 quick5 的 yaml（可用 `RIME_USER_SRC` 換）。腳本在暫存目錄工作，不寫入 Rime 使用者目錄。

- `probe.c`：用 `rime_api.h` 的 `simulate_key_sequence` 餵輸入碼、列候選；`碼#i,j` 依序選第 i、j 個候選並上屏。
- `lua/dbg.lua`：在候選的 comment 附上 `type`、`quality`、區間。

## 上次結果（2026-10-30）

環境：鼠鬚管 1.1.2（librime 1.16.0），rime-quick `5dcdb9e`、rime-cangjie `fcb3d7d`、rime-essay `054920d`，不帶 userdb。

```
== mir  1. 一店 [phrase]  2. 䂖 [phrase]  3. 歹口 [phrase]
== anr  1. 問 [phrase q=1.0005]  2. 門口 [phrase q=1.0001]  3. 日名
xform:
== anr  1. 門口 [phrase]  2. 日名
```

換鼠鬚管、librime 或 rime-quick / rime-essay 版本後要重量，結果記回 `docs/research/rime.md`。
