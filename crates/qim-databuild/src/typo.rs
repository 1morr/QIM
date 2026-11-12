//! 打錯碼的基準集合（plan §5.2，M4 票 08）：把基準題的輸入碼照錯誤模型改成「實際按的鍵」，量容錯有沒有用、
//! 會不會把對的改錯。兩種：
//!
//! - 「含打錯碼」（[`typo_items`]，實體鍵盤）：每一鍵以固定機率出錯，四種錯誤（鄰鍵替換、漏碼、多碼、兩碼對調）。
//!   鄰鍵是實體鍵盤上鍵中心在 [`TypoModel::neighbour_radius`] 內的字母鍵，**等機率**；容錯用的鄰鍵先驗
//!   （`qim_layout::PhysicalModel`）是依距離的 Gaussian、按下的鍵佔九成，兩者故意不同（容錯不能對著產生錯誤的
//!   模型調）。
//! - 「觸控模擬」（[`touch_items`]）：照 Fowler 2015 在佈局幾何上每一鍵抽一個點擊，但雜訊照 Goodman 2002：
//!   平均位置往下、往鍵盤中心偏，x、y 的 σ 不同、有旋轉（[`TouchNoise`]）。空間模型（`qim_layout::SpatialModel`）
//!   以觸控範圍為中心、軸對齊、沒有偏移，兩者也故意不同。基準把點擊經空間模型轉成 `Key::Weighted` 再打。
//!
//! 參數與來源在各 `Default`（`docs/research/typo-bench-models.md`）。每題的亂數由題目那一行的 sha256 決定
//! （ChaCha8，可攜、跨平台結果相同），所以同一行不論在檔裡哪裡、別的行怎麼變，產生的錯誤都一樣。

use qim_layout::{
    Action, Device, KeyState, Keyboard, Layout, Orientation, Page, When, physical_letter_keys,
};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

/// 觸控模擬的佈局檔：預設 Gboard 速成佈局（[`crate::gboard::FILE`]），基準讀它把點擊轉成機率；
/// `qim_bench::TOUCH_LAYOUT` 是同一個路徑（plan §1.3 不讓 `qim-bench` 依賴本 crate），`shared_paths` 比對。
pub const TOUCH_LAYOUT: &str = "build/layouts/gboard-quick.json";

/// 觸控模擬假設的畫布尺寸：一單位是 1/160 吋，同網頁（`web/src/touch.ts` 的 `MM_PER_CSS_PX`：手機的 CSS px
/// 約等於 Android 的 dp）；直向畫布 390 單位寬，字母鍵約 5.3 × 7.1 mm（Fowler 2015 的 Nexus 5 是
/// 6.16 × 9.42 mm）。`qim_bench::TOUCH_MM_PER_UNIT` 是同一個值，`shared_paths` 比對。
pub const TOUCH_MM_PER_UNIT: f64 = 25.4 / 160.0;

/// 觸控模擬用的頁：手機直向的鍵盤一開始顯示的頁（字母頁），同殼開鍵盤時（`qim_layout::Keyboard::new`）；
/// 基準照同樣的方式選。
pub fn touch_page(layout: Layout) -> Page {
    let (keyboard, _) = Keyboard::new(layout, TOUCH_WHEN);
    keyboard.page().clone()
}

/// 觸控模擬假設的裝置與方向。
const TOUCH_WHEN: When = When {
    device: Device::Phone,
    orientation: Orientation::Portrait,
};

/// 一份基準題產生的兩種集合。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    pub typo: String,
    pub typo_stats: Stats,
    pub touch: String,
    pub touch_stats: Stats,
}

/// 用預設的模型產生 [`typo_items`] 與 [`touch_items`]。
pub fn generate(bench: &str, page: &Page) -> Result<Generated, BadLine> {
    let (typo, typo_stats) = typo_items(bench, &TypoModel::default())?;
    let surface = TouchSurface {
        page,
        mm_per_unit: TOUCH_MM_PER_UNIT,
    };
    let (touch, touch_stats) = touch_items(bench, surface, &TouchNoise::default())?;
    Ok(Generated {
        typo,
        typo_stats,
        touch,
        touch_stats,
    })
}

/// 實體鍵盤的錯誤模型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypoModel {
    /// 每一個要按的鍵出錯的機率。
    pub rate: f64,
    /// 出錯時四種錯誤的相對比例：替換、漏碼、多碼（多按一個鄰鍵，在前或在後）、兩碼對調（這一鍵和下一鍵）。
    pub substitute: f64,
    pub omit: f64,
    pub insert: f64,
    pub transpose: f64,
    /// 替換時換成鄰鍵的機率（鄰鍵之間等機率）；其餘換成任一個非鄰鍵的字母，等機率。
    pub neighbour_share: f64,
    /// 鍵中心在這個距離內（鍵距）的字母鍵算鄰鍵：同排左右（1）、上下排的兩個斜角（約 1.03 與 1.25）。
    pub neighbour_radius: f64,
}

impl Default for TypoModel {
    /// 依據見 `docs/research/typo-bench-models.md`〈對 QIM 參數的建議〉A：
    /// - `rate` 0.03：Dhakal 2018 的 TextTest 替換、漏打、多打合計 3.12%（打的當下出錯，含之後修正掉的）。
    /// - 比例 0.5 : 0.2 : 0.2 : 0.1：Dhakal 三類 53 : 26 : 21，對調取 Grudin 1983 專家約 7% 進位成 10%，
    ///   從三類按比例扣（自己定的折衷，沒有一份資料給齊四類）。
    /// - `neighbour_share` 0.65：Grudin 新手 75%、專家約 47% 的替換落在緊鄰鍵，取中間。其餘的照 Grudin 偏同一隻手，
    ///   這裡簡化成等機率（自己定）。
    /// - 多碼按的是鄰鍵：Grudin 的專家多打絕大多數是一指碰到兩鍵。
    fn default() -> Self {
        Self {
            rate: 0.03,
            substitute: 0.5,
            omit: 0.2,
            insert: 0.2,
            transpose: 0.1,
            neighbour_share: 0.65,
            neighbour_radius: 1.3,
        }
    }
}

/// 觸控點擊的雜訊（Goodman 2002 的三個觀察：平均位置往下、往鍵盤中心偏；x、y 變異不同；有旋轉），單位毫米。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchNoise {
    pub sigma_x_mm: f64,
    pub sigma_y_mm: f64,
    /// 分布主軸轉幾度：鍵盤右半邊的鍵順時針、左半邊逆時針（畫布的 y 往下），正中間的不轉。
    pub rotation_deg: f64,
    /// 平均位置比鍵中心往下多少。
    pub down_mm: f64,
    /// 往鍵盤水平中心拉：最外側的鍵（中心離鍵盤中心最遠的字母鍵）拉 `edge_pull_sigma × σx`，其餘照它到中心的
    /// 水平距離線性縮小、正中間的不拉。
    pub edge_pull_sigma: f64,
}

impl Default for TouchNoise {
    /// 依據見 `docs/research/typo-bench-models.md`〈對 QIM 參數的建議〉B：
    /// - `sigma_x_mm` 1.8、`sigma_y_mm` 1.5：Azenkot & Zhai 2012 食指的 1.81、1.48 mm（Yi 2017 轉引）。空間模型的初值
    ///   取自 Fowler 2015 的 1.97、1.88 mm，兩者不同。
    /// - `down_mm` 0.6：Azenkot & Zhai 食指往下 0.59 mm（Yi 2017 轉引）。
    /// - `edge_pull_sigma` 0.3：鍵盤上查不到數字；Henze 2011 在螢幕角落的偏移是 SD 的 0.18–0.55 倍，取偏低的 0.3
    ///   （自己定）。
    /// - `rotation_deg` 15：查不到數字，Goodman 只說有旋轉；左右半邊反向照拇指的弧形（自己定）。
    fn default() -> Self {
        Self {
            sigma_x_mm: 1.8,
            sigma_y_mm: 1.5,
            rotation_deg: 15.0,
            down_mm: 0.6,
            edge_pull_sigma: 0.3,
        }
    }
}

/// 觸控模擬的佈局：基準把同一個佈局經空間模型轉成機率，兩邊要用同一個畫布尺寸。
#[derive(Debug, Clone, Copy)]
pub struct TouchSurface<'a> {
    pub page: &'a Page,
    /// 畫布一單位幾毫米。
    pub mm_per_unit: f64,
}

/// 產生的集合的統計，`cargo xtask data` 印出來。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub items: usize,
    /// 有任何一鍵打錯的題。
    pub items_with_errors: usize,
    /// 原本要按的鍵數。
    pub keys: usize,
    /// 出錯的鍵：含打錯碼是讓實際按的鍵變了的錯誤次數，觸控是點到別的字母鍵的點擊。
    pub key_errors: usize,
}

impl Stats {
    pub fn key_error_rate(&self) -> f64 {
        ratio(self.key_errors, self.keys)
    }

    pub fn item_error_rate(&self) -> f64 {
        ratio(self.items_with_errors, self.items)
    }
}

fn ratio(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

/// 讀不懂的基準行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadLine {
    pub number: usize,
    pub reason: String,
}

impl std::fmt::Display for BadLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "第 {} 行：{}", self.number, self.reason)
    }
}

impl std::error::Error for BadLine {}

/// 由基準題（`<逐字速成碼，空格分隔><TAB><句子>`，`bench/colloquial/<名>.tsv` 與 dev 題的格式）產生
/// 「含打錯碼」的題：`<逐字實際按的鍵，空格分隔><TAB><逐字速成碼><TAB><句子>`。實際按的鍵仍照原本的字分組
/// （對調跨字時兩個字各拿到對方的鍵），漏光了的字是空字串。
pub fn typo_items(bench: &str, model: &TypoModel) -> Result<(String, Stats), BadLine> {
    let neighbours = Neighbours::new(model.neighbour_radius);
    let mut out = String::new();
    let mut stats = Stats::default();
    for (number, line) in lines(bench) {
        let (codes, sentence) = split(number, line)?;
        let mut rng = rng_for("typo", line);
        let keys: Vec<(usize, char)> = codes
            .iter()
            .enumerate()
            .flat_map(|(unit, code)| code.chars().map(move |c| (unit, c)))
            .collect();
        let mut typed: Vec<(usize, char)> = Vec::with_capacity(keys.len() + 2);
        let mut errors = 0;
        let mut i = 0;
        while i < keys.len() {
            let (unit, c) = keys[i];
            i += 1;
            if uniform(&mut rng) >= model.rate {
                typed.push((unit, c));
                continue;
            }
            let total = model.substitute + model.omit + model.insert + model.transpose;
            let mut pick = uniform(&mut rng) * total;
            let mut is = |weight: f64| {
                let hit = pick < weight;
                pick -= weight;
                hit
            };
            // 實際按的鍵有沒有變：同字母對調（`oo`）或找不到別的鍵可換時沒變，不算出錯。
            let changed = if is(model.substitute) {
                let replaced = neighbours.substitute(c, model.neighbour_share, &mut rng);
                typed.push((unit, replaced));
                replaced != c
            } else if is(model.omit) {
                // 漏碼：這一鍵不按。
                true
            } else if is(model.insert) {
                let extra = neighbours.pick(c, &mut rng);
                if uniform(&mut rng) < 0.5 {
                    typed.push((unit, extra));
                    typed.push((unit, c));
                } else {
                    typed.push((unit, c));
                    typed.push((unit, extra));
                }
                true
            } else if let Some(&(next_unit, next)) = keys.get(i) {
                // 兩碼對調：這一鍵和下一鍵換位置，各自留在原本那個字的位置上。
                typed.push((unit, next));
                typed.push((next_unit, c));
                i += 1;
                next != c
            } else {
                // 最後一鍵沒有下一鍵可以對調，改成替換。
                let replaced = neighbours.substitute(c, model.neighbour_share, &mut rng);
                typed.push((unit, replaced));
                replaced != c
            };
            errors += usize::from(changed);
        }
        let typed_units: Vec<String> = (0..codes.len())
            .map(|unit| {
                typed
                    .iter()
                    .filter(|&&(u, _)| u == unit)
                    .map(|&(_, c)| c)
                    .collect()
            })
            .collect();
        stats.items += 1;
        stats.keys += keys.len();
        stats.key_errors += errors;
        stats.items_with_errors += usize::from(errors > 0);
        out.push_str(&format!(
            "{}\t{}\t{sentence}\n",
            typed_units.join(" "),
            codes.join(" ")
        ));
    }
    Ok((out, stats))
}

/// 由基準題產生「觸控模擬」的題：`<逐字的點擊，空格分隔><TAB><逐字速成碼><TAB><句子>`。一個字的點擊以 `;`
/// 分隔，每個點擊是畫布座標 `x,y`（一位小數）。每一鍵在 `surface` 上照 [`TouchNoise`] 抽一個點擊；
/// 點在縫上、字母鍵以外的鍵或鍵盤外就重抽（Fowler 2015 對落在所有鍵外的點擊也是重抽）。
pub fn touch_items(
    bench: &str,
    surface: TouchSurface<'_>,
    noise: &TouchNoise,
) -> Result<(String, Stats), BadLine> {
    let page = surface.page;
    // 觸控模擬只打字母：組字中、Shift 沒開（基準的輸入碼都是小寫）。
    let state = KeyState {
        composing: true,
        shift: Default::default(),
    };
    let letter_at = |x: f64, y: f64| {
        let index = page.hit(x, y)?;
        match page.keys[index].tap_action(state) {
            Action::Letter(c) => Some(c),
            _ => None,
        }
    };
    let (left, right) = page.keys.iter().fold((f64::MAX, f64::MIN), |(l, r), key| {
        (l.min(key.frame.x), r.max(key.frame.x + key.frame.w))
    });
    let centre_x = (left + right) / 2.0;
    // 最外側的字母鍵中心離鍵盤中心多遠：往中心拉的偏移照它線性縮放。
    let edge = page
        .keys
        .iter()
        .filter(|key| matches!(key.tap_action(state), Action::Letter(_)))
        .map(|key| (key.frame.x + key.frame.w / 2.0 - centre_x).abs())
        .fold(0.0, f64::max);
    let mm = surface.mm_per_unit;
    let mut out = String::new();
    let mut stats = Stats::default();
    for (number, line) in lines(bench) {
        let (codes, sentence) = split(number, line)?;
        let mut rng = rng_for("touch", line);
        let mut errors = 0;
        let mut keys = 0;
        let mut units = Vec::with_capacity(codes.len());
        for code in &codes {
            let mut taps = Vec::with_capacity(code.len());
            for c in code.chars() {
                let key = page
                    .keys
                    .iter()
                    .find(|key| key.tap_action(state) == Action::Letter(c))
                    .ok_or_else(|| BadLine {
                        number,
                        reason: format!("佈局沒有 {c:?} 鍵"),
                    })?;
                let cx = key.frame.x + key.frame.w / 2.0;
                let cy = key.frame.y + key.frame.h / 2.0;
                let pull_mm = if edge > 0.0 {
                    noise.edge_pull_sigma * noise.sigma_x_mm * (centre_x - cx) / edge
                } else {
                    0.0
                };
                let mean_x = cx + pull_mm / mm;
                let mean_y = cy + noise.down_mm / mm;
                let side = (cx - centre_x).signum() * f64::from(u8::from(cx != centre_x));
                let (sin, cos) = (side * noise.rotation_deg).to_radians().sin_cos();
                let mut tries = 0;
                let (x, y, hit) = loop {
                    tries += 1;
                    if tries > 1000 {
                        return Err(BadLine {
                            number,
                            reason: format!("{c:?} 抽了 1000 次都沒點到字母鍵"),
                        });
                    }
                    let (a, b) = gaussian_pair(&mut rng);
                    let (dx, dy) = (a * noise.sigma_x_mm, b * noise.sigma_y_mm);
                    let x = round_tenth(mean_x + (dx * cos - dy * sin) / mm);
                    let y = round_tenth(mean_y + (dx * sin + dy * cos) / mm);
                    if let Some(hit) = letter_at(x, y) {
                        break (x, y, hit);
                    }
                };
                keys += 1;
                errors += usize::from(hit != c);
                taps.push(format!("{x:.1},{y:.1}"));
            }
            units.push(taps.join(";"));
        }
        stats.items += 1;
        stats.keys += keys;
        stats.key_errors += errors;
        stats.items_with_errors += usize::from(errors > 0);
        out.push_str(&format!(
            "{}\t{}\t{sentence}\n",
            units.join(" "),
            codes.join(" ")
        ));
    }
    Ok((out, stats))
}

/// 實體鍵盤每個字母鍵的鄰鍵（鍵位取自 `qim_layout`，不另寫一份 QWERTY）。
struct Neighbours(Vec<(char, Vec<char>)>);

impl Neighbours {
    fn new(radius: f64) -> Self {
        let keys = physical_letter_keys();
        Neighbours(
            keys.iter()
                .map(|&(c, x, y)| {
                    let near = keys
                        .iter()
                        .filter(|&&(d, kx, ky)| d != c && (kx - x).hypot(ky - y) <= radius)
                        .map(|&(d, _, _)| d)
                        .collect();
                    (c, near)
                })
                .collect(),
        )
    }

    /// `c` 的一個鄰鍵，等機率；`c` 不是字母鍵或沒有鄰鍵時是它自己。
    fn pick(&self, c: char, rng: &mut ChaCha8Rng) -> char {
        match self.0.iter().find(|(d, _)| *d == c) {
            Some((_, near)) => choose(near, rng).unwrap_or(c),
            None => c,
        }
    }

    /// 替換 `c`：機率 `neighbour_share` 換成鄰鍵（[`Neighbours::pick`]），其餘換成任一個非鄰鍵的字母。
    fn substitute(&self, c: char, neighbour_share: f64, rng: &mut ChaCha8Rng) -> char {
        if uniform(rng) < neighbour_share {
            return self.pick(c, rng);
        }
        let near = self
            .0
            .iter()
            .find(|(d, _)| *d == c)
            .map_or(&[][..], |(_, near)| near.as_slice());
        let far: Vec<char> = self
            .0
            .iter()
            .map(|&(d, _)| d)
            .filter(|&d| d != c && !near.contains(&d))
            .collect();
        choose(&far, rng).unwrap_or(c)
    }
}

/// 等機率挑一個；空的時是 `None`。
fn choose(items: &[char], rng: &mut ChaCha8Rng) -> Option<char> {
    let index = (uniform(rng) * items.len() as f64) as usize;
    items.get(index.min(items.len().saturating_sub(1))).copied()
}

/// 非空行與它的行號（從 1 起）。
fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line))
        .filter(|(_, line)| !line.trim().is_empty())
}

fn split(number: usize, line: &str) -> Result<(Vec<&str>, &str), BadLine> {
    let bad = |reason: &str| BadLine {
        number,
        reason: reason.into(),
    };
    let (codes, sentence) = line
        .split_once('\t')
        .ok_or_else(|| bad("少了 tab：要有輸入碼與句子兩欄"))?;
    let codes: Vec<&str> = codes.split(' ').collect();
    if codes.iter().any(|c| c.is_empty()) || codes.len() != sentence.chars().count() {
        return Err(bad("要一個字一個非空的碼"));
    }
    Ok((codes, sentence))
}

/// 這一行的亂數：種子是 `<種類>\0<整行>` 的 sha256。
fn rng_for(kind: &str, line: &str) -> ChaCha8Rng {
    let digest = Sha256::digest(format!("{kind}\0{line}").as_bytes());
    ChaCha8Rng::from_seed(digest.into())
}

/// [0, 1) 的均勻分布：取 53 位元。
fn uniform(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

/// 兩個獨立的標準常態（Box–Muller）。自己寫而不用 `rand_distr`：它的數值在小版本間可能變，產生的基準
/// 檔就跟著變。
fn gaussian_pair(rng: &mut ChaCha8Rng) -> (f64, f64) {
    let u1 = 1.0 - uniform(rng);
    let u2 = uniform(rng);
    let r = (-2.0 * u1.ln()).sqrt();
    let (sin, cos) = (std::f64::consts::TAU * u2).sin_cos();
    (r * cos, r * sin)
}

fn round_tenth(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}
