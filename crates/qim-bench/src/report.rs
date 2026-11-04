//! 印給人看、也直接貼進票做前後對照的結果：一個集合一列的 Markdown 表。
//! 之後的指標（修正次數、跳動、延遲，票 04、05）加成新的欄。

use crate::Outcome;
use crate::score::SetResult;

pub fn render(outcome: &Outcome) -> String {
    // 前 12 碼足以分辨；`cargo xtask data` 印的是完整 sha256，開頭對得上就是同一份。
    let mut out = format!(
        "資料包 {}（sha256 {}）\n\n",
        crate::PACK,
        &outcome.pack_sha256[..12]
    );
    out.push_str(
        "| 集合 | 題數 | 整句首選正確 | 字正確率 | 答錯 | baseline 答錯 |\n\
         | --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    out.push_str(&row(&outcome.fixed, "—".into()));
    for set in &outcome.sets {
        let limit = outcome
            .baseline_wrong(&set.name)
            .map_or("—".into(), |wrong| wrong.to_string());
        out.push_str(&row(set, limit));
    }
    let fixed = &outcome.fixed;
    out.push_str(&format!(
        "\n固定題 {}/{} 通過\n",
        fixed.correct, fixed.total
    ));
    for failure in &fixed.failures {
        out.push_str(&format!("- 沒過：{failure}\n"));
    }
    out
}

fn row(set: &SetResult, baseline_wrong: String) -> String {
    format!(
        "| {} | {} | {}（{}） | {} | {} | {} |\n",
        set.name,
        set.total,
        set.correct,
        percent(set.sentence_accuracy()),
        percent(set.char_accuracy()),
        set.wrong(),
        baseline_wrong
    )
}

fn percent(ratio: f64) -> String {
    format!("{:.2}%", ratio * 100.0)
}
