"""記住的字串要打幾個字母才給（M3 票 15）：常見 email 開頭的前 N 個字母和中文輸入碼撞上的比例。

讀首選字基準的題目（輸入碼以空白分字、Tab、答案），每題當一次組字；另把每個字的開頭都當成組字的開頭
（使用者逐字、逐詞打時的上限）。組字的輸入碼以 email 英文字母的前 N 個開頭時，那次組字會多出一個候選。

    python3 -I scripts/experiments/remembered-prefix/collide.py bench/corpus/*.tsv bench/colloquial/*.tsv
"""

import sys

# 香港常見的 email 開頭：姓氏、英文名與常用的帳號名。
NAMES = (
    "chan wong lee cheung lau ho ivan peter mary john info admin test hello "
    "kenneth chris tony kelly sam alex"
).split()


def main(paths):
    codes = []
    for path in paths:
        for line in open(path, encoding="utf-8"):
            if line.startswith("#") or "\t" not in line:
                continue
            codes.append(line.split("\t")[0].split(" "))
    sentences = ["".join(c) for c in codes]
    starts = ["".join(c[i:]) for c in codes for i in range(len(c))]
    print(f"題數 {len(sentences)}，字的開頭 {len(starts)}")
    print("| N | 每題平均 | 每個字的開頭平均 | 最常撞的開頭 |")
    print("| ---: | ---: | ---: | --- |")
    for n in range(2, 6):
        rows = []
        for name in NAMES:
            prefix = name[:n]
            if len(prefix) < n:
                continue
            per_sentence = sum(s.startswith(prefix) for s in sentences) / len(sentences)
            per_start = sum(s.startswith(prefix) for s in starts) / len(starts)
            rows.append((name, per_sentence, per_start))
        mean_sentence = sum(r[1] for r in rows) / len(rows)
        mean_start = sum(r[2] for r in rows) / len(rows)
        worst = max(rows, key=lambda r: r[2])
        print(
            f"| {n} | {mean_sentence:.4%} | {mean_start:.4%} | {worst[0]}（{worst[2]:.4%}） |"
        )


if __name__ == "__main__":
    main(sys.argv[1:])
