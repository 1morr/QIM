//! 觸控點對鍵的點擊（M4 票 09c）：每個點擊最高的字母就是要打的鍵、同種子同結果、鄰鍵有份。

use qim_bench::{Taps, Unit, top_char};
use qim_core::Key;

/// 三個並排的字母鍵 a、b、c（各 50 × 50）。
const THREE_KEYS: &str = r#"{
  "format": "qim-layout/1",
  "variants": [{
    "when": { "device": "phone", "orientation": "portrait" },
    "canvas": { "width": 150, "height": 50 },
    "pages": [{ "id": "letters", "keys": [
      { "id": "a", "frame": { "x": 0, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "a" } },
      { "id": "b", "frame": { "x": 50, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "b" } },
      { "id": "c", "frame": { "x": 100, "y": 0, "w": 50, "h": 50 }, "tap": { "letter": "c" } }
    ]}]
  }]
}"#;

fn unit(code: &str) -> Unit {
    Unit {
        code: code.into(),
        text: "字".into(),
    }
}

#[test]
fn every_tap_has_the_intended_key_as_its_top_letter() {
    let taps = Taps::from_json(THREE_KEYS).unwrap();
    let input: String = "abc".repeat(300);
    let keys = taps.input_keys("seed", &input);
    assert_eq!(keys.len(), input.chars().count());
    for (key, c) in keys.iter().zip(input.chars()) {
        assert!(matches!(key, Key::Weighted(_)), "{key:?}");
        assert_eq!(top_char(key), Some(c));
    }
}

#[test]
fn taps_are_touch_probabilities_with_neighbour_letters() {
    let taps = Taps::from_json(THREE_KEYS).unwrap();
    let keys = taps.input_keys("seed", &"b".repeat(200));
    let with_neighbour = keys
        .iter()
        .filter(|key| matches!(key, Key::Weighted(w) if w.entries().len() > 1))
        .count();
    assert!(with_neighbour > 0, "點在 b 鍵上的點擊應有鄰鍵的機率");
}

#[test]
fn the_same_seed_gives_the_same_taps_and_another_seed_does_not() {
    let taps = Taps::from_json(THREE_KEYS).unwrap();
    let units = [unit("ab"), unit("c")];
    assert_eq!(taps.unit_keys("s", &units), taps.unit_keys("s", &units));
    assert_ne!(taps.unit_keys("s", &units), taps.unit_keys("t", &units));
    assert_eq!(taps.input_keys("s", "abc"), taps.input_keys("s", "abc"));
}

#[test]
fn units_keep_their_grouping_and_unknown_characters_follow_for_input() {
    let taps = Taps::from_json(THREE_KEYS).unwrap();
    let keys = taps.unit_keys("s", &[unit("ab"), unit("c")]);
    assert_eq!(
        keys.iter().map(Vec::len).collect::<Vec<_>>(),
        [2, 1],
        "{keys:?}"
    );
    // 佈局上沒有的字元（分隔記號、沒有的字母）照 `Key::for_input`。
    assert_eq!(
        taps.input_keys("s", "'z"),
        [Key::for_input('\''), Key::for_input('z')]
    );
}
