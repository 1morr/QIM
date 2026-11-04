# Rime 臨時西文（`inline_ascii`）逐鍵實測

回答的問題：舊版 librime（1.17.0 以前，改寫 commit `74bd5dc4` 之前）組字中單按左 Shift 留碼接英文後，空白、Enter、退格、Esc、再按 Shift、數字與標點各做什麼。結論在 brief §12.2「舊版 ascii_composer」那條，QIM 的做法在 brief §4.3（M1 票 25b）。

## 跑法

```sh
L='{Shift_L}{Release+Shift_L}'
scripts/experiments/rime-inline-ascii/run.sh anr "$L" hello '{space}' world '{Return}'
```

每個參數是一步 librime 按鍵序列，每步之後印上屏、輸入、組字、`ascii_mode` 與前三個候選。

需要：已安裝鼠鬚管（用它的 SharedSupport 與內建 librime），`brew install librime`（只用來取標頭編 `probe.c`），`~/Library/Rime` 有 quick5 的 yaml（可用 `RIME_USER_SRC` 換）。在暫存目錄工作，不寫入 Rime 使用者目錄。

## 上次結果（2026-11-05）

環境：鼠鬚管內建 librime 1.16.0（`ascii_composer.cc` 到 `74bd5dc4^` 都沒變）。

```
anr                          input=[anr] ascii=0 preedit=[日弓口] 1.問 2.門口 3.日名
{Shift_L}{Release+Shift_L}   input=[anr] ascii=1 preedit=[anr]
hello                        input=[anrhello] ascii=1 preedit=[anrhello]
{space}                      input=[anrhello ] ascii=1 preedit=[anrhello ]
world                        input=[anrhello world] ascii=1 preedit=[anrhello world]
{Return}                     commit=[anrhello world] input=[] ascii=0 preedit=[]
```

其他鍵（退格、Esc、再按左右 Shift、數字、標點、方向鍵、已選定的分段）的結果摘在 brief §12.2。換鼠鬚管或 librime 版本後要重量。
