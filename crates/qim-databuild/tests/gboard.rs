//! 預設 Gboard 速成佈局（plan §2.6）：由排列算出，經 `qim-layout` 讀回與點擊。

use qim_databuild::gboard;
use qim_layout::{Action, Device, Layout, Orientation, StrictIssue, Warning, check_builtin};

fn portrait() -> Layout {
    // 經過 JSON：網頁讀的是寫出去的檔。
    Layout::from_json(&gboard::layout().to_json()).unwrap()
}

#[test]
fn has_one_phone_portrait_variant_with_one_page() {
    let layout = portrait();
    assert_eq!(layout.variants.len(), 1);
    let variant = &layout.variants[0];
    assert_eq!(variant.when.device, Device::Phone);
    assert_eq!(variant.when.orientation, Orientation::Portrait);
    assert_eq!(variant.pages.len(), 1);
}

#[test]
fn every_letter_and_the_edit_keys_hit_themselves_at_their_centre() {
    let layout = portrait();
    let page = &layout.variants[0].pages[0];
    let mut taps: Vec<Action> = Vec::new();
    for key in &page.keys {
        let (x, y) = (
            key.frame.x + key.frame.w / 2.0,
            key.frame.y + key.frame.h / 2.0,
        );
        assert_eq!(page.hit(x, y).map(|i| &page.keys[i].id), Some(&key.id));
        taps.push(key.tap.clone());
    }
    let mut letters: Vec<char> = taps
        .iter()
        .filter_map(|tap| match tap {
            Action::Letter(c) => Some(*c),
            _ => None,
        })
        .collect();
    letters.sort();
    assert_eq!(
        letters.iter().collect::<String>(),
        "abcdefghijklmnopqrstuvwxyz"
    );
    for edit in [Action::Space, Action::Backspace, Action::Enter] {
        assert!(taps.contains(&edit), "少了 {edit:?}");
    }
}

#[test]
fn qwerty_rows_follow_gboard() {
    let layout = portrait();
    let page = &layout.variants[0].pages[0];
    let frame = |id: &str| page.keys.iter().find(|k| k.id == id).unwrap().frame;
    // 同一排同高、由左到右。
    for row in ["qwertyuiop", "asdfghjkl", "zxcvbnm"] {
        let ids: Vec<String> = row.chars().map(String::from).collect();
        for pair in ids.windows(2) {
            let (left, right) = (frame(&pair[0]), frame(&pair[1]));
            assert_eq!(left.y, right.y, "{pair:?}");
            assert!(left.x < right.x, "{pair:?}");
        }
    }
    // 由上到下：q 排、a 排、z 排、空白。
    assert!(frame("q").y < frame("a").y);
    assert!(frame("a").y < frame("z").y);
    assert!(frame("z").y < frame("space").y);
    // 第二排錯開半鍵，a 在 q 與 w 之間。
    assert!(frame("q").x < frame("a").x && frame("a").x < frame("w").x);
    // 刪除鍵在 m 右邊，Enter 在空白右邊。
    assert!(frame("m").x < frame("backspace").x);
    assert!(frame("space").x < frame("enter").x);
}

#[test]
fn letter_keys_carry_the_letter_as_hint_and_leave_the_label_to_the_scheme() {
    let layout = portrait();
    for key in &layout.variants[0].pages[0].keys {
        if let Action::Letter(c) = key.tap {
            assert_eq!(key.label, None, "{}", key.id);
            assert_eq!(key.hint.as_deref(), Some(c.to_string().as_str()));
        }
    }
}

/// 內建佈局的嚴格檢查（plan §5.3），只有「缺橫向」先放過，由 [`has_portrait_and_landscape`] 管。
#[test]
fn passes_the_strict_check_for_builtin_layouts() {
    let issues: Vec<StrictIssue> = check_builtin(&gboard::layout().to_json())
        .into_iter()
        .filter(|issue| {
            !matches!(
                issue,
                StrictIssue::Warning(Warning::MissingOrientation { .. })
            )
        })
        .collect();
    assert_eq!(issues, []);
}

#[test]
#[ignore = "M1 票 20 加橫向版本時拿掉 ignore"]
fn has_portrait_and_landscape() {
    assert_eq!(check_builtin(&gboard::layout().to_json()), []);
}

#[test]
fn tapping_the_gap_between_two_keys_hits_one_of_them() {
    let layout = portrait();
    let page = &layout.variants[0].pages[0];
    let frame = |id: &str| page.keys.iter().find(|k| k.id == id).unwrap().frame;
    let (a, s) = (frame("a"), frame("s"));
    let gap_x = (a.x + a.w + s.x) / 2.0;
    assert!(a.x + a.w < s.x, "a、s 之間要有縫");
    let hit = page
        .hit(gap_x - 0.1, a.y + a.h / 2.0)
        .map(|i| page.keys[i].id.as_str());
    assert_eq!(hit, Some("a"));
}
