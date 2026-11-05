//! 票 17c：rime-essay 的詞用 OpenCC 標準字形（「羣」「衆」「裏」），列出和常見寫法不同的字：
//! `t2tw` 換掉的字（OpenCC 標準 → 台灣常見）與 `t2hk` 反查（香港寫法 → 常見），附兩字的速成碼、
//! 是否同碼、在不在連打字集、已不在字形對照表（`common_forms`），以及詞庫會收的詞裡含它的詞數。
//! 跑法見同目錄的 README.md。只讀快取，不下載。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ferrous_opencc::OpenCC;
use ferrous_opencc::config::BuiltinConfig;
use qim_databuild::common_forms::CommonForms;
use qim_databuild::sources::{self, Fetch};
use qim_databuild::{Error, lexicon, libcangjie, quick};

struct NoFetch;
impl Fetch for NoFetch {
    fn fetch(&self, url: &str, _: &mut dyn std::io::Write) -> Result<(), String> {
        Err(format!("快取沒有 {url}，先跑 cargo xtask data"))
    }
}

#[derive(Default)]
struct Usage {
    words: usize,
    frequency: u64,
    samples: Vec<(u64, String)>,
}

fn main() -> Result<(), Error> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let all = sources::parse(&std::fs::read_to_string(root.join(sources::SOURCES)).unwrap())?;
    let cache = sources::default_cache_dir().unwrap().join("sources");
    let load = |id: &str, path: &str| -> Result<String, Error> {
        let file = all.source.get(id).unwrap().file(path)?;
        Ok(String::from_utf8(sources::load(&cache, id, file, &NoFetch)?).unwrap())
    };
    let rows = libcangjie::parse(&load("libcangjie", "data/table.txt")?)?;
    let table = quick::code_table(&rows, &BTreeMap::new());
    let forms: BTreeMap<char, char> = CommonForms::from_opencc(&table)?.pairs().collect();

    let mut codes: BTreeMap<char, BTreeSet<&str>> = BTreeMap::new();
    let mut every = BTreeSet::new();
    for (code, entries) in table.iter() {
        for e in entries {
            every.insert(e.text);
            if !e.extension {
                codes.entry(e.text).or_default().insert(code);
            }
        }
    }
    let one = |cc: &OpenCC, c: char| {
        let out = cc.convert(c.encode_utf8(&mut [0; 4]));
        let mut it = out.chars();
        match (it.next(), it.next()) {
            (Some(x), None) => x,
            _ => c,
        }
    };
    let tw = OpenCC::from_config(BuiltinConfig::T2tw).unwrap();
    let hk = OpenCC::from_config(BuiltinConfig::T2hk).unwrap();
    // 香港寫法 → 原字
    let mut hk_rev: BTreeMap<char, char> = BTreeMap::new();
    for &c in &every {
        let h = one(&hk, c);
        if h != c {
            hk_rev.insert(h, c);
        }
    }

    // rime-essay 的詞（兩字以上、詞頻達門檻、同一詞取最大詞頻）
    let essay = load("rime_essay", "essay.txt")?;
    let mut words: BTreeMap<&str, u64> = BTreeMap::new();
    for line in essay.lines() {
        let (w, f) = line.split_once('\t').unwrap();
        let f: u64 = f.parse().unwrap();
        if w.chars().count() >= 2 {
            let k = words.entry(w).or_insert(f);
            *k = (*k).max(f);
        }
    }
    let mut usage: BTreeMap<char, Usage> = BTreeMap::new();
    let mut usage_any: BTreeMap<char, usize> = BTreeMap::new();
    for (&w, &f) in &words {
        if f < lexicon::MIN_FREQUENCY {
            continue;
        }
        // 換成常見寫法之後字都在連打字集才算「會收」
        let target = |c: char| one(&tw, c);
        let kept = w
            .chars()
            .all(|c| codes.contains_key(&c) || codes.contains_key(&target(c)));
        let seen: BTreeSet<char> = w.chars().collect();
        for c in seen {
            *usage_any.entry(c).or_default() += 1;
            if kept {
                let u = usage.entry(c).or_default();
                u.words += 1;
                u.frequency += f;
                u.samples.push((f, w.to_string()));
            }
        }
    }

    let fmt = |c: char| {
        codes
            .get(&c)
            .map(|s| s.iter().copied().collect::<Vec<_>>().join(","))
            .unwrap_or_else(|| {
                if every.contains(&c) {
                    "擴展區".into()
                } else {
                    "無".into()
                }
            })
    };
    println!(
        "來源\t字\t常見\t已在對照表\t字的碼\t常見的碼\t同碼\t會收的詞數\t詞頻和\t全部詞數\t例"
    );
    let mut rows_out: Vec<(&str, char, char)> = Vec::new();
    for &c in usage_any.keys() {
        let t = one(&tw, c);
        if t != c {
            rows_out.push(("t2tw", c, t));
        }
        if let Some(&orig) = hk_rev.get(&c) {
            rows_out.push(("t2hk 反查", c, orig));
        }
    }
    rows_out.sort_by_key(|&(_, c, _)| std::cmp::Reverse(usage.get(&c).map_or(0, |u| u.words)));
    for (src, c, t) in rows_out {
        let u = usage.get(&c);
        let mut samples = u.map(|u| u.samples.clone()).unwrap_or_default();
        samples.sort_by_key(|s| std::cmp::Reverse(s.0));
        let same = match (codes.get(&c), codes.get(&t)) {
            (Some(a), Some(b)) => {
                if a.is_disjoint(b) {
                    "否"
                } else {
                    "是"
                }
            }
            _ => "—",
        };
        println!(
            "{src}\t{c}\t{t}\t{}\t{}\t{}\t{same}\t{}\t{}\t{}\t{}",
            if forms.get(&c) == Some(&t) {
                "是"
            } else {
                "否"
            },
            fmt(c),
            fmt(t),
            u.map_or(0, |u| u.words),
            u.map_or(0, |u| u.frequency),
            usage_any[&c],
            samples
                .iter()
                .take(4)
                .map(|s| format!("{}({})", s.1, s.0))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    if std::env::var("HK").is_ok() {
        println!();
        println!("t2hk 換成不同碼香港寫法的連打字集字（原字\t香港寫法\t原字碼\t香港碼）");
        for &c in codes.keys() {
            let h = one(&hk, c);
            if h != c && codes.contains_key(&h) && codes[&c].is_disjoint(&codes[&h]) {
                println!("{c}\t{h}\t{}\t{}", fmt(c), fmt(h));
            }
        }
    }
    Ok(())
}
