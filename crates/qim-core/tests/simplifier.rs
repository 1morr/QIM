//! 簡繁轉換（plan〈英文、簡繁與 emoji〉）：照 OpenCC `hk2s` 的規則轉換，表由簡繁資料包給。

use qim_core::{ConversionTable, Pack, PackError, Simplifier, SimplifierPack};

fn table(entries: &[(&str, &str)]) -> ConversionTable {
    ConversionTable::new(entries.iter().map(|&(k, v)| (k.to_string(), v.to_string())))
}

/// 沒有正規化、分段的詞表是空的，只有 `chain` 的轉換。
fn chain(chain: Vec<Vec<ConversionTable>>) -> Simplifier {
    Simplifier::new(vec![], table(&[]), chain)
}

#[test]
fn a_table_matches_its_longest_key() {
    let s = chain(vec![vec![table(&[
        ("乾", "干"),
        ("乾隆", "乾隆"),
        ("門", "门"),
    ])]]);
    assert_eq!(s.convert("乾隆"), "乾隆");
    assert_eq!(s.convert("乾杯門"), "干杯门");
}

/// OpenCC 的 `short_circuit`：群組裡第一個有匹配的表勝出，後面的表較長也不看。
#[test]
fn the_first_table_in_a_group_with_a_match_wins() {
    let s = chain(vec![vec![
        table(&[("乾", "干")]),
        table(&[("乾隆", "乾隆")]),
    ]]);
    assert_eq!(s.convert("乾隆"), "干隆");
}

/// 群組依序套用：前一組的輸出是下一組的輸入（香港異體字先轉回標準繁體，再轉簡體）。
#[test]
fn groups_apply_in_order() {
    let s = chain(vec![
        vec![table(&[("綫", "線")])],
        vec![table(&[("線", "线")])],
    ]);
    assert_eq!(s.convert("綫"), "线");
}

/// 先用分段詞表切段（OpenCC 的 mmseg），各段分開轉換：轉換的詞不跨段。
#[test]
fn conversion_does_not_cross_segments() {
    let segmentation = table(&[("乾隆", "乾隆")]);
    let s = Simplifier::new(
        vec![],
        segmentation,
        vec![vec![table(&[("隆門", "X"), ("門", "门")])]],
    );
    assert_eq!(s.convert("乾隆門"), "乾隆门");
    assert_eq!(s.convert("隆門"), "X");
}

/// 正規化（CJK 相容表意文字轉成統一表意文字）在分段之前整串套用。
#[test]
fn normalization_comes_first() {
    let s = Simplifier::new(
        vec![vec![table(&[("\u{F900}", "豈")])]],
        table(&[]),
        vec![vec![table(&[("豈", "岂")])]],
    );
    assert_eq!(s.convert("\u{F900}"), "岂");
}

#[test]
fn characters_without_a_mapping_stay() {
    let s = chain(vec![vec![table(&[("門", "门")])]]);
    assert_eq!(s.convert("a門。𠀀"), "a门。𠀀");
    assert_eq!(s.convert(""), "");
}

#[test]
fn a_pack_round_trips() {
    let pack = SimplifierPack {
        id: "opencc-hk2s".into(),
        simplifier: Simplifier::new(
            vec![vec![table(&[("\u{F900}", "豈")])]],
            table(&[("乾隆", "乾隆")]),
            vec![vec![table(&[("乾隆", "乾隆")]), table(&[("乾", "干")])]],
        ),
    };
    let bytes = pack.to_bytes().unwrap();
    let decoded = SimplifierPack::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, pack);
    assert_eq!(decoded.simplifier.convert("乾隆乾"), "乾隆干");
}

#[test]
fn other_packs_are_not_simplifier_packs() {
    assert_eq!(
        SimplifierPack::from_bytes(b"QIMPxxxx").err(),
        Some(PackError::NotAPack)
    );
}
