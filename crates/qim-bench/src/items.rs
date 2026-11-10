//! 題目檔的格式：`bench/fixed.tsv`、`bench/corpus/<名>.tsv`、「有前文」的 `bench/corpus/context/<名>.tsv`
//! 與個人集 `bench/personal/rime-userdb.tsv`（`docs/development.md`〈首選字基準〉）。

/// 一題：把輸入碼逐鍵打進核心，首選應該是期望的文字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// 依序的單位；語料一個字一個單位，固定題整題一個單位。修正次數的模擬卡住時一個單位一個單位重打。
    pub units: Vec<Unit>,
    /// 題目在檔裡的行號（從 1 起），翻盤清單與個人集只用它指出題目。
    pub line: usize,
    /// 回報時指出是哪一題：固定題是「來源」欄，其他是行號。
    pub label: String,
    /// 依權重報數字時的權重：個人集是上屏次數，其他是 1。
    pub weight: u64,
    /// 輸入框裡題目前面的文字（`Session::set_context`）；只有「有前文」的題有。
    pub context: Option<String>,
}

/// 基準模擬打字的單位：一段輸入碼與它該打出的文字。是題目檔的切法，和核心組字的〈分段〉（`Segment`）無關。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit {
    pub code: String,
    pub text: String,
}

impl Item {
    /// 整題的輸入碼。
    pub fn input(&self) -> String {
        self.units.iter().map(|u| u.code.as_str()).collect()
    }

    /// 期望的首選。
    pub fn expected(&self) -> String {
        self.units.iter().map(|u| u.text.as_str()).collect()
    }
}

/// `輸入碼<TAB>期望首選<TAB>來源`；空行與 `#` 開頭的行略過。
pub fn parse_fixed(text: &str) -> Result<Vec<Item>, String> {
    let mut items = Vec::new();
    for (number, line) in numbered_lines(text) {
        if line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [input, expected, source] = fields.as_slice() else {
            return Err(format!(
                "第 {number} 行要有 3 欄（輸入碼、期望首選、來源），實際 {} 欄",
                fields.len()
            ));
        };
        if input.is_empty() || expected.is_empty() {
            return Err(format!("第 {number} 行的輸入碼或期望首選是空的"));
        }
        items.push(Item {
            units: vec![Unit {
                code: input.to_string(),
                text: expected.to_string(),
            }],
            line: number,
            label: source.to_string(),
            weight: 1,
            context: None,
        });
    }
    Ok(items)
}

/// `<逐字速成碼，空格分隔><TAB><句子>`。
pub fn parse_corpus(text: &str) -> Result<Vec<Item>, String> {
    let mut items = Vec::new();
    for (number, line) in numbered_lines(text) {
        let Some((codes, sentence)) = line.split_once('\t') else {
            return Err(format!("第 {number} 行少了 tab：要有輸入碼與句子兩欄"));
        };
        items.push(per_character(number, codes, sentence, 1)?);
    }
    Ok(items)
}

/// 「有前文」的題：`<前文><TAB><逐字速成碼，空格分隔><TAB><句子>`，前文不能是空的。
pub fn parse_context(text: &str) -> Result<Vec<Item>, String> {
    let mut items = Vec::new();
    for (number, line) in numbered_lines(text) {
        let fields: Vec<&str> = line.split('\t').collect();
        let [context, codes, sentence] = fields.as_slice() else {
            return Err(format!(
                "第 {number} 行要有 3 欄（前文、輸入碼、句子），實際 {} 欄",
                fields.len()
            ));
        };
        if context.is_empty() {
            return Err(format!("第 {number} 行的前文是空的"));
        }
        items.push(Item {
            context: Some(context.to_string()),
            ..per_character(number, codes, sentence, 1)?
        });
    }
    Ok(items)
}

/// 個人集：`<逐字速成碼，空格分隔><TAB><詞><TAB><上屏次數>`，由 `cargo xtask data` 從 Rime 學習資料轉出；
/// `#` 開頭的行（記來源 sha256 的檔頭）略過。錯誤訊息只指出行號，不帶內容（plan §5.2）。
pub fn parse_personal(text: &str) -> Result<Vec<Item>, String> {
    let mut items = Vec::new();
    for (number, line) in numbered_lines(text).filter(|(_, line)| !line.starts_with('#')) {
        let fields: Vec<&str> = line.split('\t').collect();
        let [codes, word, count] = fields.as_slice() else {
            return Err(format!(
                "第 {number} 行要有 3 欄（輸入碼、詞、上屏次數），實際 {} 欄",
                fields.len()
            ));
        };
        let weight = count
            .parse::<u64>()
            .ok()
            .filter(|&c| c >= 1)
            .ok_or_else(|| format!("第 {number} 行的上屏次數不是正整數"))?;
        items.push(per_character(number, codes, word, weight)?);
    }
    Ok(items)
}

/// 題目裡每段連續 `len` 個字各當一題（逐字的碼 → 那幾個字），不重複，權重是出現的次數；
/// 依第一次出現的順序，行號是這個順序。dev 模式量短輸入（單字、詞）的首選用。
pub fn substrings(items: &[Item], len: usize) -> Vec<Item> {
    let mut found: Vec<Item> = Vec::new();
    let mut index = std::collections::HashMap::new();
    for window in items.iter().flat_map(|item| item.units.windows(len)) {
        let i = *index.entry(window.to_vec()).or_insert_with(|| {
            let line = found.len() + 1;
            found.push(Item {
                units: window.to_vec(),
                line,
                label: format!("第 {line} 題"),
                weight: 0,
                context: None,
            });
            line - 1
        });
        found[i].weight += 1;
    }
    found
}

/// 每個字配一個碼的題目；碼數與字數不同就是壞檔。訊息不帶內容，個人集也用它。
fn per_character(number: usize, codes: &str, text: &str, weight: u64) -> Result<Item, String> {
    let codes: Vec<&str> = codes.split(' ').collect();
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() || codes.iter().any(|c| c.is_empty()) {
        return Err(format!("第 {number} 行的輸入碼或文字是空的"));
    }
    if codes.len() != chars.len() {
        return Err(format!(
            "第 {number} 行有 {} 個碼、{} 個字，要一個字一個碼",
            codes.len(),
            chars.len()
        ));
    }
    Ok(Item {
        units: codes
            .iter()
            .zip(chars)
            .map(|(code, ch)| Unit {
                code: code.to_string(),
                text: ch.to_string(),
            })
            .collect(),
        line: number,
        label: format!("第 {number} 行"),
        weight,
        context: None,
    })
}

/// 非空行與它的行號（從 1 起）。
fn numbered_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line))
        .filter(|(_, line)| !line.trim().is_empty())
}
