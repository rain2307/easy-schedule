# Easy Schedule

[![Crates.io](https://img.shields.io/crates/v/easy-schedule)](https://crates.io/crates/easy-schedule)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

A flexible task scheduler built on Tokio with multiple scheduling options and skip conditions.

## Features

- Multiple scheduling types:
  - Delayed execution
  - Interval execution
  - Scheduled execution
  - One-time execution
- Flexible skip conditions:
  - Skip specific dates
  - Skip date ranges
  - Skip weekdays
  - Skip specific times
  - Skip time ranges
- Task cancellation support
- Comprehensive logging

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
easy-schedule = "0.1"
```

## Usage Example

```rust
use easy_schedule::{Scheduler, Task, Skip};
use std::sync::Arc;
use time::Time;

#[derive(Debug)]
struct MyTask;

impl ScheduledTask for MyTask {
    fn get_schedule(&self) -> Task {
        Task::At(Time::try_from_hms(12, 0, 0).unwrap(), None)
    }

    fn on_time(&self) {
        println!("Task executed at noon");
    }

    fn on_skip(&self) {
        println!("Task skipped");
    }
}

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::new();
    scheduler.add_task(Arc::new(Box::new(MyTask))).await;
    scheduler.start().await;
}
```

## License

Dual-licensed under MIT or Apache-2.0.
