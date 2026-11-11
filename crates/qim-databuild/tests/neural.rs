//! 停鍵層的模型（plan §2.7、§4.6）：`train.py` 訓練好的模型核對 PyTorch 的分數後量化、匯入
//! `data/neural-rerank/`，再由它建模型資料包。用 `qim-neural` 的極小測試模型。

use std::path::{Path, PathBuf};

use qim_neural::{Format, Reranker, to_gguf};
use tempfile::TempDir;

const CHARS: &str = "我你的是好人天大小中";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../qim-neural/tests/fixtures")
}

/// 照 `train.py` 的放法：`<訓練目錄>/model.safetensors`、`reference.json`、`train.json`，字表在
/// `data/vocab.txt`。字表比模型多 3 個字（字表外的字）。
fn trained(dir: &Path, reference: &str) -> (PathBuf, PathBuf) {
    let model = dir.join("tiny");
    std::fs::create_dir_all(&model).unwrap();
    std::fs::copy(
        fixtures().join("tiny-lstm.safetensors"),
        model.join("model.safetensors"),
    )
    .unwrap();
    std::fs::write(model.join("reference.json"), reference).unwrap();
    std::fs::write(
        model.join("train.json"),
        r#"{"config": {"name": "tiny", "model": "lstm"}, "commit": "abc1234"}"#,
    )
    .unwrap();
    let vocab = dir.join("vocab.txt");
    let lines: Vec<String> = ["<sep>", "<unk>"]
        .into_iter()
        .map(String::from)
        .chain(CHARS.chars().map(String::from))
        .chain(["甲", "乙", "丙"].map(String::from))
        .collect();
    std::fs::write(&vocab, lines.join("\n") + "\n").unwrap();
    (model, vocab)
}

fn reference() -> String {
    std::fs::read_to_string(fixtures().join("tiny-lstm-reference.json")).unwrap()
}

#[test]
fn import_checks_pytorch_scores_and_writes_q8_weights_and_vocab() {
    let tmp = TempDir::new().unwrap();
    let (model, vocab) = trained(tmp.path(), &reference());
    let out = tmp.path().join("data/neural-rerank");
    let imported = qim_databuild::neural::import(&model, &vocab, 2.0, &out).unwrap();
    assert!(imported.reference_max_difference <= 1e-4);
    assert_eq!(imported.cases, 5);

    let safetensors = std::fs::read(fixtures().join("tiny-lstm.safetensors")).unwrap();
    assert_eq!(
        std::fs::read(out.join("model.gguf")).unwrap(),
        to_gguf(&safetensors, Format::Q8).unwrap(),
        "q8 的位元組照 to_gguf"
    );
    let written = std::fs::read_to_string(out.join("vocab.txt")).unwrap();
    assert_eq!(written.lines().count(), 12, "只留模型用到的字：{written}");
    let config = std::fs::read_to_string(out.join("model.toml")).unwrap();
    assert!(config.contains("lambda = 2.0"), "{config}");
    assert!(config.contains("unknown_chars = 3"), "{config}");
    assert!(config.contains("abc1234"), "{config}");

    // 由匯入的檔建模型資料包，交給 qim-neural 重排得了。
    let pack = qim_databuild::neural::pack(&out).unwrap();
    assert_eq!(pack.id, qim_databuild::neural::MODEL_ID);
    assert_eq!(pack.vocab, CHARS.chars().collect::<Vec<_>>());
    assert_eq!(pack.unknown_chars, 3);
    assert_eq!(pack.lambda, 2.0);
    let reranker =
        Reranker::new(&pack.weights, &pack.vocab, pack.unknown_chars, pack.lambda).unwrap();
    assert_eq!(
        reranker
            .rank("你", &[("我的", -1.0), ("好人", -1.0)])
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn import_refuses_scores_that_differ_from_pytorch() {
    let tmp = TempDir::new().unwrap();
    // 第一組的分數改掉 0.001 nat：超過 0.0001 的容許差。
    let tampered = reference().replacen("-2.6524391174316406", "-2.6534391174316406", 1);
    assert_ne!(tampered, reference());
    let (model, vocab) = trained(tmp.path(), &tampered);
    let out = tmp.path().join("out");
    let err = qim_databuild::neural::import(&model, &vocab, 2.0, &out)
        .unwrap_err()
        .to_string();
    assert!(err.contains("reference.json"), "{err}");
    assert!(!out.exists(), "核對沒過不寫檔");
}

#[test]
fn import_refuses_a_vocab_that_does_not_fit_the_model() {
    let tmp = TempDir::new().unwrap();
    let (model, vocab) = trained(tmp.path(), &reference());
    std::fs::write(&vocab, "<sep>\n<unk>\n我\n你\n").unwrap();
    let err = qim_databuild::neural::import(&model, &vocab, 2.0, &tmp.path().join("a"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("字表"), "{err}");
    std::fs::write(&vocab, "我\n你\n").unwrap();
    let err = qim_databuild::neural::import(&model, &vocab, 2.0, &tmp.path().join("b"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("<sep>"), "{err}");
}

#[test]
fn import_refuses_a_bad_lambda() {
    let tmp = TempDir::new().unwrap();
    let (model, vocab) = trained(tmp.path(), &reference());
    assert!(qim_databuild::neural::import(&model, &vocab, -1.0, &tmp.path().join("o")).is_err());
}

/// `--verify` 的抽查用 repo 裡採用的模型：同分的三條路徑裡要把沒有錯字的排第一；權重壞了就不過。
#[test]
fn verify_checks_the_adopted_model_ranks_the_right_sentence_first() {
    use qim_databuild::built::BuiltPack;
    use qim_databuild::verify::{self, CheckId};

    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(qim_databuild::neural::NEURAL_DIR);
    let mut pack = qim_databuild::neural::pack(&dir).unwrap();
    let built = |p: &qim_core::ModelPack| vec![BuiltPack::new("停鍵層模型", p).unwrap()];
    let check = verify::neural_rerank(&built(&pack));
    assert_eq!(check.id, CheckId::NeuralRerank);
    assert!(check.passed, "{}", check.detail);
    // 極小的隨機模型排不出來：抽查看的是排序，不只是載入得了。
    let tmp = TempDir::new().unwrap();
    let (model, vocab) = trained(tmp.path(), &reference());
    let out = tmp.path().join("out");
    qim_databuild::neural::import(&model, &vocab, 2.0, &out).unwrap();
    let tiny = qim_databuild::neural::pack(&out).unwrap();
    assert!(!verify::neural_rerank(&built(&tiny)).passed);
    pack.weights.truncate(100);
    assert!(!verify::neural_rerank(&built(&pack)).passed);
}
