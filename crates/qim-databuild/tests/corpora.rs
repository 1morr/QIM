use qim_core::{CodeEntry, CodeTable};
use qim_databuild::common_forms::CommonForms;
use qim_databuild::corpora::{CANTONESE_KEPT, CORPORA, Conversion, Converter, Corpus};

/// 只含「搵揾」「溫温」「群羣」與「說」的碼表建出的對照表；「説」不在碼表（不在連打字集）。
fn forms() -> CommonForms {
    let entry = |text| CodeEntry {
        text,
        extension: false,
    };
    let table = CodeTable::from_entries([
        ("qt".to_string(), vec![entry('搵'), entry('揾')]),
        ("et".into(), vec![entry('溫'), entry('温')]),
        ("sq".into(), vec![entry('群'), entry('羣')]),
        ("yu".into(), vec![entry('說')]),
    ]);
    CommonForms::from_opencc(&table).unwrap()
}

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
        .texts(&[path], &forms(), |text| {
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
fn colloquial_corpus_keeps_common_forms_instead_of_hong_kong_forms() {
    // t2hk 換成「佢説温書」，對照表換回常見寫法。
    assert_eq!(common_voice_texts("佢說溫書"), ["佢說溫書"]);
}

#[test]
fn every_kept_cantonese_character_survives_the_colloquial_conversion() {
    let text: String = CANTONESE_KEPT.iter().collect();
    assert_eq!(common_voice_texts(&text), [text]);
}

#[test]
fn simplified_text_is_still_converted_and_written_in_common_forms() {
    let s2hk = Converter::new(Conversion::S2hk, forms()).unwrap();
    // s2hk 的簡轉繁出「羣」、香港寫法出「揾」，對照表換回「群」「搵」。
    assert_eq!(s2hk.convert("晒太阳，搵工，人群"), "曬太陽，搵工，人群");
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
fn wan_written_in_the_hong_kong_form_becomes_the_common_form() {
    // HKCanCor 原文就寫「揾」；兩字同碼，使用者打「搵」。
    assert_eq!(common_voice_texts("集體揾錢"), ["集體搵錢"]);
    let s2hk = Converter::new(Conversion::S2hk, forms()).unwrap();
    assert_eq!(s2hk.convert("揾工"), "搵工");
}

/// `forms()` 的碼表再加上單向並存寫法的字：着（`tu`）著（`ta`）、臺（`gg`）台（`ir`）、菸（`ty`）煙（`fg`）。
fn forms_with_one_way() -> CommonForms {
    let entry = |text| CodeEntry {
        text,
        extension: false,
    };
    let table = CodeTable::from_entries([
        ("tu".to_string(), vec![entry('着')]),
        ("ta".into(), vec![entry('著')]),
        ("gg".into(), vec![entry('臺')]),
        ("ir".into(), vec![entry('台')]),
        ("ty".into(), vec![entry('菸')]),
        ("fg".into(), vec![entry('煙')]),
    ]);
    CommonForms::from_opencc(&table).unwrap()
}

#[test]
fn one_way_forms_stay_as_the_source_wrote_them() {
    // t2hk 把「臺」「菸」換成「台」「煙」：單向並存寫法照原文留下，原文寫「台」「煙」的也照舊，
    // 兩種寫法的字頻與 bigram 各算各的（M1 票 17d）。
    let t2hk = Converter::new(Conversion::T2hk, forms_with_one_way()).unwrap();
    assert_eq!(t2hk.convert("臺灣香菸，台州煙霧"), "臺灣香菸，台州煙霧");
    // 簡體原文沒有「臺」：轉出來的照 s2hk。
    let s2hk = Converter::new(Conversion::S2hk, forms_with_one_way()).unwrap();
    assert_eq!(s2hk.convert("台湾的舞臺"), "台灣的舞臺");
}
