//! 按鍵的幾何：觸控範圍內外、到觸控範圍的距離、畫出來的四個角（plan §2.6）。
//!
//! 座標都是畫布座標，y 往下。旋轉以 `frame` 中心順時針（同 CSS `rotate`）。

use crate::model::{Key, Page};

/// 把畫布上的點轉成按鍵自己的座標：原點在 `frame` 中心、沒有旋轉。
fn to_local(key: &Key, x: f64, y: f64) -> (f64, f64) {
    let frame = &key.frame;
    let (dx, dy) = (x - (frame.x + frame.w / 2.0), y - (frame.y + frame.h / 2.0));
    if key.rotate == 0.0 {
        return (dx, dy);
    }
    let (sin, cos) = key.rotate.to_radians().sin_cos();
    (dx * cos + dy * sin, -dx * sin + dy * cos)
}

/// 按鍵自己座標的點轉回畫布座標（[`to_local`] 的反向）。
fn to_world(key: &Key, lx: f64, ly: f64) -> (f64, f64) {
    let frame = &key.frame;
    let (cx, cy) = (frame.x + frame.w / 2.0, frame.y + frame.h / 2.0);
    let (sin, cos) = key.rotate.to_radians().sin_cos();
    (cx + lx * cos - ly * sin, cy + lx * sin + ly * cos)
}

/// 觸控範圍在按鍵座標的左、上、右、下邊與圓角半徑。
struct TouchArea {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    radius: f64,
}

fn touch_area(key: &Key) -> TouchArea {
    let (hw, hh) = (key.frame.w / 2.0, key.frame.h / 2.0);
    let inset = &key.touch_inset;
    let left = -hw - inset.left;
    let right = (hw + inset.right).max(left);
    let top = -hh - inset.top;
    let bottom = (hh + inset.bottom).max(top);
    let radius = key
        .shape
        .corner_radius()
        .clamp(0.0, (right - left).min(bottom - top) / 2.0);
    TouchArea {
        left,
        top,
        right,
        bottom,
        radius,
    }
}

impl Key {
    /// 點 `(x, y)` 在不在這個鍵的觸控範圍（形狀加 `touchInset`）內。
    /// 左、上邊算在內，右、下邊不算（旋轉前），相鄰兩鍵的共用邊只屬於一個鍵；圓角外的角落不算。
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let (lx, ly) = to_local(self, x, y);
        let area = touch_area(self);
        if !(area.left..area.right).contains(&lx) || !(area.top..area.bottom).contains(&ly) {
            return false;
        }
        let r = area.radius;
        if r == 0.0 {
            return true;
        }
        // 在四個角的 r × r 方格裡時，要在圓角的圓內。
        let cx = if lx < area.left + r {
            area.left + r
        } else if lx > area.right - r {
            area.right - r
        } else {
            return true;
        };
        let cy = if ly < area.top + r {
            area.top + r
        } else if ly > area.bottom - r {
            area.bottom - r
        } else {
            return true;
        };
        (lx - cx).powi(2) + (ly - cy).powi(2) <= r * r
    }

    /// 點到觸控範圍的距離；在範圍內是 0。
    pub fn distance(&self, x: f64, y: f64) -> f64 {
        let (lx, ly) = to_local(self, x, y);
        let area = touch_area(self);
        let r = area.radius;
        let (cx, cy) = (
            (area.left + area.right) / 2.0,
            (area.top + area.bottom) / 2.0,
        );
        let qx = ((lx - cx).abs() - ((area.right - area.left) / 2.0 - r)).max(0.0);
        let qy = ((ly - cy).abs() - ((area.bottom - area.top) / 2.0 - r)).max(0.0);
        ((qx * qx + qy * qy).sqrt() - r).max(0.0)
    }

    /// 畫出來的矩形（旋轉後、不含 `touchInset` 與圓角）的四個角，順時針。
    pub(crate) fn corners(&self) -> [(f64, f64); 4] {
        let (hw, hh) = (self.frame.w / 2.0, self.frame.h / 2.0);
        [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)].map(|(lx, ly)| to_world(self, lx, ly))
    }

    /// 觸控範圍內的取樣點（畫布座標），給「完全被蓋住」的檢查用：16 × 16 格，含貼著四邊的點。
    /// 比格距窄的空隙抓不到，所以這是警告不是錯誤。
    pub(crate) fn touch_samples(&self) -> Vec<(f64, f64)> {
        const N: usize = 16;
        // 貼邊但在範圍內：右、下邊不算在內。
        const EDGE: f64 = 1e-6;
        let area = touch_area(self);
        let along = |from: f64, to: f64, i: usize| {
            from + (to - from) * (EDGE + (1.0 - 2.0 * EDGE) * i as f64 / (N - 1) as f64)
        };
        let mut samples = Vec::new();
        for i in 0..N {
            for j in 0..N {
                let lx = along(area.left, area.right, i);
                let ly = along(area.top, area.bottom, j);
                let (x, y) = to_world(self, lx, ly);
                if self.contains(x, y) {
                    samples.push((x, y));
                }
            }
        }
        samples
    }
}

impl Page {
    /// 點在畫布座標 `(x, y)` 時按到的鍵在 `keys` 的位置：由上層往下，觸控範圍含這點的第一個；
    /// 都沒有時取距離最近的鍵，一樣近時取上層的。沒有按鍵時是 `None`。
    pub fn hit(&self, x: f64, y: f64) -> Option<usize> {
        let order = self.top_down();
        order
            .iter()
            .copied()
            .find(|&index| self.keys[index].contains(x, y))
            .or_else(|| {
                // `min_by` 一樣小時取先出現的，也就是上層的。
                order.iter().copied().min_by(|&a, &b| {
                    let distance = |index: usize| self.keys[index].distance(x, y);
                    distance(a).total_cmp(&distance(b))
                })
            })
    }

    /// 由上層往下層的按鍵位置：`z` 大的先，同一層時陣列裡後面的先。
    pub(crate) fn top_down(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.keys.len()).rev().collect();
        // 穩定排序，同一層保留反轉後的順序。
        order.sort_by_key(|&index| std::cmp::Reverse(self.keys[index].z));
        order
    }
}

/// 兩個凸多邊形重疊的面積是否大於 0（分離軸定理；只碰到邊或角不算）。
pub(crate) fn convex_overlap(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    const EPSILON: f64 = 1e-9;
    let axes = |polygon: &[(f64, f64)]| -> Vec<(f64, f64)> {
        (0..polygon.len())
            .map(|i| {
                let (p, q) = (polygon[i], polygon[(i + 1) % polygon.len()]);
                (-(q.1 - p.1), q.0 - p.0)
            })
            .collect()
    };
    let project = |polygon: &[(f64, f64)], axis: (f64, f64)| {
        polygon
            .iter()
            .map(|p| p.0 * axis.0 + p.1 * axis.1)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            })
    };
    axes(a).into_iter().chain(axes(b)).all(|axis| {
        let length = (axis.0 * axis.0 + axis.1 * axis.1).sqrt();
        if length == 0.0 {
            return true;
        }
        let axis = (axis.0 / length, axis.1 / length);
        let (a_lo, a_hi) = project(a, axis);
        let (b_lo, b_hi) = project(b, axis);
        a_hi.min(b_hi) - a_lo.max(b_lo) > EPSILON
    })
}
