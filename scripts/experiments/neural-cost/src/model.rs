//! 兩個隨機權重的字級模型與它們的權重檔。形狀的依據見 README〈模型〉。

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use candle_core::quantized::{GgmlDType, QMatMul, QTensor, gguf_file};
use candle_core::safetensors::MmapedSafetensors;
use candle_core::{D, DType, Device, IndexOp, Module, Result, Tensor, bail};
use candle_nn::LayerNorm;
use candle_nn::ops::{log_softmax, sigmoid, softmax_last_dim};

/// 句首記號；字表其餘的 id 都當成字。
pub const BOS: u32 = 0;

/// Gboard 的 CIFG-LSTM（Hard et al. 2018）：輸入閘與遺忘閘共用、cell 輸出投影回 embedding 維度、
/// 輸入輸出共用 embedding。論文是詞級、詞表 1 萬；這裡當字級用，字表同樣取 1 萬。
pub struct LstmConfig {
    pub vocab: usize,
    pub embed: usize,
    pub cells: usize,
}

pub const LSTM: LstmConfig = LstmConfig {
    vocab: 10_000,
    embed: 96,
    cells: 670,
};

/// 青簡含章 small 的 GPT-2 式字級 decoder：公開的只有約 23M 參數與上下文 256，
/// 字表、層數、寬度是湊出約 23M 的近似。
pub struct GptConfig {
    pub vocab: usize,
    pub embed: usize,
    pub layers: usize,
    pub heads: usize,
    pub context: usize,
}

pub const GPT: GptConfig = GptConfig {
    vocab: 8_000,
    embed: 512,
    layers: 6,
    heads: 8,
    context: 256,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Lstm,
    Gpt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// 全部 f32，safetensors。
    F32,
    /// 內層維度是 32 倍數的矩陣（含共用的 embedding）存 Q8_0，其餘 f32；GGUF。
    Q8,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Lstm, Kind::Gpt];

    pub fn stem(self) -> &'static str {
        match self {
            Kind::Lstm => "lstm-1.4m",
            Kind::Gpt => "gpt-23m",
        }
    }

    pub fn vocab(self) -> usize {
        match self {
            Kind::Lstm => LSTM.vocab,
            Kind::Gpt => GPT.vocab,
        }
    }

    /// 權重的名字與形狀；一維的是偏差或 LayerNorm。
    fn shapes(self) -> Vec<(String, Vec<usize>)> {
        match self {
            Kind::Lstm => {
                let c = &LSTM;
                vec![
                    ("embed".into(), vec![c.vocab, c.embed]),
                    // 三組閘：輸入 i、輸出 o、候選 g；遺忘閘是 1 − i。輸入是 [x, h]。
                    ("gates.weight".into(), vec![3 * c.cells, 2 * c.embed]),
                    ("gates.bias".into(), vec![3 * c.cells]),
                    ("proj.weight".into(), vec![c.embed, c.cells]),
                ]
            }
            Kind::Gpt => {
                let c = &GPT;
                let d = c.embed;
                let mut v = vec![
                    ("wte".into(), vec![c.vocab, d]),
                    ("wpe".into(), vec![c.context, d]),
                ];
                for l in 0..c.layers {
                    let p = format!("h.{l}");
                    v.push((format!("{p}.ln_1.weight"), vec![d]));
                    v.push((format!("{p}.ln_1.bias"), vec![d]));
                    v.push((format!("{p}.attn.c_attn.weight"), vec![3 * d, d]));
                    v.push((format!("{p}.attn.c_attn.bias"), vec![3 * d]));
                    v.push((format!("{p}.attn.c_proj.weight"), vec![d, d]));
                    v.push((format!("{p}.attn.c_proj.bias"), vec![d]));
                    v.push((format!("{p}.ln_2.weight"), vec![d]));
                    v.push((format!("{p}.ln_2.bias"), vec![d]));
                    v.push((format!("{p}.mlp.c_fc.weight"), vec![4 * d, d]));
                    v.push((format!("{p}.mlp.c_fc.bias"), vec![4 * d]));
                    v.push((format!("{p}.mlp.c_proj.weight"), vec![d, 4 * d]));
                    v.push((format!("{p}.mlp.c_proj.bias"), vec![d]));
                }
                v.push(("ln_f.weight".into(), vec![d]));
                v.push(("ln_f.bias".into(), vec![d]));
                v
            }
        }
    }
}

impl Format {
    pub const ALL: [Format; 2] = [Format::F32, Format::Q8];

    pub fn label(self) -> &'static str {
        match self {
            Format::F32 => "f32",
            Format::Q8 => "q8",
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Format::F32 => "safetensors",
            Format::Q8 => "gguf",
        }
    }
}

pub fn file_name(kind: Kind, format: Format) -> String {
    format!("{}-{}.{}", kind.stem(), format.label(), format.extension())
}

/// 從檔名認出模型與格式（`gen` 寫出的名字）。
pub fn parse_file_name(path: &Path) -> Result<(Kind, Format)> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    for kind in Kind::ALL {
        for format in Format::ALL {
            if name == file_name(kind, format) {
                return Ok((kind, format));
            }
        }
    }
    bail!("unknown model file name {name}; expected a name written by gen")
}

fn quantizable(shape: &[usize]) -> bool {
    shape.len() == 2 && shape[1].is_multiple_of(GgmlDType::Q8_0.block_size())
}

/// 隨機權重：矩陣 N(0, 0.02)，LayerNorm 的權重 1，偏差 0。值不影響開銷。
fn random_tensors(kind: Kind, dev: &Device) -> Result<Vec<(String, Tensor)>> {
    kind.shapes()
        .into_iter()
        .map(|(name, shape)| {
            let t = if shape.len() == 2 {
                Tensor::randn(0f32, 0.02, shape.as_slice(), dev)?
            } else if name.contains(".ln_") || name.starts_with("ln_f") {
                Tensor::ones(shape.as_slice(), DType::F32, dev)?
            } else {
                Tensor::zeros(shape.as_slice(), DType::F32, dev)?
            };
            Ok((name, t))
        })
        .collect()
}

/// 寫出四個權重檔（兩個模型 × 兩種格式），回傳檔案路徑。
pub fn generate(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let dev = Device::Cpu;
    std::fs::create_dir_all(dir)?;
    let mut written = Vec::new();
    for kind in Kind::ALL {
        let tensors = random_tensors(kind, &dev)?;
        let f32_path = dir.join(file_name(kind, Format::F32));
        let map: HashMap<String, Tensor> = tensors.iter().cloned().collect();
        candle_core::safetensors::save(&map, &f32_path)?;
        written.push(f32_path);

        let q: Vec<(String, QTensor)> = tensors
            .iter()
            .map(|(name, t)| {
                let dtype = if quantizable(t.dims()) {
                    GgmlDType::Q8_0
                } else {
                    GgmlDType::F32
                };
                Ok((name.clone(), QTensor::quantize(t, dtype)?))
            })
            .collect::<Result<_>>()?;
        let refs: Vec<(&str, &QTensor)> = q.iter().map(|(n, t)| (n.as_str(), t)).collect();
        let q_path = dir.join(file_name(kind, Format::Q8));
        let mut file = File::create(&q_path)?;
        gguf_file::write(&mut file, &[], &refs)?;
        written.push(q_path);
    }
    Ok(written)
}

/// 讀權重，順便數參數。
struct Loader {
    source: Source,
    params: usize,
}

enum Source {
    // mmap：只複製一次進 tensor，不先整檔讀進緩衝區（票 04：釋放的緩衝區會留在 footprint）。
    SafeTensors(MmapedSafetensors),
    Gguf(File, gguf_file::Content),
}

impl Loader {
    fn open(path: &Path, format: Format) -> Result<Self> {
        let source = match format {
            // SAFETY: 量測期間沒有別人改這個檔。
            Format::F32 => Source::SafeTensors(unsafe { MmapedSafetensors::new(path)? }),
            Format::Q8 => {
                let mut file = File::open(path)?;
                let content = gguf_file::Content::read(&mut file)?;
                Source::Gguf(file, content)
            }
        };
        Ok(Self { source, params: 0 })
    }

    /// 矩陣：f32 直接乘，Q8_0 走量化的矩陣乘法；也用來查 embedding。
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

/// 一次重排：同一段上下文後接幾條候選路徑，回每條路徑各字 log 機率的和。
pub trait Scorer: Send {
    /// 上下文只算一次，狀態複製給每條路徑（前綴快取），路徑一批算完。
    fn score(&self, context: &[u32], candidates: &[Vec<u32>]) -> Result<Vec<f32>>;

    /// 對照用：不共用前綴，每條路徑各自從句首連上下文算起。分數與 `score` 相同。
    fn score_unshared(&self, context: &[u32], candidates: &[Vec<u32>]) -> Result<Vec<f32>> {
        candidates
            .iter()
            .map(|c| self.score_one(context, c))
            .collect()
    }

    /// 一條路徑從句首算起的分數。
    fn score_one(&self, context: &[u32], candidate: &[u32]) -> Result<f32>;
}

pub struct Loaded {
    pub scorer: Box<dyn Scorer>,
    pub kind: Kind,
    pub format: Format,
    pub params: usize,
}

pub fn load(path: &Path) -> Result<Loaded> {
    let (kind, format) = parse_file_name(path)?;
    let mut loader = Loader::open(path, format)?;
    let scorer: Box<dyn Scorer> = match kind {
        Kind::Lstm => Box::new(Lstm::load(&mut loader)?),
        Kind::Gpt => Box::new(Gpt::load(&mut loader)?),
    };
    Ok(Loaded {
        scorer,
        kind,
        format,
        params: loader.params,
    })
}

fn ids(tokens: &[u32]) -> Result<Tensor> {
    Tensor::new(tokens, &Device::Cpu)
}

/// 候選路徑排成 (路徑數, 字數)；每條要一樣長。
fn candidate_ids(candidates: &[Vec<u32>]) -> Result<Tensor> {
    let len = candidates.first().map_or(0, Vec::len);
    if len == 0 || candidates.iter().any(|c| c.len() != len) {
        bail!("candidate paths must be non-empty and of equal length");
    }
    let flat: Vec<u32> = candidates.concat();
    Tensor::from_vec(flat, (candidates.len(), len), &Device::Cpu)
}

struct Lstm {
    embed: QMatMul,
    gates: Linear,
    proj: QMatMul,
}

impl Lstm {
    fn load(l: &mut Loader) -> Result<Self> {
        Ok(Self {
            embed: l.matrix("embed")?,
            gates: l.linear("gates")?,
            proj: l.matrix("proj.weight")?,
        })
    }

    /// 吃一個字，回新的 (h, c)；h 是投影後的輸出。
    fn step(&self, tokens: &Tensor, h: &Tensor, c: &Tensor) -> Result<(Tensor, Tensor)> {
        let n = LSTM.cells;
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
}

impl Scorer for Lstm {
    fn score(&self, context: &[u32], candidates: &[Vec<u32>]) -> Result<Vec<f32>> {
        let cand = candidate_ids(candidates)?;
        let (paths, len) = cand.dims2()?;
        // 上下文只跑一次，狀態再複製給每條路徑。
        let mut h = Tensor::zeros((1, LSTM.embed), DType::F32, &Device::Cpu)?;
        let mut c = Tensor::zeros((1, LSTM.cells), DType::F32, &Device::Cpu)?;
        for &t in std::iter::once(&BOS).chain(context) {
            (h, c) = self.step(&ids(&[t])?, &h, &c)?;
        }
        let mut h = h.broadcast_as((paths, LSTM.embed))?.contiguous()?;
        let mut c = c.broadcast_as((paths, LSTM.cells))?.contiguous()?;
        let mut total = Tensor::zeros(paths, DType::F32, &Device::Cpu)?;
        for t in 0..len {
            let col = cand.i((.., t))?.contiguous()?;
            let lp = self
                .log_probs(&h)?
                .gather(&col.unsqueeze(1)?, 1)?
                .squeeze(1)?;
            total = (total + lp)?;
            if t + 1 < len {
                (h, c) = self.step(&col, &h, &c)?;
            }
        }
        total.to_vec1()
    }

    fn score_one(&self, context: &[u32], candidate: &[u32]) -> Result<f32> {
        let mut h = Tensor::zeros((1, LSTM.embed), DType::F32, &Device::Cpu)?;
        let mut c = Tensor::zeros((1, LSTM.cells), DType::F32, &Device::Cpu)?;
        for &t in std::iter::once(&BOS).chain(context) {
            (h, c) = self.step(&ids(&[t])?, &h, &c)?;
        }
        let mut sum = 0.0;
        for (i, &t) in candidate.iter().enumerate() {
            sum += self.log_probs(&h)?.i((0, t as usize))?.to_scalar::<f32>()?;
            if i + 1 < candidate.len() {
                (h, c) = self.step(&ids(&[t])?, &h, &c)?;
            }
        }
        Ok(sum)
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

    /// `past` 是前綴的 K、V（批可以是 1，會廣播）；回輸出與含前綴的 K、V。
    fn forward(&self, x: &Tensor, past: Option<&Kv>, mask: &Tensor) -> Result<(Tensor, Kv)> {
        let (b, t, d) = x.dims3()?;
        let heads = GPT.heads;
        let hd = d / heads;
        let qkv = self.c_attn.forward(&self.ln_1.forward(x)?)?;
        let split = |i: usize| {
            qkv.narrow(2, i * d, d)?
                .reshape((b, t, heads, hd))?
                .transpose(1, 2)?
                .contiguous()
        };
        let (q, mut k, mut v) = (split(0)?, split(1)?, split(2)?);
        if let Some((pk, pv)) = past {
            let p = pk.dim(2)?;
            k = Tensor::cat(&[&pk.broadcast_as((b, heads, p, hd))?, &k], 2)?.contiguous()?;
            v = Tensor::cat(&[&pv.broadcast_as((b, heads, p, hd))?, &v], 2)?.contiguous()?;
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
        Ok(Self {
            wte: l.matrix("wte")?,
            wpe: l.matrix("wpe")?,
            blocks: (0..GPT.layers)
                .map(|i| Block::load(l, i))
                .collect::<Result<_>>()?,
            ln_f: l.layer_norm("ln_f")?,
        })
    }

    /// 從位置 `start` 起跑 `tokens`（批, 長度）；回 ln_f 後的隱層，沒給 `past` 時另回各層的 K、V。
    /// 給了 `past` 的那一輪（候選路徑）K、V 已經展開成每條路徑一份，留著只佔記憶體，跑完一層就丟。
    fn forward(
        &self,
        tokens: &Tensor,
        start: usize,
        past: Option<&[Kv]>,
    ) -> Result<(Tensor, Vec<Kv>)> {
        let (_, t) = tokens.dims2()?;
        if start + t > GPT.context {
            bail!("sequence of {} exceeds context {}", start + t, GPT.context);
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
    fn score(&self, context: &[u32], candidates: &[Vec<u32>]) -> Result<Vec<f32>> {
        let cand = candidate_ids(candidates)?;
        let (paths, len) = cand.dims2()?;
        // 前綴（句首 + 上下文）只跑一次，它的 K、V 給每條路徑共用。
        let prefix: Vec<u32> = std::iter::once(BOS)
            .chain(context.iter().copied())
            .collect();
        let (hidden, cache) = self.forward(&ids(&prefix)?.unsqueeze(0)?, 0, None)?;
        let last = hidden.i((.., prefix.len() - 1, ..))?;
        let first = self
            .log_probs(&last)?
            .broadcast_as((paths, GPT.vocab))?
            .contiguous()?;
        let mut total = first
            .gather(&cand.narrow(1, 0, 1)?.contiguous()?, 1)?
            .squeeze(1)?;
        if len > 1 {
            // 候選的最後一個字不用預測下一個字，只跑前 len − 1 個。
            let inputs = cand.narrow(1, 0, len - 1)?.contiguous()?;
            let (hidden, _) = self.forward(&inputs, prefix.len(), Some(&cache))?;
            let targets = cand.narrow(1, 1, len - 1)?.contiguous()?.unsqueeze(2)?;
            let lp = self
                .log_probs(&hidden)?
                .gather(&targets, 2)?
                .squeeze(2)?
                .sum(1)?;
            total = (total + lp)?;
        }
        total.to_vec1()
    }

    fn score_one(&self, context: &[u32], candidate: &[u32]) -> Result<f32> {
        // 句首 + 上下文 + 候選的前 len − 1 字一次跑完，只在預測候選字的位置算輸出層。
        let len = candidate.len();
        let seq: Vec<u32> = std::iter::once(BOS)
            .chain(context.iter().copied())
            .chain(candidate[..len - 1].iter().copied())
            .collect();
        let (hidden, _) = self.forward(&ids(&seq)?.unsqueeze(0)?, 0, None)?;
        let lp = self
            .log_probs(&hidden.narrow(1, context.len(), len)?)?
            .squeeze(0)?;
        let targets = Tensor::new(candidate, &Device::Cpu)?.unsqueeze(1)?;
        lp.gather(&targets, 1)?.sum_all()?.to_scalar::<f32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("neural-cost-test-{}", std::process::id()));
        generate(&dir).unwrap();
        dir
    }

    fn total_params(kind: Kind) -> usize {
        kind.shapes()
            .iter()
            .map(|(_, s)| s.iter().product::<usize>())
            .sum()
    }

    #[test]
    fn parameter_counts_match_the_references() {
        // Gboard 約 140 萬、青簡 small 約 23M。
        assert_eq!(total_params(Kind::Lstm), 1_412_250);
        assert_eq!(total_params(Kind::Gpt), 23_142_400);
    }

    #[test]
    fn batched_scores_match_one_path_at_a_time() {
        let dir = temp_dir();
        let context = [5, 9, 13];
        let cands = vec![vec![1, 2, 3, 4], vec![7, 7, 7, 7], vec![100, 200, 300, 400]];
        for format in Format::ALL {
            for kind in Kind::ALL {
                let path = dir.join(file_name(kind, format));
                let (k, f) = parse_file_name(&path).unwrap();
                let mut l = Loader::open(&path, f).unwrap();
                let got: Vec<f32>;
                let want: Vec<f32>;
                match k {
                    Kind::Lstm => {
                        let m = Lstm::load(&mut l).unwrap();
                        got = m.score(&context, &cands).unwrap();
                        want = m.score_unshared(&context, &cands).unwrap();
                    }
                    Kind::Gpt => {
                        let g = Gpt::load(&mut l).unwrap();
                        got = g.score(&context, &cands).unwrap();
                        want = g.score_unshared(&context, &cands).unwrap();
                    }
                }
                assert_eq!(l.params, total_params(kind), "{kind:?} {format:?}");
                for (g, w) in got.iter().zip(&want) {
                    assert!(
                        g.is_finite() && (g - w).abs() < 1e-2,
                        "{kind:?} {format:?}: {got:?} vs {want:?}"
                    );
                }
            }
        }
        // 空上下文也要能算。
        let loaded = load(&dir.join(file_name(Kind::Gpt, Format::Q8))).unwrap();
        assert_eq!(loaded.scorer.score(&[], &cands).unwrap().len(), 3);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
