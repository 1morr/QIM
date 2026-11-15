//! 句中英文混輸值不值得做（M4 票 28）：用法與量法見同目錄的 README。在 repo 根目錄執行。

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use qim_bench::{Item, Unit, load_engine, score_set};
use qim_core::{Engine, EnglishPack, EnglishWords, InlineEnglish, Pack, SchemePack};
use qim_databuild::common_forms::CommonForms;
use qim_databuild::corpora::{CORPORA, Converter, Corpus};
use qim_databuild::corpus::{Split, split_of};
use qim_databuild::hkcancor;
use qim_databuild::libcangjie;
use qim_databuild::sources::{self, Fetch};
use qim_databuild::transcode::Transcoder;
use sha2::{Digest, Sha256};

const OUT_DIR: &str = "build/inline-english";
/// 每個語料、每個切分最多取幾句（同首選字基準的 [`qim_databuild::corpus::BENCH_SENTENCES`]）。
const PER_CORPUS: usize = 2000;

const USAGE: &str = "用法：inline-english extract | mixed <dev|test> <懲罰,最短長度,最低頻率>... | chinese <off|懲罰,最短長度,最低頻率> | report <設定>... | words <dev|test>";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["extract"] => extract(),
        ["mixed", split, configs @ ..] if !configs.is_empty() => mixed(split, configs),
        ["chinese", config] => chinese(config),
        ["report", configs @ ..] if !configs.is_empty() => report(configs),
        ["words", split] => words(split),
        _ => Err(USAGE.to_string()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------- 抽題

/// 一句混輸的句子：中文一段段、英文一個個詞，依原順序。
#[derive(Debug, Clone)]
struct Mixed {
    corpus: String,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
enum Token {
    /// 一段中文和每個字的速成碼。
    Chinese(Vec<(char, String)>),
    /// 英文詞，原樣（大小寫照原文）。
    English(String),
}

struct NoFetch;

impl Fetch for NoFetch {
    fn fetch(&self, url: &str, _: &mut dyn std::io::Write) -> Result<(), String> {
        Err(format!("快取裡沒有 {url}；先跑 `cargo xtask data`"))
    }
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// 從四個語料抽混輸的句子，寫 `build/inline-english/{dev,test}.tsv`。
fn extract() -> Result<(), String> {
    let root = PathBuf::from(".");
    let all = sources::parse(&fs::read_to_string(sources::SOURCES).map_err(err)?).map_err(err)?;
    let cache = sources::default_cache_dir()
        .ok_or("找不到快取目錄")?
        .join("sources");
    let source = |id: &str| all.source.get(id).ok_or(format!("沒有 [source.{id}]"));
    let libcangjie_source = source("libcangjie")?;
    let table = sources::load(
        &cache,
        "libcangjie",
        libcangjie_source.file("data/table.txt").map_err(err)?,
        &NoFetch,
    )
    .map_err(err)?;
    let rows = libcangjie::parse(&String::from_utf8(table).map_err(err)?).map_err(err)?;
    let transcoder = Transcoder::new(&rows);
    let quick = SchemePack::from_bytes(&fs::read(root.join(qim_bench::QUICK_PACK)).map_err(err)?)
        .map_err(err)?;
    let forms = CommonForms::from_opencc(&quick.code_table).map_err(err)?;
    fs::create_dir_all(OUT_DIR).map_err(err)?;
    let mut outputs: BTreeMap<&str, String> = BTreeMap::new();
    for corpus in &CORPORA {
        let src = source(corpus.source_id)?;
        let paths = src
            .files
            .iter()
            .map(|f| sources::cached(&cache, corpus.source_id, f, &NoFetch))
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        let (mut dev, mut test): (HashSet<String>, HashSet<String>) = Default::default();
        let mut take = |text: &str| {
            for piece in pieces(text) {
                if let Some(tokens) = mixed_tokens(&piece, &transcoder) {
                    let line = render_tokens(&tokens);
                    let key = line.replace(' ', "");
                    match split_of(&key) {
                        Split::Dev => dev.insert(line),
                        Split::Test => test.insert(line),
                        Split::Train => false,
                    };
                }
            }
        };
        texts(corpus, &paths, &forms, &mut take)?;
        for (found, name) in [(dev, "dev"), (test, "test")] {
            let mut lines: Vec<String> = found.into_iter().collect();
            lines.sort_by_key(|l| (hash(l), l.clone()));
            let total = lines.len();
            lines.truncate(PER_CORPUS);
            println!("{} {name}：{total} 句，取 {}", corpus.name, lines.len());
            let out = outputs.entry(name).or_default();
            for line in lines {
                out.push_str(&format!("{}\t{line}\n", corpus.name));
            }
        }
    }
    for (name, text) in outputs {
        let path = Path::new(OUT_DIR).join(format!("{name}.tsv"));
        fs::write(&path, text).map_err(err)?;
        println!("寫出 {}", path.display());
    }
    Ok(())
}

/// 語料的每段文字，轉換同 `cargo xtask data`。HKCanCor 是分詞的語料、詞直接接起來，相鄰的英文詞會黏成一串，
/// 所以這裡從分詞的詞接、英文詞之間補空格。
fn texts(
    corpus: &Corpus,
    paths: &[PathBuf],
    forms: &CommonForms,
    take: &mut dyn FnMut(&str),
) -> Result<(), String> {
    if corpus.segmented() {
        let converter = Converter::new(corpus.conversion, forms.clone()).map_err(err)?;
        for path in paths {
            let zip = fs::read(path).map_err(err)?;
            hkcancor::archive_tagged(&zip, |words| {
                let mut raw = String::new();
                for w in words {
                    let ascii = |s: &str| s.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
                    if ascii(w.text) && raw.chars().last().is_some_and(|c| c.is_ascii_alphabetic())
                    {
                        raw.push(' ');
                    }
                    raw.push_str(w.text);
                }
                take(&converter.convert(&raw));
            })
            .map_err(err)?;
        }
        Ok(())
    } else {
        corpus
            .texts(paths, forms, |text| {
                take(text);
                Ok(())
            })
            .map_err(err)
    }
}

/// 照標點與符號切開（空白不切）：首選字基準的句子是標點、空白與符號之間的字，這裡留著空白，英文詞前後常有空白。
/// 兩邊都不是英文字母的空白照樣切開。
fn pieces(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in text.split(|c: char| !(c.is_alphanumeric() || c == ' ')) {
        let chars: Vec<char> = part.chars().collect();
        let mut current = String::new();
        for (i, &c) in chars.iter().enumerate() {
            if c == ' ' {
                let left = chars[..i].iter().rev().find(|&&c| c != ' ');
                let right = chars[i + 1..].iter().find(|&&c| c != ' ');
                let latin = |c: Option<&char>| c.is_some_and(|c| c.is_ascii_alphabetic());
                if !latin(left) && !latin(right) {
                    out.push(std::mem::take(&mut current));
                    continue;
                }
            }
            current.push(c);
        }
        out.push(current);
    }
    out.retain(|p| !p.trim().is_empty());
    out
}

/// 一段文字是混輸的句子時的中英各段：只有連打字集的字、英文字母與空白，中文與英文都至少有一個；有數字的不收。
fn mixed_tokens(piece: &str, transcoder: &Transcoder) -> Option<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    for c in piece.chars() {
        if c == ' ' {
            // 英文詞之間的空白分開兩個詞。
            if let Some(Token::English(_)) = tokens.last() {
                tokens.push(Token::English(String::new()));
            }
            continue;
        }
        if c.is_ascii_alphabetic() {
            match tokens.last_mut() {
                Some(Token::English(w)) => w.push(c),
                _ => tokens.push(Token::English(c.to_string())),
            }
            continue;
        }
        let code = transcoder.code(c)?.to_string();
        match tokens.last_mut() {
            Some(Token::Chinese(run)) => run.push((c, code)),
            _ => tokens.push(Token::Chinese(vec![(c, code)])),
        }
    }
    tokens.retain(|t| !matches!(t, Token::English(w) if w.is_empty()));
    let has_english = tokens.iter().any(|t| matches!(t, Token::English(_)));
    let has_chinese = tokens.iter().any(|t| matches!(t, Token::Chinese(_)));
    (has_english && has_chinese).then_some(tokens)
}

/// 一行：中文段寫成「字碼 字碼…」以 `/` 接字與碼，英文詞原樣，各段以空白分開。
fn render_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|t| match t {
            Token::Chinese(run) => run
                .iter()
                .map(|(c, code)| format!("{c}/{code}"))
                .collect::<Vec<_>>()
                .join("+"),
            Token::English(w) => w.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_mixed(text: &str) -> Result<Vec<Mixed>, String> {
    text.lines()
        .map(|line| {
            let (corpus, rest) = line.split_once('\t').ok_or(format!("壞的行：{line}"))?;
            let tokens = rest
                .split(' ')
                .map(|t| {
                    if t.contains('/') {
                        t.split('+')
                            .map(|cc| {
                                let (c, code) =
                                    cc.split_once('/').ok_or(format!("壞的字：{cc}"))?;
                                let c = c.chars().next().ok_or("空的字")?;
                                Ok((c, code.to_string()))
                            })
                            .collect::<Result<Vec<_>, String>>()
                            .map(Token::Chinese)
                    } else {
                        Ok(Token::English(t.to_string()))
                    }
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Mixed {
                corpus: corpus.to_string(),
                tokens,
            })
        })
        .collect()
}

fn hash(s: &str) -> u64 {
    let d = Sha256::digest(s.as_bytes());
    u64::from_be_bytes(d[..8].try_into().expect("sha256 有 32 位元組"))
}

fn load_mixed(split: &str) -> Result<Vec<Mixed>, String> {
    let path = Path::new(OUT_DIR).join(format!("{split}.tsv"));
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("讀不到 {}：{e}；先跑 extract", path.display()))?;
    parse_mixed(&text)
}

// ---------------------------------------------------------------- 混輸句的模擬

/// 使用者怎麼打一句混輸的句子。
#[derive(Debug, Clone, Copy)]
enum Typist {
    /// 現在的做法（基準）：中文選字上屏後，英文詞單獨打，規則 4 出得了的從第 2 位選（算 1 次修正），出不了的按
    /// Shift 直打。
    Rule4,
    /// 敏感度用的另一個基準：英文詞一律按 Shift 直打。
    AllShift,
    /// 句中英文混輸開著：收得進 lattice 的英文詞和前後的中文一起連打，其餘照 [`Typist::Rule4`]。
    Inline(InlineEnglish),
}

fn letters_a_to_y(w: &str) -> bool {
    !w.is_empty() && w.chars().all(|c| ('a'..='y').contains(&c))
}

/// 規則 4 出得了：拼對的完整詞、長過速成一個字的最長碼（2 碼）、整個詞都是方案的字母。
fn rule4(english: &EnglishWords, w: &str) -> bool {
    let lower = w.to_lowercase();
    lower.len() > 2 && letters_a_to_y(&lower) && english.lookup(&lower).is_some()
}

/// 句中英文混輸收得進 lattice（同 `aec9741` 的 `Engine::inline_english_edge`）。
fn inline_covers(english: &EnglishWords, inline: &InlineEnglish, w: &str) -> bool {
    let lower = w.to_lowercase();
    lower.len() >= inline.min_len
        && letters_a_to_y(&lower)
        && english
            .frequency(&lower)
            .is_some_and(|f| f >= inline.min_frequency)
}

/// 一句照 `typist` 拆成幾次組字（每次從空的組字打起、打到上屏），與 Shift 直打的英文詞。
struct Plan {
    chunks: Vec<Vec<Unit>>,
    /// Shift 直打的詞：按的鍵數（字母加切換兩次）。
    shift_keys: usize,
    shift_words: usize,
}

fn plan(m: &Mixed, typist: Typist, english: &EnglishWords) -> Plan {
    let mut plan = Plan {
        chunks: Vec::new(),
        shift_keys: 0,
        shift_words: 0,
    };
    let mut current: Vec<Unit> = Vec::new();
    let english_unit = |w: &str| {
        let code = w.to_lowercase();
        let text = english.lookup(&code).unwrap_or(w).to_string();
        Unit { code, text }
    };
    for token in &m.tokens {
        match token {
            Token::Chinese(run) => current.extend(run.iter().map(|(c, code)| Unit {
                code: code.clone(),
                text: c.to_string(),
            })),
            Token::English(w) => {
                if let Typist::Inline(inline) = typist
                    && inline_covers(english, &inline, w)
                {
                    current.push(english_unit(w));
                    continue;
                }
                if !current.is_empty() {
                    plan.chunks.push(std::mem::take(&mut current));
                }
                if !matches!(typist, Typist::AllShift) && rule4(english, w) {
                    plan.chunks.push(vec![english_unit(w)]);
                } else {
                    plan.shift_keys += w.len() + 2;
                    plan.shift_words += 1;
                }
            }
        }
    }
    if !current.is_empty() {
        plan.chunks.push(current);
    }
    plan
}

#[derive(Debug, Default, Clone, Copy)]
struct Tally {
    sentences: usize,
    words: usize,
    /// 句中英文混輸收得進 lattice 的詞（只在 [`Typist::Inline`]）。
    covered: usize,
    shift_words: usize,
    corrections: usize,
    keys: usize,
    /// 一次也不用選字、翻頁就打完的句子。
    zero: usize,
    unfinished: usize,
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.sentences += o.sentences;
        self.words += o.words;
        self.covered += o.covered;
        self.shift_words += o.shift_words;
        self.corrections += o.corrections;
        self.keys += o.keys;
        self.zero += o.zero;
        self.unfinished += o.unfinished;
    }
}

/// 照 `typist` 打完每一句，依語料加總。
fn simulate(
    engine: &Engine,
    english: &EnglishWords,
    items: &[Mixed],
    typist: Typist,
) -> BTreeMap<String, Tally> {
    let plans: Vec<Plan> = items.iter().map(|m| plan(m, typist, english)).collect();
    let mut flat: Vec<Item> = Vec::new();
    let mut owner: Vec<usize> = Vec::new();
    for (i, p) in plans.iter().enumerate() {
        for units in &p.chunks {
            owner.push(i);
            flat.push(Item {
                units: units.clone(),
                line: flat.len() + 1,
                label: String::new(),
                weight: 1,
                context: None,
                typed: None,
                retyped: None,
            });
        }
    }
    let result = score_set(engine, "mixed", &flat, false);
    let mut per: Vec<Tally> = plans
        .iter()
        .zip(items)
        .map(|(p, m)| {
            let words: Vec<&String> = m
                .tokens
                .iter()
                .filter_map(|t| match t {
                    Token::English(w) => Some(w),
                    Token::Chinese(_) => None,
                })
                .collect();
            let covered = match typist {
                Typist::Inline(inline) => words
                    .iter()
                    .filter(|w| inline_covers(english, &inline, w))
                    .count(),
                _ => 0,
            };
            Tally {
                sentences: 1,
                words: words.len(),
                covered,
                shift_words: p.shift_words,
                keys: p.shift_keys,
                ..Tally::default()
            }
        })
        .collect();
    let mut unfinished = vec![false; items.len()];
    for (r, &i) in result.items.iter().zip(&owner) {
        per[i].corrections += r.correction_count;
        per[i].keys += r.keys;
        unfinished[i] |= !r.finished;
    }
    let mut out: BTreeMap<String, Tally> = BTreeMap::new();
    for ((t, m), unfinished) in per.iter_mut().zip(items).zip(unfinished) {
        t.unfinished = usize::from(unfinished);
        t.zero = usize::from(!unfinished && t.corrections == 0);
        out.entry(m.corpus.clone()).or_default().add(t);
    }
    let mut total = Tally::default();
    for t in out.values() {
        total.add(t);
    }
    out.insert("合計".into(), total);
    out
}

fn parse_config(s: &str) -> Result<InlineEnglish, String> {
    qim_bench::parse_inline_english(s)
}

fn load_english() -> Result<EnglishWords, String> {
    let bytes = fs::read(qim_bench::ENGLISH_PACK).map_err(err)?;
    Ok(EnglishPack::from_bytes(&bytes).map_err(err)?.words)
}

fn mixed(split: &str, configs: &[&str]) -> Result<(), String> {
    let items = load_mixed(split)?;
    let english = load_english()?;
    let engine = load_engine(Path::new("."), &[], None)?.engine;
    let rule4 = simulate(&engine, &english, &items, Typist::Rule4);
    let shift = simulate(&engine, &english, &items, Typist::AllShift);
    println!("## 混輸句（{split}）\n");
    println!(
        "| 打法 | 語料 | 句數 | 英文詞 | 收進 lattice | Shift 直打 | 修正次數 | 比規則 4 | 比全按 Shift | 每句按鍵 | 零修正的句 | 打不出 |"
    );
    println!("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    let row = |name: &str, corpus: &str, t: &Tally| {
        let base = rule4[corpus].corrections as f64;
        let alt = shift[corpus].corrections as f64;
        let pct = |b: f64| {
            if b == 0.0 {
                "—".to_string()
            } else {
                format!("{:+.1}%", (t.corrections as f64 - b) / b * 100.0)
            }
        };
        println!(
            "| {name} | {corpus} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {} | {} |",
            t.sentences,
            t.words,
            t.covered,
            t.shift_words,
            t.corrections,
            pct(base),
            pct(alt),
            t.keys as f64 / t.sentences.max(1) as f64,
            t.zero,
            t.unfinished
        );
    };
    for (corpus, t) in &rule4 {
        row("規則 4（基準）", corpus, t);
    }
    for (corpus, t) in &shift {
        row("全按 Shift", corpus, t);
    }
    for config in configs {
        let inline = parse_config(config)?;
        let on = engine.clone().with_inline_english(inline);
        let result = simulate(&on, &english, &items, Typist::Inline(inline));
        for (corpus, t) in &result {
            row(&format!("混輸 {config}"), corpus, t);
        }
    }
    Ok(())
}

/// 混輸句裡的英文詞：依長度與能不能出（規則 4、頻率）分組的個數，看設定前用（不看模擬結果）。
fn words(split: &str) -> Result<(), String> {
    let items = load_mixed(split)?;
    let english = load_english()?;
    let mut by: BTreeMap<(usize, &str), usize> = BTreeMap::new();
    let mut freqs: BTreeMap<u8, usize> = BTreeMap::new();
    for m in &items {
        for t in &m.tokens {
            if let Token::English(w) = t {
                let lower = w.to_lowercase();
                let kind = if !letters_a_to_y(&lower) {
                    "有 z"
                } else if let Some(f) = english.frequency(&lower) {
                    *freqs.entry(f / 20 * 20).or_default() += 1;
                    "詞典有"
                } else {
                    "詞典沒有"
                };
                *by.entry((lower.len().min(10), kind)).or_default() += 1;
            }
        }
    }
    println!("句數 {}", items.len());
    for ((len, kind), n) in by {
        println!("長度 {len}{} {kind}：{n}", if len == 10 { "+" } else { "" });
    }
    for (f, n) in freqs {
        println!("頻率 {f}–{}：{n}", f.saturating_add(19));
    }
    Ok(())
}

// ---------------------------------------------------------------- 純中文集

/// 打首選字基準各集合的 dev 切分（`qim_bench::run_dev`），`config` 是 `off` 或混輸的設定；寫
/// `build/inline-english/chinese-<config>.tsv`：集合、題數、答錯、修正次數。
fn chinese(config: &str) -> Result<(), String> {
    if config != "off" {
        parse_config(config)?;
        // SAFETY 的前提：這時只有主執行緒，還沒有別的執行緒讀環境變數。
        unsafe { std::env::set_var(qim_bench::INLINE_ENGLISH_VAR, config) };
    }
    let outcome = qim_bench::run_dev(Path::new("."), &[], None)?;
    let mut out = String::new();
    for set in std::iter::once(&outcome.fixed).chain(&outcome.sets) {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            set.name,
            set.total(),
            set.wrong(),
            set.correction_count()
        ));
    }
    let path = Path::new(OUT_DIR).join(format!("chinese-{config}.tsv"));
    fs::create_dir_all(OUT_DIR).map_err(err)?;
    fs::File::create(&path)
        .and_then(|mut f| f.write_all(out.as_bytes()))
        .map_err(err)?;
    println!("寫出 {}", path.display());
    Ok(())
}

fn read_chinese(config: &str) -> Result<Vec<(String, usize, usize, usize)>, String> {
    let path = Path::new(OUT_DIR).join(format!("chinese-{config}.tsv"));
    let text = fs::read_to_string(&path).map_err(|e| format!("讀不到 {}：{e}", path.display()))?;
    text.lines()
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let n = |i: usize| {
                f.get(i)
                    .and_then(|s| s.parse().ok())
                    .ok_or(format!("壞的行：{l}"))
            };
            Ok((f[0].to_string(), n(1)?, n(2)?, n(3)?))
        })
        .collect()
}

/// 各設定的純中文集比關著（`chinese off`）多答錯、多修正幾題。
fn report(configs: &[&str]) -> Result<(), String> {
    let off = read_chinese("off")?;
    println!("## 純中文集（dev）：開了比關著多幾題（答錯 / 修正次數）\n");
    print!("| 集合 | 題數 | 關：答錯 | 關：修正 |");
    for c in configs {
        print!(" {c} |");
    }
    println!();
    print!("| --- | ---: | ---: | ---: |");
    for _ in configs {
        print!(" ---: |");
    }
    println!();
    let runs: Vec<Vec<(String, usize, usize, usize)>> = configs
        .iter()
        .map(|c| read_chinese(c))
        .collect::<Result<_, _>>()?;
    let mut totals = vec![(0i64, 0i64); configs.len()];
    for (i, (name, total, wrong, corr)) in off.iter().enumerate() {
        print!("| {name} | {total} | {wrong} | {corr} |");
        for (k, run) in runs.iter().enumerate() {
            let (rname, _, rw, rc) = &run[i];
            if rname != name {
                return Err(format!("集合對不上：{rname} 與 {name}"));
            }
            let (dw, dc) = (*rw as i64 - *wrong as i64, *rc as i64 - *corr as i64);
            totals[k].0 += dw;
            totals[k].1 += dc;
            print!(" {dw:+} / {dc:+} |");
        }
        println!();
    }
    print!("| 合計 | | | |");
    for (dw, dc) in totals {
        print!(" {dw:+} / {dc:+} |");
    }
    println!();
    Ok(())
}
