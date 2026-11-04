use std::collections::BTreeMap;

use qim_core::{CodeEntry, CodeTable, FORMAT_VERSION, Pack, PackError, Packs, Ranking, Scheme};

fn sample() -> Pack {
    let scheme = Scheme {
        id: "quick".into(),
        name: "速成".into(),
        version: "1".into(),
        alphabet: "ab".into(),
        max_code_length: 2,
        radicals: BTreeMap::from([('a', "日".into()), ('b', "月".into())]),
        packs: Packs::default(),
        ranking: Ranking::default(),
    };
    let code_table = CodeTable::from_entries([
        (
            "a".to_string(),
            vec![
                CodeEntry {
                    text: '日',
                    extension: false,
                },
                CodeEntry {
                    text: '𠀀',
                    extension: true,
                },
            ],
        ),
        (
            "ab".to_string(),
            vec![CodeEntry {
                text: '明',
                extension: false,
            }],
        ),
    ]);
    Pack { scheme, code_table }
}

#[test]
fn pack_round_trips_through_bytes() {
    let pack = sample();
    let bytes = pack.to_bytes().unwrap();
    assert_eq!(Pack::from_bytes(&bytes).unwrap(), pack);
}

#[test]
fn lookup_returns_entries_in_stored_order_and_empty_for_unknown_code() {
    let pack = sample();
    let texts: Vec<char> = pack.code_table.lookup("a").iter().map(|e| e.text).collect();
    assert_eq!(texts, ['日', '𠀀']);
    assert!(pack.code_table.lookup("abc").is_empty());
}

#[test]
fn other_format_version_is_an_error() {
    let mut bytes = sample().to_bytes().unwrap();
    // 魔數 4 個位元組之後是 postcard varint 的版本號；版本 4 只佔 1 個位元組。
    assert_eq!(bytes[4], 4);
    bytes[4] = 5;
    assert_eq!(
        Pack::from_bytes(&bytes),
        Err(PackError::UnsupportedVersion {
            found: 5,
            supported: FORMAT_VERSION,
        })
    );
}

#[test]
fn bytes_that_are_not_a_pack_are_an_error() {
    assert_eq!(Pack::from_bytes(b""), Err(PackError::NotAPack));
    assert_eq!(Pack::from_bytes(b"PK\x03\x04zip"), Err(PackError::NotAPack));
}

#[test]
fn truncated_or_padded_pack_is_an_error() {
    let bytes = sample().to_bytes().unwrap();
    assert!(matches!(
        Pack::from_bytes(&bytes[..bytes.len() - 1]),
        Err(PackError::Malformed(_))
    ));
    let mut padded = bytes.clone();
    padded.push(0);
    assert!(matches!(
        Pack::from_bytes(&padded),
        Err(PackError::Malformed(_))
    ));
}
