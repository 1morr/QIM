//! candle 的推論對得上 PyTorch：`scripts/neural-rerank/train.py` 寫的參考分數（路徑各字 log 機率
//! 的和），f32 差 ≤ 0.0001 nat。極小的測試模型（`train.py fixture`）一定跑；本機訓練過真的模型
//! （`build/neural-rerank/lstm-c0.5/`）時另比它。

use std::path::{Path, PathBuf};

use qim_neural::{Format, Model, to_gguf};
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    context: Vec<u32>,
    path: Vec<u32>,
    score: f64,
}

const TOLERANCE: f64 = 1e-4;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// 每組前文 + 路徑用 `score`（前綴快取、補齊遮罩那條路）算一條，和參考分數的最大差。
fn max_difference(model: &Model, cases: &[Case]) -> f64 {
    cases
        .iter()
        .map(|c| {
            let ours = model
                .score(&c.context, std::slice::from_ref(&c.path))
                .unwrap()[0];
            (f64::from(ours) - c.score).abs()
        })
        .fold(0.0, f64::max)
}

fn load(weights: &Path, reference: &Path) -> (Vec<u8>, Vec<Case>) {
    let cases = serde_json::from_str(&std::fs::read_to_string(reference).unwrap()).unwrap();
    (std::fs::read(weights).unwrap(), cases)
}

#[test]
fn tiny_model_matches_pytorch() {
    let dir = fixtures();
    let (weights, cases) = load(
        &dir.join("tiny-lstm.safetensors"),
        &dir.join("tiny-lstm-reference.json"),
    );
    let model = Model::from_gguf(&to_gguf(&weights, Format::F32).unwrap()).unwrap();
    assert_eq!(model.vocab_size(), 12);
    let diff = max_difference(&model, &cases);
    assert!(diff <= TOLERANCE, "f32 和 PyTorch 差 {diff} nat");
}

#[test]
fn tiny_model_loads_as_q8() {
    let dir = fixtures();
    let (weights, cases) = load(
        &dir.join("tiny-lstm.safetensors"),
        &dir.join("tiny-lstm-reference.json"),
    );
    let q8 = to_gguf(&weights, Format::Q8).unwrap();
    let f32 = to_gguf(&weights, Format::F32).unwrap();
    assert!(
        q8.len() < f32.len(),
        "q8 {} 位元組、f32 {}",
        q8.len(),
        f32.len()
    );
    let model = Model::from_gguf(&q8).unwrap();
    // 量化有誤差，只要求同一個量級。
    let diff = max_difference(&model, &cases);
    assert!(diff < 1.0, "q8 和 PyTorch 差 {diff} nat");
}

#[test]
fn trained_model_matches_pytorch_when_present() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../build/neural-rerank/lstm-c0.5");
    let (weights, reference) = (dir.join("model.safetensors"), dir.join("reference.json"));
    if !weights.exists() || !reference.exists() {
        eprintln!("略過：沒有 {}", dir.display());
        return;
    }
    let (weights, cases) = load(&weights, &reference);
    let model = Model::from_gguf(&to_gguf(&weights, Format::F32).unwrap()).unwrap();
    let diff = max_difference(&model, &cases);
    assert!(diff <= TOLERANCE, "f32 和 PyTorch 差 {diff} nat");
}

#[test]
fn rejects_files_that_are_not_models() {
    assert!(Model::from_gguf(b"not a gguf").is_err());
    assert!(to_gguf(b"not safetensors", Format::Q8).is_err());
}
