//! M2 票 11：鍵盤擴充跑小型神經模型的開銷。用法與結果見 README。

pub mod bench;
pub mod footprint;
pub mod model;

use std::ffi::{CStr, CString, c_char};
use std::path::Path;

/// 給 iOS 擴充呼叫：跑 `bench::run`，回 UTF-8 字串（錯誤也是字串，以 `error:` 開頭）。
/// 回傳值要交給 `nc_free` 釋放。
///
/// # Safety
/// `model_path` 是以 NUL 結尾的 UTF-8 路徑。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nc_run(
    model_path: *const c_char,
    iterations: u32,
    threads: u32,
) -> *mut c_char {
    // SAFETY: 呼叫者保證是 NUL 結尾的字串。
    let path = unsafe { CStr::from_ptr(model_path) }
        .to_string_lossy()
        .into_owned();
    let text = match bench::run(
        Path::new(&path),
        iterations as usize,
        threads.max(1) as usize,
        bench::Prefix::Shared,
    ) {
        Ok(text) => text,
        Err(e) => format!("error: {e}"),
    };
    CString::new(text.replace('\0', " ")).map_or(std::ptr::null_mut(), CString::into_raw)
}

/// 釋放 `nc_run` 回傳的字串。
///
/// # Safety
/// `s` 是 `nc_run` 回傳、還沒釋放過的指標，或 null。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nc_free(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: 由 CString::into_raw 而來。
        drop(unsafe { CString::from_raw(s) });
    }
}
