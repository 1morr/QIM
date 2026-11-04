//! `cargo xtask check`：依序跑各檢查步驟，第一個失敗就停。
//!
//! 加步驟：在 [`steps`] 的清單加一列；不是外部指令的檢查就給 [`Action`] 加一種。

use std::path::Path;
use std::process::Command;

/// 一個檢查步驟。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// 失敗時回報的名字。
    pub name: &'static str,
    /// `--fast` 時跳過。
    pub slow: bool,
    pub action: Action,
}

/// 步驟要做的事。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// 在 repo 根目錄跑 `cargo <args>`。
    Cargo(&'static [&'static str]),
    /// 在 `web/` 跑 `npm run <script>`。
    NpmScript(&'static str),
    /// 依賴規則（plan §1.3），見 [`crate::deps`]。
    Deps,
    /// progress 與里程碑 spec 的規則（plan §6），見 [`crate::docs`]。
    Docs,
    /// 產生網頁的 WebAssembly 綁定，`tsc` 要讀它的型別；見 [`crate::web::wasm_bindings`]。
    WasmBindings,
    /// 複製資料包到網頁，再跑 Playwright（`npm run e2e`）。
    E2e,
}

const ALL: &[Step] = &[
    Step {
        name: "fmt",
        slow: false,
        action: Action::Cargo(&["fmt", "--all", "--check"]),
    },
    Step {
        name: "clippy",
        slow: false,
        action: Action::Cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]),
    },
    Step {
        name: "test",
        slow: true,
        action: Action::Cargo(&["test", "--workspace"]),
    },
    Step {
        name: "deps",
        slow: false,
        action: Action::Deps,
    },
    Step {
        name: "docs",
        slow: false,
        action: Action::Docs,
    },
    Step {
        name: "wasm",
        slow: false,
        action: Action::WasmBindings,
    },
    Step {
        name: "tsc",
        slow: false,
        action: Action::NpmScript("typecheck"),
    },
    Step {
        name: "biome",
        slow: false,
        action: Action::NpmScript("lint"),
    },
    Step {
        name: "e2e",
        slow: true,
        action: Action::E2e,
    },
];

/// 要跑的步驟，依執行順序；`fast` 為真時跳過慢的步驟。
pub fn steps(fast: bool) -> Vec<Step> {
    ALL.iter()
        .filter(|step| !(fast && step.slow))
        .copied()
        .collect()
}

/// 依序執行 `plan`，`execute_step` 回傳該步驟是否通過；遇到第一個失敗就停，回傳它的名字。
pub fn run(plan: &[Step], mut execute_step: impl FnMut(&Step) -> bool) -> Result<(), &'static str> {
    for step in plan {
        if !execute_step(step) {
            return Err(step.name);
        }
    }
    Ok(())
}

/// 實際執行一個步驟，輸出直接接到終端機。
pub fn exec(step: &Step, root: &Path) -> bool {
    eprintln!("==> {}", step.name);
    let mut command = match step.action {
        Action::Cargo(args) => {
            let mut command = Command::new(crate::cargo());
            command.args(args).current_dir(root);
            command
        }
        Action::NpmScript(script) => {
            let Some(command) = npm(root, script) else {
                return false;
            };
            command
        }
        Action::E2e => {
            let Some(command) = crate::web::built_data(root)
                .then(|| npm(root, "e2e"))
                .flatten()
            else {
                return false;
            };
            command
        }
        Action::Deps => return crate::deps::report(root),
        Action::Docs => return crate::docs::report(root),
        Action::WasmBindings => return crate::web::wasm_bindings(root),
    };
    match command.status() {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("無法執行 {}：{error}", step.name);
            false
        }
    }
}

/// 在 `web/` 跑 `npm run <script>` 的指令；`node_modules` 還沒裝時印提示、回 `None`。
fn npm(root: &Path, script: &str) -> Option<Command> {
    let web = root.join("web");
    if !web.join("node_modules").is_dir() {
        eprintln!("web/node_modules 不存在：先在 web/ 跑 `npm ci`（見 docs/development.md）");
        return None;
    }
    // Windows 上 npm 是批次檔，Command 不會自己補副檔名。
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let mut command = Command::new(npm);
    command.args(["run", "--silent", script]).current_dir(web);
    Some(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn step(name: &'static str) -> Step {
        Step {
            name,
            slow: false,
            action: Action::Cargo(&[]),
        }
    }

    #[test]
    fn stops_at_first_failing_step_and_names_it() {
        let mut ran = Vec::new();
        let result = run(&[step("fmt"), step("clippy"), step("test")], |step| {
            ran.push(step.name);
            step.name != "clippy"
        });
        assert_eq!(result, Err("clippy"));
        assert_eq!(ran, ["fmt", "clippy"]);
    }

    fn names(steps: &[Step]) -> Vec<&'static str> {
        steps.iter().map(|step| step.name).collect()
    }

    #[test]
    fn full_check_runs_every_step_in_order() {
        assert_eq!(
            names(&steps(false)),
            [
                "fmt", "clippy", "test", "deps", "docs", "wasm", "tsc", "biome", "e2e"
            ]
        );
    }

    #[test]
    fn fast_check_skips_cargo_test_and_e2e() {
        assert_eq!(
            names(&steps(true)),
            ["fmt", "clippy", "deps", "docs", "wasm", "tsc", "biome"]
        );
    }

    /// 從 `docs/development.md`〈全部檢查〉的表格讀出 (步驟名, `--fast` 是否跑)。
    /// 步驟名是第一欄以反引號包住的字，`--fast` 是最後一欄的「跑」或「跳過」。
    fn documented_steps(markdown: &str) -> Vec<(String, bool)> {
        markdown
            .split("\n## ")
            .find(|section| section.starts_with("全部檢查"))
            .into_iter()
            .flat_map(str::lines)
            .filter_map(|line| {
                line.trim().starts_with('|').then_some(())?;
                let cells = crate::docs::cells(line);
                let name = cells.first()?.strip_prefix('`')?.strip_suffix('`')?;
                let fast = match *cells.last()? {
                    "跑" => true,
                    "跳過" => false,
                    _ => return None,
                };
                Some((name.to_owned(), fast))
            })
            .collect()
    }

    const DEVELOPMENT_MD: &str = include_str!("../../docs/development.md");

    /// 程式碼裡的步驟，以 (名字, `--fast` 是否跑) 表示，與文件表格比對。
    fn coded_steps() -> Vec<(String, bool)> {
        steps(false)
            .iter()
            .map(|step| (step.name.to_owned(), !step.slow))
            .collect()
    }

    #[test]
    fn development_md_lists_every_step_as_coded() {
        assert_eq!(documented_steps(DEVELOPMENT_MD), coded_steps());
    }

    #[test]
    fn missing_or_wrong_row_in_development_md_is_caught() {
        let missing = DEVELOPMENT_MD.replace("| `tsc` |", "| tsc-removed |");
        assert_ne!(documented_steps(&missing), coded_steps());

        let wrong_fast = DEVELOPMENT_MD.replace(
            "| `cargo test --workspace` | 跳過 |",
            "| `cargo test --workspace` | 跑 |",
        );
        assert_ne!(documented_steps(&wrong_fast), coded_steps());
    }

    #[test]
    fn unrelated_edits_to_development_md_are_not_flagged() {
        let edited = DEVELOPMENT_MD
            .replace("| --- | --- | --- |", "|:---|:---|:---:|")
            .replace("每張票收尾時照跑", "每張票收尾時都照跑")
            .replace(
                "| `fmt` | `cargo fmt --all --check` | 跑 |",
                "|  `fmt`  |  cargo fmt  |  跑  |",
            );
        assert_ne!(edited, DEVELOPMENT_MD);
        assert_eq!(documented_steps(&edited), coded_steps());
    }
}
