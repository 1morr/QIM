//! `deps` 步驟：依賴規則（`docs/plan.md` §1.3）。
//!
//! [`RULES`] 是 plan §1.3 表的程式版，改表時兩邊同一 commit 改；`cargo test` 比對兩邊。

use std::path::{Path, PathBuf};

use cargo_metadata::{DependencyKind, MetadataCommand};
use proc_macro2::{Delimiter, Group, Spacing, TokenStream, TokenTree};

/// plan §1.3 表的一列。
struct Rule {
    krate: &'static str,
    /// 可以依賴的 workspace crate（任何種類的依賴都適用）。
    workspace: &'static [&'static str],
    /// 一般與 build 依賴可以用的外部 crate；`None` 是不限。dev 依賴不受這條限制。
    external: Option<&'static [&'static str]>,
    /// 原始碼（`src/`）不可以用 [`FORBIDDEN_PATHS`]。
    no_io: bool,
}

/// plan §1.3 禁止 [`Rule::no_io`] 為真的 crate 用的 std 路徑。
const FORBIDDEN_PATHS: &[&[&str]] = &[
    &["std", "fs"],
    &["std", "net"],
    &["std", "thread"],
    &["std", "time", "SystemTime"],
];

/// 只會來自禁用路徑的名字：不論前面怎麼寫（`use std::time; time::SystemTime`），出現就算。
const FORBIDDEN_NAMES: &[(&str, &str)] = &[("SystemTime", "std::time::SystemTime")];

/// [`RULES`] 所在的檔案，違規說明裡指給人看。
const RULES_FILE: &str = "xtask/src/deps.rs";

const RULES: &[Rule] = &[
    Rule {
        krate: "qim-core",
        workspace: &[],
        external: Some(&["serde", "postcard", "serde_json", "sha2", "semver"]),
        no_io: true,
    },
    Rule {
        krate: "qim-layout",
        workspace: &[],
        external: Some(&["serde", "serde_json", "jsonschema"]),
        no_io: true,
    },
    Rule {
        krate: "qim-neural",
        workspace: &[],
        external: Some(&["candle-core", "candle-nn"]),
        no_io: true,
    },
    Rule {
        krate: "qim-wasm",
        workspace: &["qim-core", "qim-layout", "qim-neural"],
        external: Some(&["wasm-bindgen"]),
        no_io: false,
    },
    Rule {
        krate: "qim-ffi",
        workspace: &["qim-core", "qim-layout", "qim-neural"],
        external: Some(&["uniffi"]),
        no_io: false,
    },
    Rule {
        krate: "qim-databuild",
        workspace: &["qim-core", "qim-layout", "qim-neural"],
        external: None,
        no_io: false,
    },
    Rule {
        krate: "qim-bench",
        workspace: &["qim-core"],
        external: None,
        no_io: false,
    },
];

/// 開發工具，不受 plan §1.3 管。
const EXEMPT: &[&str] = &["xtask"];

/// 跑 [`run`] 並把違規印到終端機；回傳是否通過。
pub fn report(root: &Path) -> bool {
    match run(root) {
        Ok(violations) => {
            for violation in &violations {
                eprintln!("{violation}");
            }
            violations.is_empty()
        }
        Err(error) => {
            eprintln!("{error}");
            false
        }
    }
}

/// 檢查 `root` 的 workspace，回傳每一條違規的說明；空的就是通過。
pub fn run(root: &Path) -> Result<Vec<String>, String> {
    let metadata = MetadataCommand::new()
        .current_dir(root)
        .no_deps()
        .exec()
        .map_err(|error| format!("cargo metadata 失敗：{error}"))?;
    let members = metadata.workspace_packages();
    // 表上的 crate 還沒建立也算 workspace crate：依賴它的時候它一定已經建立了。
    let is_workspace_crate = |dep: &str| {
        RULES.iter().any(|rule| rule.krate == dep)
            || members.iter().any(|member| member.name.as_str() == dep)
    };
    let mut violations = Vec::new();
    for package in &members {
        let name = package.name.as_str();
        if EXEMPT.contains(&name) {
            continue;
        }
        let Some(rule) = RULES.iter().find(|rule| rule.krate == name) else {
            violations.push(format!(
                "workspace 成員 {name} 不在 plan §1.3 的表上：先把它加進 plan §1.3 與 {RULES_FILE} 的 RULES"
            ));
            continue;
        };
        for dependency in &package.dependencies {
            let dep = dependency.name.as_str();
            let kind = kind_name(dependency.kind);
            if is_workspace_crate(dep) {
                if !rule.workspace.contains(&dep) {
                    violations.push(format!(
                        "{name} 不可以依賴 {dep}（{kind}）：plan §1.3 只允許 {name} 依賴這些 workspace crate：{}",
                        join_names(rule.workspace)
                    ));
                }
            } else if let Some(allowed) = rule.external
                && dependency.kind != DependencyKind::Development
                && !allowed.contains(&dep)
            {
                violations.push(format!(
                    "{name} 的{kind} {dep} 不在 plan §1.3 的白名單（{}）：要加先改 plan §1.3 與 {RULES_FILE} 的 RULES",
                    join_names(allowed)
                ));
            }
        }
        if rule.no_io {
            let src = package
                .manifest_path
                .parent()
                .expect("manifest 在 crate 目錄裡")
                .join("src");
            scan_sources(
                name,
                src.as_std_path(),
                metadata.workspace_root.as_std_path(),
                &mut violations,
            )?;
        }
    }
    Ok(violations)
}

/// 掃 `src` 下每個 `.rs` 檔，用到 [`FORBIDDEN_PATHS`] 就記一條違規。
fn scan_sources(
    name: &str,
    src: &Path,
    workspace_root: &Path,
    violations: &mut Vec<String>,
) -> Result<(), String> {
    let mut files = Vec::new();
    collect_rust_files(src, &mut files)?;
    files.sort();
    let forbidden_list = FORBIDDEN_PATHS
        .iter()
        .map(|path| path.join("::"))
        .collect::<Vec<_>>()
        .join("、");
    for file in files {
        let label = file
            .strip_prefix(workspace_root)
            .unwrap_or(&file)
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let source =
            std::fs::read_to_string(&file).map_err(|error| format!("讀不到 {label}：{error}"))?;
        let uses = forbidden_uses(&source).map_err(|error| format!("{label} 無法斷詞：{error}"))?;
        for (line, path) in uses {
            violations.push(format!(
                "{label}:{line} 用了 {path}：plan §1.3 規定 {name} 不可以用 {forbidden_list}"
            ));
        }
    }
    Ok(())
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|error| format!("讀不到 {}：{error}", dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("讀不到 {}：{error}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// 原始碼裡用到禁用路徑的地方，以 (行號, 被禁的路徑) 表示。
///
/// 先斷詞，所以註解與字串裡的字不算；每個 `std` 之後沿 `::` 讀出路徑，
/// `use` 的 `{..}` 展開成各自的路徑，`*` 涵蓋它底下的一切。
/// 用斷詞而不用 `syn` 解析：巨集呼叫的內容 `syn` 不會當成路徑，斷詞則一樣掃得到。
/// 已知漏洞：把 `std` 本身改名（`use std as s`、`extern crate std as s`）後的路徑抓不到。
fn forbidden_uses(source: &str) -> Result<Vec<(usize, String)>, String> {
    let tokens: TokenStream = source.parse().map_err(|error| format!("{error}"))?;
    let mut found = Vec::new();
    find_std_paths(tokens, &mut found);
    // `std::time::SystemTime` 會被路徑與名字各抓一次。
    found.sort();
    found.dedup();
    Ok(found)
}

fn find_std_paths(stream: TokenStream, found: &mut Vec<(usize, String)>) {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        match token {
            TokenTree::Group(group) => find_std_paths(group.stream(), found),
            TokenTree::Ident(ident) if ident == "std" => {
                let line = ident.span().start().line;
                for path in expand(&tokens[index + 1..], vec!["std".to_owned()]) {
                    for forbidden in FORBIDDEN_PATHS.iter().filter(|f| covers(&path, f)) {
                        found.push((line, forbidden.join("::")));
                    }
                }
            }
            TokenTree::Ident(ident) => {
                if let Some((_, path)) = FORBIDDEN_NAMES.iter().find(|(name, _)| ident == name) {
                    found.push((ident.span().start().line, (*path).to_owned()));
                }
            }
            _ => {}
        }
    }
}

/// `prefix` 後面接著 `rest`：沿 `::` 讀下去，回傳讀到的所有完整路徑。
fn expand(rest: &[TokenTree], mut prefix: Vec<String>) -> Vec<Vec<String>> {
    let mut rest = rest;
    while let [TokenTree::Punct(first), TokenTree::Punct(second), next, ..] = rest {
        if !(first.as_char() == ':' && first.spacing() == Spacing::Joint && second.as_char() == ':')
        {
            break;
        }
        match next {
            TokenTree::Ident(ident) => prefix.push(ident.to_string()),
            TokenTree::Punct(punct) if punct.as_char() == '*' => {
                prefix.push("*".to_owned());
                break;
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                return expand_group(group, &prefix);
            }
            _ => break,
        }
        rest = &rest[3..];
    }
    vec![prefix]
}

/// `use` 的 `{a, b::c, self, *}`：逗號分開的每一項各自展開。
fn expand_group(group: &Group, prefix: &[String]) -> Vec<Vec<String>> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    tokens
        .split(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ','))
        .flat_map(|item| match item {
            [TokenTree::Ident(ident), rest @ ..] => {
                let mut path = prefix.to_vec();
                if ident != "self" {
                    path.push(ident.to_string());
                }
                expand(rest, path)
            }
            [TokenTree::Punct(punct), ..] if punct.as_char() == '*' => {
                let mut path = prefix.to_vec();
                path.push("*".to_owned());
                vec![path]
            }
            [TokenTree::Group(inner), ..] if inner.delimiter() == Delimiter::Brace => {
                expand_group(inner, prefix)
            }
            _ => Vec::new(),
        })
        .collect()
}

/// `path` 是否用到 `forbidden`：從它開始，或是涵蓋它的 `*`。
fn covers(path: &[String], forbidden: &[&str]) -> bool {
    let shared = path
        .iter()
        .zip(forbidden)
        .take_while(|(a, b)| a == b)
        .count();
    let starts_with_forbidden = shared == forbidden.len();
    let glob_above_forbidden =
        path.last().is_some_and(|last| last == "*") && shared == path.len() - 1;
    starts_with_forbidden || glob_above_forbidden
}

fn kind_name(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Development => "dev 依賴",
        DependencyKind::Build => "build 依賴",
        _ => "一般依賴",
    }
}

fn join_names(names: &[&str]) -> String {
    if names.is_empty() {
        "（無）".to_owned()
    } else {
        names.join("、")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    /// repo 的 Rust 部分複製到暫存目錄，變異只動副本。
    fn copy_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = repo_root();
        for file in ["Cargo.toml", "rust-toolchain.toml"] {
            fs::copy(root.join(file), dir.path().join(file)).unwrap();
        }
        for sub in ["crates", "xtask"] {
            copy_dir(&root.join(sub), &dir.path().join(sub));
        }
        dir
    }

    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                if entry.file_name() != "target" {
                    copy_dir(&entry.path(), &target);
                }
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    /// 在副本的檔尾追加 `text`，回傳追加的第一行的行號，讓斷言不綁原檔的長度。
    /// 追加到 `Cargo.toml` 的依賴寫成 `[dependencies.<名字>]` 子表，不受檔尾是哪一段影響。
    fn append(dir: &Path, file: &str, text: &str) -> usize {
        let path = dir.join(file);
        let mut content = fs::read_to_string(&path).unwrap();
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        let first_line = content.lines().count() + 1;
        content.push_str(text);
        fs::write(path, content).unwrap();
        first_line
    }

    fn core_line(line: usize) -> String {
        format!("{CORE_LIB}:{line}")
    }

    const CORE_TOML: &str = "crates/qim-core/Cargo.toml";

    /// 違規要被抓到，且同一條說明提到 `expected` 的每個字串。
    fn assert_red(dir: &Path, expected: &[&str]) {
        let violations = run(dir).unwrap();
        assert!(
            violations
                .iter()
                .any(|v| expected.iter().all(|word| v.contains(word))),
            "應該有提到 {expected:?} 的違規，實際：{violations:#?}"
        );
    }

    #[test]
    fn core_dependency_off_the_table_is_red() {
        let dir = copy_repo();
        append(
            dir.path(),
            CORE_TOML,
            "[dependencies.toml]\nversion = \"1\"\n",
        );
        assert_red(dir.path(), &["qim-core", "toml"]);
    }

    /// 在副本建一個空的函式庫 crate。
    fn add_crate(dir: &Path, name: &str) {
        let crate_dir = dir.join("crates").join(name);
        fs::create_dir_all(crate_dir.join("src")).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
        )
        .unwrap();
        fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    }

    #[test]
    fn core_depending_on_a_binding_crate_is_red() {
        let dir = copy_repo();
        add_crate(dir.path(), "qim-wasm");
        append(
            dir.path(),
            CORE_TOML,
            "[dependencies.qim-wasm]\npath = \"../qim-wasm\"\n",
        );
        assert_red(dir.path(), &["qim-core", "qim-wasm"]);
    }

    #[test]
    fn core_dev_dependency_on_a_binding_crate_is_red() {
        let dir = copy_repo();
        add_crate(dir.path(), "qim-wasm");
        append(
            dir.path(),
            CORE_TOML,
            "[dev-dependencies.qim-wasm]\npath = \"../qim-wasm\"\n",
        );
        assert_red(dir.path(), &["qim-core", "qim-wasm"]);
    }

    #[test]
    fn workspace_member_off_the_table_is_red() {
        let dir = copy_repo();
        add_crate(dir.path(), "qim-extra");
        assert_red(dir.path(), &["qim-extra", "不在 plan §1.3 的表上"]);
    }

    fn assert_green(dir: &Path) {
        let violations = run(dir).unwrap();
        assert!(violations.is_empty(), "不該有違規：{violations:#?}");
    }

    #[test]
    fn current_repo_is_green() {
        assert_green(&repo_root());
    }

    #[test]
    fn dev_dependency_off_the_table_is_green() {
        let dir = copy_repo();
        append(
            dir.path(),
            CORE_TOML,
            "[dev-dependencies.serde_json]\nversion = \"1\"\n",
        );
        assert_green(dir.path());
    }

    const CORE_LIB: &str = "crates/qim-core/src/lib.rs";

    #[test]
    fn use_std_fs_in_core_is_red() {
        let dir = copy_repo();
        let at = append(dir.path(), CORE_LIB, "use std::fs;\n");
        assert_red(dir.path(), &[&core_line(at), "std::fs"]);
    }

    #[test]
    fn std_fs_call_in_core_is_red() {
        let dir = copy_repo();
        let at = append(
            dir.path(),
            CORE_LIB,
            "pub fn load() -> Vec<u8> {\n    std::fs::read(\"pack\").unwrap()\n}\n",
        );
        assert_red(dir.path(), &[&core_line(at + 1), "std::fs"]);
    }

    #[test]
    fn std_fs_in_use_group_is_red() {
        let dir = copy_repo();
        let at = append(dir.path(), CORE_LIB, "use std::{fs, io};\n");
        assert_red(dir.path(), &[&core_line(at), "std::fs"]);
    }

    #[test]
    fn nested_groups_globs_and_submodules_are_red() {
        let dir = copy_repo();
        append(dir.path(), CORE_LIB, "mod clock;\n");
        fs::write(
            dir.path().join("crates/qim-core/src/clock.rs"),
            "use std::{io, time::{Duration, SystemTime}};\nuse std::thread::*;\nuse ::std::*;\n",
        )
        .unwrap();
        let src = "crates/qim-core/src/clock.rs";
        assert_red(dir.path(), &[&format!("{src}:1"), "std::time::SystemTime"]);
        assert_red(dir.path(), &[&format!("{src}:2"), "std::thread"]);
        assert_red(dir.path(), &[&format!("{src}:3"), "std::net"]);
    }

    #[test]
    fn layout_is_scanned_too() {
        let dir = copy_repo();
        add_crate(dir.path(), "qim-layout");
        fs::write(
            dir.path().join("crates/qim-layout/src/lib.rs"),
            "pub fn connect() {\n    let _ = std::net::TcpStream::connect(\"x:1\");\n}\n",
        )
        .unwrap();
        assert_red(dir.path(), &["crates/qim-layout/src/lib.rs:2", "std::net"]);
    }

    #[test]
    fn std_paths_in_comments_and_strings_are_green() {
        let dir = copy_repo();
        append(
            dir.path(),
            CORE_LIB,
            r##"
// use std::fs;
/* std::fs::read("pack") */
/// 呼叫端用 `std::fs::read` 讀檔後傳入。
pub const NOTE: &str = "use std::{fs, io};";
pub const RAW: &str = r#"std::thread::spawn"#;
pub const BYTES: &[u8] = b"std::net";
"##,
        );
        assert_green(dir.path());
    }

    #[test]
    fn formatting_naming_and_allowed_paths_are_green() {
        let dir = copy_repo();
        append(
            dir.path(),
            CORE_LIB,
            "use std::io;\nuse std :: {\n    collections::HashMap ,\n    fmt\n};\nuse std::time::Duration;\n\
             mod fs {\n    pub fn read() {}\n}\npub mod net {}\n\
             pub fn renamed_fs_helper(fs: u8) -> u8 {\n    self::fs::read();\n    crate::fs::read();\n    fs\n}\n",
        );
        // 既有的允許依賴改成怪格式、加行尾註解。
        let toml = dir.path().join(CORE_TOML);
        let manifest = fs::read_to_string(&toml).unwrap();
        let serde_line = manifest
            .lines()
            .find(|line| line.starts_with("serde = "))
            .expect("qim-core 依賴 serde");
        let odd = serde_line.replacen("serde = ", "serde   =   ", 1) + "  # 允許的依賴";
        fs::write(&toml, manifest.replace(serde_line, &odd)).unwrap();
        assert_green(dir.path());
    }

    #[test]
    fn system_time_through_imported_module_is_red() {
        let dir = copy_repo();
        let at = append(
            dir.path(),
            CORE_LIB,
            "use std::time;\npub fn now() {\n    let _ = time::SystemTime::now();\n}\n",
        );
        assert_red(dir.path(), &[&core_line(at + 2), "std::time::SystemTime"]);
    }

    #[test]
    fn core_build_dependency_off_the_table_is_red() {
        let dir = copy_repo();
        append(
            dir.path(),
            CORE_TOML,
            "\n[build-dependencies]\ncc = \"1\"\n",
        );
        assert_red(dir.path(), &["qim-core", "build 依賴", "cc"]);
    }

    #[test]
    fn std_fs_in_cfg_test_module_is_red() {
        let dir = copy_repo();
        let at = append(
            dir.path(),
            CORE_LIB,
            "#[cfg(test)]\nmod tests {\n    use std::fs;\n}\n",
        );
        assert_red(dir.path(), &[&core_line(at + 2), "std::fs"]);
    }

    #[test]
    fn std_fs_outside_src_is_green() {
        let dir = copy_repo();
        let core = dir.path().join("crates/qim-core");
        let reads_file = "fn main() {\n    let _ = std::fs::read(\"pack\");\n}\n";
        for file in [
            "build.rs",
            "tests/pack.rs",
            "benches/pack.rs",
            "examples/pack.rs",
        ] {
            let path = core.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, reads_file).unwrap();
        }
        assert_green(dir.path());
    }

    fn backticked(cell: &str) -> Vec<String> {
        cell.split('`')
            .skip(1)
            .step_by(2)
            .map(str::to_owned)
            .collect()
    }

    /// plan §1.3 表裡第一欄全是 `qim-*` 的列，依表上的順序：(這列的 crate, 各欄)。
    /// `web/` 那類不是 crate。
    fn crate_rows(plan: &str) -> Vec<(Vec<String>, Vec<&str>)> {
        plan.split("\n### ")
            .find(|section| section.starts_with("1.3 "))
            .into_iter()
            .flat_map(str::lines)
            .filter_map(|line| {
                line.trim().starts_with('|').then_some(())?;
                let cells = crate::docs::cells(line);
                let crates = backticked(cells.first()?);
                (!crates.is_empty() && crates.iter().all(|name| name.starts_with("qim-")))
                    .then_some((crates, cells))
            })
            .collect()
    }

    /// plan §1.3 表裡每個 crate 的「可以依賴」欄：(crate, 欄內反引號包住的名字, 外部 crate 是否不限)。
    fn documented_rules(plan: &str) -> Vec<(String, Vec<String>, bool)> {
        let mut rules: Vec<_> = crate_rows(plan)
            .into_iter()
            .filter_map(|(crates, cells)| {
                let allowed_cell = *cells.get(1)?;
                let mut allowed = backticked(allowed_cell);
                allowed.sort();
                Some(
                    crates
                        .into_iter()
                        .map(|krate| (krate, allowed.clone(), allowed_cell.contains("不限")))
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .collect();
        rules.sort();
        rules
    }

    fn coded_rules() -> Vec<(String, Vec<String>, bool)> {
        let mut rules: Vec<_> = RULES
            .iter()
            .map(|rule| {
                let mut allowed: Vec<String> = rule
                    .workspace
                    .iter()
                    .chain(rule.external.unwrap_or_default())
                    .map(|name| (*name).to_owned())
                    .collect();
                allowed.sort();
                (rule.krate.to_owned(), allowed, rule.external.is_none())
            })
            .collect();
        rules.sort();
        rules
    }

    const PLAN_MD: &str = include_str!("../../docs/plan.md");

    #[test]
    fn plan_table_matches_coded_rules() {
        assert_eq!(documented_rules(PLAN_MD), coded_rules());
    }

    /// 把 `text` 裡的 `from` 換成 `to`；`from` 不在就讓測試失敗，免得變異沒有生效。
    fn mutate(text: &str, from: &str, to: &str) -> String {
        assert!(text.contains(from), "變異的原文已不在 plan：{from:?}");
        text.replace(from, to)
    }

    #[test]
    fn drift_between_plan_table_and_rules_is_caught() {
        let dropped = mutate(PLAN_MD, "| `serde`、`postcard`、", "| `serde`、");
        assert_ne!(documented_rules(&dropped), coded_rules());

        let unlimited_removed = mutate(PLAN_MD, "外部 crate 不限", "外部 crate 只限 I/O");
        assert_ne!(documented_rules(&unlimited_removed), coded_rules());

        let extra_row = mutate(
            PLAN_MD,
            "| `web/`、",
            "| `qim-extra` | `qim-core` | — |\n| `web/`、",
        );
        assert_ne!(documented_rules(&extra_row), coded_rules());
    }

    #[test]
    fn unrelated_edits_to_plan_table_are_not_flagged() {
        let edited = mutate(
            PLAN_MD,
            "| crate | 可以依賴 | 不可以 |",
            "| Crate | 允許依賴 | 禁止 |",
        );
        let edited = mutate(&edited, "之後要加的依賴先改本表", "要加依賴就先改這張表");
        let edited = mutate(&edited, "| `qim-layout` | ", "|  `qim-layout`  |  ");
        assert_eq!(documented_rules(&edited), coded_rules());
    }

    /// plan §1.3 表一列的「不可以」欄，讀成可以和 [`RULES`] 比對的樣子。
    #[derive(Debug, Default, Clone)]
    struct Forbidden {
        /// 反引號包住、`std::` 開頭的禁用路徑。
        io: Vec<String>,
        /// 不可以依賴的 workspace crate。
        deps: Vec<String>,
        /// 「被 `X` 依賴」：不可以依賴這列的 crate。
        dependents: Vec<String>,
        /// 「任何 workspace crate」都不可以依賴。
        any_workspace: bool,
    }

    /// 「不可以」欄以「；」分成子句，每句要是下列之一，否則回 `Err`：新寫的規則要先能檢查。
    /// - `同上`：和上一列的整欄相同；`同上的 I/O`：上一列的禁用路徑。
    /// - `被 ... 依賴`，中間至少一個反引號包住的 `qim-*`：它們不可以依賴這列的 crate。
    /// - `任何 workspace crate`，後面只能接全形括號說明：不可以依賴任何 workspace crate。
    /// - 只有反引號包住的名字與說明：`std::` 開頭的是禁用路徑，`qim-` 開頭的是不可以依賴的 crate。
    fn documented_forbidden(plan: &str) -> Result<Vec<(String, Forbidden)>, String> {
        let mut previous = Forbidden::default();
        let mut rows = Vec::new();
        for (crates, cells) in crate_rows(plan) {
            let cell = cells.get(2).ok_or("「不可以」欄不見了")?;
            let mut forbidden = Forbidden::default();
            for clause in cell.split('；').map(str::trim) {
                let names = backticked(clause);
                let all_crates =
                    !names.is_empty() && names.iter().all(|name| name.starts_with("qim-"));
                let explained = |rest: &str| {
                    rest.is_empty()
                        || (rest.starts_with('（') && rest.ends_with('）') && !rest.contains('`'))
                };
                if clause == "同上" {
                    forbidden = previous.clone();
                } else if clause == "同上的 I/O" {
                    forbidden.io = previous.io.clone();
                } else if clause.starts_with('被') && clause.ends_with("依賴") && all_crates {
                    forbidden.dependents = names;
                } else if let Some(rest) = clause.strip_prefix("任何 workspace crate")
                    && explained(rest)
                {
                    forbidden.any_workspace = true;
                } else if !names.is_empty()
                    && names
                        .iter()
                        .all(|n| n.starts_with("std::") || n.starts_with("qim-"))
                {
                    let (io, deps): (Vec<_>, Vec<_>) =
                        names.into_iter().partition(|n| n.starts_with("std::"));
                    forbidden.io.extend(io);
                    forbidden.deps.extend(deps);
                } else {
                    return Err(format!(
                        "{crates:?} 的「不可以」欄有看不懂的子句：{clause:?}"
                    ));
                }
            }
            forbidden.io.sort();
            rows.extend(crates.into_iter().map(|krate| (krate, forbidden.clone())));
            previous = forbidden;
        }
        Ok(rows)
    }

    /// plan §1.3 表的「不可以」欄與 [`RULES`]、[`FORBIDDEN_PATHS`] 對不上的地方。
    fn forbidden_mismatches(plan: &str) -> Vec<String> {
        let rows = match documented_forbidden(plan) {
            Ok(rows) => rows,
            Err(error) => return vec![error],
        };
        let rule = |krate: &str| RULES.iter().find(|rule| rule.krate == krate);
        let mut coded_io: Vec<String> =
            FORBIDDEN_PATHS.iter().map(|path| path.join("::")).collect();
        coded_io.sort();
        let mut mismatches = Vec::new();
        for (krate, forbidden) in rows {
            let Some(own) = rule(&krate) else {
                mismatches.push(format!("{krate} 不在 RULES"));
                continue;
            };
            let expected_io = if own.no_io {
                coded_io.clone()
            } else {
                Vec::new()
            };
            if forbidden.io != expected_io {
                mismatches.push(format!(
                    "{krate} 的禁用路徑：plan {:?}，RULES / FORBIDDEN_PATHS {expected_io:?}",
                    forbidden.io
                ));
            }
            for dep in &forbidden.deps {
                if own.workspace.contains(&dep.as_str()) {
                    mismatches.push(format!("plan 說 {krate} 不可以依賴 {dep}，RULES 卻允許"));
                }
            }
            if forbidden.any_workspace && !own.workspace.is_empty() {
                mismatches.push(format!(
                    "plan 說 {krate} 不可以依賴任何 workspace crate，RULES 卻允許 {:?}",
                    own.workspace
                ));
            }
            for dependent in &forbidden.dependents {
                match rule(dependent) {
                    Some(other) if other.workspace.contains(&krate.as_str()) => mismatches.push(
                        format!("plan 說 {krate} 不可以被 {dependent} 依賴，RULES 卻允許"),
                    ),
                    Some(_) => {}
                    None => mismatches.push(format!("{dependent} 不在 RULES")),
                }
            }
        }
        mismatches
    }

    #[test]
    fn plan_forbidden_column_matches_rules() {
        assert_eq!(forbidden_mismatches(PLAN_MD), Vec::<String>::new());
    }

    #[test]
    fn drift_in_forbidden_column_is_caught() {
        let cases = [
            // 少列一個禁用路徑。
            ("`std::net`、`std::thread`、", "`std::net`、"),
            // qim-layout 不再禁 I/O，RULES 卻還掃。
            ("| `qim-core`；同上的 I/O |", "| `qim-core` |"),
            // 說 qim-wasm 不可以依賴它其實依賴的 qim-core。
            (
                "綁定工具 `wasm-bindgen` | 被",
                "綁定工具 `wasm-bindgen` | `qim-core`；被",
            ),
            // 說 qim-layout 不可以被 qim-wasm 依賴。
            (
                "| `qim-core`；同上的 I/O |",
                "| `qim-core`；同上的 I/O；被 `qim-wasm` 依賴 |",
            ),
            // 說 qim-wasm 不可以依賴任何 workspace crate。
            (
                "綁定工具 `wasm-bindgen` | 被",
                "綁定工具 `wasm-bindgen` | 任何 workspace crate；被",
            ),
            // 寫了檢查不了的新規則，或看起來像可檢查的寫法但多了東西。
            (
                "| `qim-core`；同上的 I/O |",
                "| `qim-core`；同上的 I/O；不可以太大 |",
            ),
            (
                "| `qim-core`；同上的 I/O |",
                "| `qim-core`；同上的 I/O 與網路 |",
            ),
            (
                "| `qim-core`；同上的 I/O |",
                "| `qim-core`；同上的 I/O；被平台殼依賴 |",
            ),
            (
                "任何 workspace crate（含綁定與平台 crate）",
                "任何 workspace crate 以外的 crate",
            ),
        ];
        for (from, to) in cases {
            let mutated = mutate(PLAN_MD, from, to);
            assert_ne!(
                forbidden_mismatches(&mutated),
                Vec::<String>::new(),
                "{from:?} → {to:?} 應該對不上"
            );
        }
    }

    #[test]
    fn unrelated_edits_to_forbidden_column_are_not_flagged() {
        let edited = mutate(
            PLAN_MD,
            "（時間由殼傳入：上屏次數與今天的日期，§2.4）",
            "（時間改由殼傳入上屏次數與日期）",
        );
        let edited = mutate(
            &edited,
            "| `qim-core`；同上的 I/O |",
            "|  `qim-core` ； 同上的 I/O  |",
        );
        let edited = mutate(
            &edited,
            "| crate | 可以依賴 | 不可以 |",
            "| Crate | 允許依賴 | 禁止 |",
        );
        assert_eq!(forbidden_mismatches(&edited), Vec::<String>::new());
    }
}
