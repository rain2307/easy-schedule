use easy_schedule::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use time::{OffsetDateTime, Time, macros::offset};

fn print_time(name: &str) {
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let format = time::macros::format_description!("[hour]:[minute]:[second]");
    println!(
        "[{}] {}: executed",
        now.format(&format).unwrap(),
        name
    );
}

#[derive(Debug)]
struct SkipWeekendTask(Arc<AtomicU32>);

#[async_trait]
impl Notifiable for SkipWeekendTask {
    fn get_schedule(&self) -> Task {
        let skip_weekends = vec![
            Skip::Day(vec![6, 7]), // Skip Saturday and Sunday
        ];
        Task::Interval(2, Some(skip_weekends))
    }

    async fn on_time(&self, cancel: CancellationToken) {
        let count = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        print_time(&format!("SkipWeekendTask #{} (weekday only)", count));
        if count >= 10 {
            cancel.cancel();
        }
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time("SkipWeekendTask SKIPPED (weekend)");
    }
}

#[derive(Debug)]
struct SkipNightTask(Arc<AtomicU32>);

#[async_trait]
impl Notifiable for SkipNightTask {
    fn get_schedule(&self) -> Task {
        let skip_night = vec![
            Skip::TimeRange(
                Time::from_hms(22, 0, 0).unwrap(),
                Time::from_hms(6, 0, 0).unwrap(),
            ), // Skip 22:00 - 06:00
        ];
        Task::Interval(1, Some(skip_night))
    }

    async fn on_time(&self, cancel: CancellationToken) {
        let count = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        print_time(&format!("SkipNightTask #{} (daytime only)", count));
        if count >= 15 {
            cancel.cancel();
        }
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time("SkipNightTask SKIPPED (nighttime)");
    }
}

#[derive(Debug)]
struct SkipSpecificDateTask;

#[async_trait]
impl Notifiable for SkipSpecificDateTask {
    fn get_schedule(&self) -> Task {
        let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
        let skip_today = vec![Skip::Date(now.date())];
        Task::Interval(1, Some(skip_today))
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        print_time("SkipSpecificDateTask (should not execute today)");
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time("SkipSpecificDateTask SKIPPED (today is skipped)");
    }
}

#[derive(Debug)]
struct MultipleSkipTask;

#[async_trait]
impl Notifiable for MultipleSkipTask {
    fn get_schedule(&self) -> Task {
        let multiple_skips = vec![
            Skip::Day(vec![6, 7]), // Skip weekends
            Skip::TimeRange(
                Time::from_hms(12, 0, 0).unwrap(),
                Time::from_hms(13, 0, 0).unwrap(),
            ), // Skip lunch time
            Skip::Time(Time::from_hms(15, 30, 0).unwrap()), // Skip 15:30
        ];
        Task::Interval(30 * 60, Some(multiple_skips)) // Every 30 minutes
    }

    async fn on_time(&self, _cancel: CancellationToken) {
        print_time("MultipleSkipTask (avoiding weekends, lunch, and 15:30)");
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time("MultipleSkipTask SKIPPED (multiple skip rules)");
    }
}

#[tokio::main]
async fn main() {
    println!("Easy Schedule Skip Example");
    println!("==========================");

    let scheduler = Scheduler::new();

    let weekend_counter = Arc::new(AtomicU32::new(0));
    let night_counter = Arc::new(AtomicU32::new(0));

    let skip_weekend_task = SkipWeekendTask(weekend_counter.clone());
    let skip_night_task = SkipNightTask(night_counter.clone());
    let skip_date_task = SkipSpecificDateTask;
    let multiple_skip_task = MultipleSkipTask;

    println!("Starting skip tasks...");
    println!("Note: Some tasks may skip execution based on current time/date");

    scheduler.run(skip_weekend_task).await;
    scheduler.run(skip_night_task).await;
    scheduler.run(skip_date_task).await;
    scheduler.run(multiple_skip_task).await;

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    println!("Stopping scheduler...");
    scheduler.stop();

    println!(
        "Final counts - Weekend: {}, Night: {}",
        weekend_counter.load(Ordering::SeqCst),
        night_counter.load(Ordering::SeqCst)
    );
}
