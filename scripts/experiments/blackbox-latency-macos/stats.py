"""把 lat 的輸出（每行「鍵 毫秒」）算成 p50 / p95；用法：python3 stats.py <輸出檔>。"""

import statistics
import sys


def report(name, values):
    values = sorted(values)
    p95 = values[int(0.95 * (len(values) - 1))]
    print(f"{name}: n={len(values)} p50={statistics.median(values):.2f} p95={p95:.2f} max={values[-1]:.2f} ms")


rows = [line.split() for line in open(sys.argv[1], encoding="utf-8") if line.strip() and not line.startswith("trusted")]
letters = [float(ms) for key, ms in rows if key != "␣"]
spaces = [float(ms) for key, ms in rows if key == "␣"]
report("字母鍵", letters)
report("空白", spaces)
