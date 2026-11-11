//! 停鍵層的模型（plan §2.7、§4.6）：把 `scripts/neural-rerank/train.py` 訓練好的模型匯入
//! [`NEURAL_DIR`]（`cargo xtask neural-import`），`cargo xtask data` 再由它建模型資料包。
//!
//! 訓練不是決定性的，重訓的模型分數會變，所以採用的那份放進 git，不在 `cargo xtask data` 裡訓練。
//! 匯入時用 `train.py` 寫的 `reference.json` 核對 PyTorch 與 candle（f32）的分數，再量化成 q8。

use std::path::Path;

use qim_core::ModelPack;
use qim_neural::{Format, Model, Reranker, to_gguf};
use serde::{Deserialize, Serialize};

use crate::Error;
use crate::sources::sha256_hex;

/// 採用的模型，相對於 repo 根目錄。
pub const NEURAL_DIR: &str = "data/neural-rerank";
/// q8 的 GGUF 權重。
pub const WEIGHTS: &str = "model.gguf";
/// 字表：`train.py prepare` 的字表只留模型用到的前幾行（含兩個記號）。
pub const VOCAB: &str = "vocab.txt";
/// λ、字表外的字數與訓練出處（[`Config`]）。
pub const CONFIG: &str = "model.toml";
/// 模型資料包的 id。
pub const MODEL_ID: &str = "zh-hant-neural-rerank";
/// PyTorch 與 candle（f32）的分數最多差多少 nat。
pub const REFERENCE_TOLERANCE: f64 = 1e-4;

/// [`CONFIG`] 的內容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// `路徑分 + λ·神經分` 的 λ（在 dev 切分上調，`scripts/experiments/model-ladder`）。
    pub lambda: f64,
    /// 訓練語料裡字表外的字數（`train.py` 的字表行數減去模型的字表大小）。
    pub unknown_chars: u32,
    pub trained: Trained,
}

/// 這份模型從哪來，給人查。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trained {
    /// `train.py train --name` 的名字。
    pub name: String,
    /// 訓練時的 commit（`train.json`）。
    pub commit: String,
    /// 匯入的 `model.safetensors`（f32）的 sha256。
    pub safetensors_sha256: String,
    /// 和 `reference.json` 的分數最多差多少 nat（f32）。
    pub reference_max_difference: f64,
}

/// 匯入的結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    /// 核對了幾組前文 + 路徑。
    pub cases: usize,
    pub reference_max_difference: f64,
    /// q8 權重的位元組數。
    pub weights_bytes: usize,
    pub config: Config,
}

#[derive(Deserialize)]
struct Case {
    context: Vec<u32>,
    path: Vec<u32>,
    score: f64,
}

#[derive(Deserialize)]
struct TrainJson {
    config: TrainConfig,
    commit: String,
}

#[derive(Deserialize)]
struct TrainConfig {
    name: String,
}

fn read(path: &Path) -> Result<Vec<u8>, Error> {
    std::fs::read(path).map_err(Error::io(path))
}

fn read_text(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(Error::io(path))
}

fn invalid_model(path: &Path) -> impl FnOnce(qim_neural::Error) -> Error {
    let what = path.display().to_string();
    move |e| Error::invalid(what, e.to_string())
}

/// 匯入 `model_dir`（`train.py train` 的輸出目錄）的模型與 `vocab`（`train.py prepare` 的字表）到
/// `out`：先用 f32 核對 `reference.json`，差超過 [`REFERENCE_TOLERANCE`] 就不寫任何檔。
pub fn import(model_dir: &Path, vocab: &Path, lambda: f64, out: &Path) -> Result<Imported, Error> {
    if !lambda.is_finite() || lambda < 0.0 {
        return Err(Error::invalid("λ", format!("要是有限的非負數：{lambda}")));
    }
    let safetensors_path = model_dir.join("model.safetensors");
    let safetensors = read(&safetensors_path)?;
    let reference_path = model_dir.join("reference.json");
    let cases: Vec<Case> = serde_json::from_str(&read_text(&reference_path)?)
        .map_err(|e| Error::invalid(reference_path.display().to_string(), e.to_string()))?;
    let train_path = model_dir.join("train.json");
    let train: TrainJson = serde_json::from_str(&read_text(&train_path)?)
        .map_err(|e| Error::invalid(train_path.display().to_string(), e.to_string()))?;

    let f32 = to_gguf(&safetensors, Format::F32)
        .and_then(|g| Model::from_gguf(&g))
        .map_err(invalid_model(&safetensors_path))?;
    if cases.is_empty() {
        return Err(Error::invalid(
            reference_path.display().to_string(),
            "沒有可核對的分數",
        ));
    }
    let mut max_difference: f64 = 0.0;
    for case in &cases {
        let ours = f32
            .score(&case.context, std::slice::from_ref(&case.path))
            .map_err(invalid_model(&safetensors_path))?[0];
        max_difference = max_difference.max((f64::from(ours) - case.score).abs());
    }
    if max_difference > REFERENCE_TOLERANCE {
        return Err(Error::invalid(
            reference_path.display().to_string(),
            format!(
                "candle（f32）和 PyTorch 的分數最多差 {max_difference:.6} nat，超過 \
                 {REFERENCE_TOLERANCE}：模型結構或權重名和 qim-neural 對不上"
            ),
        ));
    }

    let size = f32.vocab_size();
    let text = read_text(vocab)?;
    let lines: Vec<&str> = text.lines().collect();
    let what = vocab.display().to_string();
    if lines.first() != Some(&"<sep>") || lines.get(1) != Some(&"<unk>") {
        return Err(Error::invalid(
            what,
            "前兩行要是 <sep>、<unk>（train.py prepare 寫的字表）",
        ));
    }
    if lines.len() < size {
        return Err(Error::invalid(
            what,
            format!("字表 {} 行，模型要 {size} 個 token", lines.len()),
        ));
    }
    let kept = &lines[..size];
    let chars = vocab_chars(kept[2..].iter().copied(), &what)?;
    let weights = to_gguf(&safetensors, Format::Q8).map_err(invalid_model(&safetensors_path))?;
    let unknown_chars = (lines.len() - size).max(1) as u32;
    // 寫檔前照殼的路載入一次：字表重複的字、和模型對不上的都在這裡擋下。
    Reranker::new(&weights, &chars, unknown_chars, lambda).map_err(invalid_model(vocab))?;

    let config = Config {
        lambda,
        unknown_chars,
        trained: Trained {
            name: train.config.name,
            commit: train.commit,
            safetensors_sha256: sha256_hex(&safetensors),
            reference_max_difference: max_difference,
        },
    };
    let toml = toml::to_string(&config).map_err(|e| Error::invalid(CONFIG, e.to_string()))?;
    std::fs::create_dir_all(out).map_err(Error::io(out))?;
    let write = |name: &str, bytes: &[u8]| {
        let path = out.join(name);
        std::fs::write(&path, bytes).map_err(Error::io(path))
    };
    write(WEIGHTS, &weights)?;
    write(VOCAB, (kept.join("\n") + "\n").as_bytes())?;
    write(
        CONFIG,
        format!("# cargo xtask neural-import 寫的，不要手改（docs/development.md〈停鍵層的模型〉）\n{toml}")
            .as_bytes(),
    )?;
    Ok(Imported {
        cases: cases.len(),
        reference_max_difference: max_difference,
        weights_bytes: weights.len(),
        config,
    })
}

/// 字表兩個記號之後的行，一行一個字；`what` 是錯誤訊息裡的檔名。
fn vocab_chars<'a>(lines: impl Iterator<Item = &'a str>, what: &str) -> Result<Vec<char>, Error> {
    lines
        .map(|l| {
            let mut chars = l.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(c),
                _ => Err(Error::invalid(what, format!("一行要是一個字：{l:?}"))),
            }
        })
        .collect()
}

/// 由匯入的檔（`dir` 通常是 [`NEURAL_DIR`]）建模型資料包。
pub fn pack(dir: &Path) -> Result<ModelPack, Error> {
    let config_path = dir.join(CONFIG);
    let config: Config = toml::from_str(&read_text(&config_path)?)
        .map_err(|e| Error::invalid(config_path.display().to_string(), e.to_string()))?;
    let vocab_path = dir.join(VOCAB);
    let text = read_text(&vocab_path)?;
    let vocab = vocab_chars(text.lines().skip(2), &vocab_path.display().to_string())?;
    Ok(ModelPack {
        id: MODEL_ID.into(),
        vocab,
        unknown_chars: config.unknown_chars,
        lambda: config.lambda,
        weights: read(&dir.join(WEIGHTS))?,
    })
}
