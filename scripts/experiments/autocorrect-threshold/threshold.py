"""英文自動修正的門檻（M4 票 25）：照 `crates/qim-core/src/english_mode.rs` 的 `autocorrection` 重寫一份，
在 AOSP 英文詞表上量不同門檻修對、修錯、不修的比例。用法：

    python3 -I threshold.py <快取>/sources/latinime/<sha>/dictionaries/en_US_wordlist.combined.gz

只用標準函式庫。詞表的篩選同 `crates/qim-databuild/src/english.rs`：不收 `f=0`、`not_a_word`、可能冒犯的詞；
同一個小寫鍵留全小寫的寫法。核心的學過的加分、英文使用者詞在這裡沒有（空白學習資料）。
"""

import gzip
import random
import sys

SCALE = 8.589237  # LatinIME 的機率刻度：機率每多一倍多這麼多（事實 M50）
ALPHABET = "abcdefghijklmnopqrstuvwxyz'-"
MIN_CHARS = 3


def load(path):
    words, typos, last = {}, [], None
    for line in gzip.open(path, "rt", encoding="utf-8"):
        if line.startswith(" word="):
            fields = dict(p.split("=", 1) for p in line.strip().split(",") if "=" in p)
            last = fields
            word, f = fields["word"], int(fields["f"])
            if f > 0 and "not_a_word" not in fields and "possibly_offensive" not in fields:
                key = word.lower()
                if key not in words or word == key:
                    words[key] = (word, f)
        elif line.startswith("  shortcut=") and last and "not_a_word" in last:
            typos.append((last["word"], line.strip().split(",")[0].split("=", 1)[1]))
    return words, typos


def one_edit(word):
    """同 `english_mode::one_edit`：(候選, 種類)，種類 0 是 Slip、1 是 Typo、2 是 Extra，留最便宜的。"""
    out = {}

    def add(candidate, kind):
        if candidate and candidate != word:
            out[candidate] = min(out.get(candidate, 9), kind)

    for i in range(len(word) + 1):
        head, tail = word[:i], word[i:]
        before = head[-1] if head else None
        if tail:
            c = tail[0]
            repeated = before == c or (len(tail) > 1 and tail[1] == c)
            add(head + tail[1:], 0 if repeated else 2)
            if len(tail) > 1 and tail[1] != c:
                add(head + tail[1] + c + tail[2:], 1)
            for x in ALPHABET:
                if x != c:
                    add(head + x + tail[1:], 1)
        for x in ALPHABET:
            slip = x == "'" or before == x or (tail and tail[0] == x)
            add(head + x + tail, 0 if slip else 1)
    return out


PENALTY = {0: 0.0, 1: SCALE, 2: 3 * SCALE}


def autocorrection(words, typed, margin):
    key = typed.lower()
    if len(key) < MIN_CHARS or key in words:
        return None
    found = sorted(
        ((words[c][1] - PENALTY[k], words[c][0]) for c, k in one_edit(key).items() if c in words and k != 2),
        key=lambda x: (-x[0], x[1]),
    )
    if not found:
        return None
    best_score, best = found[0]
    runner_up = found[1][0] if len(found) > 1 else float("-inf")
    apostrophe_elsewhere = "'" not in best and any("'" in w for _, w in found)
    if best_score - runner_up < margin or apostrophe_elsewhere:
        return None
    return best


# Gboard 18.5.10 實測修了的錯字（docs/research/gboard-english-2026-11-15.md〈自動修正〉）。
GBOARD = [
    ("freind", "friend"), ("goverment", "government"), ("wierd", "weird"), ("teh", "the"),
    ("definately", "definitely"), ("knowlege", "knowledge"), ("independant", "independent"),
    ("truely", "truly"), ("sucess", "success"), ("finaly", "finally"), ("adress", "address"),
    ("suprise", "surprise"), ("libary", "library"), ("existance", "existence"),
    ("posession", "possession"), ("peice", "piece"), ("neccessary", "necessary"),
    ("gogle", "google"), ("recieve", "receive"), ("acheive", "achieve"), ("becuase", "because"),
    ("its", "it's"),
]


def random_typos(words, rng, count, top):
    common = sorted(
        ((k, f) for k, (_, f) in words.items() if f >= 120 and k.isalpha() and len(k) >= 3),
        key=lambda x: -x[1],
    )[:top]
    letters = "abcdefghijklmnopqrstuvwxyz"
    samples = []
    while len(samples) < count:
        key, _ = rng.choice(common)
        i = rng.randrange(len(key))
        kind = rng.choice(["sub", "del", "ins", "tra"])
        if kind == "sub":
            typo = key[:i] + rng.choice(letters) + key[i + 1:]
        elif kind == "del":
            typo = key[:i] + key[i + 1:]
        elif kind == "ins":
            typo = key[:i] + rng.choice(letters) + key[i:]
        else:
            typo = key[:i] + key[i + 1:i + 2] + key[i] + key[i + 2:] if i < len(key) - 1 else key
        if typo != key and typo not in words:
            samples.append((typo, key))
    return samples


def main():
    words, aosp = load(sys.argv[1])
    samples = random_typos(words, random.Random(25), 3000, 3000)
    print("| 門檻（倍） | Gboard 22 個修對 | AOSP 47 條修對 | AOSP 47 條修錯 | 隨機錯字修對 | 隨機錯字修錯 | 隨機錯字不修 |")
    print("| ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for times in (2, 4, 8):
        margin = SCALE * (times.bit_length() - 1)
        gboard = sum((autocorrection(words, t, margin) or "").lower() == want for t, want in GBOARD)
        got = [(autocorrection(words, t, margin), want) for t, want in aosp]
        aosp_ok = sum(g is not None and g.lower() == w.lower() for g, w in got)
        aosp_bad = sum(g is not None and g.lower() != w.lower() for g, w in got)
        ok = bad = none = 0
        for typo, key in samples:
            g = autocorrection(words, typo, margin)
            if g is None:
                none += 1
            elif g.lower() == key:
                ok += 1
            else:
                bad += 1
        n = len(samples)
        print(
            f"| {times} | {gboard} | {aosp_ok} | {aosp_bad} | {100 * ok / n:.1f}% | {100 * bad / n:.1f}% | {100 * none / n:.1f}% |"
        )


if __name__ == "__main__":
    main()
