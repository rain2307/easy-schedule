use crossbeam_deque::Worker;
use std::{
    boxed::Box,
    fmt::{self, Debug},
    sync::Arc,
};
use time::{Date, OffsetDateTime, Time};
use tokio::{
    select,
    time::{Duration, Instant, sleep, sleep_until},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, instrument};

#[derive(Debug, Clone)]
pub enum Skip {
    /// skip fixed date
    Date(Date),
    /// skip date range
    DateRange(Date, Date),
    /// skip days
    ///
    /// 1: Monday, 2: Tuesday, 3: Wednesday, 4: Thursday, 5: Friday, 6: Saturday, 7: Sunday
    Day(usize),
    /// skip days range
    ///
    /// 1: Monday, 2: Tuesday, 3: Wednesday, 4: Thursday, 5: Friday, 6: Saturday, 7: Sunday
    DayRange(usize, usize),
    /// skip fixed time
    Time(Time),
    /// skip time range
    ///
    /// end must be greater than start
    TimeRange(Time, Time),
    /// no skip
    None,
}

impl Default for Skip {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Skip::Date(date) => write!(f, "date: {}", date),
            Skip::DateRange(start, end) => write!(f, "date range: {} - {}", start, end),
            Skip::Day(day) => write!(f, "day: {}", day),
            Skip::DayRange(start, end) => write!(f, "day range: {} - {}", start, end),
            Skip::Time(time) => write!(f, "time: {}", time),
            Skip::TimeRange(start, end) => write!(f, "time range: {} - {}", start, end),
            Skip::None => write!(f, "none"),
        }
    }
}

impl Skip {
    /// check if the time is skipped
    pub fn is_skip(&self, time: OffsetDateTime) -> bool {
        match self {
            Skip::Date(date) => time.date() == *date,
            Skip::DateRange(start, end) => time.date() >= *start && time.date() <= *end,
            Skip::Day(day) => time.day() + 1 == *day as u8,
            Skip::DayRange(start, end) => {
                time.day() + 1 >= *start as u8 && time.day() + 1 <= *end as u8
            }
            Skip::Time(time) => time.hour() == time.hour() && time.minute() == time.minute(),
            Skip::TimeRange(start, end) => {
                assert!(start < end, "start must be less than end");
                time.hour() >= start.hour()
                    && time.hour() <= end.hour()
                    && time.minute() >= start.minute()
                    && time.minute() <= end.minute()
            }
            Skip::None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Task {
    /// wait seconds
    Wait(u64, Option<Skip>),
    /// interval seconds
    Interval(u64, Option<Skip>),
    /// at time
    At(Time, Option<Skip>),
    /// exact time
    Once(OffsetDateTime),
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Task::Wait(wait, skip) => {
                write!(f, "wait: {} {}", wait, skip.clone().unwrap_or_default())
            }
            Task::Interval(interval, skip) => {
                write!(
                    f,
                    "interval: {} {}",
                    interval,
                    skip.clone().unwrap_or_default()
                )
            }
            Task::At(time, skip) => {
                write!(f, "at: {} {}", time, skip.clone().unwrap_or_default())
            }
            Task::Once(time) => write!(f, "once: {}", time),
        }
    }
}

/// a task that can be scheduled
pub trait ScheduledTask: Debug + Sync + Send {
    /// get the schedule type
    fn get_schedule(&self) -> Task;

    /// called when the task is scheduled
    fn on_time(&self);

    /// called when the task is skipped
    fn on_skip(&self);
}

pub struct Scheduler {
    tasks: Worker<Arc<Box<dyn ScheduledTask>>>,
    cancel: CancellationToken,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Worker::new_fifo(),
            cancel: CancellationToken::new(),
        }
    }

    /// start the scheduler
    pub async fn start(&self) {
        self.check().await;

        let cancel = self.cancel.clone();
        select! {
            _ = cancel.cancelled() => {
                tracing::debug!("scheduler cancelled");
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::debug!("ctrl+c");
                if !cancel.is_cancelled() {
                    cancel.cancel();
                }
            }
        }
    }

    pub async fn add_task(&self, task: Arc<Box<dyn ScheduledTask>>) {
        self.tasks.push(task);
        self.check().await;
    }

    /// 检查当前任务
    async fn check(&self) {
        while let Some(task) = self.tasks.pop() {
            let schedule = task.get_schedule();
            let cancel = self.cancel.clone();
            let task = task.clone();
            match schedule {
                Task::Wait(..) => {
                    Scheduler::run_wait(task, cancel.clone()).await;
                }
                Task::Interval(..) => {
                    Scheduler::run_interval(task, cancel.clone()).await;
                }
                Task::At(..) => {
                    Scheduler::run_at(task, cancel.clone()).await;
                }
                Task::Once(..) => {
                    Scheduler::run_once(task, cancel.clone()).await;
                }
            }
        }
    }

    /// stop the scheduler
    ///
    /// this will cancel all the tasks
    pub fn stop(&self) {
        self.cancel.cancel();
    }
}

fn get_next_time(now: OffsetDateTime, time: Time) -> OffsetDateTime {
    let mut next = now.replace_time(time);
    if next < now {
        next = next + time::Duration::days(1);
    }
    next
}

fn get_now() -> Option<OffsetDateTime> {
    match OffsetDateTime::now_local() {
        Ok(now) => Some(now),
        Err(e) => {
            error!("failed to get local time: {}", e);
            None
        }
    }
}

impl Scheduler {
    /// run wait task
    #[instrument(skip(task, cancel))]
    async fn run_wait(task: Arc<Box<dyn ScheduledTask>>, cancel: CancellationToken) {
        if let Task::Wait(wait, skip) = task.get_schedule() {
            tokio::spawn(async move {
                select! {
                    _ = cancel.cancelled() => {
                        return;
                    }
                    _ = sleep(Duration::from_secs(wait)) => {
                        tracing::debug!(wait, "wait seconds");
                    }
                };
                if let Some(now) = get_now() {
                    if let Some(skip) = skip {
                        if skip.is_skip(now) {
                            task.on_skip();
                            return;
                        }

                        task.on_time();
                    }
                }
            });
        }
    }

    /// run interval task
    #[instrument(skip(task, cancel))]
    async fn run_interval(task: Arc<Box<dyn ScheduledTask>>, cancel: CancellationToken) {
        if let Task::Interval(interval, skip) = task.get_schedule() {
            tokio::spawn(async move {
                loop {
                    select! {
                        _ = cancel.cancelled() => {
                            return;
                        }
                        _ = sleep(Duration::from_secs(interval)) => {
                            tracing::debug!(interval, "interval");
                        }
                    };
                    if let Some(now) = get_now() {
                        if let Some(ref skip) = skip {
                            if skip.is_skip(now) {
                                task.on_skip();
                                continue;
                            }
                            task.on_time();
                        }
                    }
                }
            });
        }
    }

    /// run at task
    #[instrument(skip(task, cancel))]
    async fn run_at(task: Arc<Box<dyn ScheduledTask>>, cancel: CancellationToken) {
        if let Task::At(time, skip) = task.get_schedule() {
            tokio::spawn(async move {
                let now = if let Some(now) = get_now() {
                    now
                } else {
                    return;
                };
                let mut next = get_next_time(now, time);
                loop {
                    let now = if let Some(now) = get_now() {
                        now
                    } else {
                        return;
                    };
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
                        if skip.is_skip(now) {
                            task.on_skip();
                            return;
                        }
                    }

                    task.on_time();

                    next += time::Duration::days(1);
                }
            });
        }
    }

    /// run once task
    #[instrument(skip(task, cancel))]
    async fn run_once(task: Arc<Box<dyn ScheduledTask>>, cancel: CancellationToken) {
        if let Task::Once(next) = task.get_schedule() {
            tokio::spawn(async move {
                if let Some(now) = get_now() {
                    if next < now {
                        task.on_skip();
                        return;
                    }
                    let seconds = (next - now).as_seconds_f64() as u64;
                    let instant = Instant::now() + Duration::from_secs(seconds);

                    select! {
                        _ = cancel.cancelled() => {
                            return;
                        }
                        _ = sleep_until(instant) => {
                            tracing::debug!("once time");
                        }
                    }
                    task.on_time();
                }
            });
        }
    }
}
