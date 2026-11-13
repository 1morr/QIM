//! Gboard 式的觸控鄰鍵讀法實驗（M4 票 09b）：觸控的按鍵機率裡的鄰鍵字母換進輸入碼，換出來的讀法插在候選列
//! 的指定格（原樣讀法照舊當首選，點擊模稜兩可時才讓鄰鍵讀法當首選）。量觸控的題的首選、修正次數、改錯、
//! 想打的字詞在第一頁的比例與名次、沒打錯的題被擠後的比例，與每鍵延遲，對照 `.scratch/m4/spec.md`
//! 〈容錯採用門檻〉〈每鍵延遲上限〉。機制在 `330f4b4` 的核心裡預設關（`Engine::with_neighbor_readings`），量完
//! 之後的 commit 刪掉，所以這個 crate 只在 `330f4b4` 上編得過（README〈怎麼重跑〉）。
//!
//! 用法（repo 根目錄先跑 `cargo xtask data`），`cargo run --release -- <指令> [選項]`：
//! - `weights`：dev 觸控題的按鍵機率長怎樣（第二高 ÷ 最高的分布），定 `floor` 的範圍用，不看結果。
//! - `dev`：只看 dev 切分，照 README 的三步挑放法、`floor`、模稜兩可的門檻，印出 `test` 要的參數。
//! - `combo [設定]`：只在 dev 打一種設定，含觸控點對鍵的沒打錯集合。
//! - `test [設定]`：設定定案後在 test 切分跑一次，逐條對照門檻。
//! - `latency [設定]`：原生每鍵延遲，照觸控點對鍵的按鍵機率送，關與開交替，空白與學過、含萬用鍵的各一組。
//!
//! 設定：`--slot N --limit N --floor X --letters N --weight X --ambiguous X|none`；`--root <repo 根目錄>`
//! 預設是 `../../..`。

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;
use std::time::Instant;

use qim_bench::{
    Item, TouchSurface, learned_model, lefts_to_first_segment, load_corpora, load_engine,
    load_typo_sets, parse_context, parse_corpus, parse_fixed, parse_touch, parse_typo, percentile,
    physical_key, top_char,
};
use qim_core::{Candidate, Engine, Key, NeighborReadings, Session, Snapshot, UserModel, Weights};
use qim_databuild::typo::{TouchNoise, touch_page};
use qim_layout::{Action, KeyState, Layout, Page, Shift};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{Rng, SeedableRng};
use sha2::{Digest, Sha256};

/// 口語的兩個語料：打錯碼的集合只有它們（票 08）。
const COLLOQUIAL: [&str; 2] = ["common-voice-yue", "hkcancor"];
/// 〈容錯採用門檻〉的數字（觸控那幾條）。
const FALSE_CORRECTION_LIMIT: f64 = 0.010;
const TOUCH_ACCURACY: f64 = 0.06;
const FEWER_CORRECTIONS: f64 = 0.10;
const P_LIMIT: f64 = 0.05;
const CLEAN_WRONG_LIMIT: f64 = 0.005;
const CLEAN_CORRECTION_LIMIT: f64 = 0.01;
/// 候選列一頁幾個（spec〈共同規則〉）。
const PAGE: usize = 9;
/// 找想打的字詞時最多翻幾頁。
const PAGES: usize = 5;

// ---------------------------------------------------------------- 設定

#[derive(Debug, Clone, Copy, PartialEq)]
struct Config {
    slot: usize,
    limit: usize,
    floor: f32,
    letters: usize,
    weight: f64,
    ambiguous: Option<f32>,
}

impl Default for Config {
    /// dev 第一步的起點（README〈dev 怎麼挑〉）。
    fn default() -> Self {
        Config {
            slot: 2,
            limit: 2,
            floor: 0.05,
            letters: 2,
            weight: 1.0,
            ambiguous: None,
        }
    }
}

impl Config {
    fn readings(&self) -> NeighborReadings {
        NeighborReadings {
            floor: self.floor,
            letters: self.letters,
            weight: self.weight,
            slot: self.slot,
            limit: self.limit,
            ambiguous: self.ambiguous,
            accepts: is_touch,
        }
    }

    fn describe(&self) -> String {
        format!(
            "第 {} 格起最多 {} 個、floor {}、ambiguous {}",
            self.slot,
            self.limit,
            self.floor,
            self.ambiguous.map_or("關".into(), |a| a.to_string())
        )
    }

    fn args(&self) -> String {
        format!(
            "--slot {} --limit {} --floor {} --letters {} --weight {} --ambiguous {}",
            self.slot,
            self.limit,
            self.floor,
            self.letters,
            self.weight,
            self.ambiguous.map_or("none".into(), |a| a.to_string())
        )
    }
}

/// 實體鍵盤送的按鍵機率（`qim_bench::physical_key`）不算觸控：照票面，實體鍵盤照舊只取最高的字母。核心分不出
/// 兩者，實驗用「是不是某個字母的實體鍵機率」來分。
fn is_touch(weights: &Weights) -> bool {
    static PHYSICAL: OnceLock<Vec<Weights>> = OnceLock::new();
    let physical = PHYSICAL.get_or_init(|| {
        ('a'..='z')
            .filter_map(|c| match physical_key(c) {
                Key::Weighted(w) => Some(w),
                _ => None,
            })
            .collect()
    });
    !physical.contains(weights)
}

// ---------------------------------------------------------------- 觸控點對鍵的點擊

/// 照票 08 的觸控錯誤模型（`qim_databuild::typo::TouchNoise` 的預設值，`docs/research/typo-bench-models.md`）
/// 抽點擊，點到別的鍵就重抽：「打對了」的觸控點擊。抽法同 `qim_databuild::typo::touch_items`（往下偏、往中心拉、
/// 左右半邊反向旋轉，座標取一位小數），只是重抽的條件從「點到字母鍵」改成「點到要打的鍵」。
struct Taps {
    page: Page,
    surface: TouchSurface,
    mm: f64,
    centre_x: f64,
    edge: f64,
    noise: TouchNoise,
}

const COMPOSING: KeyState = KeyState {
    composing: true,
    shift: Shift::Off,
};

impl Taps {
    fn new(root: &Path) -> Self {
        let json = read(&root.join(qim_bench::TOUCH_LAYOUT));
        let page = touch_page(Layout::from_json(&json).unwrap());
        let surface = TouchSurface::from_json(&json, qim_bench::TOUCH_MM_PER_UNIT).unwrap();
        let (left, right) = page.keys.iter().fold((f64::MAX, f64::MIN), |(l, r), key| {
            (l.min(key.frame.x), r.max(key.frame.x + key.frame.w))
        });
        let centre_x = (left + right) / 2.0;
        let edge = page
            .keys
            .iter()
            .filter(|key| matches!(key.tap_action(COMPOSING), Action::Letter(_)))
            .map(|key| (key.frame.x + key.frame.w / 2.0 - centre_x).abs())
            .fold(0.0, f64::max);
        Taps {
            page,
            surface,
            mm: qim_bench::TOUCH_MM_PER_UNIT,
            centre_x,
            edge,
            noise: TouchNoise::default(),
        }
    }

    fn letter_at(&self, x: f64, y: f64) -> Option<char> {
        let index = self.page.hit(x, y)?;
        match self.page.keys[index].tap_action(COMPOSING) {
            Action::Letter(c) => Some(c),
            _ => None,
        }
    }

    /// 點對 `c` 鍵的一下；佈局沒有的字元（分隔記號）照 `Key::for_input`。
    fn key(&self, c: char, rng: &mut ChaCha8Rng) -> Key {
        let Some(key) = self
            .page
            .keys
            .iter()
            .find(|key| key.tap_action(COMPOSING) == Action::Letter(c))
        else {
            return Key::for_input(c);
        };
        let noise = &self.noise;
        let cx = key.frame.x + key.frame.w / 2.0;
        let cy = key.frame.y + key.frame.h / 2.0;
        let pull_mm = if self.edge > 0.0 {
            noise.edge_pull_sigma * noise.sigma_x_mm * (self.centre_x - cx) / self.edge
        } else {
            0.0
        };
        let mean_x = cx + pull_mm / self.mm;
        let mean_y = cy + noise.down_mm / self.mm;
        let side = (cx - self.centre_x).signum() * f64::from(u8::from(cx != self.centre_x));
        let (sin, cos) = (side * noise.rotation_deg).to_radians().sin_cos();
        for _ in 0..10_000 {
            let (a, b) = gaussian_pair(rng);
            let (dx, dy) = (a * noise.sigma_x_mm, b * noise.sigma_y_mm);
            let x = round_tenth(mean_x + (dx * cos - dy * sin) / self.mm);
            let y = round_tenth(mean_y + (dx * sin + dy * cos) / self.mm);
            if self.letter_at(x, y) == Some(c) {
                return self.surface.tap(x, y).expect("點在字母鍵上");
            }
        }
        panic!("{c:?} 抽了一萬次都沒點到");
    }

    /// 每個單位的碼都點對鍵；`seed` 決定抽到的點（同一題每次一樣）。
    fn clean(&self, seed: &str, item: &Item) -> Vec<Vec<Key>> {
        let digest = Sha256::digest(format!("touch-clean\0{seed}").as_bytes());
        let mut rng = ChaCha8Rng::from_seed(digest.into());
        item.units
            .iter()
            .map(|u| u.code.chars().map(|c| self.key(c, &mut rng)).collect())
            .collect()
    }
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

// ---------------------------------------------------------------- 題目

/// 一題：第一次按的鍵（`typed`）與發現打錯後重打的鍵（`retype`，每個單位一組）。
#[derive(Clone)]
struct Task {
    item: Item,
    typed: Vec<Vec<Key>>,
    retype: Vec<Vec<Key>>,
    /// 觸控模擬的題：`typed` 每一鍵最高的字母都等於原本的碼才是沒打錯；其餘集合一律算沒打錯。
    clean: bool,
}

struct Set {
    name: String,
    tasks: Vec<Task>,
}

fn is_clean(item: &Item, typed: &[Vec<Key>]) -> bool {
    item.units.len() == typed.len()
        && item
            .units
            .iter()
            .zip(typed)
            .all(|(unit, keys)| keys.iter().map(top_char).eq(unit.code.chars().map(Some)))
}

/// 觸控模擬的題：第一次照集合的點擊，重打點對鍵。
fn touch_set(name: String, items: Vec<Item>, taps: &Taps) -> Set {
    let tasks = items
        .into_iter()
        .map(|item| {
            let typed = item.typed.clone().expect("觸控模擬的題");
            let retype = taps.clean(&format!("{name}\0{}", item.line), &item);
            Task {
                clean: is_clean(&item, &typed),
                item,
                typed,
                retype,
            }
        })
        .collect();
    Set { name, tasks }
}

/// 沒打錯的集合改成觸控點對鍵送（第一次與重打同一組點擊）。
fn touch_clean_set(name: String, items: Vec<Item>, taps: &Taps) -> Set {
    let tasks = items
        .into_iter()
        .map(|item| {
            let typed = taps.clean(&format!("{name}\0{}", item.line), &item);
            Task {
                item,
                retype: typed.clone(),
                typed,
                clean: true,
            }
        })
        .collect();
    Set { name, tasks }
}

/// 照實體鍵送的集合（含打錯碼與原本沒打錯的集合，設計上不適用，只核對開關前後相同）。
fn physical_set(name: String, items: Vec<Item>) -> Set {
    let tasks = items
        .into_iter()
        .map(|item| {
            let retype: Vec<Vec<Key>> = item
                .units
                .iter()
                .map(|u| u.code.chars().map(physical_key).collect())
                .collect();
            let typed = item.typed.clone().unwrap_or_else(|| retype.clone());
            Task {
                clean: is_clean(&item, &typed),
                item,
                typed,
                retype,
            }
        })
        .collect();
    Set { name, tasks }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("讀不到 {}：{e}；先跑 cargo xtask data", path.display()))
}

fn surface(root: &Path) -> TouchSurface {
    TouchSurface::from_json(
        &read(&root.join(qim_bench::TOUCH_LAYOUT)),
        qim_bench::TOUCH_MM_PER_UNIT,
    )
    .unwrap()
}

/// 有前文的題打兩次：給前文、不給（同基準）。
fn with_and_without_context(name: &str, items: Vec<Item>) -> [(String, Vec<Item>); 2] {
    let without = items
        .iter()
        .map(|i| Item {
            context: None,
            ..i.clone()
        })
        .collect();
    let [given, not_given] = qim_bench::CONTEXT_SUFFIXES;
    [
        (format!("{name}{given}"), items),
        (format!("{name}{not_given}"), without),
    ]
}

/// 一個切分的題：觸控模擬、含打錯碼（實體）、沒打錯的集合（原本的與改成觸控的）、固定題。
struct Split {
    touch: Vec<Set>,
    physical_typo: Vec<Set>,
    clean: Vec<(String, Vec<Item>)>,
    fixed: Option<Vec<Item>>,
}

fn dev_split(root: &Path) -> Split {
    let surface = surface(root);
    let taps = Taps::new(root);
    let mut touch = Vec::new();
    let mut physical_typo = Vec::new();
    for name in COLLOQUIAL {
        let text = read(&root.join(qim_bench::dev_touch_items(name)));
        touch.push(touch_set(
            format!("{name}{}", qim_bench::TOUCH_SUFFIX),
            parse_touch(&text, &surface).unwrap(),
            &taps,
        ));
        let text = read(&root.join(qim_bench::dev_typo_items(name)));
        physical_typo.push(physical_set(
            format!("{name}{}", qim_bench::TYPO_SUFFIX),
            parse_typo(&text).unwrap(),
        ));
    }
    let corpora: Vec<String> = load_corpora(root)
        .unwrap()
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    let mut clean = Vec::new();
    for name in &corpora {
        let items = parse_corpus(&read(&root.join(qim_bench::dev_items(name)))).unwrap();
        clean.push((name.clone(), items));
    }
    for name in &corpora {
        let path = root.join(qim_bench::dev_context_items(name));
        if path.exists() {
            clean.extend(with_and_without_context(
                name,
                parse_context(&read(&path)).unwrap(),
            ));
        }
    }
    Split {
        touch,
        physical_typo,
        clean,
        fixed: None,
    }
}

fn test_split(root: &Path) -> Split {
    let taps = Taps::new(root);
    let mut touch = Vec::new();
    let mut physical_typo = Vec::new();
    for (name, items) in load_typo_sets(root).unwrap() {
        if name.ends_with(qim_bench::TOUCH_SUFFIX) {
            touch.push(touch_set(name, items, &taps));
        } else {
            physical_typo.push(physical_set(name, items));
        }
    }
    let mut clean: Vec<(String, Vec<Item>)> = load_corpora(root).unwrap();
    for dir in qim_bench::CONTEXT_DIRS {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "tsv"))
            .collect();
        files.sort();
        for path in files {
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            clean.extend(with_and_without_context(
                &name,
                parse_context(&read(&path)).unwrap(),
            ));
        }
    }
    let fixed = parse_fixed(&read(&root.join(qim_bench::FIXED))).unwrap();
    Split {
        touch,
        physical_typo,
        clean,
        fixed: Some(fixed),
    }
}

// ---------------------------------------------------------------- 打一題

/// 一題的結果。
#[derive(Debug, Clone)]
struct Outcome {
    line: usize,
    chars: usize,
    correct: bool,
    corrections: usize,
    clean: bool,
    /// 按完第一次的鍵、不選字時的候選，最多 [`PAGES`] 頁（第一個是首選）。
    list: Vec<String>,
}

/// 逐鍵按完 `keys`、不選字：最多 [`PAGES`] 頁的候選。有 `context` 時先給前文。
fn type_through(engine: &Engine, context: Option<&str>, keys: &[Vec<Key>]) -> Vec<String> {
    let mut session = Session::default();
    if let Some(context) = context {
        session.set_context(context);
    }
    let mut user = UserModel::default();
    let mut snapshot = Snapshot::default();
    for key in keys.iter().flatten() {
        snapshot = session.handle(engine, &mut user, key.clone());
    }
    let mut list: Vec<String> = Vec::new();
    for page in 0..PAGES {
        list.extend(snapshot.candidates.iter().map(|c| c.text.clone()));
        if !snapshot.has_more || page + 1 == PAGES {
            break;
        }
        snapshot = session.handle(engine, &mut user, Key::PageDown);
    }
    list
}

/// `qim_bench` 的修正次數模擬（`crates/qim-bench/src/score.rs` 的 `simulate`，學習關）的複本，只差在照碼重打
/// 的鍵換成 `task.retype`（觸控的題重打也點觸控，協調者 2026-11-13 要求；原本一律送實體鍵，會低估觸控的代價）。
/// 規則見原本的註解：首選接得上就空白、否則選這頁接得上的最長候選、都接不上就翻頁；打錯的鍵組出的這頁接不上
/// 時 Esc 照碼重打剩下的字；卡住時一個單位一個單位打。
fn simulate(engine: &Engine, task: &Task) -> usize {
    let item = &task.item;
    let context = item.context.as_deref();
    let units = item.units.as_slice();
    let mut first = Some(task.typed.as_slice());
    let mistyped = !task.clean;
    let mut in_typo_round = false;
    let mut session = Session::default();
    let mut user = UserModel::default();
    let mut corrections = 0;
    let mut press = |session: &mut Session, key: Key| {
        let snapshot = session.handle(engine, &mut user, key);
        if snapshot.commit.is_some() {
            user = UserModel::default();
        }
        snapshot
    };
    let target: String = units.iter().map(|u| u.text.as_str()).collect();
    let starts: Vec<usize> = units
        .iter()
        .scan(0, |at, u| {
            let start = *at;
            *at += u.text.chars().count();
            Some(start)
        })
        .collect();
    let mut done = 0;
    let mut one_by_one = false;
    let mut snapshot = Snapshot::default();
    loop {
        let rest: String = target.chars().skip(done).collect();
        if rest.is_empty() {
            return corrections;
        }
        if snapshot.composition.input.is_empty() {
            let Some(next) = starts.iter().position(|&s| s == done) else {
                return corrections;
            };
            let end = if one_by_one { next + 1 } else { units.len() };
            if let Some(context) = context {
                let typed: String = target.chars().take(done).collect();
                session.set_context(&format!("{context}{typed}"));
            }
            let keys = match first.take() {
                Some(typed) => {
                    in_typo_round = mistyped;
                    typed
                }
                None => {
                    if std::mem::take(&mut in_typo_round) {
                        corrections += 1;
                    }
                    task.retype.as_slice()
                }
            };
            for key in keys[next..end].iter().flatten() {
                snapshot = press(&mut session, key.clone());
            }
            if snapshot.composition.input.is_empty() {
                if in_typo_round {
                    continue;
                }
                return corrections;
            }
            continue;
        }
        let selected = snapshot.composition.selected_text();
        let Some(open) = rest.strip_prefix(selected.as_str()) else {
            return corrections;
        };
        let connects = |c: &Candidate| !c.text.is_empty() && open.starts_with(&c.text);
        let chosen = |index: usize| snapshot.candidates[index].text.clone();
        let (key, text) = if snapshot.page == 0 && snapshot.candidates.first().is_some_and(connects)
        {
            (Key::Space, Some(chosen(0)))
        } else if let Some(index) = longest_connecting(&snapshot.candidates, connects) {
            corrections += 1;
            let digit = u8::try_from(index + 1).expect("每頁最多 9 個候選");
            (Key::Digit(digit), Some(chosen(index)))
        } else if std::mem::take(&mut in_typo_round) {
            corrections += 1;
            (Key::Escape, None)
        } else if snapshot.has_more {
            corrections += 1;
            (Key::PageDown, None)
        } else if !one_by_one && starts.iter().filter(|&&s| s >= done).count() > 1 {
            corrections += 1;
            one_by_one = true;
            (Key::Escape, None)
        } else {
            return corrections;
        };
        snapshot = press(&mut session, key);
        let Some(text) = text else {
            continue;
        };
        let expected = format!("{selected}{text}");
        match &snapshot.commit {
            Some(commit) if *commit == expected => done += commit.chars().count(),
            None if snapshot.composition.selected_text() == expected => {}
            _ => return corrections,
        }
    }
}

fn longest_connecting(
    candidates: &[Candidate],
    connects: impl Fn(&Candidate) -> bool,
) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| connects(c))
        .max_by_key(|(i, c)| (c.text.chars().count(), std::cmp::Reverse(*i)))
        .map(|(i, _)| i)
}

fn run_task(engine: &Engine, task: &Task) -> Outcome {
    let item = &task.item;
    let list = type_through(engine, item.context.as_deref(), &task.typed);
    let expected = item.expected();
    Outcome {
        line: item.line,
        chars: expected.chars().count(),
        correct: list.first().is_some_and(|t| *t == expected),
        corrections: simulate(engine, task),
        clean: task.clean,
        list,
    }
}

/// 題目分給各核心並行打（每題互不影響）。
fn score(engine: &Engine, set: &Set) -> Vec<Outcome> {
    let threads = thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = set.tasks.len().div_ceil(threads).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = set
            .tasks
            .chunks(chunk)
            .map(|tasks| {
                s.spawn(move || {
                    tasks
                        .iter()
                        .map(|t| run_task(engine, t))
                        .collect::<Vec<Outcome>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect::<Vec<Outcome>>()
    })
}

// ---------------------------------------------------------------- 比較

/// 想打的字詞在 `list` 的名次（從 1 起）；最多翻 [`PAGES`] 頁都沒有是 `None`。
fn rank_of(list: &[String], text: &str) -> Option<usize> {
    list.iter().position(|t| t == text).map(|i| i + 1)
}

/// 沒打錯的題：關的時候第一步要選的候選（首選接得上就是首選，否則第一頁接得上的最長候選），與它在關、開的
/// 名次。第一頁沒有接得上的是 `None`（這題本來就要翻頁或清掉，不算擠）。
fn crowding(expected: &str, off: &[String], on: &[String]) -> Option<(usize, Option<usize>)> {
    let first_page = &off[..off.len().min(PAGE)];
    let index = if off
        .first()
        .is_some_and(|t| expected.starts_with(t.as_str()))
    {
        0
    } else {
        first_page
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.is_empty() && expected.starts_with(t.as_str()))
            .max_by_key(|(i, t)| (t.chars().count(), std::cmp::Reverse(*i)))?
            .0
    };
    Some((index + 1, rank_of(on, &off[index])))
}

/// 被擠的統計：題數、名次往後、被擠出第一頁。
#[derive(Debug, Clone, Copy, Default)]
struct Crowd {
    items: usize,
    later: usize,
    off_page: usize,
}

impl Crowd {
    fn add(&mut self, expected: &str, off: &[String], on: &[String]) {
        if let Some((before, after)) = crowding(expected, off, on) {
            self.items += 1;
            self.later += usize::from(after.is_none_or(|a| a > before));
            self.off_page += usize::from(before <= PAGE && after.is_none_or(|a| a > PAGE));
        }
    }

    fn merge(&mut self, o: Crowd) {
        self.items += o.items;
        self.later += o.later;
        self.off_page += o.off_page;
    }

    fn cell(&self) -> String {
        format!(
            "{:.2}% / {:.2}%（{} 題）",
            100.0 * ratio(self.later, self.items),
            100.0 * ratio(self.off_page, self.items),
            self.items
        )
    }
}

/// 打錯的題的名次統計（想打的整句在候選列的位置）。
#[derive(Debug, Clone, Copy, Default)]
struct Ranks {
    items: usize,
    first_page: usize,
    found: usize,
    rank_sum: usize,
}

impl Ranks {
    fn add(&mut self, rank: Option<usize>) {
        self.items += 1;
        if let Some(r) = rank {
            self.found += 1;
            self.rank_sum += r;
            self.first_page += usize::from(r <= PAGE);
        }
    }

    fn merge(&mut self, o: Ranks) {
        self.items += o.items;
        self.first_page += o.first_page;
        self.found += o.found;
        self.rank_sum += o.rank_sum;
    }

    fn cell(&self) -> String {
        format!(
            "{:.1}%（{}/{}）、前 {PAGES} 頁 {:.1}%、平均第 {:.1} 名",
            100.0 * ratio(self.first_page, self.items),
            self.first_page,
            self.items,
            100.0 * ratio(self.found, self.items),
            if self.found == 0 {
                0.0
            } else {
                self.rank_sum as f64 / self.found as f64
            }
        )
    }
}

/// 打錯的題依期望的字數分組（同基準的長度分組，5 字以上併成一組）。
const GROUPS: [(usize, &str); 4] = [
    (1, "1 字"),
    (2, "2 字"),
    (4, "3–4 字"),
    (usize::MAX, "5 字以上"),
];

fn group_of(chars: usize) -> usize {
    GROUPS
        .iter()
        .position(|&(upper, _)| chars <= upper)
        .unwrap()
}

/// 觸控模擬的集合在關（`off`）與開（`on`）之下的比較，可以幾批相加（兩個語料合計）。
#[derive(Debug, Clone, Copy, Default)]
struct TouchCompare {
    mistyped: usize,
    mistyped_correct_off: usize,
    mistyped_correct_on: usize,
    better: usize,
    worse: usize,
    corrections_off: usize,
    corrections_on: usize,
    literal_correct: usize,
    false_corrections: usize,
    ranks_off: Ranks,
    ranks_on: Ranks,
    groups_off: [Ranks; 4],
    groups_on: [Ranks; 4],
    crowd: Crowd,
}

impl TouchCompare {
    fn of(set: &Set, off: &[Outcome], on: &[Outcome]) -> Self {
        let mut c = TouchCompare::default();
        for ((task, a), b) in set.tasks.iter().zip(off).zip(on) {
            assert_eq!(a.line, b.line);
            c.corrections_off += a.corrections;
            c.corrections_on += b.corrections;
            let expected = task.item.expected();
            if a.clean {
                if a.correct {
                    c.literal_correct += 1;
                    c.false_corrections += usize::from(!b.correct);
                }
                c.crowd.add(&expected, &a.list, &b.list);
            } else {
                c.mistyped += 1;
                c.mistyped_correct_off += usize::from(a.correct);
                c.mistyped_correct_on += usize::from(b.correct);
                c.better += usize::from(!a.correct && b.correct);
                c.worse += usize::from(a.correct && !b.correct);
                let (r_off, r_on) = (rank_of(&a.list, &expected), rank_of(&b.list, &expected));
                c.ranks_off.add(r_off);
                c.ranks_on.add(r_on);
                let g = group_of(a.chars);
                c.groups_off[g].add(r_off);
                c.groups_on[g].add(r_on);
            }
        }
        c
    }

    fn add(mut self, o: TouchCompare) -> Self {
        self.mistyped += o.mistyped;
        self.mistyped_correct_off += o.mistyped_correct_off;
        self.mistyped_correct_on += o.mistyped_correct_on;
        self.better += o.better;
        self.worse += o.worse;
        self.corrections_off += o.corrections_off;
        self.corrections_on += o.corrections_on;
        self.literal_correct += o.literal_correct;
        self.false_corrections += o.false_corrections;
        self.ranks_off.merge(o.ranks_off);
        self.ranks_on.merge(o.ranks_on);
        for g in 0..GROUPS.len() {
            self.groups_off[g].merge(o.groups_off[g]);
            self.groups_on[g].merge(o.groups_on[g]);
        }
        self.crowd.merge(o.crowd);
        self
    }

    fn accuracy_on(&self) -> f64 {
        ratio(self.mistyped_correct_on, self.mistyped)
    }

    fn correction_change(&self) -> f64 {
        self.corrections_on as f64 / self.corrections_off as f64 - 1.0
    }

    fn false_correction_rate(&self) -> f64 {
        ratio(self.false_corrections, self.literal_correct)
    }

    fn p(&self) -> f64 {
        mcnemar(self.better, self.worse)
    }
}

fn ratio(a: usize, b: usize) -> f64 {
    if b == 0 { 0.0 } else { a as f64 / b as f64 }
}

/// McNemar 精確檢定（雙尾），同票 09 的實驗。
fn mcnemar(b: usize, c: usize) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let k = b.min(c);
    let ln_half_n = n as f64 * 0.5f64.ln();
    let mut ln_choose = 0.0;
    let mut tail = 0.0;
    for i in 0..=k {
        if i > 0 {
            ln_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        tail += (ln_choose + ln_half_n).exp();
    }
    (2.0 * tail).min(1.0)
}

/// 沒打錯的集合在關與開之下：答錯數、修正次數、被擠。
struct CleanCompare {
    name: String,
    total: usize,
    wrong_off: usize,
    wrong_on: usize,
    corrections_off: usize,
    corrections_on: usize,
    crowd: Crowd,
}

impl CleanCompare {
    fn of(name: &str, set: &Set, off: &[Outcome], on: &[Outcome]) -> Self {
        let mut crowd = Crowd::default();
        for ((task, a), b) in set.tasks.iter().zip(off).zip(on) {
            crowd.add(&task.item.expected(), &a.list, &b.list);
        }
        CleanCompare {
            name: name.to_string(),
            total: off.len(),
            wrong_off: off.iter().filter(|o| !o.correct).count(),
            wrong_on: on.iter().filter(|o| !o.correct).count(),
            corrections_off: off.iter().map(|o| o.corrections).sum(),
            corrections_on: on.iter().map(|o| o.corrections).sum(),
            crowd,
        }
    }

    fn passes(&self) -> bool {
        let wrong = self.wrong_on as f64 - self.wrong_off as f64;
        let corrections = self.corrections_on as f64 - self.corrections_off as f64;
        wrong <= CLEAN_WRONG_LIMIT * self.total as f64
            && corrections <= CLEAN_CORRECTION_LIMIT * self.corrections_off as f64
    }
}

fn clean_table(rows: &[CleanCompare]) -> String {
    let mut out = String::from(
        "| 集合（觸控點對鍵） | 題數 | 答錯（關 → 開） | 修正次數（關 → 開） | 被擠後 / 擠出第一頁 | 不退步 |\n| --- | ---: | --- | --- | --- | --- |\n",
    );
    for r in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} → {} | {} → {} ({:+.1}%) | {} | {} |",
            r.name,
            r.total,
            r.wrong_off,
            r.wrong_on,
            r.corrections_off,
            r.corrections_on,
            100.0 * (r.corrections_on as f64 / r.corrections_off as f64 - 1.0),
            r.crowd.cell(),
            if r.passes() { "過" } else { "**沒過**" }
        );
    }
    out
}

const TOUCH_HEADER: &str = "| 設定 | 打錯的題首選 | 修正次數 | 改錯 | 翻好/翻壞 | 想打的整句在第一頁（打錯的題） | 沒打錯的題被擠後 / 擠出第一頁 |\n| --- | --- | --- | --- | --- | --- | --- |";

fn touch_row(label: &str, c: &TouchCompare) -> String {
    format!(
        "| {label} | {}/{} ({:.1}%) | {:+.1}% | {}/{} ({:.2}%) | {}/{} p={:.3} | {} | {} |",
        c.mistyped_correct_on,
        c.mistyped,
        100.0 * c.accuracy_on(),
        100.0 * c.correction_change(),
        c.false_corrections,
        c.literal_correct,
        100.0 * c.false_correction_rate(),
        c.better,
        c.worse,
        c.p(),
        c.ranks_on.cell(),
        c.crowd.cell()
    )
}

fn groups_table(c: &TouchCompare) -> String {
    let mut out = String::from(
        "| 打錯的題（期望的字數） | 關：第一頁、前 5 頁、平均名次 | 開：第一頁、前 5 頁、平均名次 |\n| --- | --- | --- |\n",
    );
    for (g, (_, label)) in GROUPS.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {label} | {} | {} |",
            c.groups_off[g].cell(),
            c.groups_on[g].cell()
        );
    }
    let _ = writeln!(
        out,
        "| 合計 | {} | {} |",
        c.ranks_off.cell(),
        c.ranks_on.cell()
    );
    out
}

// ---------------------------------------------------------------- 執行

fn load(root: &Path) -> Engine {
    load_engine(root, &[], None).unwrap().engine
}

/// 關的結果先打好，各設定都和它比。
struct Runner<'a> {
    engine: Engine,
    split: &'a Split,
    taps: Taps,
    off_touch: Vec<Vec<Outcome>>,
    clean_sets: Option<Vec<Set>>,
    off_clean: Option<Vec<Vec<Outcome>>>,
}

impl<'a> Runner<'a> {
    fn new(root: &Path, engine: Engine, split: &'a Split) -> Self {
        let off_touch = split.touch.iter().map(|s| score(&engine, s)).collect();
        Runner {
            engine,
            split,
            taps: Taps::new(root),
            off_touch,
            clean_sets: None,
            off_clean: None,
        }
    }

    fn on(&self, config: &Config) -> Engine {
        self.engine
            .clone()
            .with_neighbor_readings(config.readings())
    }

    /// 觸控模擬兩個語料合計。
    fn touch(&self, config: &Config) -> TouchCompare {
        let engine = self.on(config);
        self.split
            .touch
            .iter()
            .zip(&self.off_touch)
            .map(|(set, off)| TouchCompare::of(set, off, &score(&engine, set)))
            .fold(TouchCompare::default(), TouchCompare::add)
    }

    /// 沒打錯的集合改成觸控點對鍵送。
    fn clean(&mut self, config: &Config) -> Vec<CleanCompare> {
        let sets = self.clean_sets.get_or_insert_with(|| {
            self.split
                .clean
                .iter()
                .map(|(name, items)| touch_clean_set(name.clone(), items.clone(), &self.taps))
                .collect()
        });
        let off = self
            .off_clean
            .get_or_insert_with(|| sets.iter().map(|s| score(&self.engine, s)).collect());
        let engine = self
            .engine
            .clone()
            .with_neighbor_readings(config.readings());
        sets.iter()
            .zip(off.iter())
            .map(|(set, off)| CleanCompare::of(&set.name, set, off, &score(&engine, set)))
            .collect()
    }
}

/// dev 的挑法（README〈dev 怎麼挑〉）：修正次數最少的；一樣多取打錯的題第一頁比例高的，再一樣取前面的
/// （偏向少插、晚插）。`limit_false` 為真時改錯要 ≤ 1.0%。
fn pick(rows: &[(Config, TouchCompare)], limit_false: bool) -> Config {
    rows.iter()
        .filter(|(_, c)| !limit_false || c.false_correction_rate() <= FALSE_CORRECTION_LIMIT)
        .min_by(|(_, a), (_, b)| {
            a.corrections_on
                .cmp(&b.corrections_on)
                .then(b.ranks_on.first_page.cmp(&a.ranks_on.first_page))
        })
        .map(|(config, _)| *config)
        .expect("至少一個設定")
}

fn sweep(runner: &Runner, title: &str, configs: &[Config]) -> Vec<(Config, TouchCompare)> {
    println!("\n### {title}\n\n{TOUCH_HEADER}");
    configs
        .iter()
        .map(|config| {
            let c = runner.touch(config);
            println!("{}", touch_row(&config.describe(), &c));
            (*config, c)
        })
        .collect()
}

fn dev(root: &Path) {
    let engine = load(root);
    let split = dev_split(root);
    let runner = Runner::new(root, engine, &split);
    let off = runner.touch(&Config {
        limit: 0,
        ..Config::default()
    });
    println!("## dev：觸控模擬（兩個語料合計）\n");
    println!(
        "關：打錯的題首選 {}/{}、修正 {} 次；想打的整句在第一頁 {}",
        off.mistyped_correct_off,
        off.mistyped,
        off.corrections_off,
        off.ranks_off.cell()
    );
    // 第一步：放法。
    let placements: Vec<Config> = [2, 3, 5]
        .into_iter()
        .flat_map(|slot| {
            [1, 2, 3, 4].into_iter().map(move |limit| Config {
                slot,
                limit,
                ..Config::default()
            })
        })
        .collect();
    let rows = sweep(&runner, "第一步：放法（floor 0.05、不搶首選）", &placements);
    let placed = pick(&rows, false);
    println!("\n挑到：{}", placed.describe());
    // 第二步：floor。
    let floors: Vec<Config> = [0.02, 0.05, 0.1, 0.2, 0.3]
        .into_iter()
        .map(|floor| Config { floor, ..placed })
        .collect();
    let rows = sweep(&runner, "第二步：floor", &floors);
    let floored = pick(&rows, false);
    println!("\n挑到：{}", floored.describe());
    // 第三步：模稜兩可的門檻。
    let ambiguous: Vec<Config> = [
        None,
        Some(0.45),
        Some(0.4),
        Some(0.35),
        Some(0.3),
        Some(0.25),
    ]
    .into_iter()
    .map(|ambiguous| Config {
        ambiguous,
        ..floored
    })
    .collect();
    let rows = sweep(&runner, "第三步：模稜兩可時搶首選的門檻", &ambiguous);
    let chosen = pick(&rows, true);
    println!("\n挑到：{}", chosen.describe());
    let c = rows.iter().find(|(cfg, _)| *cfg == chosen).unwrap().1;
    println!("\n### 挑到的設定：打錯的題依字數\n\n{}", groups_table(&c));
    println!("test 用：`cargo run --release -- test {}`", chosen.args());
}

fn combo(root: &Path, config: &Config) {
    let engine = load(root);
    let split = dev_split(root);
    let mut runner = Runner::new(root, engine, &split);
    let c = runner.touch(config);
    println!("## dev：{}\n\n{TOUCH_HEADER}", config.describe());
    println!("{}", touch_row(&config.describe(), &c));
    println!("\n{}", groups_table(&c));
    println!("{}", clean_table(&runner.clean(config)));
}

/// 實體鍵送的集合：開關前後要完全相同（設計上不適用，只核對觸控的判斷沒有漏到實體鍵）。
fn physical_identical(engine: &Engine, on: &Engine, sets: &[Set]) -> Vec<(String, bool)> {
    sets.iter()
        .map(|set| {
            let (a, b) = (score(engine, set), score(on, set));
            let same = a.iter().zip(&b).all(|(x, y)| {
                x.correct == y.correct && x.corrections == y.corrections && x.list == y.list
            });
            (set.name.clone(), same)
        })
        .collect()
}

fn test(root: &Path, config: &Config) {
    let engine = load(root);
    let split = test_split(root);
    let mut runner = Runner::new(root, engine.clone(), &split);
    let on = runner.on(config);
    println!("## test：{}（只跑這一次）\n", config.describe());
    let c = runner.touch(config);
    println!("### 觸控模擬（兩個語料合計）\n\n{TOUCH_HEADER}");
    println!(
        "{}",
        touch_row(
            "關",
            &TouchCompare {
                mistyped_correct_on: c.mistyped_correct_off,
                corrections_on: c.corrections_off,
                false_corrections: 0,
                better: 0,
                worse: 0,
                ranks_on: c.ranks_off,
                crowd: Crowd::default(),
                ..c
            }
        )
    );
    println!("{}", touch_row("開", &c));
    println!(
        "\n修正次數：關 {} 次、開 {} 次；打錯的題 {} 題、沒打錯的題裡關的時候首選正確 {} 題",
        c.corrections_off, c.corrections_on, c.mistyped, c.literal_correct
    );
    println!("\n{}", groups_table(&c));
    println!("### 對照〈容錯採用門檻〉（觸控那幾條）\n");
    let line = |name: &str, value: String, limit: String, ok: bool| {
        println!(
            "- {name}：{value}（{limit}）：{}",
            if ok { "過" } else { "**沒過**" }
        );
    };
    line(
        "打錯的題首選",
        format!("{:.1}%", 100.0 * c.accuracy_on()),
        format!("≥ {:.0}%", 100.0 * TOUCH_ACCURACY),
        c.accuracy_on() >= TOUCH_ACCURACY,
    );
    line(
        "修正次數",
        format!("{:+.1}%", 100.0 * c.correction_change()),
        format!("≤ −{:.0}%", 100.0 * FEWER_CORRECTIONS),
        c.correction_change() <= -FEWER_CORRECTIONS,
    );
    line(
        "翻好／翻壞 McNemar",
        format!("{}/{} p={:.4}", c.better, c.worse, c.p()),
        format!("p < {P_LIMIT}"),
        c.p() < P_LIMIT,
    );
    line(
        "原樣正確卻被改錯",
        format!("{:.2}%", 100.0 * c.false_correction_rate()),
        format!("≤ {:.1}%", 100.0 * FALSE_CORRECTION_LIMIT),
        c.false_correction_rate() <= FALSE_CORRECTION_LIMIT,
    );
    println!("\n### 沒打錯的集合改成觸控點對鍵送\n");
    let rows = runner.clean(config);
    println!("{}", clean_table(&rows));
    if let Some(fixed) = &split.fixed {
        let set = touch_clean_set("fixed".into(), fixed.clone(), &runner.taps);
        let (a, b) = (score(&engine, &set), score(&on, &set));
        println!(
            "固定題（觸控點對鍵）：關 {}/{}、開 {}/{}",
            a.iter().filter(|o| o.correct).count(),
            a.len(),
            b.iter().filter(|o| o.correct).count(),
            b.len()
        );
        println!("\n### 照實體鍵送的集合（設計上不適用，核對開關前後相同）\n");
        let mut physical_sets: Vec<Set> = vec![physical_set("fixed".into(), fixed.clone())];
        physical_sets.extend(split.physical_typo.iter().map(|s| Set {
            name: s.name.clone(),
            tasks: s.tasks.clone(),
        }));
        physical_sets.extend(
            split
                .clean
                .iter()
                .map(|(name, items)| physical_set(name.clone(), items.clone())),
        );
        for (name, same) in physical_identical(&engine, &on, &physical_sets) {
            println!("- {name}：{}", if same { "相同" } else { "**不同**" });
        }
    }
}

// ---------------------------------------------------------------- 延遲

/// 照 `keys` 打一個輸入，量每一鍵與「選開頭一段」（同 `qim_bench::native_timing_with`，按鍵換成觸控的）。
fn time_input(engine: &Engine, user: &mut UserModel, keys: &[Key]) -> (Vec<f64>, Option<f64>) {
    let mut session = Session::default();
    let mut last = Snapshot::default();
    let per_key = keys
        .iter()
        .map(|key| {
            let start = Instant::now();
            last = std::hint::black_box(session.handle(engine, user, key.clone()));
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    let select = lefts_to_first_segment(&last).map(|lefts| {
        for _ in 0..lefts {
            session.handle(engine, user, Key::Left);
        }
        let start = Instant::now();
        std::hint::black_box(session.select(engine, user, 0));
        start.elapsed().as_secs_f64() * 1000.0
    });
    (per_key, select)
}

fn p95s(per_key: &[Vec<f64>], selects: &[f64]) -> [f64; 3] {
    let all: Vec<f64> = per_key.concat();
    let long: Vec<f64> = per_key
        .iter()
        .filter_map(|k| k.get(qim_bench::LONG_INPUT_KEYS..))
        .flatten()
        .copied()
        .collect();
    [
        percentile(&all, 95).unwrap_or(0.0),
        percentile(&long, 95).unwrap_or(0.0),
        percentile(selects, 95).unwrap_or(0.0),
    ]
}

fn load_average() -> String {
    std::process::Command::new("uptime")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn latency(root: &Path, config: &Config) {
    let engine = load(root);
    let on = engine.clone().with_neighbor_readings(config.readings());
    let taps = Taps::new(root);
    let inputs = qim_bench::latency_inputs(root).unwrap();
    let wildcard = engine.scheme().wildcard.expect("方案有萬用鍵");
    let wildcard_inputs = qim_bench::wildcard_inputs(&inputs, wildcard);
    // 每個輸入照觸控點對鍵送：當成一個單位抽點擊（萬用鍵 `z` 也點 z 鍵，它是鄰鍵讀法不換的碼）。
    let keys_of = |label: &str, inputs: &[String]| -> Vec<Vec<Key>> {
        inputs
            .iter()
            .enumerate()
            .map(|(i, input)| {
                let item = Item {
                    units: vec![qim_bench::Unit {
                        code: input.clone(),
                        text: String::new(),
                    }],
                    line: i + 1,
                    label: String::new(),
                    weight: 1,
                    context: None,
                    typed: None,
                };
                taps.clean(&format!("latency\0{label}\0{i}"), &item)
                    .concat()
            })
            .collect()
    };
    let batches = [
        ("基準的輸入", keys_of("plain", &inputs)),
        ("首碼換成 z", keys_of("wildcard", &wildcard_inputs)),
    ];
    let corpora = load_corpora(root).unwrap();
    let mut context_items = Vec::new();
    for dir in qim_bench::CONTEXT_DIRS {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            let mut files: Vec<PathBuf> = entries.map(|e| e.unwrap().path()).collect();
            files.sort();
            for path in files
                .iter()
                .filter(|p| p.extension().is_some_and(|e| e == "tsv"))
            {
                context_items.extend(parse_context(&read(path)).unwrap());
            }
        }
    }
    let learned = learned_model(
        &engine,
        corpora
            .iter()
            .flat_map(|(_, items)| items)
            .chain(&context_items),
    );
    println!(
        "## 每鍵延遲（原生 release；觸控點對鍵；{}）\n",
        config.describe()
    );
    println!("開始時：{}\n", load_average());
    println!(
        "| 學習資料 | 輸入 | 鄰鍵讀法 | 全部 p95 | 第 20 碼起 p95 | 選開頭一段 p95 |\n| --- | --- | --- | ---: | ---: | ---: |"
    );
    for (user_label, user) in [("空白", UserModel::default()), ("學過", learned)] {
        for (input_label, batch) in &batches {
            // 關與開逐題交替，機器負載的起伏兩邊一起吃。
            let mut runs: [(Vec<Vec<f64>>, Vec<f64>); 2] = Default::default();
            for keys in batch {
                for (i, e) in [&engine, &on].into_iter().enumerate() {
                    let mut user = user.clone();
                    let (per_key, select) = time_input(e, &mut user, keys);
                    runs[i].0.push(per_key);
                    runs[i].1.extend(select);
                }
            }
            for (i, label) in ["關", "開"].into_iter().enumerate() {
                let [all, long, select] = p95s(&runs[i].0, &runs[i].1);
                println!(
                    "| {user_label} | {input_label} | {label} | {:.0} µs | {:.0} µs | {:.2} ms |",
                    all * 1000.0,
                    long * 1000.0,
                    select
                );
            }
        }
    }
    println!("\n結束時：{}", load_average());
}

// ---------------------------------------------------------------- 按鍵機率的分布

/// dev 觸控題每一下的「第二高 ÷ 最高」分布，與點錯的那幾下想打的字母排第幾、比例多少（定 floor 範圍用）。
fn weights(root: &Path) {
    let surface = surface(root);
    let mut ratios: Vec<f64> = Vec::new();
    let mut wrong: Vec<(Option<usize>, f64)> = Vec::new();
    for name in COLLOQUIAL {
        let text = read(&root.join(qim_bench::dev_touch_items(name)));
        for item in parse_touch(&text, &surface).unwrap() {
            for (unit, keys) in item.units.iter().zip(item.typed.as_ref().unwrap()) {
                for (c, key) in unit.code.chars().zip(keys) {
                    let Key::Weighted(w) = key else { continue };
                    let e = w.entries();
                    ratios.push(e.get(1).map_or(0.0, |s| f64::from(s.1 / e[0].1)));
                    if e[0].0 != c {
                        let at = e.iter().position(|&(d, _)| d == c);
                        let r = at.map_or(0.0, |i| f64::from(e[i].1 / e[0].1));
                        wrong.push((at, r));
                    }
                }
            }
        }
    }
    let q = |v: &[f64], p: u32| percentile(v, p).unwrap_or(0.0);
    println!(
        "每一下（{} 下）第二高 ÷ 最高：p10 {:.3}、p50 {:.3}、p90 {:.3}",
        ratios.len(),
        q(&ratios, 10),
        q(&ratios, 50),
        q(&ratios, 90)
    );
    let mut by_rank: BTreeMap<String, usize> = BTreeMap::new();
    for (at, _) in &wrong {
        *by_rank
            .entry(at.map_or("沒有".into(), |i| format!("第 {} 高", i + 1)))
            .or_default() += 1;
    }
    let rs: Vec<f64> = wrong.iter().map(|w| w.1).collect();
    println!(
        "點錯的 {} 下，想打的字母：{by_rank:?}；它 ÷ 最高：p10 {:.3}、p50 {:.3}、p90 {:.3}",
        wrong.len(),
        q(&rs, 10),
        q(&rs, 50),
        q(&rs, 90)
    );
}

// ---------------------------------------------------------------- 參數

fn parse_args(args: &[String]) -> (PathBuf, Config) {
    let mut root = PathBuf::from("../../..");
    let mut config = Config::default();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next().unwrap_or_else(|| panic!("{flag} 要給值"));
        match flag.as_str() {
            "--root" => root = PathBuf::from(value),
            "--slot" => config.slot = value.parse().unwrap(),
            "--limit" => config.limit = value.parse().unwrap(),
            "--floor" => config.floor = value.parse().unwrap(),
            "--letters" => config.letters = value.parse().unwrap(),
            "--weight" => config.weight = value.parse().unwrap(),
            "--ambiguous" => {
                config.ambiguous = (value != "none").then(|| value.parse().unwrap());
            }
            other => panic!("沒有這個選項：{other}"),
        }
    }
    (root, config)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else {
        eprintln!(
            "用法：cargo run --release -- weights|dev|combo|test|latency [選項]（見 src/main.rs 開頭）"
        );
        std::process::exit(2);
    };
    let (root, config) = parse_args(rest);
    let started = Instant::now();
    match command.as_str() {
        "weights" => weights(&root),
        "dev" => dev(&root),
        "combo" => combo(&root, &config),
        "test" => test(&root, &config),
        "latency" => latency(&root, &config),
        other => panic!("沒有這個指令：{other}"),
    }
    eprintln!("花了 {:.0} 秒", started.elapsed().as_secs_f64());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcnemar_matches_known_values() {
        assert_eq!(mcnemar(0, 0), 1.0);
        assert!((mcnemar(10, 0) - 2.0 * 0.5f64.powi(10)).abs() < 1e-12);
        assert_eq!(mcnemar(5, 5), 1.0);
        assert!((mcnemar(3, 12) - 2.0 * 576.0 / 32768.0).abs() < 1e-12);
    }

    fn texts(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn crowding_follows_the_candidate_the_user_picks_first() {
        // 首選接得上：看首選；鄰鍵讀法插在它前面才算往後。
        let off = texts(&["門口", "門", "口"]);
        assert_eq!(
            crowding("門口", &off, &texts(&["門口", "刷口", "門"])),
            Some((1, Some(1)))
        );
        // 首選接不上：第一頁接得上的最長候選。
        let off = texts(&["刷口", "門", "日"]);
        assert_eq!(
            crowding("門口", &off, &texts(&["刷口", "日口", "門"])),
            Some((2, Some(3)))
        );
        // 第一頁沒有接得上的：不算。
        assert_eq!(crowding("門口", &texts(&["刷"]), &texts(&["刷"])), None);
    }

    #[test]
    fn crowd_counts_items_pushed_off_the_first_page() {
        let mut crowd = Crowd::default();
        let off: Vec<String> = (0..12).map(|i| i.to_string()).collect();
        // 期望以 "8" 開頭：第 9 名；開了之後前面多兩個，變第 11 名。
        let mut on = off.clone();
        on.insert(1, "x".into());
        on.insert(1, "y".into());
        crowd.add("8", &off, &on);
        assert_eq!((crowd.items, crowd.later, crowd.off_page), (1, 1, 1));
    }

    #[test]
    fn physical_weights_are_not_touch() {
        let Key::Weighted(w) = physical_key('a') else {
            panic!("實體鍵送按鍵機率")
        };
        assert!(!is_touch(&w));
        let touch = Weights::new(vec![('a', 0.7), ('s', 0.3)]).unwrap();
        assert!(is_touch(&touch));
    }
}
