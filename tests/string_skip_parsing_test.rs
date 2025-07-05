use easy_schedule::Task;

#[test]
fn test_basic_tasks_without_skip() {
    let task = Task::parse("wait(10)").unwrap();
    assert!(matches!(task, Task::Wait(10, None)));

    let task = Task::parse("interval(5)").unwrap();
    assert!(matches!(task, Task::Interval(5, None)));

    let task = Task::parse("at(09:30)").unwrap();
    assert!(matches!(task, Task::At(_, None)));
}

#[test]
fn test_single_skip_conditions() {
    // Test weekday skip
    let task = Task::parse("wait(10, weekday 6)").unwrap();
    if let Task::Wait(10, Some(skips)) = task {
        assert_eq!(skips.len(), 1);
        assert!(matches!(skips[0], easy_schedule::Skip::Day(ref days) if days == &vec![6]));
    } else {
        panic!("Expected Wait task with skip");
    }

    // Test date skip
    let task = Task::parse("interval(5, date 2024-12-25)").unwrap();
    if let Task::Interval(5, Some(skips)) = task {
        assert_eq!(skips.len(), 1);
        assert!(matches!(skips[0], easy_schedule::Skip::Date(_)));
    } else {
        panic!("Expected Interval task with skip");
    }

    // Test time range skip
    let task = Task::parse("at(09:30, time 12:00..13:00)").unwrap();
    if let Task::At(_, Some(skips)) = task {
        assert_eq!(skips.len(), 1);
        assert!(matches!(skips[0], easy_schedule::Skip::TimeRange(_, _)));
    } else {
        panic!("Expected At task with skip");
    }
}

#[test]
fn test_multiple_skip_conditions() {
    // Test multiple weekday skips
    let task = Task::parse("wait(10, [weekday 6, weekday 7])").unwrap();
    if let Task::Wait(10, Some(skips)) = task {
        assert_eq!(skips.len(), 2);
        assert!(matches!(skips[0], easy_schedule::Skip::Day(ref days) if days == &vec![6]));
        assert!(matches!(skips[1], easy_schedule::Skip::Day(ref days) if days == &vec![7]));
    } else {
        panic!("Expected Wait task with multiple skips");
    }

    // Test mixed skip types
    let task = Task::parse("interval(5, [date 2024-12-25, time 12:00..13:00])").unwrap();
    if let Task::Interval(5, Some(skips)) = task {
        assert_eq!(skips.len(), 2);
        assert!(matches!(skips[0], easy_schedule::Skip::Date(_)));
        assert!(matches!(skips[1], easy_schedule::Skip::TimeRange(_, _)));
    } else {
        panic!("Expected Interval task with mixed skips");
    }
}

#[test]
fn test_error_cases() {
    // Invalid weekday
    assert!(Task::parse("wait(10, weekday 8)").is_err());

    // Invalid month
    assert!(Task::parse("wait(10, date 2024-13-01)").is_err());

    // Invalid time
    assert!(Task::parse("wait(10, time 25:00..26:00)").is_err());

    // Invalid skip type
    assert!(Task::parse("wait(10, [weekday 6, invalid 7])").is_err());
}

#[test]
fn test_parsing_display() {
    // Test that parsed tasks can be displayed
    let tests = vec![
        "wait(10, weekday 6)",
        "interval(5, date 2024-12-25)",
        "at(09:30, time 12:00..13:00)",
        "wait(10, [weekday 6, weekday 7])",
    ];

    for test in tests {
        let task = Task::parse(test).unwrap();
        let display = format!("{}", task);
        println!("Parsed '{}' -> '{}'", test, display);
        // Just ensure it doesn't panic
        assert!(!display.is_empty());
    }
}
