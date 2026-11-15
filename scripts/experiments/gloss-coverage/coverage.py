"""CC-CEDICT／CC-Canto 當譯詞來源：對 QIM 詞庫的覆蓋率，與多少釋義能直接上屏
（docs/research/translation-sources.md〈CC-CEDICT 能蓋多少〉）。

在 repo 根目錄、`cargo xtask data` 填好來源快取後跑：

    python3 -I scripts/experiments/gloss-coverage/coverage.py <cedict.txt> <cccanto-webdist.txt>

<cedict.txt> 是 mdbg.net 的 cedict_1_0_ts_utf-8_mdbg.txt 解壓檔；下載沒有版本，
記數字時一併記檔頭的 `date=` 與 sha256。<cccanto-webdist.txt> 取自
cantonese.org/cccanto-170202.zip。快取路徑照 docs/development.md〈資料建置〉：
有 QIM_CACHE_DIR 就用它，否則用 macOS 的 ~/Library/Caches/qim。

詞庫的詞取 rime-essay 詞頻 >= 100、兩字以上的漢字詞（詞庫門檻）；兩邊都把香港寫法與
OpenCC 標準寫法折成同一寫法，比對是近似的。「可直接上屏」：依檔內順序（同一詞的各讀音
條目串起來，加 CC-Canto 時接在後面）逐個看義項，跳過參照、量詞、姓氏、異體的說明，
取第一個分號或逗號前、去括號與開頭 "to " 後剩 1–3 個英文詞的義項；前面的化不成時往後
找，所以是上限。
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
# QIM 的雙向並存寫法（手抄自 crates/qim-databuild/src/common_forms.rs，那邊改了這裡要跟著改），
# 加上 rime-essay 用 OpenCC 標準寫法、CC-CEDICT 用常見寫法的字。
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
    has_entry = has_gloss = 0
    for word in keys:
        senses = [s for d in dicts for s in d.get(word, [])]
        if senses:
            has_entry += 1
            has_gloss += committable(senses) is not None
    n = len(keys)
    print(f"{name}\tn={n}\t有條目 {has_entry} ({has_entry / n:.1%})\t可直接上屏 {has_gloss} ({has_gloss / n:.1%})")


def main(cedict_path, canto_path):
    essay = load_essay()
    words = [w for w, n in essay.most_common() if len(w) >= 2 and n >= 100]
    chars = [w for w, _ in essay.most_common() if len(w) == 1]
    cedict, canto = load_dict(cedict_path), load_dict(canto_path)
    print(f"essay 詞 {len(words)}、單字 {len(chars)}；CC-CEDICT 詞頭 {len(cedict)}、CC-Canto 詞頭 {len(canto)}")
    for top in (5000, 20000, 50000, len(words)):
        report(f"essay 詞前 {top}：CC-CEDICT", words[:top], [cedict])
        report(f"essay 詞前 {top}：CC-CEDICT+CC-Canto", words[:top], [cedict, canto])
    for top in (1000, 3000, 6000):
        report(f"單字前 {top}：CC-CEDICT", chars[:top], [cedict])
    with open("data/lexicon/yue-colloquial.tsv", encoding="utf-8") as f:
        yue = [norm(l.split("\t")[0]) for l in f if l.strip() and not l.startswith("#")]
    report("粵語口語小詞表：CC-CEDICT", yue, [cedict])
    report("粵語口語小詞表：CC-CEDICT+CC-Canto", yue, [cedict, canto])
    words_hk = set()
    with open(cached("words_hk/*/wordslist.csv"), encoding="utf-8") as f:
        for line in f:
            w = norm(line.split(",")[0].replace("*", ""))
            if len(w) >= 2 and HAN.match(w) and w not in essay:
                words_hk.add(w)
    report("粵典詞頭（essay 沒有）：CC-CEDICT+CC-Canto", sorted(words_hk), [cedict, canto])

    print("\n單字前 20 的第一個可上屏釋義（CC-CEDICT 依拼音排列，不依常用度）：")
    print(" ".join(f"{c}={committable(cedict[c])}" for c in chars[:20]))

    print("\n抽查樣本（可直接上屏的詞，前 5000 與 5000–20000 各 50，種子 2020）：")
    rng = random.Random(2020)
    for lo, hi in ((0, 5000), (5000, 20000)):
        glossed = ((w, committable(cedict[w])) for w in words[lo:hi] if w in cedict)
        pool = [(w, g) for w, g in glossed if g]
        print(f"[{lo}–{hi}] " + "  ".join(f"{w}={g}" for w, g in rng.sample(pool, 50)))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: coverage.py <cedict.txt> <cccanto-webdist.txt>")
    main(sys.argv[1], sys.argv[2])
