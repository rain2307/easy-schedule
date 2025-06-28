use easy_schedule::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use time::{OffsetDateTime, Time, macros::offset};

fn print_time(name: &str) {
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let format = time::macros::format_description!("[hour]:[minute]:[second]");
    println!("[{}] {}: executed", now.format(&format).unwrap(), name);
}

#[derive(Debug)]
struct WaitTask(Arc<AtomicU32>);

#[async_trait]
impl Notifiable for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(2, None)
    }

    async fn on_time(&self, cancel: CancellationToken) {
        let count = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        print_time(&format!("WaitTask #{count}"));
        if count >= 3 {
            cancel.cancel();
        }
    }
}

#[derive(Debug)]
struct IntervalTask(Arc<AtomicU32>);

#[async_trait]
impl Notifiable for IntervalTask {
    fn get_schedule(&self) -> Task {
        Task::Interval(3, None)
    }

    async fn on_time(&self, cancel: CancellationToken) {
        let count = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        print_time(&format!("IntervalTask #{count}"));
        if count >= 5 {
            cancel.cancel();
        }
    }
}

#[derive(Debug)]
struct AtTask;

#[async_trait]
impl Notifiable for AtTask {
    fn get_schedule(&self) -> Task {
        Task::At(Time::from_hms(23, 59, 50).unwrap(), None)
    }

    async fn on_time(&self, cancel: CancellationToken) {
        print_time("AtTask");
        cancel.cancel();
    }
}

#[derive(Debug)]
struct OnceTask;

#[async_trait]
impl Notifiable for OnceTask {
    fn get_schedule(&self) -> Task {
        let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
        let next = now + time::Duration::seconds(5);
        Task::Once(next, None)
    }

    async fn on_time(&self, cancel: CancellationToken) {
        print_time("OnceTask");
        cancel.cancel();
    }
}

#[tokio::main]
async fn main() {
    println!("Easy Schedule Basic Example");
    println!("===========================");

    let scheduler = Scheduler::new();

    let wait_counter = Arc::new(AtomicU32::new(0));
    let interval_counter = Arc::new(AtomicU32::new(0));

    let wait_task = WaitTask(wait_counter.clone());
    let interval_task = IntervalTask(interval_counter.clone());
    let at_task = AtTask;
    let once_task = OnceTask;

    println!("Starting tasks...");

    scheduler.run(wait_task).await;
    scheduler.run(interval_task).await;
    scheduler.run(at_task).await;
    scheduler.run(once_task).await;

    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    println!("Stopping scheduler...");
    scheduler.stop();

    println!(
        "Final counts - Wait: {}, Interval: {}",
        wait_counter.load(Ordering::SeqCst),
        interval_counter.load(Ordering::SeqCst)
    );
}
