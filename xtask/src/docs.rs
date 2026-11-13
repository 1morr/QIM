//! `docs` 步驟：跑 `scripts/check_docs.py`（文件規則寫在腳本開頭）與它的測試。
//! 另留 Markdown 表格分欄的 [`cells`]，`check` 與 `deps` 的測試讀文件裡的表時用。

use std::path::Path;
use std::process::Command;

/// 文件檢查與它自己的測試都通過才回傳 true；違規由腳本印到終端機。
pub fn report(root: &Path) -> bool {
    python(root, &["scripts/check_docs.py"])
        && python(root, &["-m", "unittest", "discover", "-s", "scripts/tests"])
}

fn python(root: &Path, args: &[&str]) -> bool {
    match Command::new("python3")
        .args(args)
        .current_dir(root)
        .status()
    {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("跑不了 python3（文件檢查要 Python 3，見 docs/development.md）：{error}");
            false
        }
    }
}

/// 表格列的各欄，去掉前後空白；`\|` 是欄內的直線，不分欄。
#[cfg(test)]
pub(crate) fn cells(text: &str) -> Vec<&str> {
    let text = text.trim();
    let text = text.strip_prefix('|').unwrap_or(text);
    let text = text.strip_suffix('|').unwrap_or(text);
    let mut cells = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (index, c) in text.char_indices() {
        match c {
            '|' if !escaped => {
                cells.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
        escaped = c == '\\' && !escaped;
    }
    cells.push(text[start..].trim());
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_splits_on_unescaped_bars_and_trims() {
        assert_eq!(cells("| a | b c |  d |"), ["a", "b c", "d"]);
        assert_eq!(cells("a | b"), ["a", "b"]);
    }

    #[test]
    fn an_escaped_bar_stays_inside_its_cell() {
        assert_eq!(cells(r"| a \| b | c |"), [r"a \| b", "c"]);
    }

    #[test]
    fn the_repo_passes_the_docs_step() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        assert!(report(root));
    }
}
