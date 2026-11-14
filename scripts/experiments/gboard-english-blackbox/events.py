#!/usr/bin/env python3
"""印 server.py 記下的 input 事件：不是單一字元 insertText 的每一筆，最後印欄位的值，
游標寫成 '|'。

用法：events.py [記錄檔]   （預設 ./events.log）
"""
import json
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "events.log"
rows = [json.loads(line.split(" ", 1)[1]) for line in open(path)]


def shown(d):
    v, s = d["value"], d["sel"]
    return repr(v if s is None else v[:s] + "|" + v[s:])


for d in rows:
    if d["type"] != "input":
        continue
    if d["it"] != "insertText" or (d["data"] and len(d["data"]) > 1):
        print("  ", d["it"], repr(d["data"]), shown(d))
if rows:
    print("FINAL", shown(rows[-1]))
