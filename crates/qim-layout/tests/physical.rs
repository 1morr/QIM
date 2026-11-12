//! 實體鍵盤的鄰鍵先驗（plan §2.6，M4 票 07）：按一個實體鍵（`KeyboardEvent.code`）算成各字母的機率。
//! 照實體鍵位（各排錯位，含上下排與斜角）；輸入只有 `code`，換 Dvorak 等配列（`key` 不同）結果不變，
//! 網頁從事件挑出 `code` 的那一半測在 `web/test/keys.test.ts`。

use qim_layout::PhysicalModel;

fn weights(code: &str) -> Vec<(char, f32)> {
    PhysicalModel::default()
        .letter_weights(code)
        .unwrap_or_else(|| panic!("{code} 是字母鍵"))
}

fn letters(weights: &[(char, f32)]) -> Vec<char> {
    weights.iter().map(|&(c, _)| c).collect()
}

fn probability(weights: &[(char, f32)], c: char) -> f32 {
    weights
        .iter()
        .find(|&&(d, _)| d == c)
        .map_or(0.0, |&(_, p)| p)
}

const CODES: [&str; 26] = [
    "KeyA", "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG", "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL",
    "KeyM", "KeyN", "KeyO", "KeyP", "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU", "KeyV", "KeyW", "KeyX",
    "KeyY", "KeyZ",
];

#[test]
fn key_f_neighbours_are_the_six_adjacent_keys_across_rows() {
    let f = weights("KeyF");
    assert_eq!(f[0].0, 'f');
    let mut neighbours = letters(&f[1..]);
    neighbours.sort_unstable();
    assert_eq!(neighbours, ['c', 'd', 'g', 'r', 't', 'v']);
}

#[test]
fn stagger_makes_the_upper_left_diagonal_nearer_than_the_upper_right() {
    // F 上一排的 R 只往左錯 0.25 鍵、T 往右錯 0.75 鍵；同排左右的 D、G 最近。
    let f = weights("KeyF");
    assert!(probability(&f, 'd') > probability(&f, 'r'));
    assert!((probability(&f, 'd') - probability(&f, 'g')).abs() < 1e-6);
    assert!(probability(&f, 'r') > probability(&f, 't'));
    // 下一排的 C、V 一樣遠，在 R 與 T 之間。
    assert!((probability(&f, 'c') - probability(&f, 'v')).abs() < 1e-6);
    assert!(probability(&f, 'r') > probability(&f, 'c'));
    assert!(probability(&f, 'c') > probability(&f, 't'));
}

#[test]
fn edge_keys_only_have_their_real_neighbours() {
    let mut q = letters(&weights("KeyQ"));
    q.sort_unstable();
    assert_eq!(q, ['a', 'q', 'w']);
    let mut p = letters(&weights("KeyP"));
    p.sort_unstable();
    // P 下面的是分號鍵，不是字母，不算。
    assert_eq!(p, ['l', 'o', 'p']);
    let mut m = letters(&weights("KeyM"));
    m.sort_unstable();
    assert_eq!(m, ['j', 'k', 'm', 'n']);
}

#[test]
fn every_letter_key_is_highest_and_sums_to_one() {
    let model = PhysicalModel::default();
    for code in CODES {
        let w = weights(code);
        let pressed = code.strip_prefix("Key").unwrap().to_ascii_lowercase();
        assert_eq!(w[0].0.to_string(), pressed, "{code}");
        assert!(w[1..].iter().all(|&(_, p)| p < w[0].1), "{code}: {w:?}");
        assert!(w.windows(2).all(|pair| pair[0].1 >= pair[1].1), "{code}");
        // 錯位的排上每個鍵最多 6 個鄰鍵（同排左右、上下排各二）。
        assert!(w.len() <= 7, "{code}");
        let sum: f32 = w.iter().map(|&(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 1e-5, "{code}: {sum}");
        assert!(
            (w[0].1 - model.hit_probability as f32).abs() < 1e-5,
            "{code}"
        );
    }
}

#[test]
fn neighbourhood_is_symmetric() {
    for code in CODES {
        let me = code.strip_prefix("Key").unwrap().to_ascii_lowercase();
        let me = me.chars().next().unwrap();
        for &(other, _) in &weights(code)[1..] {
            let back = weights(&format!("Key{}", other.to_ascii_uppercase()));
            assert!(letters(&back).contains(&me), "{code} → {other}");
        }
    }
}

#[test]
fn non_letter_codes_have_no_weights() {
    let model = PhysicalModel::default();
    for code in [
        "Digit1",
        "Semicolon",
        "Space",
        "ShiftLeft",
        "Keyf",
        "KeyAB",
        "Key",
        "",
    ] {
        assert_eq!(model.letter_weights(code), None, "{code}");
    }
}

/// 打錯碼的基準（M4 票 08）在 `qim-databuild` 照實體鍵位抽鄰鍵，鍵位要由這裡給、不另寫一份。
#[test]
fn letter_key_centres_cover_the_alphabet_with_staggered_rows() {
    let keys = qim_layout::physical_letter_keys();
    let mut letters: Vec<char> = keys.iter().map(|&(c, _, _)| c).collect();
    letters.sort_unstable();
    assert_eq!(letters, ('a'..='z').collect::<Vec<_>>());
    let centre = |c: char| {
        let &(_, x, y) = keys.iter().find(|&&(d, _, _)| d == c).unwrap();
        (x, y)
    };
    assert_eq!(centre('q'), (2.0, 0.0));
    assert_eq!(centre('a'), (2.25, 1.0));
    assert_eq!(centre('z'), (2.75, 2.0));
    assert_eq!(centre('f'), (5.25, 1.0));
}
