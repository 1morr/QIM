use qim_databuild::corpus::{Split, clauses, split_of};

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
