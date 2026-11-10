//! 線上更新的 release 目錄（plan §4.7）：把 `build/packs/` 的每份資料包改名成不撞名的檔，加一份
//! `index.json`；給了上一版索引時，內容改了卻沒加版本的資料包讓它失敗。

use std::path::Path;

use qim_core::{
    EmojiPack, EmojiTable, IndexEntry, Lexicon, LexiconPack, MANIFEST_FILE, Manifest,
    PACK_INDEX_FILE, Pack, PackIndex, PackKind, Word,
};
use qim_databuild::release;

fn lexicon_bytes(word: &str) -> Vec<u8> {
    LexiconPack {
        id: "test-lexicon".into(),
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
    .to_bytes()
    .unwrap()
}

fn emoji_bytes() -> Vec<u8> {
    EmojiPack {
        id: "test-emoji".into(),
        emoji: EmojiTable::new([("笑", vec!["😄"])]),
    }
    .to_bytes()
    .unwrap()
}

/// 照 `cargo xtask data` 的放法寫一份資料包目錄：`<id>/` 裡有 `.pack`、manifest、LICENSE、NOTICE。
fn put(packs: &Path, kind: PackKind, id: &str, version: &str, bytes: &[u8]) -> Manifest {
    let dir = packs.join(id);
    std::fs::create_dir_all(&dir).unwrap();
    let manifest = Manifest::new(kind, id, bytes, version, "CC0-1.0");
    std::fs::write(dir.join(manifest.file_name()), bytes).unwrap();
    std::fs::write(dir.join(MANIFEST_FILE), manifest.to_json()).unwrap();
    std::fs::write(dir.join("LICENSE"), format!("{id} license")).unwrap();
    std::fs::write(dir.join("NOTICE"), format!("{id} notice")).unwrap();
    manifest
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn release_has_renamed_assets_and_an_index() {
    let tmp = tempfile::tempdir().unwrap();
    let packs = tmp.path().join("packs");
    let out = tmp.path().join("release");
    let lexicon = put(
        &packs,
        PackKind::Lexicon,
        "test-lexicon",
        "1.0.0",
        &lexicon_bytes("門口"),
    );
    let emoji = put(
        &packs,
        PackKind::Emoji,
        "test-emoji",
        "1.0.0",
        &emoji_bytes(),
    );
    // 上一次留下的檔不留在新的 release 裡。
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("stale.pack"), b"old").unwrap();

    let index = release::write(&packs, &out, None).unwrap();

    let expected = PackIndex::new(vec![
        IndexEntry {
            manifest: emoji.clone(),
            url: "test-emoji.pack".into(),
        },
        IndexEntry {
            manifest: lexicon.clone(),
            url: "test-lexicon.pack".into(),
        },
    ]);
    assert_eq!(index, expected);
    assert_eq!(
        PackIndex::from_json(&read(&out.join(PACK_INDEX_FILE))).unwrap(),
        expected
    );
    let mut names: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "index.json",
            "test-emoji.LICENSE",
            "test-emoji.NOTICE",
            "test-emoji.manifest.json",
            "test-emoji.pack",
            "test-lexicon.LICENSE",
            "test-lexicon.NOTICE",
            "test-lexicon.manifest.json",
            "test-lexicon.pack",
        ]
    );
    assert_eq!(
        read(&out.join("test-lexicon.LICENSE")),
        "test-lexicon license"
    );
    assert_eq!(read(&out.join("test-emoji.NOTICE")), "test-emoji notice");
    assert_eq!(
        Manifest::from_json(&read(&out.join("test-lexicon.manifest.json"))).unwrap(),
        lexicon
    );
    assert_eq!(
        std::fs::read(out.join("test-lexicon.pack")).unwrap(),
        lexicon_bytes("門口")
    );
}

#[test]
fn changed_pack_without_a_new_version_fails_against_the_previous_index() {
    let tmp = tempfile::tempdir().unwrap();
    let packs = tmp.path().join("packs");
    let out = tmp.path().join("release");
    put(
        &packs,
        PackKind::Lexicon,
        "test-lexicon",
        "1.0.0",
        &lexicon_bytes("門口"),
    );
    let previous = release::write(&packs, &out, None).unwrap();

    // 內容改了、版本沒加：失敗，舊的 release 目錄不動。
    put(
        &packs,
        PackKind::Lexicon,
        "test-lexicon",
        "1.0.0",
        &lexicon_bytes("問口"),
    );
    let error = release::write(&packs, &out, Some(&previous)).unwrap_err();
    assert!(error.contains("test-lexicon"), "{error}");
    assert_eq!(
        PackIndex::from_json(&read(&out.join(PACK_INDEX_FILE))).unwrap(),
        previous
    );

    // 加了版本就過。
    put(
        &packs,
        PackKind::Lexicon,
        "test-lexicon",
        "1.0.1",
        &lexicon_bytes("問口"),
    );
    let index = release::write(&packs, &out, Some(&previous)).unwrap();
    assert_eq!(index.packs[0].manifest.version, "1.0.1");
}

#[test]
fn pack_that_does_not_match_its_manifest_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let packs = tmp.path().join("packs");
    put(
        &packs,
        PackKind::Lexicon,
        "test-lexicon",
        "1.0.0",
        &lexicon_bytes("門口"),
    );
    std::fs::write(
        packs.join("test-lexicon").join("test-lexicon.pack"),
        lexicon_bytes("問口"),
    )
    .unwrap();
    let error = release::write(&packs, &tmp.path().join("release"), None).unwrap_err();
    assert!(error.contains("test-lexicon"), "{error}");
    assert!(error.contains("sha256"), "{error}");
}

#[test]
fn pack_without_license_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let packs = tmp.path().join("packs");
    put(
        &packs,
        PackKind::Emoji,
        "test-emoji",
        "1.0.0",
        &emoji_bytes(),
    );
    std::fs::remove_file(packs.join("test-emoji").join("LICENSE")).unwrap();
    let error = release::write(&packs, &tmp.path().join("release"), None).unwrap_err();
    assert!(error.contains("LICENSE"), "{error}");
}

/// e2e 扮演 GitHub release 的 `web/e2e/fixtures/pack-release/`（plan §4.7）：照 `cargo xtask pack-release`
/// 的路寫出 `e2e-update-lexicon` 1.1.0（`anr` 是「新版詞庫」；舊版 1.0.0 在
/// `crates/qim-core/tests/web_fixture.rs`）。檔案進 git，格式改了會紅；
/// `QIM_UPDATE_FIXTURES=1 cargo test -p qim-databuild --test release` 重產。
#[test]
fn web_release_fixture_is_current() {
    const ID: &str = "e2e-update-lexicon";
    let tmp = tempfile::tempdir().unwrap();
    let packs = tmp.path().join("packs");
    let out = tmp.path().join("release");
    let bytes = LexiconPack {
        id: ID.into(),
        lexicon: Lexicon::new(
            [(
                "anr".to_string(),
                vec![Word {
                    text: "新版詞庫".into(),
                    frequency: 1000,
                }],
            )],
            [],
        ),
    }
    .to_bytes()
    .unwrap();
    put(&packs, PackKind::Lexicon, ID, "1.1.0", &bytes);
    release::write(&packs, &out, None).unwrap();

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/e2e/fixtures/pack-release");
    if std::env::var_os("QIM_UPDATE_FIXTURES").is_some() {
        if fixture.exists() {
            std::fs::remove_dir_all(&fixture).unwrap();
        }
        std::fs::create_dir_all(&fixture).unwrap();
        for entry in std::fs::read_dir(&out).unwrap() {
            let path = entry.unwrap().path();
            std::fs::copy(&path, fixture.join(path.file_name().unwrap())).unwrap();
        }
    }
    let listing = |dir: &Path| {
        let mut files: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .map(|e| {
                        let path = e.unwrap().path();
                        let name = path.file_name().unwrap().to_string_lossy().into_owned();
                        (name, std::fs::read(&path).unwrap())
                    })
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        files
    };
    assert!(
        listing(&fixture) == listing(&out),
        "{} 和 release::write 寫出的不同：QIM_UPDATE_FIXTURES=1 cargo test -p qim-databuild --test release",
        fixture.display()
    );
}
