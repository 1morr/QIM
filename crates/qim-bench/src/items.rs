//! 題目檔的格式：`bench/fixed.tsv` 與 `bench/corpus/<名>.tsv`（`docs/development.md`〈首選字基準〉）。

/// 一題：把 `input` 逐鍵打進核心，首選應該是 `expected`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub input: String,
    pub expected: String,
    /// 回報時指出是哪一題：固定題是「來源」欄，語料是行號。
    pub label: String,
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
        items.push(item(number, input, expected, source)?);
    }
    Ok(items)
}

/// `<逐字速成碼，空格分隔><TAB><句子>`，輸入碼是各字的碼接起來。
pub fn parse_corpus(text: &str) -> Result<Vec<Item>, String> {
    let mut items = Vec::new();
    for (number, line) in numbered_lines(text) {
        let Some((codes, sentence)) = line.split_once('\t') else {
            return Err(format!("第 {number} 行少了 tab：要有輸入碼與句子兩欄"));
        };
        let input: String = codes.split(' ').collect();
        items.push(item(number, &input, sentence, &format!("第 {number} 行"))?);
    }
    Ok(items)
}

fn item(number: usize, input: &str, expected: &str, label: &str) -> Result<Item, String> {
    if input.is_empty() || expected.is_empty() {
        return Err(format!("第 {number} 行的輸入碼或期望首選是空的"));
    }
    Ok(Item {
        input: input.to_string(),
        expected: expected.to_string(),
        label: label.to_string(),
    })
}

/// 非空行與它的行號（從 1 起）。
fn numbered_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line))
        .filter(|(_, line)| !line.trim().is_empty())
}
