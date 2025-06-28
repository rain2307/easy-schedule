# Easy Schedule

[![Crates.io](https://img.shields.io/crates/v/easy-schedule)](https://crates.io/crates/easy-schedule)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![CI](https://github.com/rain2307/easy-schedule/actions/workflows/rust.yml/badge.svg)](https://github.com/rain2307/easy-schedule/actions/workflows/rust.yml)

A flexible and powerful task scheduler built on Tokio, providing multiple scheduling options with advanced skip conditions and timezone support.

## ✨ Features

### 🕐 Multiple Scheduling Types
- **Wait** - Execute once after a delay
- **Interval** - Execute repeatedly at fixed intervals  
- **At** - Execute daily at specific times
- **Once** - Execute at an exact datetime

### 🚫 Flexible Skip Conditions
- **Date-based**: Skip specific dates or date ranges
- **Weekday-based**: Skip specific weekdays or weekday ranges
- **Time-based**: Skip specific times or time ranges (including overnight ranges)
- **Combinable**: Use multiple skip rules together

### 🌍 Advanced Features
- **Timezone Support** - Full timezone support with minute-level precision
- **String Parsing** - Create tasks from intuitive strings like `wait(5)`, `at(14:30)`
- **Cancellation** - Comprehensive task cancellation support
- **Error Handling** - Robust error handling with sensible defaults
- **Async/Await** - Full async support with Tokio integration

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
easy-schedule = "0.10"
tokio = { version = "1", features = ["full"] }
time = { version = "0.3", features = ["macros"] }
```

## 📖 Usage Examples

### Basic Task Types

```rust
use easy_schedule::prelude::*;
use time::{Time, OffsetDateTime, macros::offset};

#[derive(Debug)]
struct MyTask {
    name: String,
}

#[async_trait]
impl Notifiable for MyTask {
    fn get_schedule(&self) -> Task {
        match self.name.as_str() {
            "wait" => Task::Wait(5, None),                    // Wait 5 seconds
            "interval" => Task::Interval(10, None),           // Every 10 seconds
            "daily" => Task::At(Time::from_hms(9, 0, 0).unwrap(), None), // Daily at 9:00 AM
            "once" => {
                let future = OffsetDateTime::now_utc() + time::Duration::minutes(5);
                Task::Once(future, None)                      // Once, 5 minutes from now
            }
            _ => Task::Wait(1, None),
        }
    }

    async fn on_time(&self, cancel: CancellationToken) {
        println!("{} executed!", self.name);
        cancel.cancel(); // Stop after first execution
    }
}

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::new();
    
    scheduler.run(MyTask { name: "wait".to_string() }).await;
    scheduler.run(MyTask { name: "interval".to_string() }).await;
    
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    scheduler.stop();
}
```

### Skip Conditions

```rust
use easy_schedule::prelude::*;
use time::{Time, macros::time};

#[derive(Debug)]
struct BusinessHoursTask;

#[async_trait]
impl Notifiable for BusinessHoursTask {
    fn get_schedule(&self) -> Task {
        let skip_rules = vec![
            Skip::Day(vec![6, 7]),                           // Skip weekends
            Skip::TimeRange(                                 // Skip night hours
                time!(22:00), 
                time!(06:00)
            ),
            Skip::Time(time!(12:00)),                        // Skip lunch time
        ];
        
        Task::Interval(3600, Some(skip_rules))              // Every hour, with skips
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        println!("Business hours task executed!");
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        println!("Skipped execution (outside business hours)");
    }
}
```

### String Parsing

```rust
use easy_schedule::prelude::*;

// ✅ Safe parsing with error handling
match Task::parse("wait(30)") {
    Ok(task) => println!("Task created: {}", task),
    Err(err) => println!("Parse error: {}", err),
}

// ⚠️ Direct parsing (panics on error)
let task = Task::from("wait(30)");                          // Wait 30 seconds

// Multiple tasks with error handling
let task_strings = vec!["wait(30)", "interval(60)", "at(14:30)"];
let tasks: Result<Vec<Task>, String> = task_strings
    .iter()
    .map(|s| Task::parse(s))
    .collect();

match tasks {
    Ok(tasks) => println!("All tasks parsed successfully: {} tasks", tasks.len()),
    Err(err) => println!("Parse failed: {}", err),
}
```

### Timezone Support

```rust
use easy_schedule::prelude::*;

// Different timezone configurations
let utc_scheduler = Scheduler::with_timezone(0, 0);         // UTC
let tokyo_scheduler = Scheduler::with_timezone(9, 0);       // JST
let india_scheduler = Scheduler::with_timezone(5, 30);      // IST
let ny_scheduler = Scheduler::with_timezone(-5, 0);         // EST

// Or use minute offsets directly
let custom_scheduler = Scheduler::with_timezone_minutes(330); // UTC+5:30
```

## 📋 Task Types Reference

| Type                      | Description               | Example                        |
| ------------------------- | ------------------------- | ------------------------------ |
| `Wait(seconds, skip)`     | Execute once after delay  | `Task::Wait(30, None)`         |
| `Interval(seconds, skip)` | Execute repeatedly        | `Task::Interval(60, None)`     |
| `At(time, skip)`          | Execute daily at time     | `Task::At(time!(14:30), None)` |
| `Once(datetime, skip)`    | Execute at exact datetime | `Task::Once(datetime, None)`   |

## 🚫 Skip Rules Reference

| Skip Type               | Description                  | Example                                       |
| ----------------------- | ---------------------------- | --------------------------------------------- |
| `Date(date)`            | Skip specific date           | `Skip::Date(date!(2024-12-25))`               |
| `DateRange(start, end)` | Skip date range              | `Skip::DateRange(start, end)`                 |
| `Day(weekdays)`         | Skip weekdays (1=Mon, 7=Sun) | `Skip::Day(vec![6, 7])`                       |
| `DayRange(start, end)`  | Skip weekday range           | `Skip::DayRange(1, 5)`                        |
| `Time(time)`            | Skip specific time           | `Skip::Time(time!(12:00))`                    |
| `TimeRange(start, end)` | Skip time range              | `Skip::TimeRange(time!(22:00), time!(06:00))` |

## 🛠️ Advanced Usage

### Custom Cancellation Logic

```rust
#[async_trait]
impl Notifiable for SmartTask {
    async fn on_time(&self, cancel: CancellationToken) {
        // Custom logic to decide when to stop
        if self.should_stop() {
            cancel.cancel();
        }
    }
}
```

### Multiple Skip Conditions

```rust
let complex_skips = vec![
    Skip::Day(vec![6, 7]),                    // No weekends
    Skip::TimeRange(time!(01:00), time!(06:00)), // No early morning
    Skip::Date(date!(2024-12-25)),            // No Christmas
];

Task::Interval(1800, Some(complex_skips))     // Every 30 minutes with conditions
```

## 🏗️ Architecture

The library follows a clean separation of concerns:

- **`Task`** - Defines when to execute (Wait, Interval, At, Once)
- **`Skip`** - Defines when NOT to execute (dates, times, weekdays)
- **`Notifiable`** - Your business logic (what to execute)
- **`Scheduler`** - Orchestrates everything with timezone support

## 🧪 Testing

Run the comprehensive test suite:

```bash
cargo test                    # Run all tests
cargo test --test skip_tests  # Test skip functionality
cargo test --test task_tests  # Test task parsing
cargo run --example basic     # Run basic example
```

## 📚 Examples

Check out the `examples/` directory for complete working examples:

- `basic.rs` - Basic scheduling with all task types
- `skip_example.rs` - Advanced skip conditions
- `string_parsing.rs` - String-based task creation
- `error_handling.rs` - Robust error handling patterns

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
