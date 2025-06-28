use easy_schedule::prelude::Skip;
use time::{
    OffsetDateTime,
    macros::{date, time},
};

#[test]
fn test_skip_date() {
    let skip = Skip::Date(date!(2024 - 12 - 25));
    let christmas = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(10:00:00));
    let other_day = OffsetDateTime::new_utc(date!(2024 - 12 - 24), time!(10:00:00));

    assert!(skip.is_skip(christmas));
    assert!(!skip.is_skip(other_day));
}

#[test]
fn test_skip_date_range() {
    let skip = Skip::DateRange(date!(2024 - 12 - 24), date!(2024 - 12 - 26));
    let christmas_eve = OffsetDateTime::new_utc(date!(2024 - 12 - 24), time!(10:00:00));
    let christmas = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(10:00:00));
    let boxing_day = OffsetDateTime::new_utc(date!(2024 - 12 - 26), time!(10:00:00));
    let before = OffsetDateTime::new_utc(date!(2024 - 12 - 23), time!(10:00:00));
    let after = OffsetDateTime::new_utc(date!(2024 - 12 - 27), time!(10:00:00));

    assert!(skip.is_skip(christmas_eve));
    assert!(skip.is_skip(christmas));
    assert!(skip.is_skip(boxing_day));
    assert!(!skip.is_skip(before));
    assert!(!skip.is_skip(after));
}

#[test]
fn test_skip_day_weekday() {
    let skip = Skip::Day(vec![6, 7]); // Saturday and Sunday

    // Create a known Saturday (2024-12-21 is a Saturday)
    let saturday = OffsetDateTime::new_utc(date!(2024 - 12 - 21), time!(10:00:00));
    // Create a known Sunday (2024-12-22 is a Sunday)
    let sunday = OffsetDateTime::new_utc(date!(2024 - 12 - 22), time!(10:00:00));
    // Create a known Monday (2024-12-23 is a Monday)
    let monday = OffsetDateTime::new_utc(date!(2024 - 12 - 23), time!(10:00:00));

    assert!(skip.is_skip(saturday));
    assert!(skip.is_skip(sunday));
    assert!(!skip.is_skip(monday));
}

#[test]
fn test_skip_day_range() {
    let skip = Skip::DayRange(1, 5); // Monday to Friday

    // Create a known Monday (2024-12-23 is a Monday, weekday = 1)
    let monday = OffsetDateTime::new_utc(date!(2024 - 12 - 23), time!(10:00:00));
    // Create a known Friday (2024-12-27 is a Friday, weekday = 5)
    let friday = OffsetDateTime::new_utc(date!(2024 - 12 - 27), time!(10:00:00));
    // Create a known Saturday (2024-12-21 is a Saturday, weekday = 6)
    let saturday = OffsetDateTime::new_utc(date!(2024 - 12 - 21), time!(10:00:00));

    assert!(skip.is_skip(monday));
    assert!(skip.is_skip(friday));
    assert!(!skip.is_skip(saturday));
}

#[test]
fn test_skip_time() {
    let skip = Skip::Time(time!(14:30:00));
    let target_time = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(14:30:00));
    let other_time = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(14:31:00));

    assert!(skip.is_skip(target_time));
    assert!(!skip.is_skip(other_time));
}

#[test]
fn test_skip_time_range_same_day() {
    let skip = Skip::TimeRange(time!(09:00:00), time!(17:00:00)); // 9 AM to 5 PM
    let morning = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(10:00:00));
    let evening = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(18:00:00));
    let start_time = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(09:00:00));
    let end_time = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(17:00:00));

    assert!(skip.is_skip(morning));
    assert!(skip.is_skip(start_time));
    assert!(skip.is_skip(end_time));
    assert!(!skip.is_skip(evening));
}

#[test]
fn test_skip_time_range_overnight() {
    let skip = Skip::TimeRange(time!(22:00:00), time!(06:00:00)); // 10 PM to 6 AM
    let night = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(23:00:00));
    let early_morning = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(05:00:00));
    let morning = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(07:00:00));
    let afternoon = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(14:00:00));

    assert!(skip.is_skip(night));
    assert!(skip.is_skip(early_morning));
    assert!(!skip.is_skip(morning));
    assert!(!skip.is_skip(afternoon));
}

#[test]
fn test_skip_none() {
    let skip = Skip::None;
    let any_time = OffsetDateTime::new_utc(date!(2024 - 12 - 25), time!(10:00:00));

    assert!(!skip.is_skip(any_time));
}

#[test]
fn test_skip_display() {
    let date_skip = Skip::Date(date!(2024 - 12 - 25));
    let time_skip = Skip::Time(time!(14:30:00));
    let day_skip = Skip::Day(vec![1, 2, 3]);
    let none_skip = Skip::None;

    assert_eq!(format!("{date_skip}"), "date: 2024-12-25");
    assert!(format!("{time_skip}").starts_with("time: 14:30:00"));
    assert_eq!(format!("{day_skip}"), "day: [1, 2, 3]");
    assert_eq!(format!("{none_skip}"), "none");
}
