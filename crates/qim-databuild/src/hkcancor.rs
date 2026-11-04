//! HKCanCor（香港粵語語料庫）的轉寫檔 → 話輪文字（plan §4.3 的口語語料）。
//!
//! 取 fcbond/hkcancor 的 `data/hkcancor-utf8.zip`：`utf8/` 下每個檔是一段錄音的轉寫，
//! 一個話輪一個 `<sent>`，`<sent_tag>` 裡一行一詞，寫成 `詞/詞性/粵拼/`。只取詞，
//! 照原順序接成一個話輪；標點也是詞（`，/w/VQ2/`），留給切句當分隔。夾英文詞或粵拼的話輪
//! 在切句後轉碼失敗、整句丟掉，同其他語料。

use std::io::Read;

use crate::Error;

const WHAT: &str = "HKCanCor zip";

/// 一個轉寫檔裡每個話輪的文字，依原順序。
pub fn utterances(transcript: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Option<String> = None;
    for line in transcript.lines().map(str::trim) {
        match line {
            "<sent_tag>" => current = Some(String::new()),
            "</sent_tag>" => out.extend(current.take()),
            word => {
                if let Some(text) = &mut current {
                    text.push_str(word.split('/').next().unwrap_or_default());
                }
            }
        }
    }
    out
}

/// zip 裡 `utf8/` 下每個轉寫檔的話輪，檔案依名稱排序，和 zip 的存放順序無關。
pub fn archive(
    zip: &[u8],
    mut utterance: impl FnMut(&str) -> Result<(), Error>,
) -> Result<(), Error> {
    let invalid = |e: &dyn std::fmt::Display| Error::invalid(WHAT, e.to_string());
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|e| invalid(&e))?;
    let mut names = Vec::new();
    for name in archive.file_names() {
        let name = name.map_err(|e| invalid(&e))?;
        if name.starts_with("utf8/") && !name.ends_with('/') {
            names.push(name.into_owned());
        }
    }
    names.sort();
    for name in names {
        let mut text = String::new();
        archive
            .by_name(&name)
            .map_err(|e| invalid(&e))?
            .read_to_string(&mut text)
            .map_err(|e| Error::invalid(format!("{WHAT} 的 {name}"), e.to_string()))?;
        for u in utterances(&text) {
            utterance(&u)?;
        }
    }
    Ok(())
}
