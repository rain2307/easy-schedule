use std::sync::Arc;

#[allow(unused_imports)]
use easy_schedule::{CancellationToken, ScheduledTask, Scheduler, Skip, Task};
use std::sync::atomic::{AtomicU32, Ordering};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
struct WaitTask;

impl ScheduledTask for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(3, None)
    }

    fn on_time(&self, _cancel: CancellationToken) {
        print_time("WaitTask on_time");
    }

    fn on_skip(&self, _cancel: CancellationToken) {
        print_time("WaitTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct IntervalTask {
    count: Arc<AtomicU32>,
}

impl IntervalTask {
    fn new() -> Self {
        Self {
            count: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl ScheduledTask for IntervalTask {
    fn get_schedule(&self) -> Task {
        Task::Interval(3, None)
        // Task::Interval(10, Some(Skip::Day(7)))
    }

    fn on_time(&self, _cancel: CancellationToken) {
        let n = self.count.fetch_add(1, Ordering::Relaxed);
        if n > 10 {
            println!("IntervalTask cancel");
            _cancel.cancel();
        }
        print_time(&format!("IntervalTask {}", n));
    }

    fn on_skip(&self, _cancel: CancellationToken) {
        print_time("IntervalTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct AtTask;

impl ScheduledTask for AtTask {
    fn get_schedule(&self) -> Task {
        let now = OffsetDateTime::now_local().unwrap();
        let next = now + time::Duration::seconds(5);
        Task::At(next.time(), None)
    }

    fn on_time(&self, _cancel: CancellationToken) {
        print_time("AtTask Execute");
    }

    fn on_skip(&self, _cancel: CancellationToken) {
        print_time("AtTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct OnceTask;

impl ScheduledTask for OnceTask {
    fn get_schedule(&self) -> Task {
        let now = OffsetDateTime::now_local().unwrap();
        let next = now + time::Duration::seconds(10);
        Task::Once(next)
    }

    fn on_time(&self, _cancel: CancellationToken) {
        print_time("OnceTask Execute");
    }

    fn on_skip(&self, _cancel: CancellationToken) {
        print_time("OnceTask on_skip");
    }
}

fn print_time(key: &str) {
    println!("{}", key);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Scheduler::start(WaitTask).await;
    Scheduler::start(IntervalTask::new()).await;
    Scheduler::start(AtTask).await;
    Scheduler::start(OnceTask).await;

    tokio::signal::ctrl_c().await.unwrap();
}
