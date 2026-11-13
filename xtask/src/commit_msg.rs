//! `cargo xtask commit-msg <訊息檔>`：commit message 的 subject 照 Conventional Commits（plan〈開發環境與閘門〉）。
//!
//! `.githooks/commit-msg` 呼叫它；規則說明見 `docs/development.md`〈Commit message〉。

use std::path::Path;
use std::process::Command;

/// 允許的 type。
pub const TYPES: &[&str] = &[
    "feat", "fix", "docs", "refactor", "test", "chore", "build", "ci", "perf", "style", "revert",
];

/// subject 整行的字元上限。
const MAX_SUBJECT: usize = 72;

/// git 自動產生的訊息開頭，不檢查。
/// `Reapply "` 是 git 2.43 起 revert 一個 revert 時產生的。
const GIT_PREFIXES: &[&str] = &[
    "Merge ",
    "Revert \"",
    "Reapply \"",
    "fixup! ",
    "squash! ",
    "amend! ",
];

/// `git commit -v` 的剪刀線：之後是 diff，不算訊息。
const SCISSORS: &str = "# ------------------------ >8 ------------------------";

/// 訊息的 subject：剪刀線之前、跳過 `#` 註解行後第一個非空白行。
fn subject(message: &str) -> Option<&str> {
    message
        .lines()
        .take_while(|line| *line != SCISSORS)
        .filter(|line| !line.starts_with('#'))
        .map(str::trim_end)
        .find(|line| !line.trim().is_empty())
}

/// 訊息違反的規則，每條一句；空的就是通過。
pub fn problems(message: &str) -> Vec<String> {
    let Some(subject) = subject(message) else {
        return vec!["訊息是空的".to_owned()];
    };
    if GIT_PREFIXES
        .iter()
        .any(|prefix| subject.starts_with(prefix))
    {
        return Vec::new();
    }
    let mut found = Vec::new();
    let length = subject.chars().count();
    if length > MAX_SUBJECT {
        found.push(format!("subject {length} 字元，上限 {MAX_SUBJECT}"));
    }
    let Some((header, rest)) = subject.split_once(':') else {
        found.push("缺冒號：subject 要是 `type(scope)!: description`".to_owned());
        return found;
    };
    let kind = header.split(['(', '!']).next().unwrap_or_default();
    if !TYPES.contains(&kind) {
        found.push(format!("type `{kind}` 不在清單：{}", TYPES.join("、")));
    } else if !valid_scope(&header[kind.len()..]) {
        found.push(format!(
            "`{header}` 的 scope 寫法不對：要是 `type(scope)` 或 `type(scope)!`，scope 不可空白"
        ));
    }
    let description = rest.strip_prefix(' ').unwrap_or_default();
    match description.chars().next() {
        None => found.push("冒號後要接一個空格再寫 description".to_owned()),
        Some(first) if first.is_whitespace() => {
            found.push("冒號後只接一個空格".to_owned());
        }
        Some(first) if first.is_uppercase() => {
            found.push(format!("description 要小寫開頭（現在是 `{first}`）"));
        }
        Some(_) => {}
    }
    if description.ends_with(['.', '。']) {
        found.push("description 結尾不要句號".to_owned());
    }
    found
}

/// type 之後到冒號前的部分：空的、`!`、`(scope)` 或 `(scope)!`。
fn valid_scope(after_type: &str) -> bool {
    let rest = after_type.strip_suffix('!').unwrap_or(after_type);
    if rest.is_empty() {
        return true;
    }
    rest.strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
        .is_some_and(|scope| {
            !scope.is_empty() && !scope.contains(['(', ')']) && !scope.contains(char::is_whitespace)
        })
}

/// 在 `dir` 所屬的 repo 是否正在 `git merge`（`MERGE_HEAD` 存在）。
/// 合併的訊息一律放行：ticket-coordinator 用 `merge: ticket NN slug`，git 預設用 `Merge …`。
pub fn merge_in_progress(dir: &Path) -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--git-path", "MERGE_HEAD"])
        .current_dir(dir)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout);
            dir.join(path.trim_end()).is_file()
        }
        _ => false,
    }
}

/// 檢查訊息檔並把違規印到終端機；回傳是否通過。
pub fn report(message_file: &Path) -> bool {
    let message = match std::fs::read_to_string(message_file) {
        Ok(message) => message,
        Err(error) => {
            eprintln!("讀不到 {}：{error}", message_file.display());
            return false;
        }
    };
    // git 在 worktree 根目錄跑 hook。
    if merge_in_progress(Path::new(".")) {
        return true;
    }
    let found = problems(&message);
    if found.is_empty() {
        return true;
    }
    eprintln!("commit message 不符規則（docs/development.md〈Commit message〉）：");
    eprintln!("  {}", subject(&message).unwrap_or_default());
    for problem in found {
        eprintln!("- {problem}");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_red(message: &str, expected: &str) {
        let found = problems(message);
        assert!(
            found.iter().any(|problem| problem.contains(expected)),
            "{message:?} 應該因「{expected}」被擋，實際：{found:?}"
        );
    }

    fn assert_green(message: &str) {
        assert_eq!(problems(message), Vec::<String>::new(), "{message:?}");
    }

    #[test]
    fn unknown_type_is_red() {
        assert_red("feature: add stuff", "type `feature`");
        assert_red("Feat: add stuff", "type `Feat`");
    }

    #[test]
    fn missing_colon_is_red() {
        assert_red("Add stuff.", "缺冒號");
        assert_red("feat add stuff", "缺冒號");
    }

    #[test]
    fn uppercase_description_is_red() {
        assert_red("feat: Add stuff", "小寫開頭");
    }

    #[test]
    fn trailing_period_is_red() {
        assert_red("fix: add stuff.", "句號");
    }

    #[test]
    fn subject_of_73_chars_is_red() {
        assert_red(&format!("feat: {}", "a".repeat(67)), "73 字元");
    }

    #[test]
    fn malformed_header_is_red() {
        assert_red("feat(): add stuff", "scope");
        assert_red("feat(core: add stuff", "scope");
        assert_red("feat!(core): add stuff", "scope");
        assert_red("feat:add stuff", "空格");
        assert_red("feat:  add stuff", "只接一個空格");
        assert_red("feat: ", "空格");
    }

    #[test]
    fn empty_or_comment_only_message_is_red() {
        assert_red("", "空的");
        assert_red("\n# Please enter the commit message\n#\n", "空的");
        assert_red(
            "# ------------------------ >8 ------------------------\ndiff --git a/X b/X\n",
            "空的",
        );
    }

    #[test]
    fn comment_lines_do_not_hide_a_bad_subject() {
        assert_red("# feat: add stuff\nAdd stuff.\n", "缺冒號");
    }

    #[test]
    fn scope_and_breaking_marker_are_green() {
        assert_green("feat(core): add stuff");
        assert_green("feat!: drop old pack format");
        assert_green("feat(core)!: drop old pack format");
    }

    #[test]
    fn every_listed_type_is_green() {
        for kind in TYPES {
            assert_green(&format!("{kind}: add stuff"));
        }
    }

    #[test]
    fn body_is_not_checked() {
        assert_green("fix: add stuff\n\nBody Starts Uppercase and ends with a period.\n");
    }

    #[test]
    fn subject_of_exactly_72_chars_is_green() {
        assert_green(&format!("feat: {}", "a".repeat(66)));
        assert_green(&format!("feat: {}", "字".repeat(66)));
    }

    #[test]
    fn git_generated_messages_are_green() {
        assert_green("Merge branch '1morr/t03'");
        assert_green(&format!(
            "Merge remote-tracking branch 'origin/{}'",
            "x".repeat(80)
        ));
        assert_green("Revert \"feat: add stuff\"\n\nThis reverts commit 991814d.\n");
        assert_green("Reapply \"feat: add stuff\"");
        assert_green("fixup! feat: add stuff");
        assert_green(&format!("squash! feat: {}", "a".repeat(66)));
        assert_green("amend! feat: add stuff");
    }

    #[test]
    fn comment_lines_are_ignored() {
        assert_green(
            "# Leading Comment.\n\nchore: add stuff\n# Please enter the commit message.\n#\n",
        );
    }

    #[test]
    fn unrelated_formatting_is_green() {
        assert_green("\n\nchore: add stuff  \r\n\r\nbody\r\n");
        assert_green("docs: 72-char subjects and `AGENTS.md` map");
        assert_green("feat(qim-core): add stuff: with a colon");
        assert_green(
            "chore: add stuff\n# ------------------------ >8 ------------------------\n\
             diff --git a/X b/X\nAdd Stuff.\n",
        );
    }

    const DEVELOPMENT_MD: &str = include_str!("../../docs/development.md");

    /// `docs/development.md`〈Commit message〉裡「type 只能是…：」那一行冒號後、以「、」分隔的 type。
    fn documented_types(markdown: &str) -> Vec<String> {
        markdown
            .split("\n## ")
            .find(|section| section.starts_with("Commit message"))
            .into_iter()
            .flat_map(str::lines)
            .filter_map(|line| line.strip_prefix("- type 只能是"))
            .filter_map(|line| line.split_once('：'))
            .flat_map(|(_, list)| list.trim().trim_end_matches('。').split('、'))
            .map(|kind| kind.trim().to_owned())
            .collect()
    }

    #[test]
    fn development_md_lists_the_coded_types() {
        assert_eq!(documented_types(DEVELOPMENT_MD), TYPES);
    }

    #[test]
    fn missing_or_extra_type_in_development_md_is_caught() {
        let missing = DEVELOPMENT_MD.replace("、revert。", "。");
        assert_ne!(documented_types(&missing), TYPES);
        let extra = DEVELOPMENT_MD.replace("、revert。", "、revert、merge。");
        assert_ne!(documented_types(&extra), TYPES);
    }

    #[test]
    fn unrelated_edits_to_development_md_types_are_green() {
        let edited = DEVELOPMENT_MD
            .replace("feat、fix、docs", "feat、 fix 、docs")
            .replace(
                "description 不以大寫字母開頭",
                "description 開頭不用大寫字母",
            );
        assert_ne!(edited, DEVELOPMENT_MD);
        assert_eq!(documented_types(&edited), TYPES);
    }

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git 可執行");
        assert!(status.success(), "git {args:?}");
    }

    #[test]
    fn merge_in_progress_follows_merge_head() {
        let repo = tempfile::tempdir().unwrap();
        git(repo.path(), &["init", "--quiet"]);
        let sub = repo.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        assert!(!merge_in_progress(repo.path()));

        std::fs::write(repo.path().join(".git/MERGE_HEAD"), "0\n").unwrap();
        assert!(merge_in_progress(repo.path()));
        assert!(merge_in_progress(&sub));
    }

    #[test]
    fn outside_a_repo_is_not_merging() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!merge_in_progress(dir.path()));
    }
}
