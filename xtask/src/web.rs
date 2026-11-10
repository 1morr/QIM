//! `cargo xtask web`：準備網頁要的 WebAssembly 綁定、資料包與佈局。
//!
//! - [`wasm_bindings`]：用 release 編 `qim-wasm`，`wasm-bindgen --target web` 產生到 [`BINDINGS_DIR`]。
//! - [`serve_data_outputs`]：把 `cargo xtask data` 的產物複製到 `web/public/` 下，網頁 fetch 它們。

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// `wasm-bindgen` 產生的 JS 與 `.wasm`，`web/src/` 直接 import；不進 git。
pub const BINDINGS_DIR: &str = "web/src/wasm";
/// 網頁的資料包與佈局，Vite 原樣提供；不進 git（plan §1.2）。
const PACKS_DIR: &str = "web/public/packs";
const LAYOUTS_DIR: &str = "web/public/layouts";
/// `cargo xtask data` 的產物目錄 → 網頁的位置。
const DATA_OUTPUTS: &[(&str, &str)] = &[("build/packs", PACKS_DIR), ("build/layouts", LAYOUTS_DIR)];
const TARGET: &str = "wasm32-unknown-unknown";

/// 編 `qim-wasm` 並產生 JS 綁定；回傳是否成功，失敗原因印到終端機。
pub fn wasm_bindings(root: &Path) -> bool {
    match try_wasm_bindings(root) {
        Ok(()) => true,
        Err(message) => {
            eprintln!("{message}");
            false
        }
    }
}

fn try_wasm_bindings(root: &Path) -> Result<(), String> {
    let lock = fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|e| format!("讀不到 Cargo.lock：{e}"))?;
    let wanted = locked_version(&lock, "wasm-bindgen")
        .ok_or("Cargo.lock 沒有 wasm-bindgen；qim-wasm 應該依賴它")?;
    let installed = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok());
    let install_hint =
        format!("用 `cargo install wasm-bindgen-cli --version {wanted} --locked` 安裝");
    match installed.as_deref().and_then(cli_version) {
        None => return Err(format!("找不到 wasm-bindgen：{install_hint}")),
        Some(found) if found != wanted => {
            return Err(format!(
                "wasm-bindgen 是 {found}，與 Cargo.lock 的 {wanted} 不同：{install_hint}"
            ));
        }
        Some(_) => {}
    }

    crate::run_command(
        Command::new(crate::cargo())
            .args(["build", "--release", "-p", "qim-wasm", "--target", TARGET])
            .current_dir(root),
    )?;
    let wasm = root.join(format!("target/{TARGET}/release/qim_wasm.wasm"));
    crate::run_command(
        Command::new("wasm-bindgen")
            .args(["--target", "web", "--out-dir"])
            .arg(root.join(BINDINGS_DIR))
            .arg(wasm),
    )
}

/// 把 [`DATA_OUTPUTS`] 的每個目錄整個複製到網頁，先清掉舊的；回傳是否成功。
pub fn serve_data_outputs(root: &Path) -> bool {
    for (built, served) in DATA_OUTPUTS {
        let from = root.join(built);
        if !from.is_dir() {
            eprintln!("{built} 不存在：先跑 `cargo xtask data`");
            return false;
        }
        let to = root.join(served);
        let result = match fs::remove_dir_all(&to) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => copy_dir(&from, &to),
        };
        if let Err(e) = result {
            eprintln!("複製 {built} 到 {served} 失敗：{e}");
            return false;
        }
    }
    true
}

/// `Cargo.lock` 裡 `name` 這個套件的版本；同名有多個版本時取第一個。
fn locked_version<'a>(lock: &'a str, name: &str) -> Option<&'a str> {
    let wanted = format!("name = \"{name}\"");
    let mut lines = lock.lines();
    lines.find(|line| line.trim() == wanted)?;
    lines
        .next()?
        .trim()
        .strip_prefix("version = \"")?
        .strip_suffix('"')
}

/// `wasm-bindgen --version` 的輸出（`wasm-bindgen 0.2.130`）裡的版本。
fn cli_version(output: &str) -> Option<&str> {
    output
        .trim()
        .strip_prefix("wasm-bindgen ")?
        .split(' ')
        .next()
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK: &str = r#"
[[package]]
name = "wasm-bindgen-shared"
version = "0.2.130"

[[package]]
name = "wasm-bindgen"
version = "0.2.130"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    #[test]
    fn reads_the_locked_version_of_the_exact_package() {
        assert_eq!(locked_version(LOCK, "wasm-bindgen"), Some("0.2.130"));
        assert_eq!(locked_version(LOCK, "wasm"), None);
    }

    #[test]
    fn reads_the_cli_version() {
        assert_eq!(cli_version("wasm-bindgen 0.2.130\n"), Some("0.2.130"));
        assert_eq!(cli_version("zsh: command not found"), None);
    }

    #[test]
    fn copies_packs_and_layouts_and_drops_stale_files() {
        let root = tempfile::tempdir().unwrap();
        let built = root.path().join("build/packs/quick");
        fs::create_dir_all(&built).unwrap();
        fs::write(built.join("quick.pack"), b"pack").unwrap();
        let layouts = root.path().join("build/layouts");
        fs::create_dir_all(&layouts).unwrap();
        fs::write(layouts.join("gboard-quick.json"), b"{}").unwrap();
        let stale = root.path().join(PACKS_DIR).join("old");
        fs::create_dir_all(&stale).unwrap();
        let stale_layout = root.path().join(LAYOUTS_DIR).join("old.json");
        fs::create_dir_all(stale_layout.parent().unwrap()).unwrap();
        fs::write(&stale_layout, b"{}").unwrap();

        assert!(serve_data_outputs(root.path()));
        let copied = root.path().join(PACKS_DIR).join("quick/quick.pack");
        assert_eq!(fs::read(copied).unwrap(), b"pack");
        let layout = root.path().join(LAYOUTS_DIR).join("gboard-quick.json");
        assert_eq!(fs::read(layout).unwrap(), b"{}");
        assert!(!stale.exists());
        assert!(!stale_layout.exists());
    }

    #[test]
    fn serve_data_outputs_fails_when_data_was_not_built() {
        let root = tempfile::tempdir().unwrap();
        assert!(!serve_data_outputs(root.path()));
        // 只有資料包、沒有佈局（`cargo xtask data` 是舊版時建的）也失敗。
        fs::create_dir_all(root.path().join("build/packs/quick")).unwrap();
        assert!(!serve_data_outputs(root.path()));
    }
}
