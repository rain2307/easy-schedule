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
- **Next Run Time** - Query when tasks will next execute
- **Error Handling** - Robust error handling with sensible defaults
- **Async/Await** - Full async support with Tokio integration

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
easy-schedule = "0.11"
tokio = { version = "1", features = ["full"] }
time = { version = "0.3", features = ["macros"] }
```

## 📖 Usage

### Basic Setup

```rust
use easy_schedule::prelude::*;
use time::{Time, OffsetDateTime, macros::offset};

#[derive(Debug)]
struct MyTask {
    name: String,
}

#[async_trait]
impl Notifiable for MyTask {
    fn get_task(&self) -> Task {
        Task::Wait(5, None) // Wait 5 seconds
    }

    async fn on_time(&self, cancel: CancellationToken) {
        println!("{} executed!", self.name);
        cancel.cancel(); // Stop after first execution
    }
}

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::new();
    
    scheduler.run(MyTask { name: "my_task".to_string() }).await;
    
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    scheduler.stop();
}
```

### Task Types

```rust
use easy_schedule::prelude::*;
use time::{Time, macros::time};

// Wait 30 seconds then execute once
let wait_task = Task::Wait(30, None);
let scheduler = Scheduler::new();

// Execute every 60 seconds
let interval_task = Task::Interval(60, None);
let scheduler = Scheduler::new();

// Execute daily at 9:00 AM
let at_task = Task::At(time!(09:00), None);
let scheduler = Scheduler::new();

// Execute once at specific datetime
let future = OffsetDateTime::now_utc() + time::Duration::minutes(5);
let once_task = Task::Once(future, None);
let scheduler = Scheduler::new();
```

### Skip Conditions

```rust
use easy_schedule::prelude::*;
use time::macros::time;

let skip_rules = vec![
    Skip::Day(vec![6, 7]),                           // Skip weekends
    Skip::TimeRange(time!(22:00), time!(06:00)),     // Skip night hours
    Skip::Time(time!(12:00)),                        // Skip lunch time
];

let task = Task::Interval(3600, Some(skip_rules));  // Every hour, with skips
let scheduler = Scheduler::new();
```

### Next Run Time

```rust
use easy_schedule::prelude::*;

let task = Task::Interval(60, None);
let scheduler = Scheduler::new();

// Create a test task that implements Notifiable
struct TestTask(Task);

#[async_trait]
impl Notifiable for TestTask {
    fn get_task(&self) -> Task { self.0.clone() }
    async fn on_time(&self, _cancel: CancellationToken) {}
}

let test_task = TestTask(task);
if let Some(next_time) = scheduler.get_next_run_time(test_task) {
    println!("Next execution: {}", next_time);
} else {
    println!("Task will not run (likely skipped)");
}
```

### Timezone Support

```rust
use easy_schedule::prelude::*;

// Different timezone configurations
let utc_scheduler = Scheduler::with_timezone(0, 0);         // UTC
let tokyo_scheduler = Scheduler::with_timezone(9, 0);       // JST
let india_scheduler = Scheduler::with_timezone(5, 30);      // IST

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

let task = Task::Interval(1800, Some(complex_skips));     // Every 30 minutes with conditions
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
cargo test                    # Run all tests
cargo test --test skip_tests  # Test skip functionality
cargo test --test task_tests  # Test task parsing
```

## 📄 License

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)