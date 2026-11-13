//! 資料包 manifest（plan〈資料包 manifest〉）：載入時核對最低核心版本、大小、sha256、種類與 id。

use qim_core::{
    CORE_VERSION, EmojiPack, EmojiTable, Lexicon, LexiconPack, MANIFEST_VERSION, Manifest,
    ModelPack, Pack, PackError, PackKind, Word,
};

fn lexicon_pack() -> LexiconPack {
    LexiconPack {
        id: "test-lexicon".into(),
        lexicon: Lexicon::new(
            [(
                "anr".to_string(),
                vec![Word {
                    text: "門口".into(),
                    frequency: 9435,
                }],
            )],
            [('門', 3)],
        ),
    }
}

fn manifest_of(bytes: &[u8]) -> Manifest {
    Manifest::new(PackKind::Lexicon, "test-lexicon", bytes, "1.2.0", "CC0-1.0")
}

#[test]
fn manifest_describes_the_pack_and_opens_it() {
    let pack = lexicon_pack();
    let bytes = pack.to_bytes().unwrap();
    let manifest = manifest_of(&bytes);
    assert_eq!(manifest.id, "test-lexicon");
    assert_eq!(manifest.kind, PackKind::Lexicon);
    assert_eq!(manifest.version, "1.2.0");
    assert_eq!(manifest.min_core_version, CORE_VERSION);
    assert_eq!(manifest.manifest_version, MANIFEST_VERSION);
    assert_eq!(manifest.file_name(), "test-lexicon.pack");
    assert_eq!(manifest.size, bytes.len() as u64);
    assert_eq!(manifest.license, "CC0-1.0");
    // sha256 of the bytes, as `shasum -a 256` prints it.
    assert_eq!(manifest.sha256.len(), 64);
    assert!(
        manifest
            .sha256
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );

    let reread = Manifest::from_json(&manifest.to_json()).unwrap();
    assert_eq!(reread, manifest);
    assert_eq!(LexiconPack::open(&reread, &bytes).unwrap(), pack);
}

#[test]
fn known_sha256_is_written_in_lowercase_hex() {
    // `printf abc | shasum -a 256`
    let manifest = Manifest::new(
        PackKind::Lexicon,
        "test-lexicon",
        b"abc",
        "1.0.0",
        "CC0-1.0",
    );
    assert_eq!(
        manifest.sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn changed_bytes_are_rejected_by_sha256() {
    let mut bytes = lexicon_pack().to_bytes().unwrap();
    let manifest = manifest_of(&bytes);
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    assert!(matches!(
        LexiconPack::open(&manifest, &bytes),
        Err(PackError::ChecksumMismatch { .. })
    ));
}

#[test]
fn edited_sha256_in_the_manifest_is_rejected() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let mut manifest = manifest_of(&bytes);
    manifest.sha256 = "0".repeat(64);
    let error = LexiconPack::open(&manifest, &bytes).unwrap_err();
    assert!(matches!(error, PackError::ChecksumMismatch { .. }));
    assert!(error.to_string().contains("sha256"), "{error}");
}

#[test]
fn truncated_bytes_are_rejected_by_size_first() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let manifest = manifest_of(&bytes);
    assert_eq!(
        LexiconPack::open(&manifest, &bytes[..bytes.len() - 1]),
        Err(PackError::SizeMismatch {
            expected: bytes.len() as u64,
            found: bytes.len() as u64 - 1,
        })
    );
}

#[test]
fn pack_needing_a_newer_core_is_rejected() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let mut manifest = manifest_of(&bytes);
    manifest.min_core_version = "999.0.0".into();
    let error = LexiconPack::open(&manifest, &bytes).unwrap_err();
    assert_eq!(
        error,
        PackError::CoreTooOld {
            required: "999.0.0".into(),
            current: CORE_VERSION.into(),
        }
    );
    assert!(error.to_string().contains("999.0.0"), "{error}");
}

#[test]
fn pack_for_this_or_an_older_core_is_accepted() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let mut manifest = manifest_of(&bytes);
    manifest.min_core_version = "0.0.0".into();
    assert!(LexiconPack::open(&manifest, &bytes).is_ok());
}

#[test]
fn malformed_min_core_version_is_rejected() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let mut manifest = manifest_of(&bytes);
    manifest.min_core_version = "latest".into();
    assert!(matches!(
        LexiconPack::open(&manifest, &bytes),
        Err(PackError::InvalidManifest(_))
    ));
}

#[test]
fn manifest_of_another_kind_is_rejected() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let manifest = manifest_of(&bytes);
    assert_eq!(
        EmojiPack::open(&manifest, &bytes),
        Err(PackError::KindMismatch {
            expected: PackKind::Emoji,
            found: PackKind::Lexicon,
        })
    );
}

#[test]
fn manifest_id_must_match_the_pack() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let mut manifest = manifest_of(&bytes);
    manifest.id = "other".into();
    assert_eq!(
        LexiconPack::open(&manifest, &bytes),
        Err(PackError::IdMismatch {
            manifest: "other".into(),
            pack: "test-lexicon".into(),
        })
    );
}

#[test]
fn kinds_are_written_in_kebab_case() {
    let pack = EmojiPack {
        id: "e".into(),
        emoji: EmojiTable::default(),
    };
    let bytes = pack.to_bytes().unwrap();
    let json = Manifest::new(PackKind::Emoji, "e", &bytes, "1.0.0", "Unicode-3.0").to_json();
    assert!(json.contains(r#""kind": "emoji""#), "{json}");
    let lexicon = manifest_of(b"x").to_json();
    assert!(lexicon.contains(r#""kind": "lexicon""#), "{lexicon}");
}

#[test]
fn unknown_fields_are_ignored_and_missing_ones_rejected() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    let json = manifest_of(&bytes).to_json();
    let extended = json.replacen('{', r#"{ "homepage": "https://example.com","#, 1);
    assert_eq!(Manifest::from_json(&extended).unwrap(), manifest_of(&bytes));
    let missing = json.replace(r#""license": "CC0-1.0""#, r#""x": 1"#);
    assert!(matches!(
        Manifest::from_json(&missing),
        Err(PackError::InvalidManifest(_))
    ));
}

/// manifest 本身的欄位不合格式時拒絕，不去讀資料包。
#[test]
fn malformed_fields_are_rejected() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    type Edit = fn(&mut Manifest);
    let cases: [(&str, Edit); 4] = [
        ("newer manifest_version", |m| {
            m.manifest_version = MANIFEST_VERSION + 1
        }),
        ("version not semver", |m| m.version = "2026-11-11".into()),
        ("id with a path", |m| m.id = "../quick".into()),
        ("empty id", |m| m.id = String::new()),
    ];
    for (case, edit) in cases {
        let mut manifest = manifest_of(&bytes);
        edit(&mut manifest);
        assert!(
            matches!(
                LexiconPack::open(&manifest, &bytes),
                Err(PackError::InvalidManifest(_))
            ),
            "{case}"
        );
    }
}

/// sha256 不是 64 位小寫十六進位時說 manifest 不合格式，不說檔案損壞。
#[test]
fn sha256_in_another_format_is_an_invalid_manifest() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    for sha256 in [
        manifest_of(&bytes).sha256.to_uppercase(),
        format!("sha256:{}", manifest_of(&bytes).sha256),
        "abc".into(),
    ] {
        let mut manifest = manifest_of(&bytes);
        manifest.sha256 = sha256.clone();
        assert!(
            matches!(
                LexiconPack::open(&manifest, &bytes),
                Err(PackError::InvalidManifest(_))
            ),
            "{sha256}"
        );
    }
}

#[test]
fn kind_names_match_the_manifest_json() {
    for kind in [
        PackKind::Scheme,
        PackKind::Lexicon,
        PackKind::LanguageModel,
        PackKind::WordTrigram,
        PackKind::RadicalHint,
        PackKind::Simplifier,
        PackKind::English,
        PackKind::Emoji,
        PackKind::Model,
    ] {
        let json = Manifest::new(kind, "x", b"", "1.0.0", "CC0-1.0").to_json();
        assert!(
            json.contains(&format!(r#""kind": "{}""#, kind.as_str())),
            "{json}"
        );
    }
}

/// 不知道種類時照 manifest 核對（匯入前檢查、建置端讀回）。
#[test]
fn verify_checks_the_pack_of_the_kind_the_manifest_names() {
    let bytes = lexicon_pack().to_bytes().unwrap();
    assert_eq!(manifest_of(&bytes).verify(&bytes), Ok(()));
    let mut emoji = manifest_of(&bytes);
    emoji.kind = PackKind::Emoji;
    // 種類對，內容卻是別種：魔數不符。
    assert_eq!(emoji.verify(&bytes), Err(PackError::NotAPack));
    let mut tampered = manifest_of(&bytes);
    tampered.sha256 = "0".repeat(64);
    assert!(matches!(
        tampered.verify(&bytes),
        Err(PackError::ChecksumMismatch { .. })
    ));
}

#[test]
fn newer_compares_versions_as_semver() {
    let at =
        |version: &str| Manifest::new(PackKind::Lexicon, "test-lexicon", b"x", version, "CC0-1.0");
    assert!(at("1.0.2").is_newer_than(&at("1.0.1")));
    assert!(!at("1.0.1").is_newer_than(&at("1.0.2")));
    // 同版不算較新（殼在同版時用內建的）。
    assert!(!at("1.0.1").is_newer_than(&at("1.0.1")));
    // semver，不是字串："1.10.0" > "1.9.0"；預發佈版比正式版舊。
    assert!(at("1.10.0").is_newer_than(&at("1.9.0")));
    assert!(at("1.0.0").is_newer_than(&at("1.0.0-beta.1")));
    // 讀不懂的版本兩邊都不算較新。
    assert!(!at("one").is_newer_than(&at("1.0.0")));
    assert!(!at("1.0.0").is_newer_than(&at("one")));
}

/// 停鍵層的模型資料包：核心只核對與解開，權重原樣交給殼（再交給 qim-neural）。
#[test]
fn model_packs_round_trip_and_open_by_manifest() {
    let pack = ModelPack {
        id: "zh-hant-neural-rerank".into(),
        vocab: vec!['的', '我', '𠮩'],
        unknown_chars: 966,
        lambda: 2.0,
        weights: (0..=255).collect(),
    };
    let bytes = pack.to_bytes().unwrap();
    let manifest = Manifest::new(
        PackKind::Model,
        "zh-hant-neural-rerank",
        &bytes,
        "1.0.0",
        "CC-BY-SA-4.0",
    );
    assert!(manifest.to_json().contains(r#""kind": "model""#));
    assert_eq!(ModelPack::open(&manifest, &bytes), Ok(pack));
    assert_eq!(manifest.verify(&bytes), Ok(()));
    assert_eq!(
        LexiconPack::open(&manifest, &bytes),
        Err(PackError::KindMismatch {
            expected: PackKind::Lexicon,
            found: PackKind::Model,
        })
    );
}
