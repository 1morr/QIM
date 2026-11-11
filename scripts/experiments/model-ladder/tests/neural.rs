//! 神經重排的打分：前綴快取、補齊遮罩的批次分數要等於一條一條從句首算的分數（f32 與 q8）。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use candle_core::{DType, Device, Tensor};
use model_ladder::neural::{Format, load, quantize};

/// 小形狀的隨機權重，名字照 `train.py` 的匯出。
fn write(path: &Path, shapes: &[(&str, Vec<usize>)]) {
    let dev = Device::Cpu;
    let map: HashMap<String, Tensor> = shapes
        .iter()
        .map(|(name, shape)| {
            let t = if shape.len() == 2 || !name.contains("ln") {
                Tensor::randn(0f32, 0.3, shape.as_slice(), &dev).unwrap()
            } else {
                Tensor::ones(shape.as_slice(), DType::F32, &dev).unwrap()
            };
            (name.to_string(), t)
        })
        .collect();
    candle_core::safetensors::save(&map, path).unwrap();
}

fn gpt_shapes(vocab: usize, d: usize, layers: usize, context: usize) -> Vec<(String, Vec<usize>)> {
    let mut v = vec![
        ("wte".into(), vec![vocab, d]),
        ("wpe".into(), vec![context, d]),
    ];
    for l in 0..layers {
        let p = format!("h.{l}");
        for (n, s) in [
            ("ln_1.weight", vec![d]),
            ("ln_1.bias", vec![d]),
            ("attn.c_attn.weight", vec![3 * d, d]),
            ("attn.c_attn.bias", vec![3 * d]),
            ("attn.c_proj.weight", vec![d, d]),
            ("attn.c_proj.bias", vec![d]),
            ("ln_2.weight", vec![d]),
            ("ln_2.bias", vec![d]),
            ("mlp.c_fc.weight", vec![4 * d, d]),
            ("mlp.c_fc.bias", vec![4 * d]),
            ("mlp.c_proj.weight", vec![d, 4 * d]),
            ("mlp.c_proj.bias", vec![d]),
        ] {
            v.push((format!("{p}.{n}"), s));
        }
    }
    v.push(("ln_f.weight".into(), vec![d]));
    v.push(("ln_f.bias".into(), vec![d]));
    v
}

fn files(tag: &str) -> (PathBuf, Vec<PathBuf>) {
    let dir =
        std::env::temp_dir().join(format!("model-ladder-neural-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let gpt = dir.join("gpt.safetensors");
    let shapes = gpt_shapes(50, 64, 2, 32);
    write(
        &gpt,
        &shapes
            .iter()
            .map(|(n, s)| (n.as_str(), s.clone()))
            .collect::<Vec<_>>(),
    );
    let lstm = dir.join("lstm.safetensors");
    write(
        &lstm,
        &[
            ("embed", vec![50, 32]),
            ("gates.weight", vec![3 * 40, 64]),
            ("gates.bias", vec![3 * 40]),
            ("proj.weight", vec![32, 40]),
        ],
    );
    let mut all = vec![gpt.clone(), lstm.clone()];
    for f in [gpt, lstm] {
        let q = f.with_extension("gguf");
        quantize(&f, &q).unwrap();
        all.push(q);
    }
    (dir, all)
}

#[test]
fn batched_scores_with_padding_match_one_path_at_a_time() {
    let (dir, paths) = files("batch");
    let context = [5, 9, 13, 0, 7];
    // 長度不同的路徑：補齊的位置不能算進分數。
    let cands = vec![
        vec![2, 3, 4, 5],
        vec![7, 7],
        vec![10, 20, 30, 40, 41, 42],
        vec![1],
    ];
    for path in &paths {
        let loaded = load(path).unwrap();
        let want_format = if path.extension().unwrap() == "gguf" {
            Format::Q8
        } else {
            Format::F32
        };
        assert_eq!(loaded.format, want_format);
        for ctx in [&context[..], &[]] {
            let got = loaded.scorer.score(ctx, &cands).unwrap();
            for (g, c) in got.iter().zip(&cands) {
                let w = loaded.scorer.score_one(ctx, c).unwrap();
                assert!(
                    g.is_finite() && (g - w).abs() < 1e-3,
                    "{}: {g} vs {w}",
                    path.display()
                );
            }
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn shapes_are_read_from_the_weights() {
    let (dir, paths) = files("shapes");
    let gpt = load(&paths[0]).unwrap();
    assert_eq!(
        (gpt.kind, gpt.scorer.vocab(), gpt.scorer.max_positions()),
        ("gpt", 50, Some(32))
    );
    let lstm = load(&paths[1]).unwrap();
    assert_eq!(
        (lstm.kind, lstm.scorer.vocab(), lstm.scorer.max_positions()),
        ("lstm", 50, None)
    );
    assert_eq!(lstm.params, 50 * 32 + 120 * 64 + 120 + 32 * 40);
    std::fs::remove_dir_all(dir).unwrap();
}
