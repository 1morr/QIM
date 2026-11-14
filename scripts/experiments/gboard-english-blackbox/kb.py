#!/usr/bin/env python3
"""用 `adb shell input tap` 點 Gboard 的鍵（目標裝置照 ANDROID_SERIAL）。

用法：kb.py "<鍵>" [間隔秒數]        英文 QWERTY 字母頁
      kb.py --sym "<鍵>" [間隔秒數]  ?123 頁（要先打開）

字母頁的符號：a-z，大寫是 Shift 加字母，' ' 空白，',' '.'，'<' 退格，'^' Shift，
'~' Enter，'#' 開 ?123 頁。?123 頁的符號：數字、@#$_&-+()/ *"':;!?，' ' 空白，
',' '.'，'<' 退格，'A' 回字母頁，'~' Enter。

鍵的中心從 1080x2400、420 dpi 模擬器上 Gboard 18.5.10 預設設定（沒有數字列）的截圖
量出；下面的座標是半比例（540 寬），點的時候乘 2。佈局不同就重量。
"""
import subprocess
import sys
import time

COLS = [30, 83, 136, 189, 243, 296, 349, 403, 456, 510]
BOTTOM = {",": (109, 1089), " ": (296, 1089), ".": (429, 1089), "~": (496, 1089)}

LETTERS = dict(BOTTOM)
LETTERS.update({c: (COLS[i], 860) for i, c in enumerate("qwertyuiop")})
LETTERS.update({c: ([56, 109, 162, 216, 269, 322, 376, 429, 482][i], 935) for i, c in enumerate("asdfghjkl")})
LETTERS.update({c: ([109, 162, 216, 269, 322, 376, 429][i], 1012) for i, c in enumerate("zxcvbnm")})
LETTERS.update({"^": (42, 1012), "<": (495, 1012), "#": (42, 1089)})

SYMBOLS = dict(BOTTOM)
SYMBOLS.update({c: (COLS[i], 858) for i, c in enumerate("1234567890")})
SYMBOLS.update({c: (COLS[i], 935) for i, c in enumerate("@#$_&-+()/")})
SYMBOLS.update({c: ([109, 162, 216, 269, 322, 376, 429][i], 1012) for i, c in enumerate("*\"':;!?")})
SYMBOLS.update({"<": (495, 1012), "A": (42, 1089)})


def tap(x, y):
    subprocess.run(["adb", "shell", "input", "tap", str(x * 2), str(y * 2)], check=True)


def main(argv):
    page = LETTERS
    if argv and argv[0] == "--sym":
        page, argv = SYMBOLS, argv[1:]
    keys = argv[0]
    delay = float(argv[1]) if len(argv) > 1 else 0.25
    for ch in keys:
        seq = ["^", ch.lower()] if page is LETTERS and ch.isalpha() and ch.isupper() else [ch]
        for k in seq:
            tap(*page[k])
            time.sleep(delay)


if __name__ == "__main__":
    main(sys.argv[1:])
