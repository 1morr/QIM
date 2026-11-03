"""Random quick-code strings vs. English words: exact hits and one-edit hits.

Answers how often a Chinese-mode English candidate would appear if it
tolerated one typo (brief §4.7, §12.1). Run: python3 -I edit1.py
Uses macOS /usr/share/dict/words; random letters do not follow the real
quick-code distribution, so read the numbers as an order of magnitude.
"""

import random

DICT = "/usr/share/dict/words"
QUICK_ALPHABET = "abcdefghijklmnopqrstuvwxy"
EDIT_ALPHABET = "abcdefghijklmnopqrstuvwxyz"
SAMPLES = 3000


def one_edit(s):
    out = set()
    for i in range(len(s) + 1):
        for c in EDIT_ALPHABET:
            out.add(s[:i] + c + s[i:])
    for i in range(len(s)):
        out.add(s[:i] + s[i + 1 :])
        for c in EDIT_ALPHABET:
            out.add(s[:i] + c + s[i + 1 :])
    for i in range(len(s) - 1):
        out.add(s[:i] + s[i + 1] + s[i] + s[i + 2 :])
    out.discard(s)
    return out


def main():
    random.seed(1)
    with open(DICT) as f:
        words = {w.strip() for w in f if w.strip().isalpha() and w.strip().islower()}
    print("words", len(words))
    for n in (4, 5, 6, 7, 8):
        exact = near = 0
        for _ in range(SAMPLES):
            s = "".join(random.choice(QUICK_ALPHABET) for _ in range(n))
            if s in words:
                exact += 1
            elif one_edit(s) & words:
                near += 1
        print(n, f"exact {100 * exact / SAMPLES:.1f}%", f"edit1 {100 * near / SAMPLES:.1f}%")


if __name__ == "__main__":
    main()
