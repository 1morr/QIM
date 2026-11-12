//! 今天的日期（plan §1.3、§2.4）：殼交給核心，核心不讀系統時間；學習資料的匯出寫成 `YYYY-MM-DD`。

use qim_core::Date;

#[test]
fn a_date_reads_and_writes_as_iso_8601() {
    let date: Date = "2026-11-12".parse().unwrap();
    assert_eq!(date.to_string(), "2026-11-12");
    assert_eq!(Date::from_ymd(2026, 11, 12), Some(date));
    assert_eq!(Date::from_ymd(1, 1, 1).unwrap().to_string(), "0001-01-01");
}

#[test]
fn dates_order_by_calendar_across_months_years_and_leap_days() {
    let dates = [
        "1970-01-01",
        "1999-12-31",
        "2000-02-29",
        "2000-03-01",
        "2026-11-12",
        "9999-12-31",
    ]
    .map(|s| s.parse::<Date>().unwrap());
    assert!(dates.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn only_real_calendar_dates_in_the_exact_format_are_accepted() {
    let bad = [
        "",
        "2026-11-1",
        "2026-1-12",
        "26-11-12",
        "2026/11/12",
        "2026-11-12 ",
        "+2026-11-12",
        "2026-13-01",
        "2026-00-10",
        "2026-11-00",
        "2026-11-31",
        "2025-02-29",
        "1900-02-29",
        "0000-01-01",
        "２０２６-11-12",
    ];
    for text in bad {
        assert!(text.parse::<Date>().is_err(), "{text:?}");
    }
    assert!("2024-02-29".parse::<Date>().is_ok());
    assert_eq!(Date::from_ymd(2026, 2, 30), None);
    assert_eq!(Date::from_ymd(10_000, 1, 1), None);
}

/// 每年每月的頭、尾兩天：寫出來再讀回同一天，而且一天比一天晚（換算的 400 年週期、閏年邊界）。
#[test]
fn every_month_of_every_year_writes_and_reads_back_in_order() {
    let mut previous: Option<Date> = None;
    for year in 1..=9999 {
        for month in 1..=12 {
            let last = (28..=31)
                .rev()
                .find(|&d| Date::from_ymd(year, month, d).is_some())
                .unwrap();
            for day in [1, last] {
                let date = Date::from_ymd(year, month, day).unwrap();
                let text = date.to_string();
                assert_eq!(text, format!("{year:04}-{month:02}-{day:02}"));
                assert_eq!(text.parse::<Date>(), Ok(date));
                assert!(previous < Some(date), "{text}");
                previous = Some(date);
            }
        }
    }
}
