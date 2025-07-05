use easy_schedule::{prelude::*, task};
use time::{OffsetDateTime, macros::offset};

pub fn print_time(name: &str) {
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let format = time::macros::format_description!("[hour]:[minute]:[second]");
    println!("[{}] {}: executed", now.format(&format).unwrap(), name);
}

fn main() {
    println!("Easy Schedule String Skip Parsing Example");
    println!("========================================");

    // Test various string formats with skip conditions
    let test_cases = vec![
        // Basic tasks without skip
        ("wait(3)", "Basic Wait"),
        ("interval(2)", "Basic Interval"),
        ("at(23:59)", "Basic At"),
        // Single skip conditions
        ("wait(2, weekday 6)", "Skip Saturday"),
        ("interval(3, date 2024-12-25)", "Skip Christmas"),
        ("at(09:30, time 12:00..13:00)", "Skip Lunch Hour"),
        // Multiple skip conditions
        ("wait(2, [weekday 6, weekday 7])", "Skip Weekends"),
        (
            "interval(3, [date 2024-12-25, time 12:00..13:00])",
            "Skip Christmas & Lunch",
        ),
        (
            "at(09:30, [weekday 6, time 22:00..06:00])",
            "Skip Saturday & Night",
        ),
        // Complex scenarios
        (
            "wait(1, [weekday 1, weekday 2, weekday 3])",
            "Skip Mon/Tue/Wed",
        ),
        (
            "interval(2, [date 2024-12-25, date 2024-01-01, time 12:00..13:00])",
            "Skip Holidays & Lunch",
        ),
    ];

    println!("\n🔍 Parsing Demonstration:");
    println!("=========================");

    for (task_str, _name) in &test_cases {
        match Task::parse(task_str) {
            Ok(task) => {
                println!("✅ '{}' -> {}", task_str, task);
            }
            Err(err) => {
                println!("❌ '{}' -> Error: {}", task_str, err);
            }
        }
    }

    println!("\n🚫 Error Cases:");
    println!("==============");

    let error_cases = vec![
        ("wait(10, weekday 8)", "Invalid weekday"),
        ("wait(10, date 2024-13-01)", "Invalid month"),
        ("wait(10, time 25:00..26:00)", "Invalid time"),
        ("wait(10, [weekday 6, invalid 7])", "Invalid skip type"),
        ("wait(10, [weekday 0])", "Invalid weekday number"),
    ];

    for (task_str, description) in error_cases {
        match Task::parse(task_str) {
            Ok(task) => {
                println!(
                    "⚠️  '{}' ({}) -> Should have failed but got: {}",
                    task_str, description, task
                );
            }
            Err(err) => {
                println!(
                    "✅ '{}' ({}) -> Expected error: {}",
                    task_str, description, err
                );
            }
        }
    }

    println!("\n💡 Key Features:");
    println!("================");
    println!("📝 Single skip: wait(10, weekday 6)");
    println!("📝 Multiple skip: wait(10, [weekday 6, weekday 7])");
    println!("📝 Date skip: interval(5, date 2024-12-25)");
    println!("📝 Time range skip: at(09:30, time 12:00..13:00)");
    println!("📝 Single time skip: at(09:30, time 15:30)");
    println!("📝 Mixed skip types: wait(10, [weekday 6, date 2024-12-25, time 12:00..13:00])");

    println!("\n🎯 Comparison with Macro:");
    println!("=========================");

    // Compare string parsing with macro
    let string_task = Task::parse("wait(5, [weekday 6, weekday 7])").unwrap();
    let macro_task = task!(wait 5, [weekday 6, weekday 7]);

    println!("String: wait(5, [weekday 6, weekday 7])");
    println!("Macro:  task!(wait 5, [weekday 6, weekday 7])");
    println!("String result: {}", string_task);
    println!("Macro result:  {}", macro_task);
    println!("Equal: {}", string_task == macro_task);

    println!("\n✅ String parsing now supports skip conditions just like macros!");
    println!("🚀 You can now use both approaches interchangeably!");
}
