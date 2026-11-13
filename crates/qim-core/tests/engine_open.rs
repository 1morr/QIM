//! `Engine::open`：殼交一組 manifest 與資料包檔，核心依各自的種類組起來（plan〈資料包 manifest〉）。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Manifest, ModelPack, Pack, PackError,
    PackKind, Packs, RadicalHintPack, RadicalHints, Ranking, Scheme, SchemePack, Session,
    UserModel, Word,
};

fn scheme_pack() -> SchemePack {
    let entry = |text| CodeEntry {
        text,
        extension: false,
        simplified: false,
    };
    SchemePack {
        scheme: Scheme {
            id: "quick".into(),
            name: "速成".into(),
            version: "1".into(),
            alphabet: "anr".into(),
            max_code_length: 2,
            wildcard: None,
            radicals: BTreeMap::new(),
            packs: Packs {
                lexicon: Some("quick-lexicon".into()),
                ..Packs::default()
            },
            ranking: Ranking::default(),
        },
        code_table: CodeTable::from_entries([
            ("an".to_string(), vec![entry('門')]),
            ("r".to_string(), vec![entry('口')]),
        ]),
    }
}

fn lexicon(id: &str, word: &str) -> LexiconPack {
    LexiconPack {
        id: id.into(),
        lexicon: Lexicon::new(
            [(
                "anr".to_string(),
                vec![Word {
                    text: word.into(),
                    frequency: 10,
                }],
            )],
            [],
        ),
    }
}

/// 編成位元組，配上它的 manifest。
fn file<P: Pack>(pack: &P) -> (Manifest, Vec<u8>) {
    let bytes = pack.to_bytes().unwrap();
    (
        Manifest::new(P::KIND, pack.id(), &bytes, "1.0.0", "CC0-1.0"),
        bytes,
    )
}

fn open(files: &[(Manifest, Vec<u8>)]) -> Result<Engine, PackError> {
    Engine::open(
        &files
            .iter()
            .map(|(m, b)| (m, b.as_slice()))
            .collect::<Vec<_>>(),
    )
}

fn top(engine: &Engine) -> String {
    let mut session = Session::default();
    let mut user = UserModel::default();
    let mut snapshot = None;
    for c in "anr".chars() {
        snapshot = Some(session.handle(engine, &mut user, Key::Letter(c)));
    }
    snapshot.unwrap().candidates[0].text.clone()
}

#[test]
fn packs_are_placed_by_their_kind_in_any_order() {
    let engine = open(&[
        file(&lexicon("other-lexicon", "問號")),
        file(&scheme_pack()),
    ])
    .unwrap();
    assert_eq!(top(&engine), "問號");
}

#[test]
fn two_packs_of_one_kind_are_rejected() {
    assert_eq!(
        open(&[
            file(&scheme_pack()),
            file(&lexicon("a", "問號")),
            file(&lexicon("b", "門口")),
        ])
        .err(),
        Some(PackError::DuplicateKind(PackKind::Lexicon))
    );
}

#[test]
fn radical_hints_are_not_for_the_engine() {
    let hints = RadicalHintPack {
        id: "cangjie-hint".into(),
        hints: RadicalHints::default(),
    };
    assert_eq!(
        open(&[file(&scheme_pack()), file(&lexicon("a", "x")), file(&hints)]).err(),
        Some(PackError::NotForEngine(PackKind::RadicalHint))
    );
}

#[test]
fn model_packs_are_not_for_the_engine() {
    // 停鍵層的模型由殼交給 qim-neural（plan〈模型分層〉），不經 Engine。
    let model = ModelPack {
        id: "zh-hant-neural-rerank".into(),
        vocab: vec!['我'],
        unknown_chars: 1,
        lambda: 2.0,
        weights: vec![1, 2, 3],
    };
    assert_eq!(
        open(&[file(&scheme_pack()), file(&lexicon("a", "x")), file(&model)]).err(),
        Some(PackError::NotForEngine(PackKind::Model))
    );
}

#[test]
fn a_scheme_pack_is_required() {
    assert_eq!(
        open(&[file(&lexicon("a", "x"))]).err(),
        Some(PackError::Missing(PackKind::Scheme))
    );
}

#[test]
fn every_pack_is_checked_against_its_manifest() {
    let mut lexicon = file(&lexicon("a", "x"));
    lexicon.0.sha256 = "0".repeat(64);
    assert!(matches!(
        open(&[file(&scheme_pack()), lexicon]),
        Err(PackError::ChecksumMismatch { .. })
    ));
}
