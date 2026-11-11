//! Gboard 級的 CIFG-LSTM（`scripts/neural-rerank/train.py` 的 `CifgLstm`）：權重從 GGUF 位元組讀，
//! 形狀從權重讀。前文只算一次、狀態給每條路徑共用（前綴快取），路徑補齊成一樣長後一批算完、補的
//! 位置遮掉。

use std::io::Cursor;

use candle_core::quantized::{GgmlDType, QMatMul, QTensor, gguf_file};
use candle_core::{D, DType, Device, IndexOp, Module, Tensor};
use candle_nn::ops::{log_softmax, sigmoid};

use crate::{Error, SEP};

type CandleResult<T> = candle_core::Result<T>;

fn inference(e: candle_core::Error) -> Error {
    Error::Inference(e.to_string())
}

fn invalid(e: candle_core::Error) -> Error {
    Error::InvalidModel(e.to_string())
}

/// 權重的存法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 全部 f32；核對 PyTorch 的分數用。
    F32,
    /// 內層維度是 32 倍數的矩陣（含共用的 embedding）存 Q8_0，其餘 f32；隨 App 發的這種。
    Q8,
}

/// 把 `train.py` 寫的 f32 safetensors 轉成 GGUF；張量依名字排序，同樣的輸入得到同樣的位元組。
pub fn to_gguf(safetensors: &[u8], format: Format) -> Result<Vec<u8>, Error> {
    let tensors =
        candle_core::safetensors::load_buffer(safetensors, &Device::Cpu).map_err(invalid)?;
    let mut names: Vec<&String> = tensors.keys().collect();
    names.sort();
    let quantized: Vec<(String, QTensor)> = names
        .into_iter()
        .map(|name| {
            let t = &tensors[name];
            let dims = t.dims();
            let q8 = format == Format::Q8
                && dims.len() == 2
                && dims[1].is_multiple_of(GgmlDType::Q8_0.block_size());
            let dtype = if q8 { GgmlDType::Q8_0 } else { GgmlDType::F32 };
            Ok((name.clone(), QTensor::quantize(t, dtype)?))
        })
        .collect::<CandleResult<_>>()
        .map_err(invalid)?;
    let refs: Vec<(&str, &QTensor)> = quantized.iter().map(|(n, t)| (n.as_str(), t)).collect();
    let mut out = Cursor::new(Vec::new());
    gguf_file::write(&mut out, &[], &refs).map_err(invalid)?;
    Ok(out.into_inner())
}

struct Linear {
    w: QMatMul,
    b: Tensor,
}

impl Module for Linear {
    fn forward(&self, xs: &Tensor) -> CandleResult<Tensor> {
        self.w.forward(xs)?.broadcast_add(&self.b)
    }
}

/// 字級 CIFG-LSTM：閘 [i, o, g]、遺忘閘 1 − i、輸入 [x, h]，cell 輸出投影回 embedding 維度，輸入
/// 輸出共用 embedding。
pub struct Model {
    embed: QMatMul,
    gates: Linear,
    proj: QMatMul,
    vocab: usize,
    embed_dim: usize,
    cells: usize,
}

impl Model {
    /// 讀 GGUF 權重（[`to_gguf`]）：`embed`、`gates.weight`、`gates.bias`、`proj.weight`。
    pub fn from_gguf(bytes: &[u8]) -> Result<Self, Error> {
        let mut reader = Cursor::new(bytes);
        let content = gguf_file::Content::read(&mut reader).map_err(invalid)?;
        let shape = |name: &str| match content.tensor_infos.get(name) {
            Some(info) => Ok(info.shape.dims().to_vec()),
            None => Err(Error::InvalidModel(format!(
                "沒有 {name}，不是 CIFG-LSTM 的權重"
            ))),
        };
        let (e, g, p) = (
            shape("embed")?,
            shape("gates.weight")?,
            shape("proj.weight")?,
        );
        let [vocab, embed_dim] = e[..] else {
            return Err(Error::InvalidModel(format!("embed 的形狀 {e:?}")));
        };
        let cells = g.first().copied().unwrap_or(0) / 3;
        if g != [3 * cells, 2 * embed_dim] || p != [embed_dim, cells] || cells == 0 {
            return Err(Error::InvalidModel(format!(
                "形狀對不上：embed {e:?}、gates {g:?}、proj {p:?}"
            )));
        }
        let mut tensor = |name: &str| {
            content
                .tensor(&mut reader, name, &Device::Cpu)
                .map_err(invalid)
        };
        let matrix = |t: QTensor| QMatMul::from_qtensor(t).map_err(invalid);
        let embed = matrix(tensor("embed")?)?;
        let gates_w = matrix(tensor("gates.weight")?)?;
        let gates_b = tensor("gates.bias")?
            .dequantize(&Device::Cpu)
            .map_err(invalid)?;
        let proj = matrix(tensor("proj.weight")?)?;
        Ok(Self {
            embed,
            gates: Linear {
                w: gates_w,
                b: gates_b,
            },
            proj,
            vocab,
            embed_dim,
            cells,
        })
    }

    /// 含兩個記號的字表大小（embedding 的列數）。
    pub fn vocab_size(&self) -> usize {
        self.vocab
    }

    /// 同一段前文後接的每條路徑各字 log 機率（nat）的和。前文是句首記號之後的 token（不含句首）。
    /// 路徑不能是空的；token 要小於 [`Model::vocab_size`]。
    pub fn score(&self, context: &[u32], paths: &[Vec<u32>]) -> Result<Vec<f32>, Error> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        self.check_tokens(context.iter().chain(paths.iter().flatten()))?;
        if paths.iter().any(Vec::is_empty) {
            return Err(Error::Inference("路徑不能是空的".into()));
        }
        self.score_batch(context, paths).map_err(inference)
    }

    fn check_tokens<'a>(&self, mut tokens: impl Iterator<Item = &'a u32>) -> Result<(), Error> {
        match tokens.find(|&&t| t as usize >= self.vocab) {
            Some(t) => Err(Error::Inference(format!(
                "token {t} 超出字表（{}）",
                self.vocab
            ))),
            None => Ok(()),
        }
    }

    fn score_batch(&self, context: &[u32], paths: &[Vec<u32>]) -> CandleResult<Vec<f32>> {
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

    /// 對照用：一條路徑從句首連前文算起，不共用狀態也不補齊。
    #[cfg(test)]
    pub(crate) fn score_one(&self, context: &[u32], path: &[u32]) -> Result<f32, Error> {
        let run = || -> CandleResult<f32> {
            let (mut h, mut c) = self.prefix_state(context)?;
            let mut sum = 0.0;
            for (i, &t) in path.iter().enumerate() {
                sum += self.log_probs(&h)?.i((0, t as usize))?.to_scalar::<f32>()?;
                if i + 1 < path.len() {
                    (h, c) = self.step(&ids(&[t])?, &h, &c)?;
                }
            }
            Ok(sum)
        };
        run().map_err(inference)
    }

    fn step(&self, tokens: &Tensor, h: &Tensor, c: &Tensor) -> CandleResult<(Tensor, Tensor)> {
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

    fn log_probs(&self, h: &Tensor) -> CandleResult<Tensor> {
        log_softmax(&self.embed.forward(h)?, D::Minus1)
    }

    /// 句首記號加前文跑完後的狀態。
    fn prefix_state(&self, context: &[u32]) -> CandleResult<(Tensor, Tensor)> {
        let mut h = Tensor::zeros((1, self.embed_dim), DType::F32, &Device::Cpu)?;
        let mut c = Tensor::zeros((1, self.cells), DType::F32, &Device::Cpu)?;
        for &t in std::iter::once(&SEP).chain(context) {
            (h, c) = self.step(&ids(&[t])?, &h, &c)?;
        }
        Ok((h, c))
    }
}

fn ids(tokens: &[u32]) -> CandleResult<Tensor> {
    Tensor::new(tokens, &Device::Cpu)
}

/// 路徑補齊成 (路徑數, 最長字數)，補 `<sep>`；另回同形狀的遮罩（真的字 1、補的 0）。
fn padded(paths: &[Vec<u32>]) -> CandleResult<(Tensor, Tensor)> {
    let len = paths.iter().map(Vec::len).max().unwrap_or(0);
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
