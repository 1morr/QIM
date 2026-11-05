//! 預設 Gboard 速成佈局（plan §2.6）：由排列算出，經 `qim-layout` 讀回與點擊。

use qim_databuild::gboard;
use qim_layout::{
    Action, Device, Frame, KeyState, Layout, LayoutVariant, Orientation, Page, check_builtin,
};

fn layout() -> Layout {
    // 經過 JSON：網頁讀的是寫出去的檔。
    Layout::from_json(&gboard::layout().to_json()).unwrap()
}

fn page<'a>(variant: &'a LayoutVariant, id: &str) -> &'a Page {
    &variant.pages[variant
        .page_index(id)
        .unwrap_or_else(|| panic!("沒有頁 {id}"))]
}

fn frame(page: &Page, id: &str) -> Frame {
    page.keys
        .iter()
        .find(|k| k.id == id)
        .unwrap_or_else(|| panic!("頁 {} 沒有鍵 {id}", page.id))
        .frame
}

fn key<'a>(page: &'a Page, id: &str) -> &'a qim_layout::Key {
    page.keys.iter().find(|k| k.id == id).unwrap()
}

/// 內建佈局的嚴格檢查（plan §5.3）：含有直向與橫向的版本、每個鍵都有淺色與深色。
#[test]
fn passes_the_strict_check_for_builtin_layouts_including_portrait_and_landscape() {
    assert_eq!(check_builtin(&gboard::layout().to_json()), []);
}

#[test]
fn has_phone_portrait_and_landscape_with_the_gboard_pages_and_phrases() {
    let layout = layout();
    let whens: Vec<_> = layout
        .variants
        .iter()
        .map(|v| (v.when.device, v.when.orientation))
        .collect();
    assert_eq!(
        whens,
        [
            (Device::Phone, Orientation::Portrait),
            (Device::Phone, Orientation::Landscape)
        ]
    );
    for variant in &layout.variants {
        let ids: Vec<&str> = variant.pages.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                gboard::LETTERS,
                gboard::SYMBOLS,
                gboard::MORE_SYMBOLS,
                gboard::NUMBERS,
                gboard::PHRASES
            ]
        );
        assert_eq!(variant.initial_page_index(), Some(0));
    }
}

#[test]
fn landscape_is_wider_and_shorter_than_portrait() {
    let layout = layout();
    let (portrait, landscape) = (layout.variants[0].canvas, layout.variants[1].canvas);
    assert!(landscape.width > portrait.width);
    assert!(landscape.height < portrait.height);
}

#[test]
fn every_key_hits_itself_at_its_centre_on_every_page() {
    for variant in &layout().variants {
        for page in &variant.pages {
            for key in &page.keys {
                let (x, y) = (
                    key.frame.x + key.frame.w / 2.0,
                    key.frame.y + key.frame.h / 2.0,
                );
                assert_eq!(
                    page.hit(x, y).map(|i| &page.keys[i].id),
                    Some(&key.id),
                    "{:?} 頁 {}",
                    variant.when,
                    page.id
                );
            }
        }
    }
}

#[test]
fn letters_page_has_every_letter_and_the_edit_keys() {
    for variant in &layout().variants {
        let page = page(variant, gboard::LETTERS);
        let mut letters: Vec<char> = page
            .keys
            .iter()
            .filter_map(|key| match key.tap {
                Action::Letter(c) => Some(c),
                _ => None,
            })
            .collect();
        letters.sort();
        assert_eq!(
            letters.iter().collect::<String>(),
            "abcdefghijklmnopqrstuvwxyz"
        );
        for edit in [Action::Space, Action::Backspace, Action::Enter] {
            assert!(page.keys.iter().any(|k| k.tap == edit), "少了 {edit:?}");
        }
    }
}

#[test]
fn qwerty_rows_follow_gboard_in_both_orientations() {
    for variant in &layout().variants {
        let page = page(variant, gboard::LETTERS);
        let frame = |id: &str| frame(page, id);
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
        // 刪除鍵在 m 右邊；最下排由左到右：?123、逗號、空白、句號、Enter。
        assert!(frame("m").x < frame("backspace").x);
        let bottom = ["to-symbols", "，", "space", "。", "enter"].map(|id| frame(id).x);
        assert!(
            bottom.windows(2).all(|pair| pair[0] < pair[1]),
            "{bottom:?}"
        );
    }
}

#[test]
fn letter_keys_carry_the_letter_as_hint_and_leave_the_label_to_the_scheme() {
    for variant in &layout().variants {
        for key in &page(variant, gboard::LETTERS).keys {
            if let Action::Letter(c) = key.tap {
                if c != 'z' {
                    assert_eq!(key.label, None, "{}", key.id);
                }
                assert_eq!(key.hint.as_deref(), Some(c.to_string().as_str()));
            }
        }
    }
}

#[test]
fn long_press_on_a_letter_types_only_the_english_letter() {
    // brief §4.4：長按字母鍵只彈出英文字母（直接輸入，不經組字）；數字改由數字列（M1 票 27）。
    let text = |s: &str| Action::Text(s.into());
    for variant in &layout().variants {
        let page = page(variant, gboard::LETTERS);
        assert_eq!(key(page, "q").long_press, [text("q")]);
        assert_eq!(key(page, "p").long_press, [text("p")]);
        for key in &page.keys {
            if let Action::Letter(c) = key.tap {
                assert_eq!(key.long_press, [text(&c.to_string())], "{}", key.id);
            }
        }
    }
}

/// `z` 照 Gboard 標「重」：方案沒有它的字根，不寫的話標籤會是字母（M1 票 27）。
#[test]
fn z_is_labelled_like_gboard() {
    for variant in &layout().variants {
        let z = key(page(variant, gboard::LETTERS), "z");
        assert_eq!(z.label.as_deref(), Some("重"));
        assert_eq!(z.hint.as_deref(), Some("z"));
        assert_eq!(z.tap, Action::Letter('z'));
    }
}

/// 一頁的鍵 id 依位置由上到下、由左到右排好，每排一個字串，鍵 id 以空白分開。
fn rows(page: &Page) -> Vec<String> {
    let mut keys: Vec<&qim_layout::Key> = page.keys.iter().collect();
    keys.sort_by(|a, b| {
        (a.frame.y, a.frame.x)
            .partial_cmp(&(b.frame.y, b.frame.x))
            .unwrap()
    });
    let mut rows: Vec<(f64, Vec<&str>)> = Vec::new();
    for key in keys {
        match rows.last_mut() {
            Some((y, ids)) if *y == key.frame.y => ids.push(&key.id),
            _ => rows.push((key.frame.y, vec![&key.id])),
        }
    }
    rows.into_iter().map(|(_, ids)| ids.join(" ")).collect()
}

/// 字母頁的鍵位照 Gboard 速成（M1 票 27 的實測）。
#[test]
fn letters_page_matches_gboard_quick() {
    for variant in &layout().variants {
        let letters = page(variant, gboard::LETTERS);
        assert_eq!(
            rows(letters),
            [
                "q w e r t y u i o p",
                "a s d f g h j k l",
                "shift z x c v b n m backspace",
                "to-symbols ， space 。 enter",
            ]
        );
        assert_eq!(key(letters, "shift").tap, Action::Shift);
        // 組字中 ⇧ 換成分隔鍵「'詞」（同 Gboard 速成，M1 票 28），送核心的分隔記號。
        let composing = KeyState {
            composing: true,
            ..KeyState::default()
        };
        let shift = key(letters, "shift");
        assert_eq!(shift.display_label(composing, |_| None), "'詞");
        assert_eq!(
            shift.tap_action(composing),
            Action::Punct(qim_core::SEPARATOR)
        );
        assert_eq!(shift.accessibility_label(composing, "'詞"), "分隔");
        assert_eq!(shift.tap_action(KeyState::default()), Action::Shift);
        assert_eq!(key(letters, "space").label.as_deref(), Some("速成"));
        assert_eq!(key(letters, "space").a11y_label.as_deref(), Some("空白"));
        // 逗號、句號照鍵上的字輸出全形（使用者 2026-11-05 決定，M1 票 27b）。
        assert_eq!(key(letters, "，").tap, Action::Text("，".into()));
        assert_eq!(key(letters, "。").tap, Action::Text("。".into()));
        // 逗號鍵長按進常用頁（代替 Gboard 的表情入口），句號鍵長按的預設是「……」。
        assert_eq!(
            key(letters, "，").long_press,
            [Action::SwitchPage(gboard::PHRASES.into())]
        );
        let period = &key(letters, "。").long_press;
        assert_eq!(period.len(), 16);
        assert_eq!(period[0], Action::Text("……".into()));
    }
}

#[test]
fn symbol_pages_match_gboard_quick_and_switch_between_each_other() {
    for variant in &layout().variants {
        let symbols = page(variant, gboard::SYMBOLS);
        assert_eq!(
            rows(symbols),
            [
                "1 2 3 4 5 6 7 8 9 0",
                "@ # $ _ & - + （ ） /",
                "to-more-symbols 、 “ ” ： ； ！ ？ backspace",
                "to-letters , to-numbers space . enter",
            ]
        );
        assert_eq!(
            rows(page(variant, gboard::MORE_SYMBOLS)),
            [
                "~ ` | • √ π ÷ × * §",
                "£ ¢ € ¥ ^ ° = 「 」 \\",
                "to-symbols % ‘ ’ ™ ✓ ［ ］ backspace",
                "to-letters < to-numbers space > enter",
            ]
        );
        let numbers = page(variant, gboard::NUMBERS);
        for id in ["+", "1", "5", "9", "%", "space", "backspace", "0", "="] {
            key(numbers, id);
        }
        assert_eq!(
            rows(numbers).last().map(String::as_str),
            Some("to-letters , to-symbols 0 = . enter")
        );
        // 切頁：?123 → 符號，=\< → 更多符號，?123 回符號，12/34 → 數字鍵盤，!?# 回符號，返回回字母。
        let to = |page_id: &str, key_id: &str| key(page(variant, page_id), key_id).tap.clone();
        let switch = |id: &str| Action::SwitchPage(id.into());
        assert_eq!(to(gboard::LETTERS, "to-symbols"), switch(gboard::SYMBOLS));
        assert_eq!(
            to(gboard::SYMBOLS, "to-more-symbols"),
            switch(gboard::MORE_SYMBOLS)
        );
        assert_eq!(
            to(gboard::MORE_SYMBOLS, "to-symbols"),
            switch(gboard::SYMBOLS)
        );
        assert_eq!(to(gboard::SYMBOLS, "to-numbers"), switch(gboard::NUMBERS));
        assert_eq!(to(gboard::NUMBERS, "to-symbols"), switch(gboard::SYMBOLS));
        for id in [gboard::SYMBOLS, gboard::MORE_SYMBOLS, gboard::NUMBERS] {
            let back = key(page(variant, id), "to-letters");
            assert_eq!(back.label.as_deref(), Some("返回"));
            assert_eq!(back.tap, switch(gboard::LETTERS));
        }
        // 打完不切頁，留在符號頁（brief §4.4）。符號照鍵上的字輸出（M1 票 27b）：Gboard 實測打全形的
        // `：；！？（）` 標成全形，底排的 `, .` 與其他 ASCII 標點半形。
        for id in ["1", "0", "_", "、", "@", "？", "！", "（", ",", "."] {
            assert_eq!(key(symbols, id).tap, Action::Text(id.into()), "{id}");
        }
    }
}

/// 配色照 Gboard 實測：字母、符號與空白是白鍵，Shift、刪除、切頁、Enter 與逗號句號是功能鍵。
#[test]
fn keys_use_gboard_light_and_dark_colours() {
    let layout = layout();
    let letters = page(&layout.variants[0], gboard::LETTERS);
    let fill = |id: &str| {
        let fill = key(letters, id).style.clone().unwrap().fill.unwrap();
        (fill.light, fill.dark)
    };
    let plain = ("#ffffff".to_owned(), "#292c34".to_owned());
    let function = ("#dce2f9".to_owned(), "#353b4d".to_owned());
    for id in ["a", "z", "space"] {
        assert_eq!(fill(id), plain, "{id}");
    }
    for id in ["shift", "backspace", "to-symbols", "，", "。", "enter"] {
        assert_eq!(fill(id), function, "{id}");
    }
    // ?123 與 Enter 是膠囊形：圓角等於半高。
    for id in ["to-symbols", "enter"] {
        let key = key(letters, id);
        assert_eq!(key.shape.corner_radius(), key.frame.h / 2.0, "{id}");
    }
    assert!(key(letters, "shift").shape.corner_radius() < key(letters, "shift").frame.h / 2.0);
}

#[test]
fn phrases_page_types_multi_character_text_and_switches_back() {
    for variant in &layout().variants {
        let phrases = page(variant, gboard::PHRASES);
        assert_eq!(key(phrases, "唔該").tap, Action::Text("唔該".into()));
        assert_eq!(key(phrases, "👍").tap, Action::Text("👍".into()));
        for (id, to) in [
            ("to-letters", gboard::LETTERS),
            ("to-symbols", gboard::SYMBOLS),
        ] {
            assert_eq!(key(phrases, id).tap, Action::SwitchPage(to.into()));
        }
    }
}

#[test]
fn tapping_the_gap_between_two_keys_hits_one_of_them() {
    let layout = layout();
    let page = page(&layout.variants[0], gboard::LETTERS);
    let (a, s) = (frame(page, "a"), frame(page, "s"));
    let gap_x = (a.x + a.w + s.x) / 2.0;
    assert!(a.x + a.w < s.x, "a、s 之間要有縫");
    let hit = page
        .hit(gap_x - 0.1, a.y + a.h / 2.0)
        .map(|i| page.keys[i].id.as_str());
    assert_eq!(hit, Some("a"));
}

/// 標籤是符號的刪除、Enter 給文字的無障礙標籤，讀屏不會唸「⌫」「↵」（plan §2.6，M1 票 24）。
#[test]
fn glyph_labelled_edit_keys_have_spoken_accessibility_labels_on_every_page() {
    for variant in &layout().variants {
        for page in &variant.pages {
            for key in &page.keys {
                let expected = match key.tap {
                    Action::Backspace => Some("刪除"),
                    Action::Enter => Some("換行"),
                    _ => None,
                };
                if let Some(expected) = expected {
                    assert_eq!(key.a11y_label.as_deref(), Some(expected), "{}", key.id);
                }
            }
        }
    }
    let letters = &layout().variants[0].pages[0];
    assert_eq!(key(letters, "a").a11y_label, None);
}

/// 數字列（brief §4.4、§7，同 Gboard 的 Number row）：另一份佈局，字母頁上方多一排數字，
/// 點一下直接輸入數字；其他排不變（M1 票 27）。
#[test]
fn number_row_layout_adds_a_row_of_digits_above_the_letters() {
    assert_eq!(check_builtin(&gboard::number_row_layout().to_json()), []);
    let plain = layout();
    let with_row = Layout::from_json(&gboard::number_row_layout().to_json()).unwrap();
    for (plain, with_row) in plain.variants.iter().zip(&with_row.variants) {
        assert_eq!(plain.when, with_row.when);
        assert_eq!(plain.canvas.width, with_row.canvas.width);
        assert!(with_row.canvas.height > plain.canvas.height);
        let letters = page(with_row, gboard::LETTERS);
        let digits: Vec<&str> = "1234567890".split("").filter(|s| !s.is_empty()).collect();
        for pair in digits.windows(2) {
            let (left, right) = (frame(letters, pair[0]), frame(letters, pair[1]));
            assert_eq!(left.y, right.y, "{pair:?}");
            assert!(left.x < right.x, "{pair:?}");
        }
        for digit in &digits {
            assert_eq!(key(letters, digit).tap, Action::Text((*digit).into()));
            assert!(frame(letters, digit).y < frame(letters, "q").y);
        }
        // 只多了數字列：字母鍵的數量與其他頁的 id 不變。
        assert_eq!(
            letters.keys.len(),
            page(plain, gboard::LETTERS).keys.len() + 10
        );
        let ids = |v: &LayoutVariant| v.pages.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(plain), ids(with_row));
    }
}

/// 觸控鍵照鍵上的字輸出（使用者 2026-11-05 決定，M1 票 27b）：每一頁直接輸入文字的鍵，打出的就是
/// 標籤；沒有送核心標點鍵（全形半形由核心猜）的鍵，只有組字中的分隔鍵例外（`composing`）。
#[test]
fn touch_keys_type_what_their_label_shows() {
    for layout in [gboard::layout(), gboard::number_row_layout()] {
        for variant in &layout.variants {
            for page in &variant.pages {
                for key in &page.keys {
                    let at = format!("{} {}", page.id, key.id);
                    assert!(!matches!(key.tap, Action::Punct(_)), "{at}");
                    if let Action::Text(text) = &key.tap {
                        assert_eq!(key.label.as_deref(), Some(text.as_str()), "{at}");
                    }
                }
            }
        }
    }
}
