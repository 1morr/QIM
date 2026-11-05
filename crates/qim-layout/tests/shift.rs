//! Shift 鍵（brief §4.4，照 Gboard 速成，M1 票 27）：點一下，下一個字母鍵直接輸入大寫英文字母，
//! 打完自動解除；快速點兩下鎖定大寫，再點一下解除。長按一律照長按清單（小寫）。

use qim_layout::{Action, KeyState, Layout, Shift, Target, validate};

/// 經過 JSON Schema 與錯誤級的檢查讀進來，確定 Schema 認得 `"shift"`。
fn layout() -> Layout {
    let (layout, _warnings) = validate(
        r#"{ "format": "qim-layout/1", "variants": [{
              "when": { "device": "phone", "orientation": "portrait" },
              "canvas": { "width": 40, "height": 10 },
              "pages": [{ "id": "letters", "keys": [
                { "id": "a", "frame": { "x": 0, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "a" },
                  "hint": "a", "longPress": [{ "text": "a" }] },
                { "id": "z", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "z" },
                  "label": "重", "hint": "z" },
                { "id": "shift", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": "shift", "label": "⇧" },
                { "id": "space", "frame": { "x": 30, "y": 0, "w": 10, "h": 10 }, "tap": "space", "hint": "␣" }
              ] }] }] }"#,
        Target::default(),
    )
    .into_layout()
    .unwrap();
    layout
}

fn radical(letter: char) -> Option<&'static str> {
    (letter == 'a').then_some("日")
}

fn shifted(shift: Shift) -> KeyState {
    KeyState {
        composing: false,
        shift,
    }
}

#[test]
fn shift_is_an_action_named_shift() {
    let layout = layout();
    let key = &layout.variants[0].pages[0].keys[2];
    assert_eq!(key.tap, Action::Shift);
    assert_eq!(Action::Shift.name(), "shift");
    assert_eq!(Action::Shift.spoken(), "大寫");
}

#[test]
fn tapping_shift_turns_it_on_once_a_quick_second_tap_locks_and_the_next_tap_unlocks() {
    assert_eq!(Shift::Off.after_shift_tap(false), Shift::Once);
    assert_eq!(Shift::Off.after_shift_tap(true), Shift::Once);
    assert_eq!(Shift::Once.after_shift_tap(true), Shift::Locked);
    // 隔太久的第二下是關掉，不是鎖定。
    assert_eq!(Shift::Once.after_shift_tap(false), Shift::Off);
    assert_eq!(Shift::Locked.after_shift_tap(true), Shift::Off);
    assert_eq!(Shift::Locked.after_shift_tap(false), Shift::Off);
}

#[test]
fn while_shifted_letter_keys_show_and_type_the_capital_letter_without_hint() {
    let layout = layout();
    let keys = &layout.variants[0].pages[0].keys;
    for shift in [Shift::Once, Shift::Locked] {
        let state = shifted(shift);
        assert_eq!(keys[0].display_label(state, radical), "A");
        // 有寫死標籤的字母鍵也換成大寫字母（Gboard 的「重」變 Z）。
        assert_eq!(keys[1].display_label(state, radical), "Z");
        assert_eq!(keys[0].tap_action(state), Action::Text("A".into()));
        assert_eq!(keys[0].display_hint(state), None);
        // 不是字母鍵的照舊。
        assert_eq!(keys[2].display_label(state, radical), "⇧");
        assert_eq!(keys[3].tap_action(state), Action::Space);
        assert_eq!(keys[3].display_hint(state), Some("␣"));
    }
    let off = KeyState::default();
    assert_eq!(keys[0].display_label(off, radical), "日");
    assert_eq!(keys[1].display_label(off, radical), "重");
    assert_eq!(keys[0].tap_action(off), Action::Letter('a'));
    assert_eq!(keys[0].display_hint(off), Some("a"));
}

#[test]
fn once_wears_off_after_a_letter_key_but_lock_and_other_keys_keep_it() {
    let layout = layout();
    let keys = &layout.variants[0].pages[0].keys;
    let (letter, space) = (&keys[0], &keys[3]);
    assert_eq!(Shift::Once.after_key(letter), Shift::Off);
    assert_eq!(Shift::Locked.after_key(letter), Shift::Locked);
    assert_eq!(Shift::Once.after_key(space), Shift::Once);
    assert_eq!(Shift::Off.after_key(letter), Shift::Off);
}
