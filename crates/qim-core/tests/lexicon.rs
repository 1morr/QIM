//! 詞庫資料包經公開介面的行為：兩份資料包位元組 → `Engine` → `Session::handle` → `Snapshot`。

use std::collections::BTreeMap;

use qim_core::{
    CodeEntry, CodeTable, Engine, Key, Lexicon, LexiconPack, Pack, PackError, Packs, Ranking,
    Scheme, Session, Snapshot, UserModel, Word,
};

fn entry(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: false,
    }
}

fn extension(text: char) -> CodeEntry {
    CodeEntry {
        text,
        extension: true,
    }
}

fn scheme(lexicon: Option<&str>) -> Scheme {
    Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "abcdefghijklmnopqrstuvwxy".into(),
        max_code_length: 2,
        radicals: BTreeMap::new(),
        packs: Packs {
            lexicon: lexicon.map(Into::into),
            language_model: None,
        },
        ranking: Ranking::default(),
    }
}

/// 碼表的字序是建置端給的「年級 → Big5」：`a` 是日曰旦，`ar` 是問、擴展區的𠀀。
fn pack(lexicon: Option<&str>) -> Pack {
    Pack {
        scheme: scheme(lexicon),
        code_table: CodeTable::from_entries([
            ("a".to_string(), "日曰旦".chars().map(entry).collect()),
            ("an".to_string(), vec![entry('門')]),
            ("r".to_string(), vec![entry('口')]),
            ("ar".to_string(), vec![entry('問'), extension('𠀀')]),
        ]),
    }
}

fn word(text: &str, frequency: u64) -> Word {
    Word {
        text: text.into(),
        frequency,
    }
}

fn lexicon_pack(id: &str, char_frequency: &[(char, u64)]) -> LexiconPack {
    LexiconPack {
        id: id.into(),
        lexicon: Lexicon::new(
            [
                ("anr".to_string(), vec![word("門口", 9435), word("明叩", 3)]),
                ("ar".to_string(), vec![word("日口", 120)]),
            ],
            char_frequency.iter().copied(),
        ),
    }
}

fn engine(char_frequency: &[(char, u64)]) -> Engine {
    Engine::from_pack_bytes(
        &pack(Some("quick-lexicon")).to_bytes().unwrap(),
        Some(
            &lexicon_pack("quick-lexicon", char_frequency)
                .to_bytes()
                .unwrap(),
        ),
        None,
    )
    .unwrap()
}

fn type_input(engine: &Engine, input: &str) -> Snapshot {
    let mut user = UserModel::default();
    let mut session = Session::default();
    let mut snapshot = Snapshot::default();
    for c in input.chars() {
        snapshot = session.handle(engine, &mut user, Key::Letter(c));
    }
    snapshot
}

fn texts(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .candidates
        .iter()
        .map(|c| c.text.as_str())
        .collect()
}

#[test]
fn lexicon_pack_round_trips_through_bytes() {
    let pack = lexicon_pack("quick-lexicon", &[('日', 5)]);
    assert_eq!(
        LexiconPack::from_bytes(&pack.to_bytes().unwrap()).unwrap(),
        pack
    );
}

#[test]
fn scheme_pack_and_lexicon_pack_are_not_mistaken_for_each_other() {
    let lexicon = lexicon_pack("quick-lexicon", &[]).to_bytes().unwrap();
    let scheme = pack(None).to_bytes().unwrap();
    assert_eq!(Pack::from_bytes(&lexicon), Err(PackError::NotAPack));
    assert_eq!(LexiconPack::from_bytes(&scheme), Err(PackError::NotAPack));
}

#[test]
fn whole_input_longer_than_a_character_code_lists_its_words_then_the_characters_of_its_beginning() {
    // 「門」+「口」拼出的句子和詞「門口」同文字，只列一次。
    assert_eq!(
        texts(&type_input(&engine(&[]), "anr")),
        ["門口", "明叩", "門", "日", "曰", "旦"]
    );
}

#[test]
fn characters_with_a_higher_frequency_come_first() {
    let engine = engine(&[('旦', 30), ('曰', 7)]);
    assert_eq!(texts(&type_input(&engine, "a")), ["旦", "曰", "日"]);
}

#[test]
fn characters_without_a_frequency_keep_the_code_table_order() {
    // 碼表的順序由建置端依年級、再依 Big5 排好（qim-databuild 的 quick 測試）。
    assert_eq!(texts(&type_input(&engine(&[]), "a")), ["日", "曰", "旦"]);
    assert_eq!(
        texts(&type_input(&engine(&[('曰', 1)]), "a")),
        ["曰", "日", "旦"]
    );
}

#[test]
fn extension_characters_follow_everything_covering_the_whole_input() {
    // 擴展區的字頻再高也不進 lattice、不和整串的候選比分數（plan §2.3）。
    assert_eq!(
        texts(&type_input(&engine(&[('𠀀', 1000)]), "ar")),
        ["問", "日口", "曰口", "旦口", "𠀀", "日", "曰", "旦"]
    );
}

#[test]
fn space_commits_a_word() {
    let engine = engine(&[]);
    let mut user = UserModel::default();
    let mut session = Session::default();
    for c in "anr".chars() {
        session.handle(&engine, &mut user, Key::Letter(c));
    }
    let snapshot = session.handle(&engine, &mut user, Key::Space);
    assert_eq!(snapshot.commit.as_deref(), Some("門口"));
    assert_eq!(snapshot.composition.input, "");
}

#[test]
fn lexicon_must_be_the_one_the_scheme_names() {
    let scheme_pack = pack(Some("quick-lexicon")).to_bytes().unwrap();
    let other = lexicon_pack("cangjie-lexicon", &[]).to_bytes().unwrap();
    assert_eq!(
        Engine::from_pack_bytes(&scheme_pack, Some(&other), None).err(),
        Some(PackError::LexiconMismatch {
            expected: Some("quick-lexicon".into()),
            found: Some("cangjie-lexicon".into()),
        })
    );
    assert_eq!(
        Engine::from_pack_bytes(&scheme_pack, None, None).err(),
        Some(PackError::LexiconMismatch {
            expected: Some("quick-lexicon".into()),
            found: None,
        })
    );
    let lexicon = lexicon_pack("quick-lexicon", &[]).to_bytes().unwrap();
    assert_eq!(
        Engine::from_pack_bytes(&pack(None).to_bytes().unwrap(), Some(&lexicon), None).err(),
        Some(PackError::LexiconMismatch {
            expected: None,
            found: Some("quick-lexicon".into()),
        })
    );
}
