use std::sync::Arc;

use time::OffsetDateTime;
use easy_schedule::{Create, ScheduledTask, Scheduler, Skip};

#[derive(Debug, Clone)]
struct WaitTask;

impl ScheduledTask for WaitTask {
    fn get_schedule(&self) -> Create {
        Create::Wait(3, None)
    }

    fn on_time(&self) {
        println!("WaitTask");
    }

    fn on_skip(&self) {
        println!("WaitTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct IntervalTask;

impl ScheduledTask for IntervalTask {
    fn get_schedule(&self) -> Create {
        // Create::Interval(10, None)
        Create::Interval(10, Some(Skip::Day(7)))
    }

    fn on_time(&self) {
        println!("IntervalTask");
    }

    fn on_skip(&self) {
        println!("IntervalTask on_skip");
    }
}

#[derive(Debug, Clone)]
struct AtTask;

impl ScheduledTask for AtTask {
    fn get_schedule(&self) -> Create {
        let now = OffsetDateTime::now_local().unwrap();
        let next = now + time::Duration::minutes(1);
        Create::At(next.time(), None)
    }

    fn on_time(&self) {
        println!("AtTask");
    }

    fn on_skip(&self) {
        println!("on_skip");
    }
}

#[derive(Debug, Clone)]
struct OnceTask;

impl ScheduledTask for OnceTask {
    fn get_schedule(&self) -> Create {
        let now = OffsetDateTime::now_local().unwrap();
        let next = now + time::Duration::minutes(2);
        Create::Once(next)
    }

    fn on_time(&self) {
        println!("OnceTask");
    }

    fn on_skip(&self) {
        println!("OnceTask on_skip");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let scheduler = Scheduler::new();

    scheduler.add_task(Arc::new(Box::new(WaitTask))).await;
    scheduler.add_task(Arc::new(Box::new(IntervalTask))).await;
    scheduler.add_task(Arc::new(Box::new(AtTask))).await;
    scheduler.add_task(Arc::new(Box::new(OnceTask))).await;
    scheduler.start().await;
}
