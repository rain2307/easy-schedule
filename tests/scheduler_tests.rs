use async_trait::async_trait;
use easy_schedule::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use time::{OffsetDateTime, macros::offset};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
struct TestTask {
    task: Task,
    counter: Arc<AtomicU32>,
    skip_counter: Arc<AtomicU32>,
    should_cancel: Arc<AtomicBool>,
}

impl TestTask {
    fn new(task: Task) -> Self {
        Self {
            task,
            counter: Arc::new(AtomicU32::new(0)),
            skip_counter: Arc::new(AtomicU32::new(0)),
            should_cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    fn with_auto_cancel(task: Task, max_executions: u32) -> Self {
        let test_task = Self::new(task);
        let counter = test_task.counter.clone();
        let should_cancel = test_task.should_cancel.clone();

        tokio::spawn(async move {
            while counter.load(Ordering::SeqCst) < max_executions {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            should_cancel.store(true, Ordering::SeqCst);
        });

        test_task
    }

    fn execution_count(&self) -> u32 {
        self.counter.load(Ordering::SeqCst)
    }

    fn skip_count(&self) -> u32 {
        self.skip_counter.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Notifiable for TestTask {
    fn get_task(&self) -> Task {
        self.task.clone()
    }

    async fn on_time(&self, cancel: CancellationToken) {
        self.counter.fetch_add(1, Ordering::SeqCst);
        if self.should_cancel.load(Ordering::SeqCst) {
            cancel.cancel();
        }
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        self.skip_counter.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn test_scheduler_creation() {
    let scheduler = Scheduler::new();
    assert!(!scheduler.get_cancel().is_cancelled());

    let scheduler_with_timezone = Scheduler::with_timezone(5, 30);
    assert!(!scheduler_with_timezone.get_cancel().is_cancelled());

    let scheduler_with_minutes = Scheduler::with_timezone_minutes(330);
    assert!(!scheduler_with_minutes.get_cancel().is_cancelled());
}

#[tokio::test]
async fn test_scheduler_stop() {
    let scheduler = Scheduler::new();
    assert!(!scheduler.get_cancel().is_cancelled());

    scheduler.stop();
    assert!(scheduler.get_cancel().is_cancelled());
}

#[tokio::test]
async fn test_wait_task() {
    let scheduler = Scheduler::new();
    let task = TestTask::with_auto_cancel(Task::Wait(1, None), 3);

    scheduler.run(task.clone()).await;

    tokio::time::sleep(Duration::from_secs(4)).await;

    assert!(task.execution_count() >= 1);
    assert_eq!(task.skip_count(), 0);
}

#[tokio::test]
async fn test_interval_task() {
    let scheduler = Scheduler::new();
    let task = TestTask::with_auto_cancel(Task::Interval(1, None), 3);

    scheduler.run(task.clone()).await;

    tokio::time::sleep(Duration::from_secs(4)).await;

    assert!(task.execution_count() >= 3);
    assert_eq!(task.skip_count(), 0);
}

#[tokio::test]
async fn test_once_task_future() {
    let scheduler = Scheduler::new();
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let future_time = now + time::Duration::seconds(2);
    let task = TestTask::new(Task::Once(future_time, None));

    scheduler.run(task.clone()).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    assert_eq!(task.execution_count(), 1);
    assert_eq!(task.skip_count(), 0);
}

#[tokio::test]
async fn test_once_task_past() {
    let scheduler = Scheduler::new();
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let past_time = now - time::Duration::seconds(10);
    let task = TestTask::new(Task::Once(past_time, None));

    scheduler.run(task.clone()).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    assert_eq!(task.execution_count(), 0);
    assert_eq!(task.skip_count(), 1);
}

#[tokio::test]
async fn test_task_with_skip() {
    let scheduler = Scheduler::new();
    let skip = Some(vec![Skip::Day(vec![1, 2, 3, 4, 5, 6, 7])]); // Skip all days
    let task = TestTask::new(Task::Interval(1, skip));

    scheduler.run(task.clone()).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    assert_eq!(task.execution_count(), 0);
    assert!(task.skip_count() > 0);
}

#[tokio::test]
async fn test_multiple_tasks() {
    let scheduler = Scheduler::new();

    let task1 = TestTask::with_auto_cancel(Task::Wait(1, None), 2);
    let task2 = TestTask::with_auto_cancel(Task::Interval(1, None), 2);

    scheduler.run(task1.clone()).await;
    scheduler.run(task2.clone()).await;

    tokio::time::sleep(Duration::from_secs(4)).await;

    assert!(task1.execution_count() >= 1);
    assert!(task2.execution_count() >= 2);
}

#[tokio::test]
async fn test_scheduler_cancel_all() {
    let scheduler = Scheduler::new();

    let task1 = TestTask::new(Task::Interval(2, None)); // Use longer intervals
    let task2 = TestTask::new(Task::Interval(2, None));

    scheduler.run(task1.clone()).await;
    scheduler.run(task2.clone()).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    let count1_before = task1.execution_count();
    let count2_before = task2.execution_count();

    // Ensure both tasks have executed at least once
    assert!(count1_before > 0);
    assert!(count2_before > 0);

    scheduler.stop();

    // Wait a bit to ensure cancellation takes effect
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Then wait longer and verify no new executions
    tokio::time::sleep(Duration::from_secs(3)).await;

    assert_eq!(task1.execution_count(), count1_before);
    assert_eq!(task2.execution_count(), count2_before);
}

#[tokio::test]
async fn test_at_task_skip_logic() {
    let scheduler = Scheduler::new();

    // Use current time + 1 second to ensure it triggers soon
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let future_time = (now + time::Duration::seconds(1)).time();

    // Create a skip rule that will definitely trigger
    let skip = Some(vec![Skip::Day(vec![1, 2, 3, 4, 5, 6, 7])]); // Skip all days
    let task = TestTask::new(Task::At(future_time, skip));

    scheduler.run(task.clone()).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Since we skip all days, it should not execute but should skip
    assert_eq!(task.execution_count(), 0);
    assert!(task.skip_count() > 0);
}

#[tokio::test]
async fn test_get_next_run_time_wait() {
    let task = Task::Wait(60, None);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    assert!(next_time.is_some());

    let now = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+8));
    let expected_next = now + time::Duration::seconds(60);
    let diff = (next_time.unwrap() - expected_next).abs();

    // Allow for small timing differences
    assert!(diff < time::Duration::seconds(2));
}

#[tokio::test]
async fn test_get_next_run_time_interval() {
    let task = Task::Interval(30, None);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    assert!(next_time.is_some());

    let now = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+8));
    let expected_next = now + time::Duration::seconds(30);
    let diff = (next_time.unwrap() - expected_next).abs();

    // Allow for small timing differences
    assert!(diff < time::Duration::seconds(2));
}

#[tokio::test]
async fn test_get_next_run_time_at() {
    let test_time = time::Time::from_hms(14, 30, 0).unwrap();
    let task = Task::At(test_time, None);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    assert!(next_time.is_some());

    let next = next_time.unwrap();
    assert_eq!(next.time(), test_time);

    let now = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+8));
    if next.date() == now.date() {
        // If today, should be later than now
        assert!(next > now);
    } else {
        // If tomorrow, should be tomorrow
        assert_eq!(next.date(), now.date() + time::Duration::days(1));
    }
}

#[tokio::test]
async fn test_get_next_run_time_once_future() {
    let now = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+8));
    let future_time = now + time::Duration::seconds(120);
    let task = Task::Once(future_time, None);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    assert!(next_time.is_some());
    assert_eq!(next_time.unwrap(), future_time);
}

#[tokio::test]
async fn test_get_next_run_time_once_past() {
    let now = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+8));
    let past_time = now - time::Duration::seconds(60);
    let task = Task::Once(past_time, None);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    assert!(next_time.is_none());
}

#[tokio::test]
async fn test_get_next_run_time_with_skip() {
    let skip = Some(vec![Skip::Day(vec![1, 2, 3, 4, 5, 6, 7])]); // Skip all days
    let task = Task::Interval(5, skip);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    // Should return None when all days are skipped
    assert!(next_time.is_none());
}

#[tokio::test]
async fn test_get_next_run_time_once_with_skip() {
    let now = OffsetDateTime::now_utc().to_offset(time::macros::offset!(+8));
    let future_time = now + time::Duration::seconds(60);
    let skip = Some(vec![Skip::Day(vec![1, 2, 3, 4, 5, 6, 7])]); // Skip all days
    let task = Task::Once(future_time, skip);
    let scheduler = Scheduler::new();

    let test_task = TestTask::new(task);
    let next_time = scheduler.get_next_run_time(test_task);
    // Should return None because the once task is skipped
    assert!(next_time.is_none());
}
