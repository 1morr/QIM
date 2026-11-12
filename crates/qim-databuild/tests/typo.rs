//! 打錯碼的基準集合（M4 票 08）：「含打錯碼」與「觸控模擬」的產生器。
//!
//! 錯誤模型與空間模型故意不同（`src/typo.rs` 的模組說明）：這裡驗產生出來的錯誤長得像宣稱的模型，
//! 基準那一半在 `crates/qim-bench/tests/typo.rs`。

use qim_databuild::gboard;
use qim_databuild::typo::{
    self, Stats, TouchNoise, TouchSurface, TypoModel, touch_items, typo_items,
};
use qim_layout::{Action, KeyState, Page, Shift, physical_letter_keys};

/// 一行 `碼<TAB>句子`；`n` 個字，碼輪流用幾個常見的兩碼。
fn bench(lines: usize, chars: usize) -> String {
    const CODES: [&str; 8] = ["oi", "ma", "hr", "yr", "nd", "ok", "ln", "jd"];
    (0..lines)
        .map(|i| {
            let codes: Vec<&str> = (0..chars)
                .map(|j| CODES[(i * 7 + j) % CODES.len()])
                .collect();
            // 句子要每行不同（亂數由整行決定），字數同碼數。
            let sentence: String = (0..chars)
                .map(|j| char::from_u32(0x4e00 + (i * chars + j) as u32).unwrap())
                .collect();
            format!("{}\t{sentence}\n", codes.join(" "))
        })
        .collect()
}

fn fields(line: &str) -> (Vec<&str>, Vec<&str>, &str) {
    let parts: Vec<&str> = line.split('\t').collect();
    assert_eq!(parts.len(), 3, "{line}");
    (
        parts[0].split(' ').collect(),
        parts[1].split(' ').collect(),
        parts[2],
    )
}

#[test]
fn typo_items_keep_codes_and_sentence_and_group_keys_per_character() {
    let input = bench(200, 6);
    let (out, stats) = typo_items(&input, &TypoModel::default()).unwrap();
    assert_eq!(out.lines().count(), 200);
    assert_eq!(stats.items, 200);
    assert_eq!(stats.keys, 200 * 12);
    for (line, source) in out.lines().zip(input.lines()) {
        let (typed, codes, sentence) = fields(line);
        let (source_codes, source_sentence) = source.split_once('\t').unwrap();
        assert_eq!(codes.join(" "), source_codes);
        assert_eq!(sentence, source_sentence);
        assert_eq!(typed.len(), codes.len(), "每個字一組實際按的鍵：{line}");
    }
}

#[test]
fn generation_is_deterministic_and_per_line() {
    let input = bench(50, 5);
    let model = TypoModel::default();
    let (first, _) = typo_items(&input, &model).unwrap();
    let (second, _) = typo_items(&input, &model).unwrap();
    assert_eq!(first, second);
    // 拿掉第一行、其餘照舊：每行的錯誤只看自己那一行。
    let rest: String = input.lines().skip(1).map(|l| format!("{l}\n")).collect();
    let (shifted, _) = typo_items(&rest, &model).unwrap();
    let tail: Vec<&str> = first.lines().skip(1).collect();
    assert_eq!(shifted.lines().collect::<Vec<_>>(), tail);
}

#[test]
fn error_rate_and_kinds_follow_the_model() {
    let input = bench(4000, 5);
    let model = TypoModel::default();
    let (out, stats) = typo_items(&input, &model).unwrap();
    let rate = stats.key_error_rate();
    assert!(
        (rate - model.rate).abs() < 0.005,
        "每鍵出錯 {rate}，模型 {}",
        model.rate
    );
    // 只有一處錯的題，從實際按的鍵數看得出種類：少一鍵是漏碼、多一鍵是多碼，同數是替換或對調。
    let (mut fewer, mut more, mut same) = (0, 0, 0);
    for line in out.lines() {
        let (typed, codes, _) = fields(line);
        let typed: String = typed.concat();
        let codes: String = codes.concat();
        if typed == codes {
            continue;
        }
        match typed.len().cmp(&codes.len()) {
            std::cmp::Ordering::Less => fewer += 1,
            std::cmp::Ordering::Greater => more += 1,
            std::cmp::Ordering::Equal => same += 1,
        }
    }
    let total = f64::from(fewer + more + same);
    let share = |n: i32| f64::from(n) / total;
    let kinds = model.substitute + model.omit + model.insert + model.transpose;
    // 一題有兩處錯時互相抵銷（多一鍵加少一鍵），比例只看大約。
    assert!(
        (share(fewer) - model.omit / kinds).abs() < 0.05,
        "漏碼 {}",
        share(fewer)
    );
    assert!(
        (share(more) - model.insert / kinds).abs() < 0.05,
        "多碼 {}",
        share(more)
    );
}

#[test]
fn substitutions_are_equally_likely_physical_neighbours() {
    let keys = physical_letter_keys();
    let centre = |c: char| {
        let &(_, x, y) = keys.iter().find(|&&(d, _, _)| d == c).unwrap();
        (x, y)
    };
    let model = TypoModel {
        rate: 0.3,
        substitute: 1.0,
        omit: 0.0,
        insert: 0.0,
        transpose: 0.0,
        neighbour_share: 1.0,
        ..TypoModel::default()
    };
    let (out, _) = typo_items(&bench(500, 4), &model).unwrap();
    let mut substituted = std::collections::BTreeSet::new();
    for line in out.lines() {
        let (typed, codes, _) = fields(line);
        for (t, c) in typed.concat().chars().zip(codes.concat().chars()) {
            if t != c {
                let ((tx, ty), (cx, cy)) = (centre(t), centre(c));
                let distance = (tx - cx).hypot(ty - cy);
                assert!(
                    distance <= model.neighbour_radius,
                    "{c} → {t} 相隔 {distance}"
                );
                substituted.insert((c, t));
            }
        }
    }
    // 等機率：o 的鄰鍵（i、p、k、l）都抽得到，不偏向最近的。
    for near in ['i', 'p', 'k', 'l'] {
        assert!(substituted.contains(&('o', near)), "o → {near} 沒出現過");
    }
}

#[test]
fn substitutions_land_on_neighbours_at_the_model_share() {
    let keys = physical_letter_keys();
    let centre = |c: char| {
        let &(_, x, y) = keys.iter().find(|&&(d, _, _)| d == c).unwrap();
        (x, y)
    };
    let model = TypoModel {
        rate: 0.5,
        substitute: 1.0,
        omit: 0.0,
        insert: 0.0,
        transpose: 0.0,
        ..TypoModel::default()
    };
    let (out, _) = typo_items(&bench(2000, 4), &model).unwrap();
    let (mut near, mut total) = (0, 0);
    for line in out.lines() {
        let (typed, codes, _) = fields(line);
        for (t, c) in typed.concat().chars().zip(codes.concat().chars()) {
            if t != c {
                let ((tx, ty), (cx, cy)) = (centre(t), centre(c));
                total += 1;
                near += usize::from((tx - cx).hypot(ty - cy) <= model.neighbour_radius);
            }
        }
    }
    let share = near as f64 / total as f64;
    assert!((share - model.neighbour_share).abs() < 0.03, "鄰鍵 {share}");
}

#[test]
fn transpositions_swap_adjacent_keys_across_characters() {
    let model = TypoModel {
        rate: 1.0,
        substitute: 0.0,
        omit: 0.0,
        insert: 0.0,
        transpose: 1.0,
        ..TypoModel::default()
    };
    let (out, _) = typo_items("ab cd\t一二\n", &model).unwrap();
    // 每一鍵都出錯：a↔b、c↔d 兩兩對調；最後一鍵若落單會改成替換，這裡剛好成對。
    assert_eq!(out, "ba dc\tab cd\t一二\n");
    let (out, _) = typo_items("a bc\t一二\n", &model).unwrap();
    // 跨字對調：a 與 b 換位，各留在原本的字。
    assert!(out.starts_with("b a"), "{out}");
}

#[test]
fn bad_lines_are_reported_with_their_number() {
    let error = typo_items("ab\t一\nab cd\t一\n", &TypoModel::default()).unwrap_err();
    assert_eq!(error.number, 2);
    let error = typo_items("ab 一\n", &TypoModel::default()).unwrap_err();
    assert_eq!(error.number, 1);
}

fn letters_page() -> Page {
    typo::touch_page(gboard::layout())
}

const MM_PER_UNIT: f64 = typo::TOUCH_MM_PER_UNIT;

fn touch(input: &str, noise: &TouchNoise) -> (String, Stats) {
    let page = letters_page();
    touch_items(
        input,
        TouchSurface {
            page: &page,
            mm_per_unit: MM_PER_UNIT,
        },
        noise,
    )
    .unwrap()
}

fn taps(line: &str) -> Vec<Vec<(f64, f64)>> {
    let (units, _, _) = fields(line);
    units
        .iter()
        .map(|unit| {
            unit.split(';')
                .map(|tap| {
                    let (x, y) = tap.split_once(',').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

#[test]
fn touch_items_put_one_tap_on_a_letter_key_per_code_letter() {
    let page = letters_page();
    let state = KeyState {
        composing: true,
        shift: Shift::Off,
    };
    let input = bench(100, 5);
    let (out, stats) = touch(&input, &TouchNoise::default());
    assert_eq!(stats.keys, 100 * 10);
    let mut misses = 0;
    for line in out.lines() {
        let (_, codes, _) = fields(line);
        let taps = taps(line);
        assert_eq!(taps.len(), codes.len());
        for (unit, code) in taps.iter().zip(&codes) {
            assert_eq!(unit.len(), code.len(), "{line}");
            for (&(x, y), c) in unit.iter().zip(code.chars()) {
                let index = page.hit(x, y).expect("點在鍵上");
                let Action::Letter(hit) = page.keys[index].tap_action(state) else {
                    panic!("({x}, {y}) 點到的不是字母鍵");
                };
                misses += usize::from(hit != c);
            }
        }
    }
    assert_eq!(misses, stats.key_errors);
    assert!(stats.key_errors > 0, "σ 約 2 mm 時總有點到鄰鍵的");
}

#[test]
fn touch_noise_is_shifted_down_and_towards_the_centre() {
    let page = letters_page();
    let state = KeyState {
        composing: true,
        shift: Shift::Off,
    };
    let centre_of = |c: char| {
        let key = page
            .keys
            .iter()
            .find(|k| k.tap_action(state) == Action::Letter(c))
            .unwrap();
        (
            key.frame.x + key.frame.w / 2.0,
            key.frame.y + key.frame.h / 2.0,
        )
    };
    // 左邊的 q 與右邊的 p 各點 2000 次：平均都在鍵中心下方，q 往右、p 往左。
    let noise = TouchNoise::default();
    let input: String = (0..2000)
        .map(|i| {
            let s = char::from_u32(0x4e00 + i).unwrap();
            format!("q p\t{s}{s}\n")
        })
        .collect();
    let (out, _) = touch(&input, &noise);
    let (mut q, mut p) = ((0.0, 0.0), (0.0, 0.0));
    for line in out.lines() {
        let taps = taps(line);
        q = (q.0 + taps[0][0].0, q.1 + taps[0][0].1);
        p = (p.0 + taps[1][0].0, p.1 + taps[1][0].1);
    }
    let n = 2000.0;
    let ((qx, qy), (px, py)) = ((q.0 / n, q.1 / n), (p.0 / n, p.1 / n));
    let ((qcx, qcy), (pcx, pcy)) = (centre_of('q'), centre_of('p'));
    assert!(
        qy > qcy && py > pcy,
        "往下偏：q {qy} vs {qcy}，p {py} vs {pcy}"
    );
    assert!(qx > qcx, "q 往中心（右）偏：{qx} vs {qcx}");
    assert!(px < pcx, "p 往中心（左）偏：{px} vs {pcx}");
}

#[test]
fn touch_generation_is_deterministic() {
    let input = bench(30, 4);
    let noise = TouchNoise::default();
    assert_eq!(touch(&input, &noise), touch(&input, &noise));
}

#[test]
fn touch_rejects_letters_missing_from_the_layout() {
    let page = letters_page();
    let error = typo::touch_items(
        "z!\t一\n",
        TouchSurface {
            page: &page,
            mm_per_unit: MM_PER_UNIT,
        },
        &TouchNoise::default(),
    )
    .unwrap_err();
    assert_eq!(error.number, 1);
}
