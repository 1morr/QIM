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
                  "hint": "a", "longPress": [{ "text": "a" }], "style": { "labelSize": 10 } },
                { "id": "z", "frame": { "x": 10, "y": 0, "w": 10, "h": 10 }, "tap": { "letter": "z" },
                  "label": "重", "hint": "z" },
                { "id": "shift", "frame": { "x": 20, "y": 0, "w": 10, "h": 10 }, "tap": "shift", "label": "⇧" },
                { "id": "space", "frame": { "x": 30, "y": 0, "w": 10, "h": 10 }, "tap": "space", "hint": "␣",
                  "style": { "labelSize": 10 } }
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
        // 讀屏念畫面上的字：沒有角標就不念角標（不是「A，a」，M1 票 27b）。
        assert_eq!(keys[0].accessibility_label(state, "A"), "A");
        assert_eq!(keys[1].accessibility_label(state, "Z"), "Z");
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
    assert_eq!(keys[0].accessibility_label(off, "日"), "日，a");
}

#[test]
fn once_wears_off_after_any_other_key_but_lock_keeps_it() {
    // 照 Android 模擬器上的 Gboard 速成實測（M1 票 27b）：開一次的 Shift 打了字母、逗號、句號、
    // 空白、刪除或切頁都解除；鎖定的都不解除。
    assert_eq!(Shift::Once.after_other_key(), Shift::Off);
    assert_eq!(Shift::Locked.after_other_key(), Shift::Locked);
    assert_eq!(Shift::Off.after_other_key(), Shift::Off);
}

#[test]
fn while_shifted_letter_keys_draw_the_capital_bigger_like_gboard() {
    // Gboard 速成的大寫字母比字根大得多（M1 票 27b，`.scratch/m1/evidence/27-letters.png`）。
    let layout = layout();
    let keys = &layout.variants[0].pages[0].keys;
    let off = KeyState::default();
    assert_eq!(keys[0].display_label_size(off), Some(10.0));
    for shift in [Shift::Once, Shift::Locked] {
        let size = keys[0].display_label_size(shifted(shift)).unwrap();
        assert!((18.0..=19.0).contains(&size), "{size}");
        // 不是字母鍵的、沒寫字級的照舊。
        assert_eq!(keys[3].display_label_size(shifted(shift)), Some(10.0));
        assert_eq!(keys[1].display_label_size(shifted(shift)), None);
    }
    // 組字中 Shift 開著也是大寫字母（點了先上屏組字再接大寫），一樣放大。
    let composing = KeyState {
        composing: true,
        shift: Shift::Once,
    };
    assert_eq!(keys[0].display_label(composing, radical), "A");
    assert_eq!(
        keys[0].display_label_size(composing),
        keys[0].display_label_size(shifted(Shift::Once))
    );
}
