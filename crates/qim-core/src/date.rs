//! 日期（plan §1.3、§2.4）：殼每次交給核心今天的日期，核心不讀系統時間；學習資料記每個條目最後
//! 使用的日期，給一年的時限用（M4 spec〈細節〉）。

use std::fmt;
use std::str::FromStr;

/// 一個日曆日（西曆），只到日、不帶時區：殼給的是使用者當地的今天。內部記距 1970-01-01 的天數，
/// 相減就是相隔幾天。年份限 0001–9999，文字是 ISO 8601 的 `YYYY-MM-DD`。沒有預設值：殼沒給的日期
/// 是「不知道」（`Option<Date>` 的 `None`），不是某個假日期，否則之後的一年時限會把它當成很久以前。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    days: i32,
}

impl Date {
    /// 西曆 `year` 年 `month` 月 `day` 日；不是真的日期（2 月 30 日）或年份不在 1–9999 時是 `None`。
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<Date> {
        let valid = (1..=9999).contains(&year)
            && (1..=12).contains(&month)
            && (1..=days_in_month(year, month)).contains(&day);
        valid.then(|| Date {
            days: days_from_civil(year, month, day),
        })
    }

    /// `earlier` 之後過了幾天；`earlier` 比自己晚時是負的。
    pub(crate) fn days_after(self, earlier: Date) -> i32 {
        self.days - earlier.days
    }

    /// （年, 月, 日）。
    fn ymd(self) -> (i32, u32, u32) {
        civil_from_days(self.days)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = self.ymd();
        write!(f, "{year:04}-{month:02}-{day:02}")
    }
}

impl FromStr for Date {
    type Err = ParseDateError;

    /// 只收 `YYYY-MM-DD`：四位年、兩位月日、半形數字，不收前後空白與其他寫法。
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let digits = |s: &str| {
            s.bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| s.parse::<u32>().ok())
                .flatten()
        };
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(ParseDateError);
        }
        let year = digits(&text[..4]).ok_or(ParseDateError)?;
        let month = digits(&text[5..7]).ok_or(ParseDateError)?;
        let day = digits(&text[8..]).ok_or(ParseDateError)?;
        let year = i32::try_from(year).map_err(|_| ParseDateError)?;
        Date::from_ymd(year, month, day).ok_or(ParseDateError)
    }
}

/// 讀不出日期（[`Date::from_str`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDateError;

impl fmt::Display for ParseDateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "日期要是 YYYY-MM-DD 的真實日期（0001–9999 年）")
    }
}

impl std::error::Error for ParseDateError {}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// 下面兩個是 Howard Hinnant 的 `days_from_civil`、`civil_from_days`
// （<https://howardhinnant.github.io/date_algorithms.html>，公有領域）：以 3 月起算的 400 年週期換算。

/// 距 1970-01-01 的天數。
fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400) as u32;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era as i32 - 719_468
}

/// [`days_from_civil`] 的反向。
fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097) as u32;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era as i32 + era * 400 + i32::from(month <= 2);
    (year, month, day)
}
