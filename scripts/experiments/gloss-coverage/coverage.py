"""CC-CEDICT / CC-Canto as a gloss source: coverage of QIM's lexicon and how many
glosses can be committed as-is (docs/research/translation-sources.md).

Run from the repo root after `cargo xtask data` has filled the source cache:

    python3 -I scripts/experiments/gloss-coverage/coverage.py <cedict.txt> <cccanto-webdist.txt>

<cedict.txt> is the unzipped cedict_1_0_ts_utf-8_mdbg.txt from mdbg.net; the
download is not versioned, so record its header `date=` and sha256 with the
numbers. <cccanto-webdist.txt> comes from cantonese.org/cccanto-170202.zip.

Lexicon words are rime-essay entries with frequency >= 100 and two or more Han
characters (the lexicon threshold); both sides fold HK / OpenCC variants onto
one form, so matching is approximate. "Committable" means the first sense that
is not a cross-reference, classifier, surname or variant note reduces to at
most three plain English words after dropping brackets, everything after the
first ';' or ',', and a leading "to ".
"""

import collections
import glob
import os
import random
import re
import sys

CACHE = os.environ.get("QIM_CACHE_DIR") or os.path.expanduser("~/Library/Caches/qim")
SOURCES = os.path.join(CACHE, "sources")
HAN = re.compile(r"^[㐀-鿿\U00020000-\U0003ffff]+$")
ENTRY = re.compile(r"^(\S+) (\S+) \[([^\]]*)\]( \{[^}]*\})? /(.*)/\s*$")
SKIP = re.compile(
    r"^(CL:|(old |archaic |ancient |Japanese |Korean )?variant of|see |surname |abbr\.|"
    r"also (written|pr\.)|used in |\(bound form\)|\(onom|\(literary\)|\(dialect\)|"
    r"\(classical\)|pr\. |erhua variant|\(Tw\) variant|\(Cantonese\)|Taiwan pr\.)",
    re.I,
)
# QIM's two-way forms (crates/qim-databuild/src/common_forms.rs) plus OpenCC-standard
# forms that rime-essay uses where CC-CEDICT uses the common one.
EXTRA_FOLDS = "線綫 啟啓 麵麪 床牀 戶户 敘敍 臥卧 裏裡 污汙 泄洩 枱檯 鈎鉤 群羣 峰峯 溫温 說説 為爲 著着 衛衞 眾衆"


def cached(pattern):
    return glob.glob(os.path.join(SOURCES, pattern))[0]


def build_fold():
    fold = {}
    with open(cached("opencc/*/data/dictionary/HKVariants.txt"), encoding="utf-8") as f:
        for line in f:
            if "\t" not in line or line.startswith("#"):
                continue
            std, variants = line.rstrip("\n").split("\t")
            for v in variants.split(" "):
                if len(std) == 1 and len(v) == 1 and v != std:
                    fold[v] = std
    for pair in EXTRA_FOLDS.split():
        fold[pair[1]] = pair[0]
    return fold


FOLD = build_fold()


def norm(word):
    return "".join(FOLD.get(c, c) for c in word)


def load_essay():
    essay = collections.Counter()
    with open(cached("rime_essay/*/essay.txt"), encoding="utf-8") as f:
        for line in f:
            word, n = line.rstrip("\n").split("\t")
            if HAN.match(word):
                key = norm(word)
                essay[key] = max(essay[key], int(n))
    return essay


def load_dict(path):
    senses = collections.defaultdict(list)
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.startswith("#"):
                continue
            m = ENTRY.match(line)
            if m:
                senses[norm(m.group(1))].extend(s for s in m.group(5).split("/") if s)
    return senses


def committable(senses):
    for sense in senses:
        sense = sense.strip()
        if SKIP.match(sense):
            continue
        t = re.sub(r"\([^)]*\)|\[[^\]]*\]", "", sense)
        t = t.split(";")[0].split(",")[0].strip()
        t = re.sub(r"^to ", "", t).strip()
        if t and re.fullmatch(r"[A-Za-z][A-Za-z' .\-]*", t) and len(t.split()) <= 3:
            return t
    return None


def report(name, keys, dicts):
    hit = ok = 0
    for w in keys:
        senses = [s for d in dicts for s in d.get(w, [])]
        if senses:
            hit += 1
            ok += committable(senses) is not None
    n = len(keys)
    print(f"{name}\tn={n}\t有條目 {hit} ({hit / n:.1%})\t可直接上屏 {ok} ({ok / n:.1%})")


def main(cedict_path, canto_path):
    essay = load_essay()
    words = [w for w, n in essay.most_common() if len(w) >= 2 and n >= 100]
    chars = [w for w, _ in essay.most_common() if len(w) == 1]
    ced, can = load_dict(cedict_path), load_dict(canto_path)
    print(f"essay 詞 {len(words)}、單字 {len(chars)}；CC-CEDICT 詞頭 {len(ced)}、CC-Canto 詞頭 {len(can)}")
    for top in (5000, 20000, 50000, len(words)):
        report(f"essay 詞前 {top}：CC-CEDICT", words[:top], [ced])
        report(f"essay 詞前 {top}：CC-CEDICT+CC-Canto", words[:top], [ced, can])
    for top in (1000, 3000, 6000):
        report(f"單字前 {top}：CC-CEDICT", chars[:top], [ced])
    with open("data/lexicon/yue-colloquial.tsv", encoding="utf-8") as f:
        yue = [norm(l.split("\t")[0]) for l in f if l.strip() and not l.startswith("#")]
    report("粵語口語小詞表：CC-CEDICT", yue, [ced])
    report("粵語口語小詞表：CC-CEDICT+CC-Canto", yue, [ced, can])
    words_hk = set()
    with open(cached("words_hk/*/wordslist.csv"), encoding="utf-8") as f:
        for line in f:
            w = norm(line.split(",")[0].replace("*", ""))
            if len(w) >= 2 and HAN.match(w) and w not in essay:
                words_hk.add(w)
    report("粵典詞頭（essay 沒有）：CC-CEDICT+CC-Canto", sorted(words_hk), [ced, can])

    print("\n單字前 20 的第一個可上屏釋義（CC-CEDICT 依拼音排列，不依常用度）：")
    print(" ".join(f"{c}={committable(ced[c])}" for c in chars[:20]))

    print("\n抽查樣本（可直接上屏的詞，前 5000 與 5000–20000 各 50，種子 2020）：")
    rng = random.Random(2020)
    for lo, hi in ((0, 5000), (5000, 20000)):
        pool = [(w, committable(ced[w])) for w in words[lo:hi] if w in ced and committable(ced[w])]
        print(f"[{lo}–{hi}] " + "  ".join(f"{w}={g}" for w, g in rng.sample(pool, 50)))


if __name__ == "__main__":
    main(*sys.argv[1:3])
