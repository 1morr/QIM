//! 觸控點對鍵的點擊（M4 票 09c）：照票 08 的觸控錯誤模型抽點擊，點到別的鍵就重抽，所以每個點擊都落在要打的鍵上。
//! 觸控模擬的題打錯後重打、每鍵延遲都用它：使用者打的是自己想打的鍵，點擊帶著空間模型的機率（鄰鍵也有份），
//! 鄰鍵讀法才有東西可換（`docs/research/tolerance.md`〈Gboard 式的鄰鍵讀法〉）。
//!
//! 錯誤模型的參數和 `qim_databuild::typo::TouchNoise::default()` 是同一組（plan〈依賴規則〉 不讓本 crate 依賴
//! `qim-databuild`，所以複製一份，`xtask/tests/shared_paths.rs` 比對）；抽法同 `qim_databuild::typo::touch_items`
//! （往下偏、往中心拉、左右半邊反向旋轉、座標取一位小數），只是重抽的條件從「點到字母鍵」改成「點到要打的鍵」。
//! 亂數是 ChaCha8，種子是字串的 SHA-256，所以同一題每次抽到同樣的點。

use std::path::Path;

use qim_core::Key;
use qim_layout::{Action, KeyState, Shift};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

use crate::items::Unit;
use crate::keys::TouchSurface;
use crate::{TOUCH_LAYOUT, TOUCH_MM_PER_UNIT};

/// 觸控錯誤模型的參數（`qim_databuild::typo::TouchNoise` 的預設值，依據見該處與
/// `docs/research/typo-bench-models.md`）。
pub const TOUCH_SIGMA_X_MM: f64 = 1.8;
pub const TOUCH_SIGMA_Y_MM: f64 = 1.5;
/// 鍵盤右半邊的鍵順時針、左半邊逆時針轉這麼多度。
pub const TOUCH_ROTATION_DEG: f64 = 15.0;
/// 平均位置比鍵中心往下多少。
pub const TOUCH_DOWN_MM: f64 = 0.6;
/// 最外側的鍵往鍵盤水平中心拉 `edge_pull_sigma × σx`。
pub const TOUCH_EDGE_PULL_SIGMA: f64 = 0.3;

/// 組字中、Shift 沒開：基準的輸入碼都是小寫字母。
const COMPOSING: KeyState = KeyState {
    composing: true,
    shift: Shift::Off,
};

/// 在觸控佈局上抽「點對鍵」的點擊。
#[derive(Debug, Clone)]
pub struct Taps {
    surface: TouchSurface,
    centre_x: f64,
    /// 字母鍵中心離鍵盤中心最遠的水平距離。
    edge: f64,
}

impl Taps {
    /// 讀佈局檔 `json`（[`TOUCH_LAYOUT`] 的內容）。
    pub fn from_json(json: &str) -> Result<Self, String> {
        let surface = TouchSurface::from_json(json, TOUCH_MM_PER_UNIT)?;
        let keys = &surface.page().keys;
        let (left, right) = keys.iter().fold((f64::MAX, f64::MIN), |(l, r), key| {
            (l.min(key.frame.x), r.max(key.frame.x + key.frame.w))
        });
        let centre_x = (left + right) / 2.0;
        let edge = keys
            .iter()
            .filter(|key| matches!(key.tap_action(COMPOSING), Action::Letter(_)))
            .map(|key| (key.frame.x + key.frame.w / 2.0 - centre_x).abs())
            .fold(0.0, f64::max);
        Ok(Self {
            surface,
            centre_x,
            edge,
        })
    }

    /// 讀 repo 根目錄 `root` 下的 [`TOUCH_LAYOUT`]。
    pub fn load(root: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(root.join(TOUCH_LAYOUT))
            .map_err(|e| format!("讀不到 {TOUCH_LAYOUT}：{e}；先跑 `cargo xtask data`"))?;
        Self::from_json(&json).map_err(|e| format!("{TOUCH_LAYOUT}：{e}"))
    }

    /// 佈局的畫布，題目檔的點擊經它轉成按鍵機率。
    pub fn surface(&self) -> &TouchSurface {
        &self.surface
    }

    fn letter_at(&self, x: f64, y: f64) -> Option<char> {
        let page = self.surface.page();
        let index = page.hit(x, y)?;
        match page.keys[index].tap_action(COMPOSING) {
            Action::Letter(c) => Some(c),
            _ => None,
        }
    }

    /// 點對 `c` 鍵的一下；佈局沒有這個字母鍵的字元（分隔記號）照 `Key::for_input`。
    fn key(&self, c: char, rng: &mut ChaCha8Rng) -> Key {
        let Some(key) = self
            .surface
            .page()
            .keys
            .iter()
            .find(|key| key.tap_action(COMPOSING) == Action::Letter(c))
        else {
            return Key::for_input(c);
        };
        let mm = TOUCH_MM_PER_UNIT;
        let cx = key.frame.x + key.frame.w / 2.0;
        let cy = key.frame.y + key.frame.h / 2.0;
        let pull_mm = if self.edge > 0.0 {
            TOUCH_EDGE_PULL_SIGMA * TOUCH_SIGMA_X_MM * (self.centre_x - cx) / self.edge
        } else {
            0.0
        };
        let mean_x = cx + pull_mm / mm;
        let mean_y = cy + TOUCH_DOWN_MM / mm;
        let side = (cx - self.centre_x).signum() * f64::from(u8::from(cx != self.centre_x));
        let (sin, cos) = (side * TOUCH_ROTATION_DEG).to_radians().sin_cos();
        for _ in 0..10_000 {
            let (a, b) = gaussian_pair(rng);
            let (dx, dy) = (a * TOUCH_SIGMA_X_MM, b * TOUCH_SIGMA_Y_MM);
            let x = round_tenth(mean_x + (dx * cos - dy * sin) / mm);
            let y = round_tenth(mean_y + (dx * sin + dy * cos) / mm);
            if self.letter_at(x, y) == Some(c) {
                return self.surface.tap(x, y).expect("點在字母鍵上");
            }
        }
        panic!("{c:?} 鍵抽了一萬次都沒點到：佈局的鍵太小或畫布尺寸不對");
    }

    /// `input` 的每個字元都點對鍵；`seed` 決定抽到的點（同一個種子每次一樣）。
    pub fn input_keys(&self, seed: &str, input: &str) -> Vec<Key> {
        let mut rng = rng(seed);
        input.chars().map(|c| self.key(c, &mut rng)).collect()
    }

    /// 每個單位的碼都點對鍵（一個單位一組）。
    pub fn unit_keys(&self, seed: &str, units: &[Unit]) -> Vec<Vec<Key>> {
        let mut rng = rng(seed);
        units
            .iter()
            .map(|u| u.code.chars().map(|c| self.key(c, &mut rng)).collect())
            .collect()
    }
}

fn rng(seed: &str) -> ChaCha8Rng {
    let digest = Sha256::digest(format!("touch-clean\0{seed}").as_bytes());
    ChaCha8Rng::from_seed(digest.into())
}

fn uniform(rng: &mut ChaCha8Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

/// Box–Muller，同 `qim_databuild::typo`。
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
