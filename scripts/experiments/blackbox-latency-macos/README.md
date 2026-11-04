# macOS 內建速成的按鍵延遲

回答的問題：macOS 內建速成按一個鍵到組字更新要多久（黑箱基線 T9，M1 票 08；結果在 `docs/research/blackbox-baseline.md`）。

## 跑法

1. 在系統設定啟用「Sucheng – Traditional」並切成目前的輸入法，開「文字編輯」新文件、游標放在文字區。
2. 編譯並對文字編輯的 pid 送一串鍵（只支援小寫字母與空白）：

```sh
swiftc -O scripts/experiments/blackbox-latency-macos/lat.swift -o /tmp/lat
/tmp/lat $(pgrep -x TextEdit) "ne sv uu hu su yg yp ru hi dr "
```

執行它的終端機要有「輔助使用」權限（印出 `trusted: true`）。每鍵印一行：鍵、毫秒。把輸出存檔後算統計：

```sh
python3 scripts/experiments/blackbox-latency-macos/stats.py <輸出檔>
```

## 量的是什麼

工具用 `CGEvent.postToPid` 把按鍵送進文字編輯，接著每 0.2 ms 讀一次焦點元素的無障礙值（含組字中的字根），值一變就記下經過的時間。這樣送的鍵不會畫出候選窗，所以數字是「按鍵到輸入法更新組字」，不含候選窗繪製。換 macOS 主版本時重量。

## 上次結果（2026-11-04）

macOS 27.0.1、TCIM 104（519）、Apple M6；三句各 20 碼、各跑 3 次，共 180 個字母鍵：p50 0.70 ms、p95 9.76 ms、最大 22 ms。原始數據 `.scratch/m1/evidence/08-mac-t9-latency.txt`。
