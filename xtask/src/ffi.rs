//! `cargo xtask ffi`：給 Apple 平台的 Swift 綁定（`qim-ffi`，plan §1.1）。
//!
//! 用 release 編 `qim-ffi` 的靜態函式庫（這台機器的架構），再用 uniffi 的 `uniffi-bindgen-swift`
//! 從它產生 Swift 原始碼、C 標頭與 modulemap 到 [`SWIFT_DIR`]。Swift 程式連結 [`STATIC_LIB`]、
//! 加 `-Xcc -fmodule-map-file=` 指到 modulemap；做法見 `docs/development.md`〈Swift 綁定〉。
//! `--ios` 另編 `aarch64-apple-ios` 的靜態函式庫，包成 Xcode 專案直接連結的 [`XCFRAMEWORK`]。

use std::path::Path;
use std::process::Command;

use crate::run_command;

/// 產生的 `qim.swift`、`qimFFI.h`、`module.modulemap`；不進 git（plan §1.2）。
pub const SWIFT_DIR: &str = "build/ffi/swift";
/// Swift 程式要連結的靜態函式庫。
pub const STATIC_LIB: &str = "target/release/libqim_ffi.a";
/// modulemap 裡 C 模組的名字；`qim.swift` 以 `import qimFFI` 找它。
const MODULE: &str = "qimFFI";
/// iPhone 實機的 target；模擬器沒有用到，不編。
const IOS_TARGET: &str = "aarch64-apple-ios";
/// `--ios` 的產物：iPhone 實機的靜態函式庫加標頭與 modulemap。
pub const XCFRAMEWORK: &str = "build/ffi/QimFFI.xcframework";
/// 包進 xcframework 的標頭目錄。標頭與 modulemap 放在它底下以模組名命名的子目錄：Xcode 把它們
/// 複製到建置產物的 `include/`，clang 依模組名找 `include/qimFFI/module.modulemap`，也不和別的
/// xcframework 的 modulemap 撞名。不用 `--xcframework` 產生的 `framework module`：靜態函式庫的
/// xcframework 不是 framework，Xcode 找不到它（實測）。
const IOS_HEADERS: &str = "build/ffi/ios-headers";

/// 編靜態函式庫並產生綁定，`ios` 時另包 iPhone 的 xcframework；回傳是否成功，失敗原因印到終端機。
pub fn swift_bindings(root: &Path, ios: bool) -> bool {
    let result = build(root, None)
        .and_then(|()| {
            bindgen(
                root,
                &["--swift-sources", "--headers", "--modulemap"],
                STATIC_LIB,
                SWIFT_DIR,
            )
        })
        .and_then(|()| if ios { xcframework(root) } else { Ok(()) });
    match result {
        Ok(()) => {
            eprintln!("Swift 綁定在 {SWIFT_DIR}，靜態函式庫是 {STATIC_LIB}");
            if ios {
                eprintln!("iPhone 的 xcframework 在 {XCFRAMEWORK}");
            }
            true
        }
        Err(message) => {
            eprintln!("{message}");
            false
        }
    }
}

/// `cargo build --release -p qim-ffi`，`target` 是 `None` 時編這台機器的架構。
fn build(root: &Path, target: Option<&str>) -> Result<(), String> {
    let mut command = Command::new(crate::cargo());
    command.args(["build", "--release", "-p", "qim-ffi"]);
    if let Some(target) = target {
        command.args(["--target", target]);
    }
    run_command(command.current_dir(root))
}

/// 用 `qim-ffi` 的 `uniffi-bindgen-swift` 從 `library` 產生 `outputs` 指定的檔案到 `out_dir`。
fn bindgen(root: &Path, outputs: &[&str], library: &str, out_dir: &str) -> Result<(), String> {
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
            .args(outputs)
            .args([
                "--module-name",
                MODULE,
                "--modulemap-filename",
                "module.modulemap",
            ])
            .arg(library)
            .arg(out_dir)
            .current_dir(root),
    )
}

/// iPhone 的靜態函式庫加標頭包成 [`XCFRAMEWORK`]。
/// Swift 原始碼和 Mac 的同一份（[`SWIFT_DIR`] 的 `qim.swift`）。
fn xcframework(root: &Path) -> Result<(), String> {
    let library = format!("target/{IOS_TARGET}/release/libqim_ffi.a");
    build(root, Some(IOS_TARGET))?;
    for dir in [IOS_HEADERS, XCFRAMEWORK] {
        match std::fs::remove_dir_all(root.join(dir)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(format!("清不掉 {dir}：{e}"));
            }
            _ => {}
        }
    }
    bindgen(
        root,
        &["--headers", "--modulemap"],
        &library,
        &format!("{IOS_HEADERS}/{MODULE}"),
    )?;
    run_command(
        Command::new("xcodebuild")
            .args([
                "-create-xcframework",
                "-library",
                &library,
                "-headers",
                IOS_HEADERS,
            ])
            .args(["-output", XCFRAMEWORK])
            .current_dir(root),
    )
}
