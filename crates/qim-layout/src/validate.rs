//! 佈局檔的兩級驗證（plan §2.6、brief §4.6）：錯誤不能存檔，警告可以；
//! 內建佈局另加嚴格檢查（plan §5.3）。

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::geometry::convex_overlap;
use crate::model::{Action, Device, FORMAT, Key, Layout, LayoutVariant, Orientation, Shape};

/// 佈局檔的 JSON Schema（draft 2020-12）。
pub const SCHEMA: &str = include_str!("../schema/qim-layout-1.schema.json");

/// 佈局要在哪裡用；錯誤級的檢查有些看平台。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Target {
    /// iOS 的 `needsInputModeSwitchKey` 為真：每一頁都要能切到有「下一個鍵盤」鍵的頁面。
    pub needs_input_mode_switch_key: bool,
}

/// 不能存檔的問題。`variant` 是佈局版本在 `variants` 的位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// 檔案壞掉：不是 JSON、不符合 Schema、`id` 重複、指到不存在的頁面。
    Broken(String),
    /// 從 `page` 切不回有字母的頁面。
    NoWayBackToLetters { variant: usize, page: String },
    /// 需要切換鍵盤鍵（[`Target::needs_input_mode_switch_key`]），`page` 卻切不到有它的頁面。
    MissingNextKeyboard { variant: usize, page: String },
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::Broken(message) => write!(f, "檔案壞掉：{message}"),
            LayoutError::NoWayBackToLetters { variant, page } => {
                write!(f, "版本 {variant} 頁 {page}：切不回有字母的頁面")
            }
            LayoutError::MissingNextKeyboard { variant, page } => write!(
                f,
                "版本 {variant} 頁 {page}：切不到有「下一個鍵盤」鍵（nextKeyboard）的頁面，iOS 需要它"
            ),
        }
    }
}

impl std::error::Error for LayoutError {}

/// 可以存檔、但提醒使用者的問題。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// 兩鍵畫出來的範圍重疊。
    Overlap {
        variant: usize,
        page: String,
        keys: [String; 2],
    },
    /// 觸控範圍全被上層的鍵蓋住，點不到。
    Covered {
        variant: usize,
        page: String,
        key: String,
    },
    /// 畫出來的範圍超出畫布。
    OutOfCanvas {
        variant: usize,
        page: String,
        key: String,
    },
    /// 這個佈局版本點不出的字母（`a`–`z`，只算 `tap`）。
    MissingLetters { variant: usize, letters: String },
    /// 有這種裝置的佈局版本，卻缺這個方向。
    MissingOrientation {
        device: Device,
        orientation: Orientation,
    },
    /// 形狀還不支援（M5），現在當矩形。
    UnsupportedShape {
        variant: usize,
        page: String,
        key: String,
        shape: &'static str,
    },
    /// 滑動還沒有作用（M5）。
    UnsupportedSwipe {
        variant: usize,
        page: String,
        key: String,
    },
    /// 沒有裝置與方向相符的版本，用第一個版本等比例縮放。
    NoMatchingVariant {
        device: Device,
        orientation: Orientation,
    },
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::Overlap {
                variant,
                page,
                keys: [a, b],
            } => write!(f, "版本 {variant} 頁 {page}：{a} 與 {b} 重疊"),
            Warning::Covered { variant, page, key } => {
                write!(
                    f,
                    "版本 {variant} 頁 {page}：{key} 完全被上層的鍵蓋住，點不到"
                )
            }
            Warning::OutOfCanvas { variant, page, key } => {
                write!(f, "版本 {variant} 頁 {page}：{key} 超出畫布")
            }
            Warning::MissingLetters { variant, letters } => {
                write!(f, "版本 {variant}：缺字母 {letters}")
            }
            Warning::MissingOrientation {
                device,
                orientation,
            } => write!(
                f,
                "有 {} 的版本，卻缺 {} 的版本",
                device.name(),
                orientation.name()
            ),
            Warning::UnsupportedShape {
                variant,
                page,
                key,
                shape,
            } => write!(
                f,
                "版本 {variant} 頁 {page}：{key} 的形狀 {shape} 還不支援，現在當矩形畫與判定"
            ),
            Warning::UnsupportedSwipe { variant, page, key } => {
                write!(f, "版本 {variant} 頁 {page}：{key} 的滑動還沒有作用")
            }
            Warning::NoMatchingVariant {
                device,
                orientation,
            } => write!(
                f,
                "沒有 {} {} 的版本，用第一個版本等比例縮放",
                device.name(),
                orientation.name()
            ),
        }
    }
}

/// 驗證的結果。
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// 讀得懂時的佈局；有 [`LayoutError::Broken`] 時是 `None`。
    pub layout: Option<Layout>,
    pub errors: Vec<LayoutError>,
    pub warnings: Vec<Warning>,
}

impl Report {
    /// 沒有錯誤就可以存檔；警告不擋。
    pub fn can_save(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 驗證佈局檔：先過 JSON Schema，再讀進型別，再查錯誤與警告。
pub fn validate(text: &str, target: Target) -> Report {
    let broken = |message: String| Report {
        layout: None,
        errors: vec![LayoutError::Broken(message)],
        warnings: Vec::new(),
    };
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => return broken(format!("不是 JSON：{error}")),
    };
    // 先看 format：別的版本的檔案不必再列 Schema 的錯。
    if let Some(format) = value.get("format").and_then(serde_json::Value::as_str)
        && format != FORMAT
    {
        return broken(format!("format 是 {format:?}，只讀得懂 {FORMAT:?}"));
    }
    let schema_errors = schema_errors(&value);
    if !schema_errors.is_empty() {
        return Report {
            layout: None,
            errors: schema_errors.into_iter().map(LayoutError::Broken).collect(),
            warnings: Vec::new(),
        };
    }
    let layout: Layout = match serde_json::from_value(value) {
        Ok(layout) => layout,
        // Schema 與型別對不上才會到這裡。
        Err(error) => return broken(format!("讀不進型別：{error}")),
    };
    let errors = references(&layout);
    if !errors.is_empty() {
        return Report {
            layout: None,
            errors,
            warnings: Vec::new(),
        };
    }
    let mut errors = Vec::new();
    for (index, variant) in layout.variants.iter().enumerate() {
        for page in pages_without(variant, |key| matches!(key.tap, Action::Letter(_))) {
            errors.push(LayoutError::NoWayBackToLetters {
                variant: index,
                page,
            });
        }
        if target.needs_input_mode_switch_key {
            for page in pages_without(variant, |key| key.tap == Action::NextKeyboard) {
                errors.push(LayoutError::MissingNextKeyboard {
                    variant: index,
                    page,
                });
            }
        }
    }
    let warnings = warnings(&layout);
    Report {
        layout: Some(layout),
        errors,
        warnings,
    }
}

fn schema_errors(value: &serde_json::Value) -> Vec<String> {
    let schema: serde_json::Value =
        serde_json::from_str(SCHEMA).expect("SCHEMA 是 repo 裡的 JSON，測試保證讀得了");
    let validator = jsonschema::draft202012::new(&schema).expect("SCHEMA 符合 draft 2020-12");
    validator
        .iter_errors(value)
        .map(|error| {
            let path = error.instance_path().to_string();
            let path = if path.is_empty() { "/".into() } else { path };
            format!("{path} 不符合 Schema：{error}")
        })
        .collect()
}

/// Schema 表達不了的「壞掉」：`id` 重複、指到不存在的頁面。
fn references(layout: &Layout) -> Vec<LayoutError> {
    let mut errors = Vec::new();
    for (index, variant) in layout.variants.iter().enumerate() {
        let mut page_ids = HashSet::new();
        for page in &variant.pages {
            if !page_ids.insert(page.id.as_str()) {
                errors.push(LayoutError::Broken(format!(
                    "版本 {index}：頁 id {} 重複",
                    page.id
                )));
            }
            let mut key_ids = HashSet::new();
            for key in &page.keys {
                if !key_ids.insert(key.id.as_str()) {
                    errors.push(LayoutError::Broken(format!(
                        "版本 {index} 頁 {}：鍵 id {} 重複",
                        page.id, key.id
                    )));
                }
                for action in key.actions() {
                    if let Action::SwitchPage(target) = action
                        && variant.page_index(target).is_none()
                    {
                        errors.push(LayoutError::Broken(format!(
                            "版本 {index} 頁 {} 鍵 {}：switchPage 指到不存在的頁 {target}",
                            page.id, key.id
                        )));
                    }
                }
            }
        }
        if let Some(initial) = &variant.initial_page
            && variant.page_index(initial).is_none()
        {
            errors.push(LayoutError::Broken(format!(
                "版本 {index}：initialPage 指到不存在的頁 {initial}"
            )));
        }
    }
    errors
}

/// 從一開始的頁面切得到、卻切不到「有符合 `wanted` 的鍵」的頁面的頁 id。
/// 頁面之間只靠 `switchPage`（點、長按、滑動、組字中都算）走。
fn pages_without(variant: &LayoutVariant, wanted: impl Fn(&Key) -> bool) -> Vec<String> {
    let edges: Vec<Vec<usize>> = variant
        .pages
        .iter()
        .map(|page| {
            page.keys
                .iter()
                .flat_map(Key::actions)
                .filter_map(|action| match action {
                    Action::SwitchPage(id) => variant.page_index(id),
                    _ => None,
                })
                .collect()
        })
        .collect();
    let reachable_from = |start: usize| {
        let mut seen = vec![false; edges.len()];
        let mut stack = vec![start];
        while let Some(page) = stack.pop() {
            if !std::mem::replace(&mut seen[page], true) {
                stack.extend(&edges[page]);
            }
        }
        seen
    };
    let good: Vec<bool> = variant
        .pages
        .iter()
        .map(|page| page.keys.iter().any(&wanted))
        .collect();
    let Some(initial) = variant.initial_page_index() else {
        return Vec::new();
    };
    reachable_from(initial)
        .into_iter()
        .enumerate()
        .filter(|&(page, reached)| {
            reached
                && !reachable_from(page)
                    .iter()
                    .zip(&good)
                    .any(|(&reach, &good)| reach && good)
        })
        .map(|(page, _)| variant.pages[page].id.clone())
        .collect()
}

fn warnings(layout: &Layout) -> Vec<Warning> {
    const EPSILON: f64 = 1e-9;
    let mut warnings = Vec::new();
    for (index, variant) in layout.variants.iter().enumerate() {
        let canvas = variant.canvas;
        for page in &variant.pages {
            let at = |key: &Key| (index, page.id.clone(), key.id.clone());
            let corners: Vec<_> = page.keys.iter().map(Key::corners).collect();
            for (i, key) in page.keys.iter().enumerate() {
                let (v, page_id, id) = at(key);
                if let Shape::Ellipse | Shape::Polygon { .. } = key.shape {
                    warnings.push(Warning::UnsupportedShape {
                        variant: v,
                        page: page_id.clone(),
                        key: id.clone(),
                        shape: key.shape.name(),
                    });
                }
                if key.swipe.is_some() {
                    warnings.push(Warning::UnsupportedSwipe {
                        variant: v,
                        page: page_id.clone(),
                        key: id.clone(),
                    });
                }
                if corners[i].iter().any(|&(x, y)| {
                    x < -EPSILON
                        || y < -EPSILON
                        || x > canvas.width + EPSILON
                        || y > canvas.height + EPSILON
                }) {
                    warnings.push(Warning::OutOfCanvas {
                        variant: v,
                        page: page_id.clone(),
                        key: id.clone(),
                    });
                }
                for (j, other) in page.keys.iter().enumerate().skip(i + 1) {
                    if convex_overlap(&corners[i], &corners[j]) {
                        warnings.push(Warning::Overlap {
                            variant: v,
                            page: page_id.clone(),
                            keys: [id.clone(), other.id.clone()],
                        });
                    }
                }
            }
            // 由上往下，每個鍵的取樣點都落在更上層的鍵裡就是點不到。
            let order = page.top_down();
            for (rank, &i) in order.iter().enumerate() {
                let above = &order[..rank];
                let samples = page.keys[i].touch_samples();
                let covered = samples
                    .iter()
                    .all(|&(x, y)| above.iter().any(|&upper| page.keys[upper].contains(x, y)));
                if covered {
                    let (variant, page, key) = at(&page.keys[i]);
                    warnings.push(Warning::Covered { variant, page, key });
                }
            }
        }
        let letters: HashSet<char> = variant
            .pages
            .iter()
            .flat_map(|page| &page.keys)
            .filter_map(|key| match key.tap {
                Action::Letter(c) => Some(c),
                _ => None,
            })
            .collect();
        let missing: String = ('a'..='z').filter(|c| !letters.contains(c)).collect();
        if !missing.is_empty() {
            warnings.push(Warning::MissingLetters {
                variant: index,
                letters: missing,
            });
        }
    }
    let mut orientations: HashMap<Device, HashSet<Orientation>> = HashMap::new();
    for variant in &layout.variants {
        orientations
            .entry(variant.when.device)
            .or_default()
            .insert(variant.when.orientation);
    }
    for device in [Device::Phone, Device::Tablet] {
        let Some(present) = orientations.get(&device) else {
            continue;
        };
        for orientation in [Orientation::Portrait, Orientation::Landscape] {
            if !present.contains(&orientation) {
                warnings.push(Warning::MissingOrientation {
                    device,
                    orientation,
                });
            }
        }
    }
    warnings
}

/// 內建佈局的嚴格檢查（plan §5.3）不通過的地方。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrictIssue {
    Error(LayoutError),
    /// 內建佈局連警告都不能有：直向與橫向都要有、不重疊、不超出畫布、字母齊全。
    Warning(Warning),
    /// 鍵沒有淺色與深色的 `fill` 或 `labelColor`。
    MissingKeyColors {
        variant: usize,
        page: String,
        key: String,
    },
    /// 佈局版本沒有 `background`。
    MissingBackground {
        variant: usize,
    },
}

/// 內建佈局的嚴格檢查：錯誤與警告都算，另外每個鍵與每個版本都要有淺色與深色的顏色。
/// 長按清單的動作是否有效由 Schema 與錯誤級的檢查保證（`letter` 是 `a`–`z`、`switchPage` 的頁存在）。
pub fn check_builtin(text: &str) -> Vec<StrictIssue> {
    let report = validate(text, Target::default());
    let mut issues: Vec<StrictIssue> = report
        .errors
        .into_iter()
        .map(StrictIssue::Error)
        .chain(report.warnings.into_iter().map(StrictIssue::Warning))
        .collect();
    let Some(layout) = report.layout else {
        return issues;
    };
    for (index, variant) in layout.variants.iter().enumerate() {
        if variant.background.is_none() {
            issues.push(StrictIssue::MissingBackground { variant: index });
        }
        for page in &variant.pages {
            for key in &page.keys {
                let colored = key
                    .style
                    .as_ref()
                    .is_some_and(|style| style.fill.is_some() && style.label_color.is_some());
                if !colored {
                    issues.push(StrictIssue::MissingKeyColors {
                        variant: index,
                        page: page.id.clone(),
                        key: key.id.clone(),
                    });
                }
            }
        }
    }
    issues
}

impl Layout {
    /// 選佈局版本（plan §2.6）：裝置與方向都符合的；其次方向符合的；再其次第一個版本，
    /// 等比例縮放並給警告。回傳版本在 `variants` 的位置。
    pub fn select(&self, device: Device, orientation: Orientation) -> (usize, Option<Warning>) {
        let find =
            |matches: &dyn Fn(&LayoutVariant) -> bool| self.variants.iter().position(matches);
        if let Some(index) = find(&|v| v.when.device == device && v.when.orientation == orientation)
            .or_else(|| find(&|v| v.when.orientation == orientation))
        {
            return (index, None);
        }
        (
            0,
            Some(Warning::NoMatchingVariant {
                device,
                orientation,
            }),
        )
    }
}
