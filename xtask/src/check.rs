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
        name: "tsc",
        slow: false,
        action: Action::NpmScript("typecheck"),
    },
    Step {
        name: "biome",
        slow: false,
        action: Action::NpmScript("lint"),
    },
];

/// 要跑的步驟，依執行順序；`fast` 為真時跳過慢的步驟。
pub fn steps(fast: bool) -> Vec<Step> {
    ALL.iter()
        .filter(|step| !(fast && step.slow))
        .copied()
        .collect()
}

/// 依序執行 `steps`，`exec` 回傳該步驟是否通過；遇到第一個失敗就停，回傳它的名字。
pub fn run(steps: &[Step], mut exec: impl FnMut(&Step) -> bool) -> Result<(), &'static str> {
    for step in steps {
        if !exec(step) {
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
            let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
            let mut command = Command::new(cargo);
            command.args(args).current_dir(root);
            command
        }
        Action::NpmScript(script) => {
            let web = root.join("web");
            if !web.join("node_modules").is_dir() {
                eprintln!(
                    "web/node_modules 不存在：先在 web/ 跑 `npm ci`（見 docs/development.md）"
                );
                return false;
            }
            // Windows 上 npm 是批次檔，Command 不會自己補副檔名。
            let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
            let mut command = Command::new(npm);
            command.args(["run", "--silent", script]).current_dir(web);
            command
        }
    };
    match command.status() {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("無法執行 {}：{error}", step.name);
            false
        }
    }
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
            ["fmt", "clippy", "test", "tsc", "biome"]
        );
    }

    #[test]
    fn fast_check_skips_only_cargo_test() {
        assert_eq!(names(&steps(true)), ["fmt", "clippy", "tsc", "biome"]);
    }
}
