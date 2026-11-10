//! Swift 看到的介面（uniffi 匯出的 `Session`）在 Rust 這邊的行為：資料包位元組 → 按鍵 → 快照。
//! 只驗型別轉換接對了；排序與組字的行為由 `qim-core` 的測試守。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Lexicon, LexiconPack, Packs, Ranking, Scheme, SchemePack, Word,
};
use qim_ffi::{Session, Snapshot};

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
    }
}

/// `a` 日、`an` 門、`r` 口，詞庫有「門口」：`anr` 的首選是「門口」，`an` 那段單獨是「門」。
fn packs() -> (Vec<u8>, Vec<u8>) {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::from([('a', "日".into()), ('n', "弓".into()), ('r', "口".into())]),
        packs: Packs {
            lexicon: Some("quick-lexicon".into()),
            ..Packs::default()
        },
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries([
        ("a".to_string(), vec![entry('日')]),
        ("an".to_string(), vec![entry('門')]),
        ("r".to_string(), vec![entry('口')]),
    ]);
    let lexicon = LexiconPack {
        id: "quick-lexicon".into(),
        lexicon: Lexicon::new(
            [(
                "anr".to_string(),
                vec![Word {
                    text: "門口".into(),
                    frequency: 9435,
                }],
            )],
            [],
        ),
    };
    (
        SchemePack { scheme, code_table }.to_bytes().unwrap(),
        lexicon.to_bytes().unwrap(),
    )
}

fn session() -> std::sync::Arc<Session> {
    let (scheme, lexicon) = packs();
    Session::new(scheme, Some(lexicon), None, None, None).unwrap()
}

fn type_letters(session: &Session, letters: &str) -> Snapshot {
    let mut snapshot = None;
    for c in letters.chars() {
        snapshot = Some(session.press("letter".into(), Some(c.to_string())).unwrap());
    }
    snapshot.unwrap()
}

fn first(snapshot: &Snapshot) -> Option<&str> {
    snapshot.candidates.first().map(|c| c.text.as_str())
}

#[test]
fn typing_anr_puts_the_word_first_and_space_commits_it() {
    let session = session();
    let snapshot = type_letters(&session, "anr");
    assert!(snapshot.handled);
    assert_eq!(snapshot.input, "anr");
    assert_eq!(first(&snapshot), Some("門口"));
    assert_eq!(snapshot.candidates[0].original, "門口");
    assert_eq!(snapshot.commit, None);

    let snapshot = session.press("space".into(), None).unwrap();
    assert_eq!(snapshot.commit.as_deref(), Some("門口"));
    assert_eq!(snapshot.commit_original.as_deref(), Some("門口"));
    assert_eq!(snapshot.input, "");
}

#[test]
fn segments_carry_code_radicals_and_text() {
    let session = session();
    let snapshot = type_letters(&session, "an");
    assert_eq!(snapshot.segments.len(), 1);
    let segment = &snapshot.segments[0];
    assert_eq!(segment.code, "an");
    assert_eq!(segment.radicals, "日弓");
    assert_eq!(segment.text, "門");
    assert_eq!(segment.original, "門");
    assert!(!segment.selected);
    assert!(!segment.raw);
    assert_eq!(snapshot.focus, 1);
    assert_eq!(snapshot.page, 0);
    assert!(!snapshot.has_more);
    assert!(!snapshot.ascii);
}

#[test]
fn selecting_a_candidate_that_covers_the_first_segment_fixes_it() {
    let session = session();
    let snapshot = type_letters(&session, "anr");
    let index = snapshot
        .candidates
        .iter()
        .position(|c| c.text == "門")
        .expect("「門」在 anr 的候選裡");
    let snapshot = session.select(index as u32);
    assert_eq!(snapshot.commit, None);
    assert_eq!(snapshot.segments[0].text, "門");
    assert!(snapshot.segments[0].selected);
    assert_eq!(first(&snapshot), Some("口"));
}

#[test]
fn unknown_key_kind_is_an_error_with_the_core_message() {
    let session = session();
    let error = session.press("warp".into(), None).unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn a_broken_pack_is_an_error_not_a_panic() {
    assert!(Session::new(vec![1, 2, 3], None, None, None, None).is_err());
}

#[test]
fn a_key_the_core_passes_through_is_not_handled() {
    let session = session();
    let snapshot = session.press("enter".into(), None).unwrap();
    assert!(!snapshot.handled);
}
