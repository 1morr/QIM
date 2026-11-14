#!/usr/bin/env python3
"""Print the input events logged by server.py: every event that is not a one-character
insertText, then the final field value with the caret shown as '|'.

usage: events.py [log-path]   (default ./events.log)
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
