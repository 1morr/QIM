//! 印給人看、也直接貼進票做前後對照的結果：指標表、依長度分組表、翻盤清單。
//! 個人集只印數字與題號（plan §5.2）。

use crate::baseline::SetBaseline;
use crate::latency::p95_limits_ms;
use crate::score::{LENGTH_GROUPS, Metrics, SetResult};
use crate::{
    CONTEXT_SUFFIXES, DevOutcome, Latency, Learning, LoadedPack, Outcome, PERSONAL,
    PERSONAL_USERDB, Personal, SHORT_INPUTS, Timing, WASM_BINDINGS,
};

/// 「有前文」的每批題給前文與不給前文並列（plan §5.2）；沒有這種集合時是空字串。
fn context_pairs(sets: &[SetResult]) -> String {
    let [given, not_given] = CONTEXT_SUFFIXES;
    let pairs: Vec<(&str, &SetResult, &SetResult)> = sets
        .iter()
        .filter_map(|with| {
            let name = with.name.strip_suffix(given)?;
            let without = sets
                .iter()
                .find(|s| s.name.strip_suffix(not_given) == Some(name))?;
            Some((name, with, without))
        })
        .collect();
    if pairs.is_empty() {
        return String::new();
    }
    let row = |name: &str, counts: [usize; 5]| {
        let [
            total,
            wrong,
            wrong_without,
            corrections,
            corrections_without,
        ] = counts;
        format!(
            "| {name} | {total} | {wrong} | {wrong_without} | {corrections} | {corrections_without} |\n"
        )
    };
    let counts = |with: &SetResult, without: &SetResult| {
        [
            with.total(),
            with.wrong(),
            without.wrong(),
            with.correction_count(),
            without.correction_count(),
        ]
    };
    let mut rows: Vec<String> = pairs
        .iter()
        .map(|(name, with, without)| row(name, counts(with, without)))
        .collect();
    let mut sum = [0; 5];
    for (_, with, without) in &pairs {
        for (s, n) in sum.iter_mut().zip(counts(with, without)) {
            *s += n;
        }
    }
    rows.push(row("合計", sum));
    format!(
        "\n有前文的題：同一批題給前文與不給前文\n\n\
         | 題目 | 題數 | 答錯（給） | 答錯（不給） | 修正（給） | 修正（不給） |\n\
         | --- | ---: | ---: | ---: | ---: | ---: |\n\
         {}",
        rows.concat()
    )
}

/// 打錯碼的集合（M4 票 08）：沒打錯與打錯的題各答對幾題，與「原樣正確卻被改錯」；沒有這種集合時是空字串。
fn typo_table<'a>(sets: impl Iterator<Item = &'a SetResult>) -> String {
    let rows: String = sets
        .filter_map(|set| {
            let t = set.typo_stats()?;
            Some(format!(
                "| {} | {} | {} | {} | {} | {} | {}/{} | {:.2}% |\n",
                set.name,
                set.total(),
                t.clean,
                t.clean_correct,
                t.mistyped,
                t.mistyped_correct,
                t.false_corrections,
                t.literal_correct,
                t.false_correction_rate() * 100.0,
            ))
        })
        .collect();
    if rows.is_empty() {
        return String::new();
    }
    format!(
        "\n打錯碼的集合：沒打錯是每一鍵機率最高的字母都等於原本的碼；原樣正確卻被改錯是沒打錯、原樣讀法（容錯關）\
         首選正確的題裡，實際首選不正確的（速成不做容錯，M4 票 09，一直是 0）\n\n\
         | 集合 | 題數 | 沒打錯 | 沒打錯答對 | 打錯 | 打錯答對 | 原樣正確卻被改錯 | 改錯比例 |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n\
         {rows}"
    )
}

/// 翻盤清單每個方向最多列幾題，其餘只報題數。
const FLIPS_SHOWN: usize = 20;

/// 前 12 碼足以分辨；`cargo xtask data` 印的是完整 sha256，開頭對得上就是同一份。
fn packs_line(packs: &[LoadedPack]) -> String {
    let packs: Vec<String> = packs
        .iter()
        .map(|p| format!("{}（sha256 {}）", p.path, &p.sha256[..12]))
        .collect();
    format!("資料包 {}\n\n", packs.join("、"))
}

/// 學習模式的表：各集合的修正次數，學習關與學習開。
pub fn render_learning(learning: &Learning) -> String {
    let mut out = packs_line(&learning.packs);
    out.push_str(
        "| 集合 | 題數 | 修正（學習關） | 修正（學習開） | 修正/句（關） | 修正/句（開） |\n\
         | --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for set in &learning.sets {
        let per = |n: usize| {
            if set.total == 0 {
                0.0
            } else {
                n as f64 / set.total as f64
            }
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.2} | {:.2} |\n",
            set.name,
            set.total,
            set.corrections_off,
            set.corrections_on,
            per(set.corrections_off),
            per(set.corrections_on),
        ));
    }
    out.push_str("\n每個集合照檔案的順序打；學習開時整個集合共用一個 `UserModel`。\n");
    out
}

pub fn render(outcome: &Outcome) -> String {
    let mut out = packs_line(&outcome.packs);
    out.push_str(METRICS_HEADER);
    out.push_str(&row(&outcome.fixed, None));
    for set in outcome.scored_sets() {
        out.push_str(&row(set, outcome.baseline_of(set)));
        if set.private {
            out.push_str(&weighted_row(set));
        }
    }
    match outcome.personal {
        Personal::Ran(_) => out.push_str(&format!(
            "\n個人集（{PERSONAL}）只在本機；加權列依上屏次數加權。\n"
        )),
        Personal::Missing => out.push_str(&format!("\n個人集：沒有 {PERSONAL_USERDB}，跳過\n")),
        Personal::NotTranscoded => out.push_str(&format!(
            "\n個人集：{PERSONAL} 不是由現在的 {PERSONAL_USERDB} 轉出（還沒轉，或資料換過），跳過；先跑 `cargo xtask data`\n"
        )),
    }
    out.push_str(&length_groups(outcome.scored_sets()));
    out.push_str(&context_pairs(&outcome.sets));
    out.push_str(&typo_table(outcome.sets.iter()));

    out.push_str("\n翻盤清單（和 baseline 比）\n");
    for set in outcome.scored_sets() {
        let Some(flips) = outcome.flips(&set.name) else {
            out.push_str(&format!("- {}：沒有 baseline\n", set.name));
            continue;
        };
        if flips.is_empty() {
            out.push_str(&format!("- {}：沒有翻盤\n", set.name));
            continue;
        }
        for (direction, lines) in [("由錯變對", &flips.fixed), ("由對變錯", &flips.broken)]
        {
            if lines.is_empty() {
                continue;
            }
            out.push_str(&format!("- {} {direction} {} 題", set.name, lines.len()));
            // 個人集只列題號；其他集合附期望與首選，看得出是哪一句。
            if set.private {
                let shown: Vec<String> = lines
                    .iter()
                    .take(FLIPS_SHOWN)
                    .map(|l| l.to_string())
                    .collect();
                out.push_str(&format!("：第 {} 行", shown.join("、")));
                out.push_str(&more(lines.len()));
                out.push('\n');
            } else {
                out.push_str(&more(lines.len()));
                out.push('\n');
                for &line in lines.iter().take(FLIPS_SHOWN) {
                    out.push_str(&format!("  - {}\n", describe(set, line)));
                }
            }
        }
    }

    let fixed = &outcome.fixed;
    out.push_str(&format!(
        "\n固定題 {}/{} 通過\n",
        fixed.correct(),
        fixed.total()
    ));
    for failure in &fixed.failures {
        out.push_str(&format!("- 沒過：{failure}\n"));
    }
    out.push_str(&latency(&outcome.latency));
    out
}

/// dev 模式：打分參數、指標表（沒有 baseline 欄的值）、依長度分組；不比 baseline，沒有翻盤與延遲。
pub fn render_dev(outcome: &DevOutcome) -> String {
    let r = &outcome.ranking;
    let mut out = packs_line(&outcome.packs);
    out.push_str(&format!(
        "dev 切分；打分參數 word_bonus = {}、segment_penalty = {}、beam = {}、sentences = {}\n\n",
        r.word_bonus, r.segment_penalty, r.beam, r.sentences
    ));
    out.push_str(METRICS_HEADER);
    out.push_str(&row(&outcome.fixed, None));
    for set in &outcome.sets {
        out.push_str(&row(set, None));
        // 短輸入依出現次數加權，才是照語料打時的首選正確率。
        if is_short(set) {
            out.push_str(&weighted_row(set));
        }
    }
    // 短輸入都在 1–4 字的組，分組沒有什麼可看。
    out.push_str(&length_groups(outcome.sets.iter().filter(|s| !is_short(s))));
    out.push_str(&context_pairs(&outcome.sets));
    out.push_str(&typo_table(outcome.sets.iter()));
    let fixed = &outcome.fixed;
    out.push_str(&format!(
        "\n固定題 {}/{} 通過\n",
        fixed.correct(),
        fixed.total()
    ));
    for failure in &fixed.failures {
        out.push_str(&format!("- 沒過：{failure}\n"));
    }
    out
}

/// dev 模式由句子取出的短輸入集合（[`SHORT_INPUTS`]）。
fn is_short(set: &SetResult) -> bool {
    SHORT_INPUTS.iter().any(|suffix| set.name.ends_with(suffix))
}

const METRICS_HEADER: &str = "| 集合 | 題數 | 整句首選正確 | 字正確率 | 答錯 | baseline 答錯 | 修正/句 | 修正 | baseline 修正 | KSPC | 跳動/句 | 打不出 |\n\
     | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n";

/// 依長度分組表：每格「整句首選正確率 · 修正/句（題數）」。
fn length_groups<'a>(sets: impl Iterator<Item = &'a SetResult>) -> String {
    let mut out = String::from("\n依長度分組（整句首選正確率 · 修正/句（題數））\n\n| 集合 |");
    for (_, label) in LENGTH_GROUPS {
        out.push_str(&format!(" {label} |"));
    }
    out.push_str("\n| --- |");
    out.push_str(&" ---: |".repeat(LENGTH_GROUPS.len()));
    out.push('\n');
    for set in sets {
        out.push_str(&format!("| {} |", set.name));
        for group in set.length_groups() {
            match group {
                Some(g) => out.push_str(&format!(
                    " {} · {:.2}（{}） |",
                    percent(g.sentence_accuracy),
                    g.correction_count,
                    g.total
                )),
                None => out.push_str(" — |"),
            }
        }
        out.push('\n');
    }
    out
}

/// 每鍵延遲：兩批輸入（基準的、含萬用鍵的）各在空白與學過的學習資料下，原生與 WebAssembly 各三組（全部、第 20 碼起、選開頭一段），
/// 微秒：用毫秒看不出前後差別。原生的列附 p95 上限（[`p95_limits_ms`]），超過的註明。
fn latency(latency: &Latency) -> String {
    let mut out = format!(
        "\n每鍵延遲（µs；`--check` 擋原生的 p95，上限見 `.scratch/m4/spec.md`〈每鍵延遲上限〉；\
         WebAssembly 只報；baseline 不記）\n\n\
         機器 {}；固定題與各語料集合的每一題從空的組字逐鍵打，不含載入資料包；\
         打完有兩段以上的題再按 ← 到第一段（不計時）、選它的首選，「選開頭一段」計那一次。\
         學習資料「空白」是空白的 UserModel，「學過」是一個 UserModel 依序學完各語料集合與「有前文」的題\
         （給前文）之後（WebAssembly 先 loadUserDict）。\
         「含萬用鍵的輸入」是同一批輸入、每個輸入的第一碼換成萬用鍵（`qim_bench::wildcard_inputs`），上限相同。\
         WebAssembly 用 {WASM_BINDINGS}（`cargo xtask bench` 以 release 編）。\n\n\
         | 批次 | 學習資料 | 環境 | 輸入 | 輸入數 | 鍵數 | p50 | p95 | p95 上限 |\n\
         | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n",
        latency.machine
    );
    for (inputs, user, runs) in latency.runs() {
        for (environment, timing, limits) in [
            ("原生", &runs.native, p95_limits_ms(user).map(Some)),
            ("WebAssembly", &runs.wasm, [None; 3]),
        ] {
            for ((group, summary), limit) in timing.groups().into_iter().zip(limits) {
                let label = format!(
                    "{} | {} | {environment}（{}）",
                    inputs.label(),
                    user.label(),
                    timing.runtime
                );
                let limit_cell = match limit {
                    None => "只報".to_string(),
                    Some(limit) => {
                        let over = summary.is_some_and(|s| s.over(limit));
                        format!(
                            "≤ {:.0}{}",
                            limit * 1000.0,
                            if over { "，超過" } else { "" }
                        )
                    }
                };
                out.push_str(&match summary {
                    Some(s) => format!(
                        "| {label} | {group} | {} | {} | {:.2} | {:.2} | {limit_cell} |\n",
                        s.inputs,
                        s.keys,
                        s.p50 * 1000.0,
                        s.p95 * 1000.0
                    ),
                    None => format!("| {label} | {group} | 0 | 0 | — | — | {limit_cell} |\n"),
                });
            }
        }
    }
    let load = |timing: &Timing| {
        timing
            .load_ms
            .map_or_else(|| "沒量".to_string(), |ms| format!("{ms:.1} ms"))
    };
    out.push_str(&format!(
        "\n載入：原生 {}（解開各資料包加建 Engine，不含讀檔）、WebAssembly {}（`new Session`：核對 manifest \
         與 sha256、解開、建 Engine）\n",
        load(&latency.blank.native),
        load(&latency.blank.wasm)
    ));
    out
}

fn more(total: usize) -> String {
    if total > FLIPS_SHOWN {
        format!("（只列前 {FLIPS_SHOWN} 題）")
    } else {
        String::new()
    }
}

/// 公開集合的一題：答錯時附期望與首選，答對時只有行號（答對的題不留內容）。
fn describe(set: &SetResult, line: usize) -> String {
    match set.failures.iter().find(|f| f.item.line == line) {
        Some(failure) => failure.to_string(),
        None => format!("第 {line} 行（現在答對）"),
    }
}

fn row(set: &SetResult, baseline: Option<&SetBaseline>) -> String {
    let m = set.metrics(false);
    format!(
        "| {} | {} | {}（{}） | {} | {} | {} | {:.2} | {} | {} | {:.2} | {:.2} | {} |\n",
        set.name,
        set.total(),
        set.correct(),
        percent(m.sentence_accuracy),
        percent(m.char_accuracy),
        set.wrong(),
        dash(baseline.map(|b| b.wrong)),
        m.correction_count,
        set.correction_count(),
        dash(baseline.map(|b| b.correction_count)),
        m.kspc,
        m.jitter,
        set.unfinished(),
    )
}

/// 依權重的那一列：題數是權重總和，只有比率。
fn weighted_row(set: &SetResult) -> String {
    let Metrics {
        sentence_accuracy,
        char_accuracy,
        correction_count,
        kspc,
        jitter,
    } = set.metrics(true);
    let weight: u64 = set.items.iter().map(|i| i.weight).sum();
    format!(
        "| {}（加權） | {weight} 次 | {} | {} | — | — | {correction_count:.2} | — | — | {kspc:.2} | {jitter:.2} | — |\n",
        set.name,
        percent(sentence_accuracy),
        percent(char_accuracy),
    )
}

fn dash(value: Option<usize>) -> String {
    value.map_or_else(|| "—".into(), |v| v.to_string())
}

fn percent(ratio: f64) -> String {
    format!("{:.2}%", ratio * 100.0)
}
