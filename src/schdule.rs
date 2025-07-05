use crate::task::{Notifiable, Task, get_next_time, get_now};
use time::OffsetDateTime;
use tokio::select;
use tokio::time::{Duration, Instant, sleep, sleep_until};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

pub struct Scheduler {
    cancel: CancellationToken,
    timezone_minutes: i16,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// create a new scheduler with default timezone (+8)
    pub fn new() -> Self {
        Self::with_timezone(8, 0)
    }

    /// create a new scheduler with specified timezone hours offset
    pub fn with_timezone(timezone_hours: i8, timezone_minutes: i8) -> Self {
        Self {
            cancel: CancellationToken::new(),
            timezone_minutes: (timezone_hours as i16) * 60 + (timezone_minutes as i16),
        }
    }

    /// create a new scheduler with timezone offset in minutes
    pub fn with_timezone_minutes(timezone_minutes: i16) -> Self {
        Self {
            cancel: CancellationToken::new(),
            timezone_minutes,
        }
    }

    /// run the task
    pub async fn run<T: Notifiable + 'static>(&self, task: T) {
        let schedule = task.get_task();
        let cancel = self.cancel.clone();
        let timezone_minutes = self.timezone_minutes;

        match schedule {
            Task::Wait(..) => {
                Scheduler::run_wait(task, cancel.clone(), timezone_minutes).await;
            }
            Task::Interval(..) => {
                Scheduler::run_interval(task, cancel.clone(), timezone_minutes).await;
            }
            Task::At(..) => {
                Scheduler::run_at(task, cancel.clone(), timezone_minutes).await;
            }
            Task::Once(..) => {
                Scheduler::run_once(task, cancel.clone(), timezone_minutes).await;
            }
        }
    }

    pub fn get_next_run_time<T: Notifiable + 'static>(&self, task: T) -> Option<OffsetDateTime> {
        let schedule = task.get_task();
        schedule.get_next_run_time::<T>(self.timezone_minutes)
    }

    /// stop the scheduler
    ///
    /// this will cancel all the tasks
    pub fn stop(&self) {
        self.cancel.cancel();
    }

    /// get the cancel token
    pub fn get_cancel(&self) -> CancellationToken {
        self.cancel.clone()
    }
}

impl Scheduler {
    /// run wait task
    #[instrument(skip(cancel))]
    async fn run_wait<T: Notifiable + 'static>(
        task: T,
        cancel: CancellationToken,
        timezone_minutes: i16,
    ) {
        if let Task::Wait(wait, skip) = task.get_task() {
            let task_ref = task;
            tokio::task::spawn(async move {
                select! {
                    _ = cancel.cancelled() => {
                        return;
                    }
                    _ = sleep(Duration::from_secs(wait)) => {
                        tracing::debug!(wait, "wait seconds");
                    }
                };
                let now = get_now(timezone_minutes).unwrap_or_else(|_| OffsetDateTime::now_utc());
                if let Some(skip) = skip {
                    if skip.iter().any(|s| s.is_skip(now)) {
                        task_ref.on_skip(cancel.clone()).await;
                        return;
                    }
                }
                task_ref.on_time(cancel.clone()).await;
            });
        }
    }

    /// run interval task
    #[instrument(skip(cancel))]
    async fn run_interval<T: Notifiable + 'static>(
        task: T,
        cancel: CancellationToken,
        timezone_minutes: i16,
    ) {
        if let Task::Interval(interval, skip) = task.get_task() {
            let task_ref = task;
            tokio::task::spawn(async move {
                loop {
                    select! {
                        _ = cancel.cancelled() => {
                            return;
                        }
                        _ = sleep(Duration::from_secs(interval)) => {
                            tracing::debug!(interval, "interval");
                        }
                    };
                    let now =
                        get_now(timezone_minutes).unwrap_or_else(|_| OffsetDateTime::now_utc());
                    if let Some(ref skip) = skip {
                        if skip.iter().any(|s| s.is_skip(now)) {
                            task_ref.on_skip(cancel.clone()).await;
                            continue;
                        }
                    }
                    task_ref.on_time(cancel.clone()).await;
                }
            });
        }
    }

    /// run at task
    #[instrument(skip(cancel))]
    async fn run_at<T: Notifiable + 'static>(
        task: T,
        cancel: CancellationToken,
        timezone_minutes: i16,
    ) {
        if let Task::At(time, skip) = task.get_task() {
            let task_ref = task;
            tokio::task::spawn(async move {
                let now = get_now(timezone_minutes).unwrap_or_else(|_| OffsetDateTime::now_utc());
                let mut next = get_next_time(now, time);
                loop {
                    let now =
                        get_now(timezone_minutes).unwrap_or_else(|_| OffsetDateTime::now_utc());
                    let seconds = (next - now).as_seconds_f64() as u64;
                    let instant = Instant::now() + Duration::from_secs(seconds);
                    select! {
                        _ = cancel.cancelled() => {
                            return;
                        }
                        _ = sleep_until(instant) => {
                            tracing::debug!("at time");
                        }
                    }

                    if let Some(skip) = skip.clone() {
                        if skip.iter().any(|s| s.is_skip(next)) {
                            task_ref.on_skip(cancel.clone()).await;
                            next += time::Duration::days(1);
                            continue;
                        }
                    }

                    task_ref.on_time(cancel.clone()).await;

                    next += time::Duration::days(1);
                }
            });
        }
    }

    /// run once task
    #[instrument(skip(task, cancel))]
    async fn run_once<T: Notifiable + 'static>(
        task: T,
        cancel: CancellationToken,
        timezone_minutes: i16,
    ) {
        if let Task::Once(next, skip) = task.get_task() {
            let task_ref = task;
            tokio::task::spawn(async move {
                let now = get_now(timezone_minutes).unwrap_or_else(|_| OffsetDateTime::now_utc());
                if next < now {
                    task_ref.on_skip(cancel.clone()).await;
                    return;
                }

                if let Some(skip) = skip {
                    if skip.iter().any(|s| s.is_skip(next)) {
                        task_ref.on_skip(cancel.clone()).await;
                        return;
                    }
                }
                let seconds = (next - now).as_seconds_f64();
                let instant = Instant::now() + Duration::from_secs(seconds as u64);

                select! {
                    _ = cancel.cancelled() => {
                        return;
                    }
                    _ = sleep_until(instant) => {
                        tracing::debug!("once time");
                    }
                }
                task_ref.on_time(cancel.clone()).await;
            });
        }
    }
}
