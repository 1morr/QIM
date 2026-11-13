//! 更新索引（plan〈線上更新〉）：release 上的 `index.json` 列出各資料包的 manifest 與下載位置；殼拿它和
//! 裝著的比出要下載的，建置端拿上一版比出內容改了卻沒加版本的資料包。

use qim_core::{IndexEntry, Manifest, PackError, PackIndex, PackKind};

fn manifest(kind: PackKind, id: &str, version: &str, content: &[u8]) -> Manifest {
    Manifest::new(kind, id, content, version, "CC0-1.0")
}

fn entry(manifest: Manifest) -> IndexEntry {
    let url = manifest.file_name();
    IndexEntry { manifest, url }
}

fn ids(entries: &[&IndexEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|e| format!("{} {}", e.manifest.id, e.manifest.version))
        .collect()
}

#[test]
fn index_entries_are_manifests_with_a_url() {
    let lexicon = manifest(PackKind::Lexicon, "quick-lexicon", "1.1.0", b"new");
    let index = PackIndex::new(vec![entry(lexicon.clone())]);
    let json = index.to_json();
    assert_eq!(PackIndex::from_json(&json).unwrap(), index);

    // 每項就是 manifest 的欄位加 `url`：殼可以把一項原樣當 manifest 交給核心（不認得的欄位略過）。
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["index_version"], 1);
    let first = &value["packs"][0];
    assert_eq!(first["url"], "quick-lexicon.pack");
    assert_eq!(first["sha256"], lexicon.sha256.as_str());
    assert_eq!(Manifest::from_json(&first.to_string()).unwrap(), lexicon);
}

#[test]
fn updates_are_newer_versions_of_installed_packs() {
    let installed = [
        manifest(PackKind::Lexicon, "quick-lexicon", "1.0.0", b"old"),
        manifest(PackKind::LanguageModel, "zh-hant-bigram", "1.0.0", b"lm"),
        manifest(PackKind::Emoji, "emoji", "1.9.0", b"emoji"),
    ];
    let index = PackIndex::new(vec![
        entry(manifest(
            PackKind::Lexicon,
            "quick-lexicon",
            "1.1.0",
            b"new",
        )),
        // 同版本：不是更新。
        entry(manifest(
            PackKind::LanguageModel,
            "zh-hant-bigram",
            "1.0.0",
            b"lm",
        )),
        // 比 semver，不比字串："1.10.0" > "1.9.0"。
        entry(manifest(PackKind::Emoji, "emoji", "1.10.0", b"emoji2")),
        // 沒裝的不下載。
        entry(manifest(PackKind::English, "scowl-en-us", "2.0.0", b"en")),
    ]);
    assert_eq!(
        ids(&index.updates(&installed)),
        ["quick-lexicon 1.1.0", "emoji 1.10.0"]
    );
}

#[test]
fn older_other_kind_or_too_new_for_this_core_is_not_an_update() {
    let installed = [
        manifest(PackKind::Lexicon, "quick-lexicon", "1.2.0", b"old"),
        manifest(PackKind::Scheme, "quick", "1.0.0", b"scheme"),
        manifest(PackKind::Emoji, "emoji", "1.0.0", b"emoji"),
    ];
    let mut too_new = manifest(PackKind::Emoji, "emoji", "2.0.0", b"emoji2");
    too_new.min_core_version = "999.0.0".into();
    let index = PackIndex::new(vec![
        entry(manifest(
            PackKind::Lexicon,
            "quick-lexicon",
            "1.1.0",
            b"older",
        )),
        entry(manifest(PackKind::Lexicon, "quick", "2.0.0", b"lexicon")),
        entry(too_new),
    ]);
    assert!(index.updates(&installed).is_empty());
}

#[test]
fn malformed_index_is_rejected() {
    let good = PackIndex::new(vec![entry(manifest(
        PackKind::Lexicon,
        "quick-lexicon",
        "1.0.0",
        b"x",
    ))])
    .to_json();
    let cases = [
        ("不是 JSON", "nope".to_string()),
        (
            "index_version 2",
            good.replace("\"index_version\": 1", "\"index_version\": 2"),
        ),
        ("少了 url", good.replace("\"url\"", "\"link\"")),
        (
            "sha256 不合格式",
            good.replace("\"sha256\": \"", "\"sha256\": \"XYZ"),
        ),
        (
            "version 不是 semver",
            good.replace("\"version\": \"1.0.0\"", "\"version\": \"one\""),
        ),
    ];
    for (name, json) in cases {
        assert_ne!(json, good, "{name}：沒改到");
        assert!(
            matches!(PackIndex::from_json(&json), Err(PackError::InvalidIndex(_))),
            "{name}"
        );
    }

    let a = manifest(PackKind::Lexicon, "quick-lexicon", "1.0.0", b"a");
    let b = manifest(PackKind::Lexicon, "quick-lexicon", "1.1.0", b"b");
    let duplicate = PackIndex::new(vec![entry(a), entry(b)]).to_json();
    let error = PackIndex::from_json(&duplicate).unwrap_err();
    assert!(error.to_string().contains("quick-lexicon"), "{error}");
}

#[test]
fn changed_content_needs_a_bigger_version() {
    let previous = PackIndex::new(vec![
        entry(manifest(
            PackKind::Lexicon,
            "quick-lexicon",
            "1.0.0",
            b"old",
        )),
        entry(manifest(PackKind::Emoji, "emoji", "1.0.0", b"emoji")),
        entry(manifest(PackKind::English, "removed", "1.0.0", b"en")),
    ]);

    let bumped = PackIndex::new(vec![
        entry(manifest(
            PackKind::Lexicon,
            "quick-lexicon",
            "1.1.0",
            b"new",
        )),
        entry(manifest(PackKind::Emoji, "emoji", "1.0.0", b"emoji")),
        entry(manifest(PackKind::Scheme, "added", "1.0.0", b"scheme")),
    ]);
    assert_eq!(bumped.check_successor(&previous), Ok(()));

    let forgot = PackIndex::new(vec![entry(manifest(
        PackKind::Lexicon,
        "quick-lexicon",
        "1.0.0",
        b"new",
    ))]);
    let problems = forgot.check_successor(&previous).unwrap_err();
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("quick-lexicon"), "{problems:?}");
    assert!(problems[0].contains("1.0.0"), "{problems:?}");

    let downgraded = PackIndex::new(vec![entry(manifest(
        PackKind::Emoji,
        "emoji",
        "0.9.0",
        b"emoji",
    ))]);
    let problems = downgraded.check_successor(&previous).unwrap_err();
    assert!(problems[0].contains("emoji"), "{problems:?}");
}
