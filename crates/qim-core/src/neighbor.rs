//! 觸控鄰鍵讀法（M4 票 09c，照 Gboard 速成的做法，`docs/research/tolerance.md`〈Gboard 式的鄰鍵讀法〉）：
//! 觸控一下的按鍵機率裡的鄰鍵字母換進輸入碼，換出來的讀法插在候選列第 [`SLOT`] 格起；原樣讀法照舊當首選。
//! 只收觸控（[`crate::Key::Weighted`]），實體鍵盤（[`crate::Key::Physical`]）照舊只取機率最高的字母；
//! 開關是 [`crate::Options::neighbor_readings`]。參數是票 09b 在 dev 挑的（票 09c 在 dev 重看過格數）。

use crate::Weights;

/// 換成的字母的機率至少要是打出的字母的幾倍；低於它的不試。
const FLOOR: f32 = 0.2;
/// 每一碼除了打出的字母，最多試幾個鄰鍵字母（照機率由高到低）。
const LETTERS: usize = 2;
/// 第一個鄰鍵讀法放在候選列的第幾格（從 0 起；原樣的首選在第 0 格）。
pub(crate) const SLOT: usize = 1;
/// 最多列幾個鄰鍵讀法。
pub(crate) const LIMIT: usize = 2;

/// 打出的字母 `typed` 在這一下（`weights`）要試的鄰鍵字母與空間懲罰 `ln(打出的字母的機率 ÷ 換成的字母的機率)`，
/// 照機率由高到低；`allowed` 篩掉方案沒有的鍵。萬用鍵 `wildcard` 不換也不換成它（協調者定：觸控上 z 鍵旁的
/// x 換成 z 會冒出萬用鍵展開，又慢又雜）。
pub(crate) fn alternatives(
    typed: char,
    weights: &Weights,
    wildcard: Option<char>,
    allowed: impl Fn(char) -> bool,
) -> Vec<(char, f64)> {
    if Some(typed) == wildcard {
        return Vec::new();
    }
    let entries = weights.entries();
    let pressed = entries
        .iter()
        .find(|&&(c, _)| c == typed)
        .map_or(entries[0].1, |&(_, p)| p);
    entries
        .iter()
        .filter(|&&(c, p)| c != typed && Some(c) != wildcard && allowed(c) && p >= pressed * FLOOR)
        .take(LETTERS)
        .map(|&(c, p)| (c, f64::from(pressed / p).ln()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weights(entries: &[(char, f32)]) -> Weights {
        Weights::new(entries.to_vec()).unwrap()
    }

    fn letters(alternatives: &[(char, f64)]) -> Vec<char> {
        alternatives.iter().map(|&(c, _)| c).collect()
    }

    #[test]
    fn at_most_two_letters_at_or_above_the_floor_in_order_of_probability() {
        let w = weights(&[('s', 0.5), ('a', 0.2), ('d', 0.15), ('w', 0.1), ('x', 0.05)]);
        let found = alternatives('s', &w, None, |_| true);
        assert_eq!(letters(&found), ['a', 'd']);
        assert!((found[0].1 - (0.5f64 / 0.2).ln()).abs() < 1e-6);
        // x 只有 s 的 0.14 倍，floor 0.2 以下。
        let w = weights(&[('s', 0.7), ('x', 0.1), ('q', 0.2)]);
        assert_eq!(letters(&alternatives('s', &w, None, |_| true)), ['q']);
    }

    #[test]
    fn keys_outside_the_scheme_are_skipped() {
        let w = weights(&[('s', 0.5), ('a', 0.3), ('d', 0.2)]);
        assert_eq!(letters(&alternatives('s', &w, None, |c| c != 'a')), ['d']);
    }

    #[test]
    fn the_wildcard_is_never_swapped_in_or_out() {
        let w = weights(&[('x', 0.5), ('z', 0.4), ('c', 0.1)]);
        assert!(!letters(&alternatives('x', &w, Some('z'), |_| true)).contains(&'z'));
        let w = weights(&[('z', 0.5), ('x', 0.4), ('a', 0.1)]);
        assert!(alternatives('z', &w, Some('z'), |_| true).is_empty());
    }
}
