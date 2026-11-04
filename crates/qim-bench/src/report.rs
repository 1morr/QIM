//! 印給人看、也直接貼進票做前後對照的結果：指標表、依長度分組表、翻盤清單。
//! 個人集只印數字與題號（plan §5.2）。

use crate::baseline::SetBaseline;
use crate::score::{LENGTH_GROUPS, Metrics, SetResult};
use crate::{Latency, Outcome, PERSONAL, PERSONAL_USERDB, Personal, WASM_BINDINGS};

/// 翻盤清單每個方向最多列幾題，其餘只報題數。
const FLIPS_SHOWN: usize = 20;

pub fn render(outcome: &Outcome) -> String {
    // 前 12 碼足以分辨；`cargo xtask data` 印的是完整 sha256，開頭對得上就是同一份。
    let mut out = format!(
        "資料包 {}（sha256 {}）\n\n",
        crate::PACK,
        &outcome.pack_sha256[..12]
    );
    out.push_str(
        "| 集合 | 題數 | 整句首選正確 | 字正確率 | 答錯 | baseline 答錯 | 修正/句 | 修正 | baseline 修正 | KSPC | 跳動/句 | 打不出 |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
    );
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

    out.push_str("\n依長度分組（整句首選正確率 · 修正/句（題數））\n\n| 集合 |");
    for (_, label) in LENGTH_GROUPS {
        out.push_str(&format!(" {label} |"));
    }
    out.push_str("\n| --- |");
    out.push_str(&" ---: |".repeat(LENGTH_GROUPS.len()));
    out.push('\n');
    for set in outcome.scored_sets() {
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

/// 每鍵延遲：原生與 WebAssembly 各兩組（全部、20 碼以上），微秒：現在的核心每鍵不到 1 µs，用毫秒看不出前後差別。
fn latency(latency: &Latency) -> String {
    let mut out = format!(
        "\n每鍵延遲（µs；只報不擋：`--check` 不比、baseline 不記）\n\n\
         機器 {}；固定題與各語料集合的每一題從空的組字逐鍵打、不選字，不含載入資料包。\
         WebAssembly 用 {WASM_BINDINGS}（`cargo xtask bench` 以 release 編）。\n\n\
         | 環境 | 輸入 | 輸入數 | 鍵數 | p50 | p95 |\n\
         | --- | --- | ---: | ---: | ---: | ---: |\n",
        latency.machine
    );
    for (environment, timing) in [("原生", &latency.native), ("WebAssembly", &latency.wasm)] {
        for (group, summary) in latency.groups(timing) {
            let label = format!("{environment}（{}）", timing.runtime);
            out.push_str(&match summary {
                Some(s) => format!(
                    "| {label} | {group} | {} | {} | {:.2} | {:.2} |\n",
                    s.inputs,
                    s.keys,
                    s.p50 * 1000.0,
                    s.p95 * 1000.0
                ),
                None => format!("| {label} | {group} | 0 | 0 | — | — |\n"),
            });
        }
    }
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
