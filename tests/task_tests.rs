use easy_schedule::{Skip, Task};
use time::{
    OffsetDateTime,
    macros::{offset, time},
};

#[test]
fn test_task_from_string_wait() {
    let task = Task::from("wait(10)");
    assert_eq!(task, Task::Wait(10, None));

    let task_str = Task::from("wait(5)".to_string());
    assert_eq!(task_str, Task::Wait(5, None));

    let task_ref = Task::from(&"wait(20)".to_string());
    assert_eq!(task_ref, Task::Wait(20, None));
}

#[test]
fn test_task_from_string_interval() {
    let task = Task::from("interval(30)");
    assert_eq!(task, Task::Interval(30, None));
}

#[test]
fn test_task_from_string_at() {
    let task = Task::from("at(14:30)");
    assert_eq!(task, Task::At(time!(14:30:00), None));
}

#[test]
fn test_task_from_string_once() {
    let task = Task::from("once(2024-01-01 10:00:00 +08)");
    if let Task::Once(datetime, _) = task {
        let expected = OffsetDateTime::from_unix_timestamp(1704074400).unwrap();
        assert_eq!(datetime, expected);
    } else {
        panic!("Expected Task::Once");
    }
}

#[test]
fn test_task_parse_success() {
    let task = Task::parse("wait(10)").unwrap();
    assert_eq!(task, Task::Wait(10, None));

    let task = Task::parse("interval(30)").unwrap();
    assert_eq!(task, Task::Interval(30, None));

    let task = Task::parse("at(14:30)").unwrap();
    assert_eq!(task, Task::At(time!(14:30:00), None));
}

#[test]
fn test_task_parse_errors() {
    // Invalid function name
    let result = Task::parse("invalid(123)");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown task type"));

    // Invalid number
    let result = Task::parse("wait(abc)");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid seconds value"));

    // Invalid time format
    let result = Task::parse("at(25:70)");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid time format"));

    // Missing parentheses
    let result = Task::parse("wait 10");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid task format"));

    // Missing closing parenthesis
    let result = Task::parse("wait(10");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing closing parenthesis"));
}

#[test]
#[should_panic(expected = "Failed to parse task from string")]
fn test_task_from_string_invalid_panics() {
    let _task = Task::from("invalid(123)");
}

#[test]
fn test_task_partial_eq() {
    let wait1 = Task::Wait(10, None);
    let wait2 = Task::Wait(10, None);
    let wait3 = Task::Wait(20, None);
    let interval1 = Task::Interval(10, None);

    assert_eq!(wait1, wait2);
    assert_ne!(wait1, wait3);
    assert_ne!(wait1, interval1);
}

#[test]
fn test_task_partial_eq_with_skip() {
    let skip1 = Some(vec![Skip::Day(vec![1, 2])]);
    let skip2 = Some(vec![Skip::Day(vec![1, 2])]);
    let skip3 = Some(vec![Skip::Day(vec![3, 4])]);

    let wait1 = Task::Wait(10, skip1);
    let wait2 = Task::Wait(10, skip2);
    let wait3 = Task::Wait(10, skip3);
    let wait4 = Task::Wait(10, None);

    assert_eq!(wait1, wait2);
    assert_ne!(wait1, wait3);
    assert_ne!(wait1, wait4);
}

#[test]
fn test_task_display() {
    let wait_task = Task::Wait(10, None);
    let interval_task = Task::Interval(30, None);
    let at_task = Task::At(time!(14:30:00), None);
    let once_task = Task::Once(OffsetDateTime::now_utc().to_offset(offset!(+8)), None);

    assert!(format!("{}", wait_task).starts_with("wait: 10"));
    assert!(format!("{}", interval_task).starts_with("interval: 30"));
    assert!(format!("{}", at_task).starts_with("at: 14:30:00"));
    assert!(format!("{}", once_task).starts_with("once:"));
}

#[test]
fn test_task_display_with_skip() {
    let skip = Some(vec![Skip::Day(vec![1, 2]), Skip::Time(time!(12:00:00))]);
    let wait_task = Task::Wait(10, skip);

    let display = format!("{}", wait_task);
    assert!(display.contains("wait: 10"));
    assert!(display.contains("day: [1, 2]"));
    assert!(display.contains("time: 12:00:00"));
}

#[test]
fn test_task_clone() {
    let original = Task::Wait(10, Some(vec![Skip::Day(vec![1])]));
    let cloned = original.clone();

    assert_eq!(original, cloned);
}

#[test]
fn test_task_debug() {
    let task = Task::Wait(10, None);
    let debug_str = format!("{:?}", task);

    assert!(debug_str.contains("Wait"));
    assert!(debug_str.contains("10"));
}
