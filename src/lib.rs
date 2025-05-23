use async_trait::async_trait;
use std::fmt::{self, Debug};
use time::{
    Date, OffsetDateTime, Time,
    macros::{format_description, offset},
};
use tokio::{
    select,
    time::{Duration, Instant, sleep, sleep_until},
};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

pub mod prelude {
    pub use super::{Notifiable, Scheduler, Skip, Task};
    pub use async_trait::async_trait;
    pub use tokio_util::sync::CancellationToken;
}

#[derive(Debug, Clone)]
pub enum Skip {
    /// skip fixed date
    Date(Date),
    /// skip date range
    DateRange(Date, Date),
    /// skip days
    ///
    /// 1: Monday, 2: Tuesday, 3: Wednesday, 4: Thursday, 5: Friday, 6: Saturday, 7: Sunday
    Day(Vec<u8>),
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
            Skip::Day(day) => write!(f, "day: {:?}", day),
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
            Skip::Day(day) => day.contains(&(time.day() + 1)),
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
    Wait(u64, Option<Vec<Skip>>),
    /// interval seconds
    Interval(u64, Option<Vec<Skip>>),
    /// at time
    At(Time, Option<Vec<Skip>>),
    /// exact time
    Once(OffsetDateTime),
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Task::Wait(a, _), Task::Wait(b, _)) => a == b,
            (Task::Interval(a, _), Task::Interval(b, _)) => a == b,
            (Task::At(a, _), Task::At(b, _)) => a == b,
            (Task::Once(a), Task::Once(b)) => a == b,
            _ => false,
        }
    }
}

impl From<&str> for Task {
    ///
    /// - wait=10
    /// - interval=10
    /// - at=10:00
    /// - once=2024-01-01 10:00:00
    fn from(s: &str) -> Self {
        let parts = s.split("=").collect::<Vec<&str>>();
        let task = parts[0];
        let value = parts[1..].join("");
        match task {
            "wait" => {
                let seconds = value.parse::<u64>().unwrap();
                Task::Wait(seconds, None)
            }
            "interval" => {
                let seconds = value.parse::<u64>().unwrap();
                Task::Interval(seconds, None)
            }
            "at" => {
                let format = format_description!("[hour]:[minute]");
                let time = Time::parse(&value, &format).expect("parse time failed");
                Task::At(time, None)
            }
            "once" => {
                let format = format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
                );
                let datetime =
                    OffsetDateTime::parse(&value, &format).expect("parse datetime failed");
                Task::Once(datetime)
            }
            _ => Task::Wait(5, None),
        }
    }
}

impl From<String> for Task {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<&String> for Task {
    fn from(s: &String) -> Self {
        Self::from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let task = Task::from("wait=10");
        assert_eq!(task, Task::Wait(10, None));
        let task = Task::from("wait=10".to_string());
        assert_eq!(task, Task::Wait(10, None));
        let task = Task::from(&"wait=10".to_string());
        assert_eq!(task, Task::Wait(10, None));
    }

    #[test]
    fn test_from_string_interval() {
        let task = Task::from("interval=10");
        assert_eq!(task, Task::Interval(10, None));
    }

    #[test]
    fn test_from_string_at() {
        let task = Task::from("at=10:00");
        assert_eq!(task, Task::At(Time::from_hms(10, 0, 0).unwrap(), None));
    }

    #[test]
    fn test_from_string_once() {
        let task = Task::from("once=2024-01-01 10:00:00 +08");
        assert_eq!(
            task,
            Task::Once(OffsetDateTime::from_unix_timestamp(1704074400).unwrap())
        );
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Task::Wait(wait, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "wait: {} {}", wait, skip)
            }
            Task::Interval(interval, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "interval: {} {}", interval, skip)
            }
            Task::At(time, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "at: {} {}", time, skip)
            }
            Task::Once(time) => write!(f, "once: {}", time),
        }
    }
}

/// a task that can be scheduled
#[async_trait]
pub trait Notifiable: Sync + Send + Debug {
    /// get the schedule type
    fn get_schedule(&self) -> Task;

    /// called when the task is scheduled
    ///
    /// Default cancel on first trigger
    async fn on_time(&self, cancel: CancellationToken) {
        cancel.cancel();
    }

    /// called when the task is skipped
    async fn on_skip(&self, _cancel: CancellationToken) {
        // do nothing
    }
}

pub struct Scheduler {
    cancel: CancellationToken,
}

impl Scheduler {
    /// create a new scheduler
    pub fn new() -> Self {
        Self {
            cancel: CancellationToken::new(),
        }
    }

    /// run the task
    pub async fn run<T: Notifiable + 'static>(&self, task: T) {
        let schedule = task.get_schedule();
        let cancel = self.cancel.clone();

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

fn get_next_time(now: OffsetDateTime, time: Time) -> OffsetDateTime {
    let mut next = now.replace_time(time);
    if next < now {
        next = next + time::Duration::days(1);
    }
    next
}

fn get_now() -> OffsetDateTime {
    // FIXME:
    OffsetDateTime::now_utc().to_offset(offset!(+8))
}

impl Scheduler {
    /// run wait task
    #[instrument(skip(cancel))]
    async fn run_wait<T: Notifiable + 'static>(task: T, cancel: CancellationToken) {
        if let Task::Wait(wait, skip) = task.get_schedule() {
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
                let now = get_now();
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
    async fn run_interval<T: Notifiable + 'static>(task: T, cancel: CancellationToken) {
        if let Task::Interval(interval, skip) = task.get_schedule() {
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
                    let now = get_now();
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
    async fn run_at<T: Notifiable + 'static>(task: T, cancel: CancellationToken) {
        if let Task::At(time, skip) = task.get_schedule() {
            let task_ref = task;
            tokio::task::spawn(async move {
                let now = get_now();
                let mut next = get_next_time(now, time);
                loop {
                    let now = get_now();
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
                        if skip.iter().any(|s| s.is_skip(now)) {
                            task_ref.on_skip(cancel.clone()).await;
                            return;
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
    async fn run_once<T: Notifiable + 'static>(task: T, cancel: CancellationToken) {
        if let Task::Once(next) = task.get_schedule() {
            let task_ref = task;
            tokio::task::spawn(async move {
                let now = get_now();
                if next < now {
                    task_ref.on_skip(cancel.clone()).await;
                    return;
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
