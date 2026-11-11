//! 停鍵層的小型神經模型（M3 票 09）：讀 `../neural-rerank/train.py` 訓練的權重，用 candle 給 lattice
//! 的前 N 條路徑打分，`路徑分 + λ·神經分` 重排。模型結構與前綴快取照 `../neural-cost/src/model.rs`
//! （M2 票 11 量開銷的同一組形狀），這裡多了：形狀從權重讀、路徑可以不一樣長（補齊後遮掉）、字表。

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

use candle_core::quantized::{GgmlDType, QMatMul, QTensor, gguf_file};
use candle_core::safetensors::MmapedSafetensors;
use candle_core::{D, DType, Device, IndexOp, Module, Tensor, bail};
use candle_nn::LayerNorm;
use candle_nn::ops::{log_softmax, sigmoid, softmax_last_dim};

use crate::{Query, Reranker};

type Result<T> = candle_core::Result<T>;

/// 句首與子句之間的記號：訓練語料在標點處切句，前文裡的標點、空白、符號也換成它。
pub const SEP: u32 = 0;
/// 字表外的字；前文裡連續的英文、數字也收成一個。
pub const UNK: u32 = 1;
/// GPT 的頭數（`train.py` 的 `Gpt` 與 neural-cost 的 `GPT`）；權重裡讀不出來。
const GPT_HEADS: usize = 8;

/// `train.py prepare` 寫的字表：第 0、1 行是兩個記號，之後依字次數排。模型只用前 `size` 個。
pub struct Vocab {
    index: HashMap<char, u32>,
    /// 路徑裡每個字表外的字多扣的分（nat）：`<unk>` 的機率平分給字表外的各字。
    unk_penalty: f32,
}

impl Vocab {
    pub fn load(path: &Path, size: usize) -> std::result::Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("讀不到 {}：{e}", path.display()))?;
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 3 || lines[0] != "<sep>" || lines[1] != "<unk>" {
            return Err(format!("{} 不是 train.py 寫的字表", path.display()));
        }
        let index = lines
            .iter()
            .enumerate()
            .take(size)
            .skip(2)
            .filter_map(|(i, l)| l.chars().next().map(|c| (c, i as u32)))
            .collect();
        let outside = lines.len().saturating_sub(size).max(1);
        Ok(Self {
            index,
            unk_penalty: (outside as f32).ln(),
        })
    }

    /// 前文：字表裡的字照查；其他字母、數字（英文、擴展區的字）連續的收成一個 `<unk>`；
    /// 標點、空白、符號連續的收成一個 `<sep>`。開頭的 `<sep>` 不留（模型的句首就是它）。
    pub fn context(&self, text: &str) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for c in text.chars() {
            let t = match self.index.get(&c) {
                Some(&i) => i,
                None if c.is_alphanumeric() => UNK,
                None => SEP,
            };
            if (t == SEP || t == UNK) && out.last() == Some(&t) {
                continue;
            }
            if t == SEP && out.is_empty() {
                continue;
            }
            out.push(t);
        }
        out
    }

    /// 路徑：字表外的字是 `<unk>`；回 token 與字表外的字數。
    pub fn path(&self, text: &str) -> (Vec<u32>, usize) {
        let ids: Vec<u32> = text
            .chars()
            .map(|c| *self.index.get(&c).unwrap_or(&UNK))
            .collect();
        let unknown = ids.iter().filter(|&&t| t == UNK).count();
        (ids, unknown)
    }
}

// ---------------------------------------------------------------- 權重

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// 全部 f32，safetensors（`train.py` 寫的）。
    F32,
    /// 內層維度是 32 倍數的矩陣（含共用的 embedding）存 Q8_0，其餘 f32；GGUF。同 neural-cost。
    Q8,
}

impl Format {
    pub fn of(path: &Path) -> Result<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("safetensors") => Ok(Format::F32),
            Some("gguf") => Ok(Format::Q8),
            _ => bail!(
                "model file must be .safetensors (f32) or .gguf (q8): {}",
                path.display()
            ),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Format::F32 => "f32",
            Format::Q8 => "q8",
        }
    }
}

/// 把 f32 的 safetensors 量化成 q8 的 GGUF（同 neural-cost 的 `generate`）。
pub fn quantize(src: &Path, dst: &Path) -> Result<()> {
    let tensors = candle_core::safetensors::load(src, &Device::Cpu)?;
    let mut names: Vec<&String> = tensors.keys().collect();
    names.sort();
    let q: Vec<(String, QTensor)> = names
        .into_iter()
        .map(|name| {
            let t = &tensors[name];
            let dims = t.dims();
            let dtype = if dims.len() == 2 && dims[1].is_multiple_of(GgmlDType::Q8_0.block_size()) {
                GgmlDType::Q8_0
            } else {
                GgmlDType::F32
            };
            Ok((name.clone(), QTensor::quantize(t, dtype)?))
        })
        .collect::<Result<_>>()?;
    let refs: Vec<(&str, &QTensor)> = q.iter().map(|(n, t)| (n.as_str(), t)).collect();
    let mut file = File::create(dst)?;
    gguf_file::write(&mut file, &[], &refs)
}

struct Loader {
    source: Source,
    params: usize,
}

enum Source {
    // mmap：只複製一次進 tensor（neural-cost 票 04 的理由）。
    SafeTensors(MmapedSafetensors),
    Gguf(File, gguf_file::Content),
}

impl Loader {
    fn open(path: &Path) -> Result<Self> {
        let source = match Format::of(path)? {
            // SAFETY: 跑實驗時沒有別人改這個檔。
            Format::F32 => Source::SafeTensors(unsafe { MmapedSafetensors::new(path)? }),
            Format::Q8 => {
                let mut file = File::open(path)?;
                let content = gguf_file::Content::read(&mut file)?;
                Source::Gguf(file, content)
            }
        };
        Ok(Self { source, params: 0 })
    }

    fn has(&self, name: &str) -> bool {
        match &self.source {
            Source::SafeTensors(st) => st.get(name).is_ok(),
            Source::Gguf(_, c) => c.tensor_infos.contains_key(name),
        }
    }

    fn shape(&self, name: &str) -> Result<Vec<usize>> {
        Ok(match &self.source {
            Source::SafeTensors(st) => st.get(name)?.shape().to_vec(),
            Source::Gguf(_, c) => match c.tensor_infos.get(name) {
                Some(info) => info.shape.dims().to_vec(),
                None => bail!("no tensor {name}"),
            },
        })
    }

    fn matrix(&mut self, name: &str) -> Result<QMatMul> {
        let m = match &mut self.source {
            Source::SafeTensors(st) => QMatMul::Tensor(st.load(name, &Device::Cpu)?),
            Source::Gguf(file, content) => {
                QMatMul::from_qtensor(content.tensor(file, name, &Device::Cpu)?)?
            }
        };
        self.params += match &m {
            QMatMul::QTensor(t) => t.shape().elem_count(),
            QMatMul::Tensor(t) | QMatMul::TensorF16(t) => t.elem_count(),
        };
        Ok(m)
    }

    fn vector(&mut self, name: &str) -> Result<Tensor> {
        let t = match &mut self.source {
            Source::SafeTensors(st) => st.load(name, &Device::Cpu)?,
            Source::Gguf(file, content) => content
                .tensor(file, name, &Device::Cpu)?
                .dequantize(&Device::Cpu)?,
        };
        self.params += t.elem_count();
        Ok(t)
    }

    fn linear(&mut self, prefix: &str) -> Result<Linear> {
        Ok(Linear {
            w: self.matrix(&format!("{prefix}.weight"))?,
            b: self.vector(&format!("{prefix}.bias"))?,
        })
    }

    fn layer_norm(&mut self, prefix: &str) -> Result<LayerNorm> {
        Ok(LayerNorm::new(
            self.vector(&format!("{prefix}.weight"))?,
            self.vector(&format!("{prefix}.bias"))?,
            1e-5,
        ))
    }
}

struct Linear {
    w: QMatMul,
    b: Tensor,
}

impl Module for Linear {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        self.w.forward(xs)?.broadcast_add(&self.b)
    }
}

// ---------------------------------------------------------------- 模型

/// 給同一段前文後接的幾條路徑打分：每條路徑各字 log 機率（nat）的和。
pub trait Scorer {
    /// 前文只算一次，狀態給每條路徑共用（前綴快取），路徑補齊成一樣長後一批算完、補的位置遮掉。
    fn score(&self, context: &[u32], paths: &[Vec<u32>]) -> Result<Vec<f32>>;
    /// 對照用：一條路徑從句首連前文算起。分數與 `score` 相同。
    fn score_one(&self, context: &[u32], path: &[u32]) -> Result<f32>;
    /// 前文最多幾個 token（含句首）加路徑字數；LSTM 沒有上限。
    fn max_positions(&self) -> Option<usize>;
    fn vocab(&self) -> usize;
}

pub struct Loaded {
    pub scorer: Box<dyn Scorer>,
    pub kind: &'static str,
    pub format: Format,
    pub params: usize,
}

/// 讀權重：有 `wte` 的是 GPT，有 `gates.weight` 的是 LSTM。
pub fn load(path: &Path) -> Result<Loaded> {
    let format = Format::of(path)?;
    let mut l = Loader::open(path)?;
    let (scorer, kind): (Box<dyn Scorer>, _) = if l.has("wte") {
        (Box::new(Gpt::load(&mut l)?), "gpt")
    } else if l.has("gates.weight") {
        (Box::new(Lstm::load(&mut l)?), "lstm")
    } else {
        bail!("{} is neither a GPT nor an LSTM checkpoint", path.display())
    };
    Ok(Loaded {
        scorer,
        kind,
        format,
        params: l.params,
    })
}

fn ids(tokens: &[u32]) -> Result<Tensor> {
    Tensor::new(tokens, &Device::Cpu)
}

/// 路徑補齊成 (路徑數, 最長字數)，補 `<sep>`；另回同形狀的遮罩（真的字 1、補的 0）。
fn padded(paths: &[Vec<u32>]) -> Result<(Tensor, Tensor)> {
    let len = paths.iter().map(Vec::len).max().unwrap_or(0);
    if len == 0 || paths.iter().any(Vec::is_empty) {
        bail!("paths must be non-empty");
    }
    let mut flat = Vec::with_capacity(paths.len() * len);
    let mut mask = Vec::with_capacity(paths.len() * len);
    for p in paths {
        for i in 0..len {
            flat.push(*p.get(i).unwrap_or(&SEP));
            mask.push(if i < p.len() { 1f32 } else { 0.0 });
        }
    }
    Ok((
        Tensor::from_vec(flat, (paths.len(), len), &Device::Cpu)?,
        Tensor::from_vec(mask, (paths.len(), len), &Device::Cpu)?,
    ))
}

struct Lstm {
    embed: QMatMul,
    gates: Linear,
    proj: QMatMul,
    vocab: usize,
    embed_dim: usize,
    cells: usize,
}

impl Lstm {
    fn load(l: &mut Loader) -> Result<Self> {
        let e = l.shape("embed")?;
        let g = l.shape("gates.weight")?;
        Ok(Self {
            vocab: e[0],
            embed_dim: e[1],
            cells: g[0] / 3,
            embed: l.matrix("embed")?,
            gates: l.linear("gates")?,
            proj: l.matrix("proj.weight")?,
        })
    }

    /// CIFG：閘 [i, o, g]，遺忘閘 1 − i，輸入 [x, h]；h 是投影後的輸出。
    fn step(&self, tokens: &Tensor, h: &Tensor, c: &Tensor) -> Result<(Tensor, Tensor)> {
        let n = self.cells;
        let x = self.embed.embedding(tokens)?;
        let z = self.gates.forward(&Tensor::cat(&[&x, h], 1)?)?;
        let i = sigmoid(&z.narrow(1, 0, n)?)?;
        let o = sigmoid(&z.narrow(1, n, n)?)?;
        let g = z.narrow(1, 2 * n, n)?.tanh()?;
        let c = ((i.affine(-1.0, 1.0)? * c)? + (i * g)?)?;
        let h = self.proj.forward(&(o * c.tanh()?)?)?;
        Ok((h, c))
    }

    fn log_probs(&self, h: &Tensor) -> Result<Tensor> {
        log_softmax(&self.embed.forward(h)?, D::Minus1)
    }

    fn prefix_state(&self, context: &[u32]) -> Result<(Tensor, Tensor)> {
        let mut h = Tensor::zeros((1, self.embed_dim), DType::F32, &Device::Cpu)?;
        let mut c = Tensor::zeros((1, self.cells), DType::F32, &Device::Cpu)?;
        for &t in std::iter::once(&SEP).chain(context) {
            (h, c) = self.step(&ids(&[t])?, &h, &c)?;
        }
        Ok((h, c))
    }
}

impl Scorer for Lstm {
    fn score(&self, context: &[u32], paths: &[Vec<u32>]) -> Result<Vec<f32>> {
        let (cand, mask) = padded(paths)?;
        let (n, len) = cand.dims2()?;
        let (h, c) = self.prefix_state(context)?;
        let mut h = h.broadcast_as((n, self.embed_dim))?.contiguous()?;
        let mut c = c.broadcast_as((n, self.cells))?.contiguous()?;
        let mut total = Tensor::zeros(n, DType::F32, &Device::Cpu)?;
        for t in 0..len {
            let col = cand.i((.., t))?.contiguous()?;
            let lp = self
                .log_probs(&h)?
                .gather(&col.unsqueeze(1)?, 1)?
                .squeeze(1)?;
            total = (total + (lp * mask.i((.., t))?)?)?;
            if t + 1 < len {
                (h, c) = self.step(&col, &h, &c)?;
            }
        }
        total.to_vec1()
    }

    fn score_one(&self, context: &[u32], path: &[u32]) -> Result<f32> {
        let (mut h, mut c) = self.prefix_state(context)?;
        let mut sum = 0.0;
        for (i, &t) in path.iter().enumerate() {
            sum += self.log_probs(&h)?.i((0, t as usize))?.to_scalar::<f32>()?;
            if i + 1 < path.len() {
                (h, c) = self.step(&ids(&[t])?, &h, &c)?;
            }
        }
        Ok(sum)
    }

    fn max_positions(&self) -> Option<usize> {
        None
    }

    fn vocab(&self) -> usize {
        self.vocab
    }
}

struct Block {
    ln_1: LayerNorm,
    c_attn: Linear,
    c_proj: Linear,
    ln_2: LayerNorm,
    c_fc: Linear,
    mlp_proj: Linear,
}

/// 一層的 K、V：(批, 頭, 長度, 頭寬)。
type Kv = (Tensor, Tensor);

impl Block {
    fn load(l: &mut Loader, i: usize) -> Result<Self> {
        let p = format!("h.{i}");
        Ok(Self {
            ln_1: l.layer_norm(&format!("{p}.ln_1"))?,
            c_attn: l.linear(&format!("{p}.attn.c_attn"))?,
            c_proj: l.linear(&format!("{p}.attn.c_proj"))?,
            ln_2: l.layer_norm(&format!("{p}.ln_2"))?,
            c_fc: l.linear(&format!("{p}.mlp.c_fc"))?,
            mlp_proj: l.linear(&format!("{p}.mlp.c_proj"))?,
        })
    }

    fn forward(&self, x: &Tensor, past: Option<&Kv>, mask: &Tensor) -> Result<(Tensor, Kv)> {
        let (b, t, d) = x.dims3()?;
        let hd = d / GPT_HEADS;
        let qkv = self.c_attn.forward(&self.ln_1.forward(x)?)?;
        let split = |i: usize| {
            qkv.narrow(2, i * d, d)?
                .reshape((b, t, GPT_HEADS, hd))?
                .transpose(1, 2)?
                .contiguous()
        };
        let (q, mut k, mut v) = (split(0)?, split(1)?, split(2)?);
        if let Some((pk, pv)) = past {
            let p = pk.dim(2)?;
            k = Tensor::cat(&[&pk.broadcast_as((b, GPT_HEADS, p, hd))?, &k], 2)?.contiguous()?;
            v = Tensor::cat(&[&pv.broadcast_as((b, GPT_HEADS, p, hd))?, &v], 2)?.contiguous()?;
        }
        let att = (q.matmul(&k.t()?)? * (1.0 / (hd as f64).sqrt()))?.broadcast_add(mask)?;
        let y = softmax_last_dim(&att)?
            .matmul(&v)?
            .transpose(1, 2)?
            .reshape((b, t, d))?;
        let x = (x + self.c_proj.forward(&y)?)?;
        let m = self
            .mlp_proj
            .forward(&self.c_fc.forward(&self.ln_2.forward(&x)?)?.gelu()?)?;
        Ok(((x + m)?, (k, v)))
    }
}

struct Gpt {
    wte: QMatMul,
    wpe: QMatMul,
    blocks: Vec<Block>,
    ln_f: LayerNorm,
    vocab: usize,
    context: usize,
}

/// `t` 個新位置接在 `past` 個舊位置後：新位置 i 只看得到 0..=past+i。
fn causal_mask(t: usize, past: usize) -> Result<Tensor> {
    let total = past + t;
    let v: Vec<f32> = (0..t)
        .flat_map(|i| {
            (0..total).map(move |j| {
                if j <= past + i {
                    0.0
                } else {
                    f32::NEG_INFINITY
                }
            })
        })
        .collect();
    Tensor::from_vec(v, (t, total), &Device::Cpu)
}

impl Gpt {
    fn load(l: &mut Loader) -> Result<Self> {
        let vocab = l.shape("wte")?[0];
        let context = l.shape("wpe")?[0];
        let layers = (0..)
            .take_while(|i| l.has(&format!("h.{i}.ln_1.weight")))
            .count();
        Ok(Self {
            wte: l.matrix("wte")?,
            wpe: l.matrix("wpe")?,
            blocks: (0..layers)
                .map(|i| Block::load(l, i))
                .collect::<Result<_>>()?,
            ln_f: l.layer_norm("ln_f")?,
            vocab,
            context,
        })
    }

    fn forward(
        &self,
        tokens: &Tensor,
        start: usize,
        past: Option<&[Kv]>,
    ) -> Result<(Tensor, Vec<Kv>)> {
        let (_, t) = tokens.dims2()?;
        if start + t > self.context {
            bail!("sequence of {} exceeds context {}", start + t, self.context);
        }
        let pos = Tensor::arange(start as u32, (start + t) as u32, &Device::Cpu)?;
        let mut x = self
            .wte
            .embedding(tokens)?
            .broadcast_add(&self.wpe.embedding(&pos)?)?;
        let mask = causal_mask(t, start)?;
        let mut cache = Vec::with_capacity(self.blocks.len());
        for (i, block) in self.blocks.iter().enumerate() {
            let (y, kv) = block.forward(&x, past.map(|p| &p[i]), &mask)?;
            x = y;
            if past.is_none() {
                cache.push(kv);
            }
        }
        Ok((self.ln_f.forward(&x)?, cache))
    }

    fn log_probs(&self, hidden: &Tensor) -> Result<Tensor> {
        log_softmax(&self.wte.forward(&hidden.contiguous()?)?, D::Minus1)
    }
}

impl Scorer for Gpt {
    fn score(&self, context: &[u32], paths: &[Vec<u32>]) -> Result<Vec<f32>> {
        let (cand, mask) = padded(paths)?;
        let (n, len) = cand.dims2()?;
        let prefix: Vec<u32> = std::iter::once(SEP)
            .chain(context.iter().copied())
            .collect();
        let (hidden, cache) = self.forward(&ids(&prefix)?.unsqueeze(0)?, 0, None)?;
        let last = hidden.i((.., prefix.len() - 1, ..))?;
        let first = self
            .log_probs(&last)?
            .broadcast_as((n, self.vocab))?
            .contiguous()?;
        let mut total = first
            .gather(&cand.narrow(1, 0, 1)?.contiguous()?, 1)?
            .squeeze(1)?;
        if len > 1 {
            // 路徑的最後一個字不用預測下一個字；補的位置在真的字之後，因果遮罩讓它們影響不到前面。
            let inputs = cand.narrow(1, 0, len - 1)?.contiguous()?;
            let (hidden, _) = self.forward(&inputs, prefix.len(), Some(&cache))?;
            let targets = cand.narrow(1, 1, len - 1)?.contiguous()?.unsqueeze(2)?;
            let lp = self.log_probs(&hidden)?.gather(&targets, 2)?.squeeze(2)?;
            total = (total + (lp * mask.narrow(1, 1, len - 1)?)?.sum(1)?)?;
        }
        total.to_vec1()
    }

    fn score_one(&self, context: &[u32], path: &[u32]) -> Result<f32> {
        let len = path.len();
        let seq: Vec<u32> = std::iter::once(SEP)
            .chain(context.iter().copied())
            .chain(path[..len - 1].iter().copied())
            .collect();
        let (hidden, _) = self.forward(&ids(&seq)?.unsqueeze(0)?, 0, None)?;
        let lp = self
            .log_probs(&hidden.narrow(1, context.len(), len)?)?
            .squeeze(0)?;
        let targets = Tensor::new(path, &Device::Cpu)?.unsqueeze(1)?;
        lp.gather(&targets, 1)?.sum_all()?.to_scalar::<f32>()
    }

    fn max_positions(&self) -> Option<usize> {
        Some(self.context)
    }

    fn vocab(&self) -> usize {
        self.vocab
    }
}

// ---------------------------------------------------------------- 重排

/// 神經分的權重 λ：`路徑分 + λ·神經分`；`None` 是只看神經分（同分再看路徑分）。
pub type Lambda = Option<f64>;

pub fn parse_lambda(s: &str) -> std::result::Result<Lambda, String> {
    if s == "inf" {
        return Ok(None);
    }
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .map(Some)
        .ok_or_else(|| format!("λ 要是非負的數或 inf：{s}"))
}

pub fn lambda_label(l: Lambda) -> String {
    l.map_or_else(|| "inf".into(), |v| format!("{v}"))
}

/// 一題的神經分與算它花的時間。
#[derive(Debug, Clone)]
pub struct Scored {
    pub neural: Vec<f32>,
    pub ms: f64,
    /// 交給模型的前文 token 數（截斷後）。
    pub context_tokens: usize,
}

/// 停鍵層的神經重排（[`Reranker`]）。同一題的神經分記下來：換 λ 重跑時不再算，計時只記第一次。
pub struct Neural {
    name: String,
    scorer: Box<dyn Scorer>,
    vocab: Vocab,
    pub lambda: Lambda,
    pub scored: HashMap<(String, usize), Scored>,
}

impl Neural {
    pub fn new(name: String, loaded: Loaded, vocab: Vocab) -> Self {
        Self {
            name,
            scorer: loaded.scorer,
            vocab,
            lambda: Some(1.0),
            scored: HashMap::new(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 給一題的各條路徑打神經分：前文截到放得下最長的路徑（GPT 上下文 256）。
    pub fn score(&self, query: &Query) -> Result<Scored> {
        let paths: Vec<(Vec<u32>, usize)> = query
            .paths
            .iter()
            .map(|p| self.vocab.path(&p.text))
            .collect();
        let longest = paths.iter().map(|(p, _)| p.len()).max().unwrap_or(0);
        let mut context = self.vocab.context(&query.before);
        if let Some(max) = self.scorer.max_positions() {
            // 句首 1 + 前文 + 路徑的前 len − 1 字 ≤ max。
            let keep = max.saturating_sub(longest);
            if context.len() > keep {
                context.drain(..context.len() - keep);
            }
        }
        let start = Instant::now();
        let tokens: Vec<Vec<u32>> = paths.iter().map(|(p, _)| p.clone()).collect();
        let raw = self.scorer.score(&context, &tokens)?;
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        let neural = raw
            .iter()
            .zip(&paths)
            .map(|(s, (_, unknown))| s - self.vocab.unk_penalty * *unknown as f32)
            .collect();
        Ok(Scored {
            neural,
            ms,
            context_tokens: context.len(),
        })
    }

    pub fn vocab_size(&self) -> usize {
        self.scorer.vocab()
    }
}

/// 依 `路徑分 + λ·神經分` 由高到低排路徑；`None` 只看神經分，同分看路徑分。
pub fn combine(query: &Query, neural: &[f32], lambda: Lambda) -> Vec<String> {
    let mut order: Vec<(f64, f64, &str)> = query
        .paths
        .iter()
        .zip(neural)
        .map(|(p, &n)| match lambda {
            Some(l) => (p.score + l * f64::from(n), p.score, p.text.as_str()),
            None => (f64::from(n), p.score, p.text.as_str()),
        })
        .collect();
    order.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.total_cmp(&a.1)));
    order.into_iter().map(|(_, _, t)| t.to_string()).collect()
}

impl Reranker for Neural {
    fn name(&self) -> &str {
        &self.name
    }

    fn rerank(&mut self, query: &Query) -> Vec<String> {
        if query.paths.is_empty() {
            return Vec::new();
        }
        let key = (query.set.clone(), query.line);
        if !self.scored.contains_key(&key) {
            let s = self.score(query).expect("神經模型打分失敗");
            self.scored.insert(key.clone(), s);
        }
        combine(query, &self.scored[&key].neural, self.lambda)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qim_core::ScoredPath;

    fn query(paths: &[(&str, f64)]) -> Query {
        Query {
            split: "dev".into(),
            set: "s".into(),
            line: 1,
            before: String::new(),
            input: String::new(),
            candidates: vec![],
            paths: paths
                .iter()
                .map(|(t, s)| ScoredPath {
                    text: (*t).into(),
                    segments: vec![(*t).into()],
                    score: *s,
                })
                .collect(),
        }
    }

    #[test]
    fn combine_weights_the_neural_score() {
        let q = query(&[("甲", -1.0), ("乙", -2.0)]);
        assert_eq!(combine(&q, &[-5.0, -1.0], Some(0.0)), ["甲", "乙"]);
        assert_eq!(combine(&q, &[-5.0, -1.0], Some(1.0)), ["乙", "甲"]);
        assert_eq!(
            combine(&q, &[-1.0, -1.0], None),
            ["甲", "乙"],
            "同分看路徑分"
        );
    }

    #[test]
    fn context_maps_punctuation_and_unknowns() {
        let dir = std::env::temp_dir().join(format!("model-ladder-vocab-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vocab.txt");
        std::fs::write(&path, "<sep>\n<unk>\n的\n我\n罕\n").unwrap();
        let v = Vocab::load(&path, 4).unwrap();
        // 開頭的標點不留；英文、數字收成一個 <unk>；連續標點收成一個 <sep>；「罕」在字表第 5 個，超出 size。
        assert_eq!(
            v.context("，我的ABC12。」我罕"),
            vec![3, 2, UNK, SEP, 3, UNK]
        );
        assert_eq!(v.path("我罕"), (vec![3, UNK], 1));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
