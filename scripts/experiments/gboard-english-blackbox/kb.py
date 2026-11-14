#!/usr/bin/env python3
"""Tap Gboard keys with `adb shell input tap` (the target device comes from ANDROID_SERIAL).

usage: kb.py "<keys>" [delay-seconds]       English QWERTY letter page
       kb.py --sym "<keys>" [delay-seconds] ?123 page (must already be open)

Letter page tokens: a-z, uppercase = Shift + letter, ' ' space, ',' '.', '<' backspace,
'^' Shift, '~' Enter, '#' ?123.  ?123 page tokens: digits, @#$_&-+()/ *"':;!?,
' ' space, ',' '.', '<' backspace, 'A' back to letters, '~' Enter.

Key centres were read off screenshots of Gboard 18.5.10 on a 1080x2400, 420 dpi
emulator with the default settings (no number row). Coordinates below are half-scale
(540 wide) and doubled when tapping; re-read them if the layout differs.
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
