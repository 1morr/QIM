# /// script
# requires-python = ">=3.12"
# dependencies = ["torch==2.15.0", "safetensors==0.8.0", "numpy==2.5.4"]
# ///
"""停鍵層的字級神經模型的訓練（M3 票 09 寫、票 11d 搬來；docs/development.md〈停鍵層的模型〉）。

  uv run --script train.py prepare               # 語料 → 字表與 token 檔（build/neural-rerank/data/）
  uv run --script train.py train --model lstm    # Gboard 級 CIFG-LSTM，約 1.4M 參數（採用的停鍵層）
  uv run --script train.py train --model gpt     # 青簡級 GPT-2 式 decoder，約 23M 參數（只給模型階梯）
  uv run --script train.py fixture               # qim-neural 單元測試用的極小模型與 PyTorch 分數

結構與權重名照 `scripts/experiments/neural-cost/src/model.rs`（M2 票 11 量開銷的那兩個模型）。
產品的推論是 `crates/qim-neural`（只有 LSTM）；`cargo xtask neural-import` 核對 `reference.json`、
量化後放進 `data/neural-rerank/`。模型階梯的 candle 程式（`scripts/experiments/model-ladder`）也讀
`model.safetensors`。
"""

from __future__ import annotations

import argparse
import json
import math
import platform
import subprocess
import time
from collections import Counter
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from safetensors.torch import save_file

ROOT = Path(__file__).resolve().parents[2]
BUILD = ROOT / "build" / "neural-rerank"
WIKI = ["zh-wiki", "yue-wiki"]
COLLOQUIAL = ["common-voice-yue", "hkcancor"]
# 句首與子句之間的記號（語料在標點處切句，標點本身不在 train.txt 裡）；前文裡的標點也換成它。
SEP, UNK = 0, 1
SPECIALS = ["<sep>", "<unk>"]
# 和 neural-cost 的形狀相同：LSTM 字表 1 萬、GPT 8000（含上面兩個記號）。
VOCAB = {"lstm": 10_000, "gpt": 8_000}
CONTEXT = 256
# 評估句幾個字以上才從訓練語料剔除（docs/research/neural-rerank.md §5：≥5 字，短語不剔）。
EXCLUDE_MIN_CHARS = 5


# ---------------------------------------------------------------- prepare


def eval_sentences() -> set[str]:
    """所有評估題的答案句（基準、dev 調參題、長前文題、個人集），≥ EXCLUDE_MIN_CHARS 字的。"""
    files = list((ROOT / "bench" / "corpus").glob("**/*.tsv"))
    files += list((ROOT / "bench" / "colloquial").glob("**/*.tsv"))
    files += list((ROOT / "build" / "corpus").glob("*/*.tsv"))
    out: set[str] = set()
    for f in files:
        if f.name == "chars.tsv":
            continue
        for line in f.read_text(encoding="utf-8").splitlines():
            out.add(line.split("\t")[-1])
    personal = ROOT / "bench" / "personal" / "rime-userdb.tsv"
    if personal.exists():
        for line in personal.read_text(encoding="utf-8").splitlines():
            if not line.startswith("#"):
                out.add(line.split("\t")[1])
    return {s for s in out if len(s) >= EXCLUDE_MIN_CHARS}


def prepare(args: argparse.Namespace) -> None:
    out = BUILD / "data"
    out.mkdir(parents=True, exist_ok=True)
    exclude = eval_sentences()
    lines: dict[str, list[str]] = {}
    dropped: dict[str, int] = {}
    for name in WIKI + COLLOQUIAL:
        raw = (ROOT / "build" / "corpus" / name / "train.txt").read_text(encoding="utf-8").splitlines()
        kept = [l for l in raw if l and l not in exclude]
        lines[name] = kept
        dropped[name] = len(raw) - len(kept)
    # 字表依字次數排；口語乘上權重（同 bigram 的語料權重，plan §4.3），口語常用字不會掉出字表。
    counts: Counter[str] = Counter()
    for name, ls in lines.items():
        w = args.colloquial_weight if name in COLLOQUIAL else 1
        c = Counter("".join(ls))
        for ch, n in c.items():
            counts[ch] += n * w
    chars = [ch for ch, _ in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))]
    vocab = SPECIALS + chars
    (out / "vocab.txt").write_text("\n".join(vocab) + "\n", encoding="utf-8")
    index = {ch: i for i, ch in enumerate(vocab)}
    stats = {}
    for name, ls in lines.items():
        ids: list[int] = []
        starts: list[int] = []
        for l in ls:
            starts.append(len(ids))
            ids.append(SEP)
            ids.extend(index[ch] for ch in l)
        ids.append(SEP)
        np.save(out / f"tokens-{name}.npy", np.asarray(ids, dtype=np.uint16))
        np.save(out / f"starts-{name}.npy", np.asarray(starts, dtype=np.int64))
        dev = (ROOT / "build" / "corpus" / name / "dev.txt").read_text(encoding="utf-8").splitlines()
        dev_ids: list[int] = []
        for l in dev:
            dev_ids.append(SEP)
            dev_ids.extend(index.get(ch, UNK) for ch in l)
            if len(dev_ids) >= args.dev_tokens:
                break
        np.save(out / f"dev-{name}.npy", np.asarray(dev_ids[: args.dev_tokens], dtype=np.uint16))
        stats[name] = {"lines": len(ls), "tokens": len(ids), "excluded_eval_lines": dropped[name]}
    meta = {
        "chars": len(chars),
        "colloquial_weight": args.colloquial_weight,
        "exclude_min_chars": EXCLUDE_MIN_CHARS,
        "eval_sentences": len(exclude),
        "corpora": stats,
    }
    (out / "meta.json").write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(meta, ensure_ascii=False, indent=2))


# ---------------------------------------------------------------- models


class CifgLstm(nn.Module):
    """Gboard 的 CIFG-LSTM：輸入閘與遺忘閘共用（f = 1 − i），cell 輸出投影回 embedding 維度，
    輸入輸出共用 embedding。閘的順序 [i, o, g]、輸入 [x, h]，同 neural-cost 的 `Lstm::step`。"""

    def __init__(self, vocab: int, embed: int = 96, cells: int = 670):
        super().__init__()
        self.embed_dim, self.cells = embed, cells
        self.embed = nn.Parameter(torch.randn(vocab, embed) * 0.02)
        self.gates = nn.Linear(2 * embed, 3 * cells)
        self.proj = nn.Linear(cells, embed, bias=False)

    def forward(self, tokens: torch.Tensor) -> torch.Tensor:
        b, t = tokens.shape
        e, n = self.embed_dim, self.cells
        x = F.embedding(tokens, self.embed)
        wx, wh = self.gates.weight[:, :e], self.gates.weight[:, e:]
        xg = F.linear(x, wx, self.gates.bias)  # 輸入那半一次算完
        h = x.new_zeros(b, e)
        c = x.new_zeros(b, n)
        outs = []
        # unbind：切片 xg[:, s] 的反向傳播每步都配一個整段大小的梯度，batch 一大就慢好幾倍。
        for xs in xg.unbind(1):
            z = xs + h @ wh.T
            i = torch.sigmoid(z[:, :n])
            o = torch.sigmoid(z[:, n : 2 * n])
            g = torch.tanh(z[:, 2 * n :])
            c = (1 - i) * c + i * g
            h = self.proj(o * torch.tanh(c))
            outs.append(h)
        return torch.stack(outs, 1) @ self.embed.T

    def export(self) -> dict[str, torch.Tensor]:
        return {
            "embed": self.embed,
            "gates.weight": self.gates.weight,
            "gates.bias": self.gates.bias,
            "proj.weight": self.proj.weight,
        }


class Block(nn.Module):
    def __init__(self, d: int, heads: int, dropout: float):
        super().__init__()
        self.heads, self.dropout = heads, dropout
        self.ln_1 = nn.LayerNorm(d, eps=1e-5)
        self.c_attn = nn.Linear(d, 3 * d)
        self.c_proj = nn.Linear(d, d)
        self.ln_2 = nn.LayerNorm(d, eps=1e-5)
        self.c_fc = nn.Linear(d, 4 * d)
        self.mlp_proj = nn.Linear(4 * d, d)
        self.drop = nn.Dropout(dropout)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        b, t, d = x.shape
        q, k, v = self.c_attn(self.ln_1(x)).split(d, dim=2)
        q, k, v = (z.view(b, t, self.heads, d // self.heads).transpose(1, 2) for z in (q, k, v))
        y = F.scaled_dot_product_attention(
            q, k, v, is_causal=True, dropout_p=self.dropout if self.training else 0.0
        )
        x = x + self.drop(self.c_proj(y.transpose(1, 2).reshape(b, t, d)))
        # candle 的 `gelu()` 是 tanh 近似。
        return x + self.drop(self.mlp_proj(F.gelu(self.c_fc(self.ln_2(x)), approximate="tanh")))


class Gpt(nn.Module):
    """GPT-2 式 decoder：pre-LayerNorm、學到的位置 embedding、輸入輸出共用 embedding；寬 512、6 層、
    8 頭、上下文 256，同 neural-cost 的 `GPT`。"""

    def __init__(self, vocab: int, d: int = 512, layers: int = 6, heads: int = 8, dropout: float = 0.0):
        super().__init__()
        self.wte = nn.Parameter(torch.randn(vocab, d) * 0.02)
        self.wpe = nn.Parameter(torch.randn(CONTEXT, d) * 0.01)
        self.h = nn.ModuleList(Block(d, heads, dropout) for _ in range(layers))
        self.ln_f = nn.LayerNorm(d, eps=1e-5)
        for name, p in self.named_parameters():
            if name.endswith("c_proj.weight") or name.endswith("mlp_proj.weight"):
                nn.init.normal_(p, std=0.02 / math.sqrt(2 * layers))
            elif p.dim() == 2 and name not in ("wte", "wpe"):
                nn.init.normal_(p, std=0.02)
            elif name.endswith(".bias"):
                nn.init.zeros_(p)

    def forward(self, tokens: torch.Tensor) -> torch.Tensor:
        t = tokens.shape[1]
        x = F.embedding(tokens, self.wte) + self.wpe[:t]
        for block in self.h:
            x = block(x)
        return self.ln_f(x) @ self.wte.T

    def export(self) -> dict[str, torch.Tensor]:
        out = {"wte": self.wte, "wpe": self.wpe, "ln_f.weight": self.ln_f.weight, "ln_f.bias": self.ln_f.bias}
        for l, b in enumerate(self.h):
            p = f"h.{l}"
            for ours, theirs in [
                ("ln_1", "ln_1"),
                ("c_attn", "attn.c_attn"),
                ("c_proj", "attn.c_proj"),
                ("ln_2", "ln_2"),
                ("c_fc", "mlp.c_fc"),
                ("mlp_proj", "mlp.c_proj"),
            ]:
                m = getattr(b, ours)
                out[f"{p}.{theirs}.weight"] = m.weight
                out[f"{p}.{theirs}.bias"] = m.bias
        return out


# ---------------------------------------------------------------- train


class Mixture:
    """依比例從各語料取視窗：每個視窗從一句的開頭（<sep>）起、長 CONTEXT + 1。"""

    def __init__(self, data: Path, vocab: int, colloquial_frac: float, seed: int):
        self.rng = np.random.default_rng(seed)
        self.vocab = vocab
        self.tokens = {n: np.load(data / f"tokens-{n}.npy") for n in WIKI + COLLOQUIAL}
        self.starts = {}
        for n, t in self.tokens.items():
            s = np.load(data / f"starts-{n}.npy")
            self.starts[n] = s[s + CONTEXT + 1 <= len(t)]
        wiki = sum(len(self.tokens[n]) for n in WIKI)
        coll = sum(len(self.tokens[n]) for n in COLLOQUIAL)
        if colloquial_frac < 0:  # 照自然比例
            colloquial_frac = coll / (wiki + coll)
        self.names = WIKI + COLLOQUIAL
        self.probs = np.array(
            [(1 - colloquial_frac) * len(self.tokens[n]) / wiki for n in WIKI]
            + [colloquial_frac * len(self.tokens[n]) / coll for n in COLLOQUIAL]
        )
        self.colloquial_frac = colloquial_frac

    def batch(self, size: int) -> torch.Tensor:
        picks = self.rng.choice(len(self.names), size=size, p=self.probs)
        rows = []
        for k in picks:
            n = self.names[k]
            s = self.starts[n][self.rng.integers(len(self.starts[n]))]
            rows.append(self.tokens[n][s : s + CONTEXT + 1])
        x = torch.from_numpy(np.stack(rows).astype(np.int64))
        return torch.where(x >= self.vocab, UNK, x)


def dev_losses(model: nn.Module, data: Path, vocab: int, device: torch.device) -> dict[str, float]:
    """各語料 dev 切分開頭的一段（prepare 的 --dev-tokens），不重疊的視窗，每字平均交叉熵（nat）。"""
    model.eval()
    out = {}
    with torch.no_grad():
        for n in WIKI + COLLOQUIAL:
            ids = torch.from_numpy(np.load(data / f"dev-{n}.npy").astype(np.int64))
            ids = torch.where(ids >= vocab, UNK, ids)
            w = (len(ids) - 1) // CONTEXT
            x = ids[: w * CONTEXT + 1]
            inp = x[:-1].view(w, CONTEXT).to(device)
            tgt = x[1:].view(w, CONTEXT).to(device)
            total = 0.0
            for i in range(0, w, 64):
                logits = model(inp[i : i + 64]).float()
                total += F.cross_entropy(logits.reshape(-1, vocab), tgt[i : i + 64].reshape(-1), reduction="sum").item()
            out[n] = total / (w * CONTEXT)
    model.train()
    return out


def reference(model: nn.Module, data: Path, vocab: int) -> list[dict]:
    """幾組固定的「前文 + 路徑」與 PyTorch 算的分數（路徑各字 log 機率的和），給 candle 那邊核對
    （`model-ladder neural` 讀到就比）。前文、路徑取 dev 的字，路徑長度不一。"""
    model = model.float().cpu().eval()
    ids = np.load(data / "dev-hkcancor.npy").astype(np.int64)
    ids = np.where(ids >= vocab, UNK, ids)
    out = []
    with torch.no_grad():
        for ctx_len, path_len, at in [(0, 5, 100), (12, 3, 500), (60, 9, 2000), (200, 18, 4000)]:
            ctx = [int(t) for t in ids[at : at + ctx_len] if t != SEP]
            path = [int(t) for t in ids[at + ctx_len : at + ctx_len + 40] if t not in (SEP, UNK)][:path_len]
            seq = torch.tensor([[SEP] + ctx + path])
            lp = F.log_softmax(model(seq[:, :-1]).float(), -1)[0]
            start = 1 + len(ctx) - 1
            score = sum(lp[start + i, t].item() for i, t in enumerate(path))
            out.append({"context": ctx, "path": path, "score": score})
    return out


def fixture(args: argparse.Namespace) -> None:
    """qim-neural 單元測試用：形狀很小的隨機 CIFG-LSTM 與 PyTorch 算的分數，核對 candle 的推論
    （`crates/qim-neural/tests/fixtures/`）。權重放大一些，分數才不會每條都差不多。embedding 寬 32，
    量化成 q8 時 embedding 與閘會量化、投影不會（內層維度要是 32 的倍數），同真的模型。"""
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    torch.manual_seed(args.seed)
    vocab = 12
    model = CifgLstm(vocab, embed=32, cells=8)
    with torch.no_grad():
        for p in model.parameters():
            p.copy_(torch.randn_like(p) * 0.5)
    model.eval()
    rng = np.random.default_rng(args.seed)
    cases = []
    with torch.no_grad():
        for ctx_len, path_len in [(0, 1), (0, 5), (3, 2), (7, 9), (20, 4)]:
            # 前文不含句首記號（推論端自己加），路徑不含記號。
            ctx = [int(t) for t in rng.integers(1, vocab, ctx_len)]
            path = [int(t) for t in rng.integers(2, vocab, path_len)]
            seq = torch.tensor([[SEP] + ctx + path])
            lp = F.log_softmax(model(seq[:, :-1]).float(), -1)[0]
            score = sum(lp[len(ctx) + i, t].item() for i, t in enumerate(path))
            cases.append({"context": ctx, "path": path, "score": score})
    tensors = {k: v.detach().float().contiguous() for k, v in model.export().items()}
    save_file(tensors, out / "tiny-lstm.safetensors")
    (out / "tiny-lstm-reference.json").write_text(json.dumps(cases, indent=1) + "\n", encoding="utf-8")
    print(f"寫到 {out}")


def git_commit() -> str:
    return subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def train(args: argparse.Namespace) -> None:
    data = BUILD / "data"
    out = BUILD / (args.name or args.model)
    out.mkdir(parents=True, exist_ok=True)
    torch.manual_seed(args.seed)
    device = torch.device("mps" if torch.backends.mps.is_available() else "cpu")
    vocab = VOCAB[args.model]
    model = (CifgLstm(vocab) if args.model == "lstm" else Gpt(vocab, dropout=args.dropout)).to(device)
    params = sum(p.numel() for p in model.parameters())
    mix = Mixture(data, vocab, args.colloquial_frac, args.seed)
    decay = [p for p in model.parameters() if p.dim() == 2]
    no_decay = [p for p in model.parameters() if p.dim() < 2]
    opt = torch.optim.AdamW(
        [{"params": decay, "weight_decay": args.weight_decay}, {"params": no_decay, "weight_decay": 0.0}],
        lr=args.lr,
        betas=(0.9, 0.95),
    )
    amp = args.model == "gpt" and device.type == "mps"

    def lr_at(step: int) -> float:
        if step < args.warmup:
            return args.lr * (step + 1) / args.warmup
        p = (step - args.warmup) / max(1, args.steps - args.warmup)
        return args.lr * (0.1 + 0.9 * 0.5 * (1 + math.cos(math.pi * p)))

    log = (out / "train.log").open("w", encoding="utf-8")
    config = {k: v for k, v in vars(args).items() if k != "func"}
    config.update(params=params, vocab=vocab, context=CONTEXT, colloquial_frac_used=mix.colloquial_frac)
    print(json.dumps(config, ensure_ascii=False))
    start = time.time()
    last = None
    # --keep-best：留口語 dev 損失（兩個口語語料平均）最低的那次評估的權重。
    best_state, best_rec = None, None
    for step in range(args.steps):
        for g in opt.param_groups:
            g["lr"] = lr_at(step)
        x = mix.batch(args.batch).to(device)
        with torch.autocast(device.type, dtype=torch.bfloat16, enabled=amp):
            logits = model(x[:, :-1])
        loss = F.cross_entropy(logits.float().reshape(-1, vocab), x[:, 1:].reshape(-1))
        opt.zero_grad(set_to_none=True)
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        opt.step()
        if step % 100 == 0 or step == args.steps - 1:
            rec = {"step": step, "loss": round(loss.item(), 4), "lr": lr_at(step), "elapsed_s": round(time.time() - start, 1)}
            if step % args.eval_every == 0 or step == args.steps - 1:
                rec["dev"] = {k: round(v, 4) for k, v in dev_losses(model, data, vocab, device).items()}
                coll = sum(rec["dev"][n] for n in COLLOQUIAL) / len(COLLOQUIAL)
                if args.save_checkpoints and step > 0:
                    # 每次評估存一份，事後用 dev 的前 2 名命中率挑（model-ladder neural）。
                    ckpt = out / f"step{step}"
                    ckpt.mkdir(exist_ok=True)
                    save_file({k: v.detach().float().cpu().contiguous() for k, v in model.export().items()}, ckpt / "model.safetensors")
                if args.keep_best and (best_rec is None or coll < best_rec["colloquial_dev"]):
                    best_state = {k: v.detach().clone() for k, v in model.state_dict().items()}
                    best_rec = {**rec, "colloquial_dev": round(coll, 4)}
            print(json.dumps(rec, ensure_ascii=False), flush=True)
            log.write(json.dumps(rec, ensure_ascii=False) + "\n")
            log.flush()
            last = rec
    elapsed = time.time() - start
    log.close()
    if best_state is not None:
        model.load_state_dict(best_state)
    tensors = {k: v.detach().float().cpu().contiguous() for k, v in model.export().items()}
    save_file(tensors, out / "model.safetensors")
    (out / "reference.json").write_text(json.dumps(reference(model, data, vocab)), encoding="utf-8")
    summary = {
        "config": config,
        "tokens_seen": args.steps * args.batch * CONTEXT,
        "train_seconds": round(elapsed, 1),
        "final": last,
        "kept": best_rec or last,
        "device": device.type,
        "machine": platform.platform(),
        "python": platform.python_version(),
        "torch": torch.__version__,
        "commit": git_commit(),
    }
    (out / "train.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(summary, ensure_ascii=False))


def main() -> None:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(required=True)
    p = sub.add_parser("prepare")
    p.add_argument("--colloquial-weight", type=int, default=64)
    p.add_argument("--dev-tokens", type=int, default=200_000)
    p.set_defaults(func=prepare)
    t = sub.add_parser("train")
    t.add_argument("--model", choices=["lstm", "gpt"], required=True)
    t.add_argument("--name", help="輸出目錄名，預設是 --model")
    t.add_argument("--steps", type=int, default=4000)
    t.add_argument("--batch", type=int, default=64)
    t.add_argument("--lr", type=float, default=6e-4)
    t.add_argument("--warmup", type=int, default=200)
    t.add_argument("--weight-decay", type=float, default=0.1)
    t.add_argument("--dropout", type=float, default=0.0)
    t.add_argument("--colloquial-frac", type=float, default=0.1, help="口語視窗的比例；負數照自然比例")
    t.add_argument("--eval-every", type=int, default=500)
    t.add_argument("--seed", type=int, default=0)
    t.add_argument("--keep-best", action="store_true", help="匯出口語 dev 損失最低那次評估的權重")
    t.add_argument("--save-checkpoints", action="store_true", help="每次評估另存一份權重到 step<N>/")
    t.set_defaults(func=train)
    f = sub.add_parser("fixture")
    f.add_argument("--out", default=str(ROOT / "crates" / "qim-neural" / "tests" / "fixtures"))
    f.add_argument("--seed", type=int, default=11)
    f.set_defaults(func=fixture)
    args = ap.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
