//! 個人集：把使用者 Rime 的學習資料（brief §12.2 的 `.userdb.txt`）轉成首選字基準的題目（plan §5.2）。
//!
//! 輸入碼由 [`Transcoder`] 從詞轉出，不用 Rime 記下的碼：那是詞典的倉頡全碼，不是使用者按的鍵。
//! 內容只在本機；錯誤訊息只帶行號，不帶內容。

use std::collections::HashMap;

use crate::transcode::Transcoder;

/// 使用者放的 Rime 學習資料，相對 repo 根目錄。
pub const USERDB: &str = "bench/personal/rime-userdb.txt";
/// 轉出的題目，`qim-bench` 讀它：第一行 [`SOURCE_HEADER`] 接 [`USERDB`] 的 sha256，
/// 之後一行「逐字速成碼（空格分隔）<TAB>詞<TAB>上屏次數」。
/// 路徑與檔頭在 `qim_bench` 有同樣的常數（plan §1.3 不讓 `qim-bench` 依賴本 crate），改時兩邊一起改。
pub const ITEMS: &str = "bench/personal/rime-userdb.tsv";
pub const SOURCE_HEADER: &str = "# rime-userdb.txt sha256 ";

/// 轉換的統計，只有數字。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// 轉出的題數。
    pub words: usize,
    /// 上屏次數 `c` < 1（沒用過或已刪除）而略過的列。
    pub unused: usize,
    /// 含連打字集以外的字、轉不了碼而略過的列。
    pub untranscodable: usize,
    /// 同一個詞在不同輸入碼下出現多次，併進第一次出現那題的列數（上屏次數相加）。
    pub merged: usize,
}

/// 取 `c ≥ 1` 的詞，同一個詞合併、上屏次數相加，依第一次出現的順序寫成題目檔的內容。
pub fn from_userdb(text: &str, transcoder: &Transcoder) -> Result<(String, Counts), String> {
    let mut counts = Counts::default();
    // 詞 → (逐字碼, 上屏次數)，依第一次出現的順序。
    let mut words: Vec<(&str, String, u64)> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();
    for (i, line) in text.lines().enumerate() {
        let number = i + 1;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [_, word, stats] = fields.as_slice() else {
            return Err(format!(
                "第 {number} 行要有 3 欄（輸入碼、詞、統計），實際 {} 欄",
                fields.len()
            ));
        };
        let commits = stats
            .split(' ')
            .find_map(|field| field.strip_prefix("c="))
            .and_then(|c| c.parse::<i64>().ok())
            .ok_or_else(|| format!("第 {number} 行的統計欄沒有整數的 c=（上屏次數）"))?;
        let Ok(commits @ 1..) = u64::try_from(commits) else {
            counts.unused += 1;
            continue;
        };
        if let Some(&at) = index.get(word) {
            words[at].2 += commits;
            counts.merged += 1;
            continue;
        }
        let Ok(codes) = transcoder.transcode(word) else {
            counts.untranscodable += 1;
            continue;
        };
        if codes.is_empty() {
            counts.untranscodable += 1;
            continue;
        }
        index.insert(word, words.len());
        words.push((word, codes.join(" "), commits));
    }
    counts.words = words.len();
    let tsv = words
        .iter()
        .map(|(word, codes, commits)| format!("{codes}\t{word}\t{commits}\n"))
        .collect();
    Ok((tsv, counts))
}
