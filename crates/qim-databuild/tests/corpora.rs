use qim_databuild::corpora::{CANTONESE_KEPT, CORPORA, Conversion, Converter, Corpus};

fn corpus(name: &str) -> &'static Corpus {
    CORPORA.iter().find(|c| c.name == name).unwrap()
}

/// 用 `common-voice-yue`（一行一句的純文字）讀一個暫存檔，交出前處理後的每一段。
fn common_voice_texts(lines: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sentence-collector.txt");
    std::fs::write(&path, lines).unwrap();
    let mut texts = Vec::new();
    corpus("common-voice-yue")
        .texts(&[path], |text| {
            texts.push(text.to_owned());
            Ok(())
        })
        .unwrap();
    texts
}

#[test]
fn colloquial_corpus_keeps_cantonese_characters_that_s2hk_would_replace() {
    // s2hk 會換成：曬、揾、嚇、鬱、餵、湧、鹹、響、僕（票 17b 的統計）。
    let sentence = "唔該晒，搵工，試吓，郁手，喂，東涌，咸濕，响度，仆街";
    assert_eq!(common_voice_texts(sentence), [sentence]);
}

#[test]
fn colloquial_corpus_still_gets_hong_kong_forms() {
    assert_eq!(common_voice_texts("佢說溫書"), ["佢説温書"]);
}

#[test]
fn every_kept_cantonese_character_survives_the_colloquial_conversion() {
    let text: String = CANTONESE_KEPT.iter().collect();
    assert_eq!(common_voice_texts(&text), [text]);
}

#[test]
fn simplified_text_is_still_converted_and_keeps_wan() {
    let s2hk = Converter::new(Conversion::S2hk).unwrap();
    assert_eq!(s2hk.convert("晒太阳，搵工"), "曬太陽，搵工");
}

#[test]
fn only_chinese_wikipedia_is_treated_as_simplified() {
    // 中文維基混有簡體；其餘三份原文已是繁體，s2hk 會把「晒」「吓」當簡體再轉一次。
    for c in &CORPORA {
        let expected = if c.name == "zh-wiki" {
            Conversion::S2hk
        } else {
            Conversion::T2hk
        };
        assert_eq!(c.conversion, expected, "{}", c.name);
    }
}

#[test]
fn wan_written_in_the_hong_kong_form_becomes_the_users_form() {
    // HKCanCor 原文就寫「揾」；兩字同碼，使用者打「搵」。
    assert_eq!(common_voice_texts("集體揾錢"), ["集體搵錢"]);
    let s2hk = Converter::new(Conversion::S2hk).unwrap();
    assert_eq!(s2hk.convert("揾工"), "搵工");
}
