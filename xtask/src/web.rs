//! `cargo xtask web`：準備網頁要的 WebAssembly 綁定與資料包。
//!
//! - [`bindings`]：用 release 編 `qim-wasm`，`wasm-bindgen --target web` 產生到 [`BINDINGS_DIR`]。
//! - [`packs`]：把 `build/packs/` 複製到 [`PACKS_DIR`]，網頁 fetch 它。

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

/// `wasm-bindgen` 產生的 JS 與 `.wasm`，`web/src/` 直接 import；不進 git。
pub const BINDINGS_DIR: &str = "web/src/wasm";
/// 網頁的資料包，Vite 原樣提供；不進 git（plan §1.2）。
pub const PACKS_DIR: &str = "web/public/packs";
/// `cargo xtask data` 的產物。
const BUILT_PACKS_DIR: &str = "build/packs";
const TARGET: &str = "wasm32-unknown-unknown";

/// 編 `qim-wasm` 並產生 JS 綁定；回傳是否成功，失敗原因印到終端機。
pub fn bindings(root: &Path) -> bool {
    match try_bindings(root) {
        Ok(()) => true,
        Err(message) => {
            eprintln!("{message}");
            false
        }
    }
}

fn try_bindings(root: &Path) -> Result<(), String> {
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

    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    run(Command::new(cargo)
        .args(["build", "--release", "-p", "qim-wasm", "--target", TARGET])
        .current_dir(root))?;
    let wasm = root.join(format!("target/{TARGET}/release/qim_wasm.wasm"));
    run(Command::new("wasm-bindgen")
        .args(["--target", "web", "--out-dir"])
        .arg(root.join(BINDINGS_DIR))
        .arg(wasm))
}

/// 把 `build/packs/` 整個複製到 `web/public/packs/`，先清掉舊的；回傳是否成功。
pub fn packs(root: &Path) -> bool {
    let from = root.join(BUILT_PACKS_DIR);
    if !from.is_dir() {
        eprintln!("{BUILT_PACKS_DIR} 不存在：先跑 `cargo xtask data`");
        return false;
    }
    let to = root.join(PACKS_DIR);
    let result = match fs::remove_dir_all(&to) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => copy_dir(&from, &to),
    };
    match result {
        Ok(()) => true,
        Err(e) => {
            eprintln!("複製資料包到 {PACKS_DIR} 失敗：{e}");
            false
        }
    }
}

fn run(command: &mut Command) -> Result<(), String> {
    let program = command.get_program().to_string_lossy().into_owned();
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{program} 失敗：{status}")),
        Err(e) => Err(format!("無法執行 {program}：{e}")),
    }
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
    fn copies_packs_and_drops_stale_files() {
        let root = tempfile::tempdir().unwrap();
        let built = root.path().join(BUILT_PACKS_DIR).join("quick");
        fs::create_dir_all(&built).unwrap();
        fs::write(built.join("quick.pack"), b"pack").unwrap();
        let stale = root.path().join(PACKS_DIR).join("old");
        fs::create_dir_all(&stale).unwrap();

        assert!(packs(root.path()));
        let copied = root.path().join(PACKS_DIR).join("quick/quick.pack");
        assert_eq!(fs::read(copied).unwrap(), b"pack");
        assert!(!stale.exists());
    }

    #[test]
    fn packs_fail_when_data_was_not_built() {
        let root = tempfile::tempdir().unwrap();
        assert!(!packs(root.path()));
    }
}
