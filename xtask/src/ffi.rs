//! `cargo xtask ffi`：給 Apple 平台的 Swift 綁定（`qim-ffi`，plan §1.1）。
//!
//! 用 release 編 `qim-ffi` 的靜態函式庫（這台機器的架構），再用 uniffi 的 `uniffi-bindgen-swift`
//! 從它產生 Swift 原始碼、C 標頭與 modulemap 到 [`SWIFT_DIR`]。Swift 程式連結 [`STATIC_LIB`]、
//! 加 `-Xcc -fmodule-map-file=` 指到 modulemap；做法見 `docs/development.md`〈Swift 綁定〉。

use std::path::Path;
use std::process::Command;

use crate::run_command;

/// 產生的 `qim.swift`、`qimFFI.h`、`module.modulemap`；不進 git（plan §1.2）。
pub const SWIFT_DIR: &str = "build/ffi/swift";
/// Swift 程式要連結的靜態函式庫。
pub const STATIC_LIB: &str = "target/release/libqim_ffi.a";
/// modulemap 裡 C 模組的名字；`qim.swift` 以 `import qimFFI` 找它。
const MODULE: &str = "qimFFI";

/// 編靜態函式庫並產生綁定；回傳是否成功，失敗原因印到終端機。
pub fn swift_bindings(root: &Path) -> bool {
    let result = run_command(
        Command::new(crate::cargo())
            .args(["build", "--release", "-p", "qim-ffi"])
            .current_dir(root),
    )
    .and_then(|()| {
        run_command(
            Command::new(crate::cargo())
                .args(["run", "--quiet", "--release", "-p", "qim-ffi"])
                .args([
                    "--features",
                    "bindgen",
                    "--bin",
                    "uniffi-bindgen-swift",
                    "--",
                ])
                .args(["--swift-sources", "--headers", "--modulemap"])
                .args([
                    "--module-name",
                    MODULE,
                    "--modulemap-filename",
                    "module.modulemap",
                ])
                .arg(STATIC_LIB)
                .arg(SWIFT_DIR)
                .current_dir(root),
        )
    });
    match result {
        Ok(()) => {
            eprintln!("Swift 綁定在 {SWIFT_DIR}，靜態函式庫是 {STATIC_LIB}");
            true
        }
        Err(message) => {
            eprintln!("{message}");
            false
        }
    }
}
