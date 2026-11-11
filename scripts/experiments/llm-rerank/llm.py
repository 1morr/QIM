"""LLM 重排實驗的工具（M3 票 10，README〈怎麼重跑〉）。只用標準函式庫。

  python3 llm.py prompts --style one --choices merge <要求.jsonl>… -o <提示.jsonl> [--sample N]
  python3 llm.py cloud --model deepseek-flash <提示.jsonl> <回覆.jsonl> [--workers 4]
  python3 llm.py orders <提示.jsonl> <回覆.jsonl> -o <結果.jsonl> [--deadline-ms 3000]
  python3 llm.py stats <回覆.jsonl>… [--model deepseek-flash]
  python3 llm.py tail-stats <回覆.jsonl>…
  python3 llm.py estimate <提示.jsonl>…
  python3 llm.py subset <提示.jsonl> <逐題結果目錄 A> <逐題結果目錄 B>

要求檔是 `model-ladder export` 的輸出；結果檔交給 `model-ladder eval --orders`。
Foundation Models 的回覆由 `fm/fm-rerank.swift` 產生，格式與 `cloud` 相同。
"""

import argparse
import json
import math
import os
import random
import re
import sys
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# 提示的寫法（README〈提示〉）。每種一段 instructions（system）加同一個格式的 user 訊息；
# `ask` 決定要模型回一個還是兩個序號。
BASE_ONE = (
    "你是中文輸入法的候選排序器。根據上下文，從候選中選出最可能接在上下文後面的一個，"
    "只回答它的序號（阿拉伯數字），不要其他文字。"
)
HINT = (
    "你是速成輸入法的候選排序器。使用者用速成碼打了一句話，候選是同一串輸入碼的不同轉換，"
    "內容可能是粵語口語或書面中文，通常是香港繁體字。根據上下文與語感，"
)
STYLES = {
    # M2 票 10 在 iPhone 上用的寫法（`ios-keyboard/Keyboard/ModelProbe.swift`）。
    "one": {"instructions": BASE_ONE},
    "two": {
        "instructions": (
            "你是中文輸入法的候選排序器。根據上下文，從候選中選出最可能接在上下文後面的兩個，"
            "最可能的在前，只回答兩個序號（阿拉伯數字，以逗號分隔），不要其他文字。"
        )
    },
    "hint": {
        "instructions": HINT
        + "選出使用者最可能想打的一個，只回答它的序號（阿拉伯數字），不要其他文字。"
    },
    "hint2": {
        "instructions": HINT
        + "選出使用者最可能想打的兩個，最可能的在前，只回答兩個序號（阿拉伯數字，以逗號分隔），"
        "不要其他文字。"
    },
}

# 每百萬 token 的人民幣價，高峰時段（spec〈採用門檻〉照高峰價）。DeepSeek 2026-08-16 起的價格，
# https://api-docs.deepseek.com/zh-cn/quick_start/pricing ，2026-11-11 查（brief §12.4）。
PRICES = {
    "deepseek-flash": {"hit": 0.04, "miss": 2.0, "out": 8.0},
    "deepseek-v4-pro": {"hit": 0.30, "miss": 9.0, "out": 27.0},
}
ENDPOINT = "https://api.deepseek.com/chat/completions"
PERSONAL = "personal"


def read_jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def write_jsonl(path, rows):
    with open(path, "w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")


def key(row):
    return (row["split"], row["set"], row["line"])


def choices(query, mode, limit):
    """給模型看的選項。`cand`：候選列第一頁，依核心的順序。`merge`：lattice 路徑在前（整串的
    備選），再補沒出現過的候選；第一頁只有 3 條整句，其餘是開頭幾個字的短詞。"""
    if mode == "cand":
        texts = list(query["candidates"])
    elif mode == "merge":
        texts = [p["text"] for p in query["paths"]] + list(query["candidates"])
    else:
        raise ValueError(mode)
    seen = []
    for t in texts:
        if t not in seen:
            seen.append(t)
    return seen[:limit]


def build_prompt(query, style, mode, limit=10):
    """一題的提示：上下文 + 輸入碼 + 編號的選項（spec〈模型階梯實驗〉的提示）。"""
    texts = choices(query, mode, limit)
    numbered = " ".join(f"{i}.{t}" for i, t in enumerate(texts, 1))
    before = query["before"] or "（無）"
    prompt = f"上下文：{before}\n輸入碼：{query['input']}\n候選：{numbered}\n序號："
    return {
        "split": query["split"],
        "set": query["set"],
        "line": query["line"],
        "instructions": style["instructions"],
        "prompt": prompt,
        "choices": texts,
    }


FULLWIDTH = str.maketrans("０１２３４５６７８９", "0123456789")


def parse_reply(reply, texts):
    """回覆裡的序號依出現的順序換回文字；超出範圍的、重複的不算。"""
    ranked = []
    for m in re.findall(r"\d+", reply.translate(FULLWIDTH)):
        i = int(m)
        if 1 <= i <= len(texts) and texts[i - 1] not in ranked:
            ranked.append(texts[i - 1])
    return ranked


def orders(prompts, replies, deadline_ms=None):
    """回覆換成 `model-ladder eval --orders` 的結果行。出錯、沒有序號的題不寫（照原順序）；
    給 `deadline_ms` 時超過它的也不寫（產品裡等不到的結果會丟掉）。"""
    by_key = {key(p): p for p in prompts}
    out = []
    for r in replies:
        p = by_key.get(key(r))
        if p is None or r.get("error"):
            continue
        if deadline_ms is not None and r["ms"] > deadline_ms:
            continue
        ranked = parse_reply(r["reply"], p["choices"])
        if ranked:
            out.append({"split": r["split"], "set": r["set"], "line": r["line"], "ranked": ranked})
    return out


def percentile(values, p):
    """最近序位法，同 `model-ladder` 的 `percentile`。"""
    s = sorted(values)
    if not s:
        return None
    rank = max(1, -(-p * len(s) // 100))
    return s[rank - 1]


def cost_per_thousand(usages, model):
    """每千次請求的人民幣費用（高峰價）。"""
    price = PRICES[model]
    total = sum(
        u.get("prompt_cache_hit_tokens", 0) * price["hit"]
        + u.get("prompt_cache_miss_tokens", 0) * price["miss"]
        + u.get("completion_tokens", 0) * price["out"]
        for u in usages
    )
    return total / 1e6 / len(usages) * 1000


def check_personal(rows, allow):
    """個人集不送雲端，除非使用者這次同意（plan §5.2）。"""
    if not allow and any(r["set"] == PERSONAL for r in rows):
        sys.exit("提示裡有個人集：要送雲端得先問使用者，同意後加 --allow-personal")


def mcnemar_exact(b, c):
    """McNemar 精確檢定（雙尾），同 `model-ladder` 的 `mcnemar_exact`。"""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / 2**n
    return min(1.0, 2 * tail)


def read_rows(path):
    with open(path, encoding="utf-8") as f:
        lines = f.read().splitlines()
    cols = lines[0].split("\t")
    return {int(r[0]): dict(zip(cols, r)) for r in (line.split("\t") for line in lines[1:])}


def subset(prompts, dir_a, dir_b):
    """兩個逐題結果目錄（`model-ladder eval` 寫的）只比提示檔裡的題：前 2 名與可動首選。"""
    rows = {}
    out = {"total": 0, "top2_a": 0, "top2_b": 0, "first_a": 0, "first_b": 0, "better": 0, "worse": 0}
    for p in prompts:
        for side, d in (("a", dir_a), ("b", dir_b)):
            if (side, p["set"]) not in rows:
                rows[(side, p["set"])] = read_rows(Path(d, f"{p['set']}.tsv"))
        ra, rb = rows[("a", p["set"])][p["line"]], rows[("b", p["set"])][p["line"]]
        ta, tb = ra["top2"] == "1", rb["top2"] == "1"
        out["total"] += 1
        out["top2_a"] += ta
        out["top2_b"] += tb
        out["first_a"] += ra["model_first"] == "1"
        out["first_b"] += rb["model_first"] == "1"
        out["better"] += tb and not ta
        out["worse"] += ta and not tb
    return out


# --- 子指令 ---


def cmd_prompts(a):
    queries = [q for path in a.requests for q in read_jsonl(path)]
    if a.sample:
        queries = random.Random(a.seed).sample(queries, min(a.sample, len(queries)))
        queries.sort(key=key)
    style = STYLES[a.style]
    write_jsonl(a.output, [build_prompt(q, style, a.choices, a.limit) for q in queries])
    print(f"{len(queries)} 題 → {a.output}", file=sys.stderr)


def call(model, prompt, timeout, max_tokens):
    body = {
        "model": model,
        "messages": [
            {"role": "system", "content": prompt["instructions"]},
            {"role": "user", "content": prompt["prompt"]},
        ],
        "thinking": {"type": "disabled"},
        "temperature": 0,
        "max_tokens": max_tokens,
        "stream": False,
    }
    req = urllib.request.Request(
        ENDPOINT,
        data=json.dumps(body).encode(),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ['DEEPSEEK_API_KEY']}",
        },
    )
    out = {"split": prompt["split"], "set": prompt["set"], "line": prompt["line"]}
    start = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            data = json.load(resp)
        out["reply"] = data["choices"][0]["message"].get("content") or ""
        out["usage"] = data.get("usage", {})
    except (urllib.error.URLError, TimeoutError, KeyError, ValueError) as e:
        out["reply"] = ""
        out["error"] = f"{type(e).__name__}: {e}"
    out["ms"] = round((time.perf_counter() - start) * 1000, 1)
    return out


def cmd_cloud(a):
    if not os.environ.get("DEEPSEEK_API_KEY"):
        sys.exit("沒有 DEEPSEEK_API_KEY：跳過雲端（spec〈共同規則〉）")
    prompts = read_jsonl(a.prompts)
    check_personal(prompts, a.allow_personal)
    done = set()
    if Path(a.replies).exists():
        done = {key(r) for r in read_jsonl(a.replies) if not r.get("error")}
    todo = [p for p in prompts if key(p) not in done]
    print(f"送 {len(todo)} 題（已有 {len(done)}）", file=sys.stderr)
    with open(a.replies, "a", encoding="utf-8") as f, ThreadPoolExecutor(a.workers) as pool:
        for n, r in enumerate(
            pool.map(lambda p: call(a.model, p, a.timeout, a.max_tokens), todo), 1
        ):
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
            f.flush()
            if n % 200 == 0:
                print(f"{n} / {len(todo)}", file=sys.stderr)


def cmd_orders(a):
    rows = orders(read_jsonl(a.prompts), read_jsonl(a.replies), a.deadline_ms)
    write_jsonl(a.output, rows)
    print(f"{len(rows)} 題有結果 → {a.output}", file=sys.stderr)


def fmt(v):
    return "" if v is None else f"{v:.0f}"


def cmd_stats(a):
    replies = [r for path in a.replies for r in read_jsonl(path)]
    groups = {}
    for r in replies:
        groups.setdefault(r["set"], []).append(r)
    groups["合計"] = replies
    head = "| 集合 | 請求 | 出錯 | 沒有序號 | p50 ms | p95 ms | 最慢 ms | > 2 s | > 3 s |"
    if a.model:
        head += " 每千次（元） |"
    print(head)
    print("| --- |" + " ---: |" * (head.count("|") - 2))
    for name, rows in groups.items():
        ok = [r for r in rows if not r.get("error")]
        ms = [r["ms"] for r in rows]
        no_number = sum(1 for r in ok if not re.search(r"\d", r["reply"].translate(FULLWIDTH)))
        line = (
            f"| {name} | {len(rows)} | {len(rows) - len(ok)} | {no_number} | "
            f"{fmt(percentile(ms, 50))} | {fmt(percentile(ms, 95))} | {fmt(max(ms))} | "
            f"{sum(m > 2000 for m in ms) / len(ms):.1%} | {sum(m > 3000 for m in ms) / len(ms):.1%} |"
        )
        if a.model:
            usages = [r["usage"] for r in ok if r.get("usage")]
            line += f" {cost_per_thousand(usages, a.model):.3f} |" if usages else " |"
        print(line)


def cmd_tail_stats(a):
    """`fm-rerank tail` 的結果：依停頓秒數與停頓後第幾個請求分組。"""
    rows = [r for path in a.replies for r in read_jsonl(path)]
    print("| 停頓 | 第幾個 | 次數 | p50 ms | p95 ms | 最慢 ms | > 2 s |")
    print("| ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    cells = {}
    for r in rows:
        cells.setdefault((r["gap"], r["burstIndex"]), []).append(r["ms"])
    for (gap, k), ms in sorted(cells.items()):
        print(
            f"| {gap} s | {k + 1} | {len(ms)} | {fmt(percentile(ms, 50))} | "
            f"{fmt(percentile(ms, 95))} | {fmt(max(ms))} | {sum(m > 2000 for m in ms)} |"
        )
    first = [r["ms"] for r in rows if r["burstIndex"] == 0]
    if first:
        over = sum(m > 2000 for m in first)
        print(f"\n停頓後第一個請求：{len(first)} 次，超過 2 s 的 {over} 次（{over / len(first):.1%}）")


def cmd_estimate(a):
    """估計請求的字數（給使用者估費用）：DeepSeek 說 1 個中文字約 0.6 token，這裡用 1 字 1 token
    的保守值另列一欄。"""
    prompts = [p for path in a.prompts for p in read_jsonl(path)]
    chars = [len(p["instructions"]) + len(p["prompt"]) for p in prompts]
    total = sum(chars)
    print(f"{len(prompts)} 題，平均 {total / len(prompts):.0f} 字，合計 {total} 字")
    for model, price in PRICES.items():
        for per_char in (0.6, 1.0):
            yuan = total * per_char * price["miss"] / 1e6 + len(prompts) * 4 * price["out"] / 1e6
            print(f"{model}（每字 {per_char} token、全算 cache miss、高峰價）：約 {yuan:.2f} 元")


def cmd_subset(a):
    prompts = read_jsonl(a.prompts)
    groups = {"合計": prompts}
    for p in prompts:
        groups.setdefault(p["set"], []).append(p)
    print("| 集合 | 題數 | 前 2 名 A | 前 2 名 B | 好/壞 | p | 可動首選 A | 可動首選 B |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for name, ps in groups.items():
        r = subset(ps, a.a, a.b)
        n = r["total"]
        print(
            f"| {name} | {n} | {r['top2_a'] / n:.1%} | {r['top2_b'] / n:.1%} | "
            f"{r['better']}/{r['worse']} | {mcnemar_exact(r['better'], r['worse']):.3f} | "
            f"{r['first_a'] / n:.1%} | {r['first_b'] / n:.1%} |"
        )


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawTextHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("prompts")
    p.add_argument("requests", nargs="+")
    p.add_argument("-o", "--output", required=True)
    p.add_argument("--style", choices=STYLES, default="one")
    p.add_argument("--choices", choices=["cand", "merge"], default="merge")
    p.add_argument("--limit", type=int, default=10)
    p.add_argument("--sample", type=int)
    p.add_argument("--seed", type=int, default=1)
    p.set_defaults(func=cmd_prompts)
    p = sub.add_parser("cloud")
    p.add_argument("prompts")
    p.add_argument("replies")
    p.add_argument("--model", choices=PRICES, required=True)
    p.add_argument("--workers", type=int, default=4)
    p.add_argument("--timeout", type=float, default=30)
    p.add_argument("--max-tokens", type=int, default=8)
    p.add_argument("--allow-personal", action="store_true")
    p.set_defaults(func=cmd_cloud)
    p = sub.add_parser("orders")
    p.add_argument("prompts")
    p.add_argument("replies")
    p.add_argument("-o", "--output", required=True)
    p.add_argument("--deadline-ms", type=float)
    p.set_defaults(func=cmd_orders)
    p = sub.add_parser("stats")
    p.add_argument("replies", nargs="+")
    p.add_argument("--model", choices=PRICES)
    p.set_defaults(func=cmd_stats)
    p = sub.add_parser("tail-stats")
    p.add_argument("replies", nargs="+")
    p.set_defaults(func=cmd_tail_stats)
    p = sub.add_parser("estimate")
    p.add_argument("prompts", nargs="+")
    p.set_defaults(func=cmd_estimate)
    p = sub.add_parser("subset")
    p.add_argument("prompts")
    p.add_argument("a")
    p.add_argument("b")
    p.set_defaults(func=cmd_subset)
    a = ap.parse_args(argv)
    a.func(a)


if __name__ == "__main__":
    main()
