//! `model-ladder sync`：訓練同步層的模型、掛上核心打各集合，報正確率與成本（README〈同步層〉）。
//!
//! 模型規格（可給多個 `--model`，同種的共用一次計數）：
//!
//! - `bigram`：不掛模型，現行的字級 bigram（同一台機器同一時間量延遲的對照）；
//! - `char3:<門檻>`：字級 trigram，留乘上權重後次數 ≥ 門檻的；
//! - `word2:<門檻>`、`word3:<bigram 門檻>:<trigram 門檻>`：詞級 n-gram；
//! - `kn2:<門檻>`：字級 bigram 重訓，次數與折扣照高階模型（加 `:w` 應重現現行的字級 bigram）；
//! - `mix:<λ>`：比照標準 KN 的字級 bigram（維基、口語分開訓練再內插）。
//!
//! `kn2`、`char3`、`word2`、`word3` 結尾加 `:w` 用現行的折扣（[`Discounting::Weighted`]），否則各語料各算
//! （[`Discounting::PerCorpus`]）。

use std::sync::Arc;

use model_ladder::sync::{
    CharTrigram, Cost, CurrentScoring, Discounting, KnBigram, MixedBigram, TableStats, WordCounts,
    WordNgram, char_bigrams, char_trigrams, count, count_words, train_corpora, train_mixed_parts,
};
use model_ladder::{
    DEFAULT_N, Identity, Set, Split, load_sets, report, run_set, type_item, write_rows,
};
use qim_core::{Engine, LanguageModel, LanguageModelPack, Pack, SchemePack, SegmentModel};

#[derive(Debug, Clone, PartialEq)]
enum Spec {
    Bigram,
    Kn2 {
        min: u64,
        how: Discounting,
    },
    Char3 {
        min: u64,
        how: Discounting,
    },
    Word {
        min2: u64,
        min3: Option<u64>,
        how: Discounting,
    },
    Mix {
        lambda: f64,
    },
}

fn parse_spec(text: &str) -> Result<Spec, String> {
    let bad = || format!("看不懂模型規格 {text}");
    let mut parts: Vec<&str> = text.split(':').collect();
    let how = if parts.last() == Some(&"w") {
        parts.pop();
        Discounting::Weighted
    } else {
        Discounting::PerCorpus
    };
    let num = |s: &str| s.parse::<u64>().map_err(|_| bad());
    Ok(match parts.as_slice() {
        ["bigram"] => Spec::Bigram,
        ["char3", m] => Spec::Char3 { min: num(m)?, how },
        ["kn2", m] => Spec::Kn2 { min: num(m)?, how },
        ["word2", m] => Spec::Word {
            min2: num(m)?,
            min3: None,
            how,
        },
        ["word3", m2, m3] => Spec::Word {
            min2: num(m2)?,
            min3: Some(num(m3)?),
            how,
        },
        ["mix", l] => Spec::Mix {
            lambda: l.parse().map_err(|_| bad())?,
        },
        _ => return Err(bad()),
    })
}

/// 逐題結果的目錄名：規格的冒號換成連字號。
fn dir_name(text: &str) -> String {
    text.replace(':', "-")
}

pub fn sync_command(args: &[String]) -> Result<(), String> {
    let mut split = Split::Dev;
    let mut specs = Vec::new();
    let mut latency = true;
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "--split" => {
                split = match rest.next().map(String::as_str) {
                    Some("dev") => Split::Dev,
                    Some("test") => Split::Test,
                    _ => return Err("--split 要 dev 或 test".into()),
                }
            }
            "--model" => {
                let text = rest.next().ok_or("--model 要給規格")?;
                specs.push((text.clone(), parse_spec(text)?));
            }
            "--no-latency" => latency = false,
            _ => return Err(format!("不認得 {flag}")),
        }
    }
    if specs.is_empty() {
        return Err("至少給一個 --model".into());
    }
    if split == Split::Test {
        eprintln!("注意：test 切分只在設定定案後跑一次報數，調參只用 dev（README〈調參與報數〉）");
    }
    let root = super::root();
    let (engine, _, lexicon) = qim_bench::load_engine(&root, &[], None)?;
    let lexicon = lexicon.ok_or("方案沒有詞庫")?;
    let base = base_model(&root, &engine)?;
    let ranking = engine.scheme().ranking;
    let scoring = CurrentScoring::new(
        base.clone(),
        &lexicon,
        ranking.word_bonus,
        ranking.segment_penalty,
    );
    let corpora = train_corpora(&root);
    let sets = load_sets(&root, split)?;
    let inputs = if latency {
        qim_bench::latency_inputs(&root)?
    } else {
        Vec::new()
    };
    let evaluate = |name: &str, model: Option<Arc<dyn SegmentModel>>, cost: Option<Cost>| {
        let hooked = match model {
            Some(m) => engine.clone().with_segment_model(m),
            None => engine.clone(),
        };
        finish(&root, split, &sets, &hooked, name, cost, &inputs)
    };

    // 同種的規格共用計數：字級 trigram 一次、詞級一次（trigram 有一個要就一起數）。
    if specs.iter().any(|(_, s)| matches!(s, Spec::Char3 { .. })) {
        eprintln!("數字級 trigram…");
        let counts = count(&corpora, |line, emit| char_trigrams(line, emit))?;
        for (text, spec) in &specs {
            if let Spec::Char3 { min, how } = spec {
                let (model, stats) = CharTrigram::train(&base, &counts, *how, *min);
                print_stats(text, "trigram", &stats);
                // `EXPLAIN=甲乙丙,…`：印這些 trigram 的查表結果（除錯用）。
                if let Ok(list) = std::env::var("EXPLAIN") {
                    for t in list.split(',') {
                        let c: Vec<char> = t
                            .chars()
                            .map(|c| if c == '^' { qim_core::CLAUSE_START } else { c })
                            .collect();
                        if let [a, b, x] = c[..] {
                            println!("{t}：{:?}", model.explain(a, b, x));
                        }
                    }
                }
                let cost = model.cost();
                evaluate(text, Some(Arc::new(model)), Some(cost))?;
            }
        }
    }
    let order = specs
        .iter()
        .filter_map(|(_, s)| match s {
            Spec::Word { min3, .. } => Some(if min3.is_some() { 3 } else { 2 }),
            _ => None,
        })
        .max();
    if let Some(order) = order {
        eprintln!("切詞並數詞級 n-gram（{order} 階）…");
        let counts: WordCounts = count_words(&corpora, &scoring, order)?;
        eprintln!("詞級：{} 種段", counts.vocabulary.len());
        for (text, spec) in &specs {
            if let Spec::Word { min2, min3, how } = spec {
                let (model, bi, tri) = WordNgram::train(&scoring, &counts, *how, *min2, *min3);
                print_stats(text, "bigram", &bi);
                if let Some(tri) = tri {
                    print_stats(text, "trigram", &tri);
                }
                let cost = model.cost();
                evaluate(text, Some(Arc::new(model)), Some(cost))?;
            }
        }
    }
    if specs.iter().any(|(_, s)| matches!(s, Spec::Kn2 { .. })) {
        eprintln!("數字級 bigram…");
        let counts = count(&corpora, |line, emit| char_bigrams(line, emit))?;
        let v = vocabulary(&root)?;
        for (text, spec) in &specs {
            if let Spec::Kn2 { min, how } = spec {
                let (model, stats) = KnBigram::train(&base, &counts, v, *how, *min);
                print_stats(text, "bigram", &stats);
                let cost = model.cost();
                evaluate(text, Some(Arc::new(model)), Some(cost))?;
            }
        }
    }
    if specs.iter().any(|(_, s)| matches!(s, Spec::Mix { .. })) {
        eprintln!("比照標準 KN 訓練維基、口語兩份字級 bigram…");
        let (wiki, colloquial) = train_mixed_parts(&corpora, vocabulary(&root)?)?;
        for (text, spec) in &specs {
            if let Spec::Mix { lambda } = spec {
                let model =
                    MixedBigram::new(base.clone(), wiki.clone(), colloquial.clone(), *lambda);
                let cost = model.cost();
                evaluate(text, Some(Arc::new(model)), Some(cost))?;
            }
        }
    }
    for (text, spec) in &specs {
        if *spec == Spec::Bigram {
            evaluate(text, None, None)?;
        }
    }
    Ok(())
}

fn print_stats(spec: &str, order: &str, s: &TableStats) {
    println!(
        "\n{spec} 的 {order}：不重複 {}、留下 {}、前文 {}、α 夾成 1 的前文 {}",
        s.types, s.kept, s.contexts, s.clamped
    );
}

/// 方案的字級 bigram（建置好的資料包）。
fn base_model(root: &std::path::Path, engine: &Engine) -> Result<LanguageModel, String> {
    let id = engine
        .scheme()
        .packs
        .language_model
        .as_ref()
        .ok_or("方案沒有語言模型")?;
    let path = root.join(format!("{}/{id}/{id}.pack", qim_bench::PACKS));
    let bytes = std::fs::read(&path).map_err(|e| format!("{}：{e}", path.display()))?;
    Ok(LanguageModelPack::from_bytes(&bytes)
        .map_err(|e| e.to_string())?
        .model)
}

/// 連打字集的字數（KN 的 `V`，同 `qim-databuild` 建語言模型時）。
fn vocabulary(root: &std::path::Path) -> Result<usize, String> {
    let bytes = std::fs::read(root.join(qim_bench::QUICK_PACK)).map_err(|e| e.to_string())?;
    let pack = SchemePack::from_bytes(&bytes).map_err(|e| e.to_string())?;
    let chars: std::collections::BTreeSet<char> = pack
        .code_table
        .iter()
        .flat_map(|(_, entries)| entries.iter().filter(|e| !e.extension).map(|e| e.text))
        .collect();
    Ok(chars.len())
}

/// 打各集合、印報告與成本、寫逐題結果。
fn finish(
    root: &std::path::Path,
    split: Split,
    sets: &[Set],
    engine: &Engine,
    name: &str,
    cost: Option<Cost>,
    inputs: &[String],
) -> Result<(), String> {
    let runs: Vec<_> = sets
        .iter()
        .map(|set| run_set(engine, set, DEFAULT_N, &mut Identity))
        .collect();
    println!("\n## {name}（{} 切分）\n", split.name());
    print!("{}", report(&runs, name, DEFAULT_N));
    if let Some(c) = cost {
        let memory = c.memory.map_or("沒估".to_string(), |m| {
            format!("{:.1} MB", m as f64 / 1e6)
        });
        println!(
            "\n成本：模型資料包 {} 位元組（{:.2} MB），常駐估計 {memory}，解開 {:.1} ms",
            c.bytes,
            c.bytes as f64 / 1e6,
            c.load_ms
        );
    }
    // 查個別題用：`SHOW=<集合>:<行>,<行>` 印前文結尾、期望與前 3 個候選（個人集不印）。
    if let Ok(show) = std::env::var("SHOW")
        && let Some((set_name, lines)) = show.split_once(':')
    {
        let lines: Vec<usize> = lines.split(',').filter_map(|l| l.parse().ok()).collect();
        for set in sets.iter().filter(|s| s.name == set_name && !s.private) {
            for item in set.items.iter().filter(|i| lines.contains(&i.line)) {
                let typed = type_item(engine, set, item, DEFAULT_N, &mut Identity);
                let before: String = item.context.clone().unwrap_or_default();
                let tail: String = before
                    .chars()
                    .rev()
                    .take(8)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                println!(
                    "{} 第 {} 行：前文…{tail:?} 期望 {} 候選 {:?} 路徑 {:?}",
                    set.name,
                    item.line,
                    item.expected(),
                    &typed.candidates[..typed.candidates.len().min(3)],
                    typed
                        .paths
                        .iter()
                        .take(4)
                        .map(|p| format!("{} {:.2}", p.text, p.score))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    for set in sets.iter().filter(|s| s.name == "common-voice-yue") {
        let (a, b) = say_remember(engine, set);
        println!(
            "common-voice-yue 說／記（同碼 yu）：期望「說」首選「記」{a} 處、期望「記」首選「說」{b} 處"
        );
    }
    if !inputs.is_empty() {
        let timing = qim_bench::native_timing(engine, inputs);
        for (label, summary) in timing.groups() {
            if let Some(s) = summary {
                println!(
                    "每鍵延遲（原生 release）{label}：p50 {:.1} µs、p95 {:.1} µs（{} 鍵）",
                    s.p50 * 1000.0,
                    s.p95 * 1000.0,
                    s.keys
                );
            }
        }
    }
    let dir = root
        .join("build/model-ladder")
        .join(split.name())
        .join(dir_name(name));
    write_rows(&dir, &runs)?;
    eprintln!("逐題結果：{}", dir.display());
    Ok(())
}

/// 首選與期望同長時，同一位置「說」「記」互錯的處數（M1 票 17b：兩字同碼 `yu`）。
fn say_remember(engine: &Engine, set: &Set) -> (usize, usize) {
    let (mut say_as_remember, mut remember_as_say) = (0, 0);
    for item in &set.items {
        let typed = type_item(engine, set, item, DEFAULT_N, &mut Identity);
        let Some(top) = typed.candidates.first() else {
            continue;
        };
        let expected = item.expected();
        if top.chars().count() != expected.chars().count() {
            continue;
        }
        for (e, t) in expected.chars().zip(top.chars()) {
            match (e, t) {
                ('說', '記') => say_as_remember += 1,
                ('記', '說') => remember_as_say += 1,
                _ => {}
            }
        }
    }
    (say_as_remember, remember_as_say)
}
