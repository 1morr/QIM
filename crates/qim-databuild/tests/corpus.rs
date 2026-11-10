use qim_databuild::corpus::{Builder, Split, clauses, split_of};
use qim_databuild::libcangjie;
use qim_databuild::transcode::Transcoder;

fn all(text: &str) -> Vec<&str> {
    clauses(text).collect()
}

#[test]
fn punctuation_and_whitespace_end_a_clause() {
    assert_eq!(
        all("佢哋就合稱做歐羅區，到而家。好！「引號」（括號）米高·佐敦 換行\n下一行…完"),
        [
            "佢哋就合稱做歐羅區",
            "到而家",
            "好",
            "引號",
            "括號",
            "米高",
            "佐敦",
            "換行",
            "下一行",
            "完"
        ]
    );
}

#[test]
fn letters_and_digits_stay_inside_the_clause() {
    // 字集外的字不當分隔：句子留著，之後轉碼失敗就整句丟掉，不會拼出原文沒有的相鄰字。
    assert_eq!(
        all("喺2004年10月，ATM提款，𠮶個"),
        ["喺2004年10月", "ATM提款", "𠮶個"]
    );
}

#[test]
fn leftover_markup_is_a_separator() {
    assert_eq!(all("壞咗{{模板|名=歐羅"), ["壞咗", "模板", "名", "歐羅"]);
}

#[test]
fn split_is_about_90_5_5_and_the_same_text_always_lands_in_the_same_split() {
    let texts: Vec<String> = (0..20_000).map(|i| format!("句{i}")).collect();
    let count = |split| texts.iter().filter(|t| split_of(t) == split).count();
    let (train, dev, test) = (count(Split::Train), count(Split::Dev), count(Split::Test));
    assert_eq!(train + dev + test, 20_000);
    assert!((17_600..=18_400).contains(&train), "train {train}");
    assert!((800..=1_200).contains(&dev), "dev {dev}");
    assert!((800..=1_200).contains(&test), "test {test}");
    assert!(texts.iter().all(|t| split_of(t) == split_of(&t.clone())));
}

/// 日、問各一個碼，夠拼出落在各切分的句子。
fn transcoder() -> Transcoder {
    let table = "\
日 日 1 1 0 0 1 0 0 0 0 a a NA 23061
問 问 1 1 0 0 1 0 0 0 0 anr anr NA 20001
";
    Transcoder::new(&libcangjie::parse(table).unwrap())
}

/// 由日、問拼成、落在 test 切分的前 `n` 句，短的在前。
fn test_clauses(n: usize) -> Vec<String> {
    (1..12)
        .flat_map(|len| {
            (0..1u32 << len).map(move |bits| {
                (0..len)
                    .map(|i| if bits >> i & 1 == 1 { '問' } else { '日' })
                    .collect::<String>()
            })
        })
        .filter(|s| split_of(s) == Split::Test)
        .take(n)
        .collect()
}

fn context_items(texts: &[String]) -> Vec<String> {
    let transcoder = transcoder();
    let mut builder = Builder::new(&transcoder, std::io::sink(), std::io::sink());
    for text in texts {
        builder.add(text).unwrap();
    }
    let finished = builder.finish().unwrap();
    finished.context_bench.lines().map(String::from).collect()
}

/// 「有前文」的題（plan §5.2）：test 切分的句子，前文是同一段文字裡它前面的兩句與中間的標點，
/// 原樣保留（含字集外的字）。
#[test]
fn a_test_clause_takes_the_two_clauses_before_it_as_context() {
    let [first, second] = test_clauses(2).try_into().unwrap();
    let codes = |s: &str| transcoder().transcode(s).unwrap().join(" ");
    let items = context_items(&[
        format!("開頭，ATM2004，佢話：{first}！"),
        format!("{first}，{second}"),
    ]);
    let mut expected = vec![
        format!("ATM2004，佢話：\t{}\t{first}", codes(&first)),
        format!("{first}，\t{}\t{second}", codes(&second)),
    ];
    expected.sort_by_key(|line| items.iter().position(|i| i == line));
    assert_eq!(items, expected);
}

/// 段落的第一句沒有前文：換行與 tab 之前的不算，題目檔也就沒有換行與 tab。
#[test]
fn context_does_not_reach_across_a_line_break() {
    let [first, second] = test_clauses(2).try_into().unwrap();
    let items = context_items(&[format!("前面\n{first}"), format!("前面\t{second}")]);
    assert!(items.is_empty(), "{items:?}");
}

/// 由日、問拼成、落在 dev 切分的前 `n` 句，短的在前。
fn dev_clauses(n: usize) -> Vec<String> {
    (1..12)
        .flat_map(|len| {
            (0..1u32 << len).map(move |bits| {
                (0..len)
                    .map(|i| if bits >> i & 1 == 1 { '問' } else { '日' })
                    .collect::<String>()
            })
        })
        .filter(|s| split_of(s) == Split::Dev)
        .take(n)
        .collect()
}

/// dev 切分也取「有前文」的題（調參用，plan §5.2），取法與格式同 test 的，兩邊不混。
#[test]
fn dev_clauses_get_context_items_of_their_own() {
    let [dev] = dev_clauses(1).try_into().unwrap();
    let [test] = test_clauses(1).try_into().unwrap();
    let transcoder = transcoder();
    let mut builder = Builder::new(&transcoder, std::io::sink(), std::io::sink());
    builder.add(&format!("前文，{dev}。{test}")).unwrap();
    let finished = builder.finish().unwrap();
    let codes = |s: &str| transcoder.transcode(s).unwrap().join(" ");
    assert_eq!(
        finished.dev_context_items,
        format!("前文，\t{}\t{dev}\n", codes(&dev))
    );
    assert_eq!(
        finished.context_bench,
        format!("前文，{dev}。\t{}\t{test}\n", codes(&test))
    );
}
