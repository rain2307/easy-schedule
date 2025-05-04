use easy_schedule::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
struct WaitTask;

#[async_trait]
impl Notifiable for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(3, None)
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        print_time("WaitTask on_time");
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
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

#[async_trait]
impl Notifiable for IntervalTask {
    fn get_schedule(&self) -> Task {
        Task::Interval(3, None)
        // Task::Interval(10, Some(Skip::Day(7)))
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        let n = self.count.fetch_add(1, Ordering::Relaxed);
        if n > 10 {
            println!("IntervalTask cancel");
            _cancel.cancel();
        }
        print_time(&format!("IntervalTask {}", n));
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time("IntervalTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct AtTask;

#[async_trait]
impl Notifiable for AtTask {
    fn get_schedule(&self) -> Task {
        let now = OffsetDateTime::now_local().unwrap();
        let next = now + time::Duration::seconds(5);
        Task::At(next.time(), None)
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        print_time("AtTask Execute");
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time("AtTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct OnceTask;

#[async_trait]
impl Notifiable for OnceTask {
    fn get_schedule(&self) -> Task {
        let now = OffsetDateTime::now_local().unwrap();
        let next = now + time::Duration::seconds(10);
        Task::Once(next)
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        print_time("OnceTask Execute");
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
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

    let scheduler = Scheduler::new();
    scheduler.run(WaitTask).await;
    scheduler.run(IntervalTask::new()).await;
    scheduler.run(AtTask).await;
    scheduler.run(OnceTask).await;

    tokio::signal::ctrl_c().await.unwrap();
}
