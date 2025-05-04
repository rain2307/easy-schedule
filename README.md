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
use async_trait::async_trait;
use easy_schedule::{CancellationToken, Notifiable, Scheduler, Task};

#[derive(Debug, Clone)]
struct WaitTask;

#[async_trait]
impl Notifiable for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(3, None)
    }

    async fn on_time(&self, cancel: CancellationToken) {
        // ...
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        // ...
    }
}

#[tokio::main]
async fn main() {
    let scheduler = Scheduler::new();
    scheduler.start(WaitTask).await;
}

```

## License

Dual-licensed under MIT or Apache-2.0.
