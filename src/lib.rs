use async_trait::async_trait;
use std::fmt::{self, Debug};
use time::{Date, OffsetDateTime, Time, UtcOffset, macros::format_description};
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

#[derive(Debug, Clone, PartialEq)]
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
            Skip::Day(day) => day.contains(&(time.weekday().number_from_monday())),
            Skip::DayRange(start, end) => {
                let weekday = time.weekday().number_from_monday() as usize;
                weekday >= *start && weekday <= *end
            }
            Skip::Time(skip_time) => time.time() == *skip_time,
            Skip::TimeRange(start, end) => {
                let current_time = time.time();
                if start <= end {
                    // 同一天内的时间范围
                    current_time >= *start && current_time <= *end
                } else {
                    // 跨日期的时间范围 (如 22:00 - 06:00)
                    current_time >= *start || current_time <= *end
                }
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
    Once(OffsetDateTime, Option<Vec<Skip>>),
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Task::Wait(a, skip_a), Task::Wait(b, skip_b)) => a == b && skip_a == skip_b,
            (Task::Interval(a, skip_a), Task::Interval(b, skip_b)) => a == b && skip_a == skip_b,
            (Task::At(a, skip_a), Task::At(b, skip_b)) => a == b && skip_a == skip_b,
            (Task::Once(a, skip_a), Task::Once(b, skip_b)) => a == b && skip_a == skip_b,
            _ => false,
        }
    }
}

impl Task {
    /// Parse a task from a string with detailed error reporting.
    ///
    /// # Examples
    ///
    /// ```
    /// use easy_schedule::Task;
    ///
    /// let task = Task::parse("wait(10)").unwrap();
    ///
    /// match Task::parse("invalid") {
    ///     Ok(task) => println!("Success: {}", task),
    ///     Err(err) => println!("Error: {}", err),
    /// }
    /// ```
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();

        // Find the function name and arguments
        let open_paren = s.find('(').ok_or_else(|| {
            format!(
                "Invalid task format: '{}'. Expected format like 'wait(10)'",
                s
            )
        })?;

        let close_paren = s
            .rfind(')')
            .ok_or_else(|| format!("Missing closing parenthesis in: '{}'", s))?;

        if close_paren <= open_paren {
            return Err(format!("Invalid parentheses in: '{}'", s));
        }

        let function_name = s[..open_paren].trim();
        let args = s[open_paren + 1..close_paren].trim();

        match function_name {
            "wait" => {
                let seconds = args
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid seconds value '{}' in wait({})", args, args))?;
                Ok(Task::Wait(seconds, None))
            }
            "interval" => {
                let seconds = args.parse::<u64>().map_err(|_| {
                    format!("Invalid seconds value '{}' in interval({})", args, args)
                })?;
                Ok(Task::Interval(seconds, None))
            }
            "at" => {
                let format = format_description!("[hour]:[minute]");
                let time = Time::parse(args, &format).map_err(|_| {
                    format!(
                        "Invalid time format '{}' in at({}). Expected format: HH:MM",
                        args, args
                    )
                })?;
                Ok(Task::At(time, None))
            }
            "once" => {
                let format = format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
                );
                let datetime = OffsetDateTime::parse(args, &format)
                    .map_err(|_| format!("Invalid datetime format '{}' in once({}). Expected format: YYYY-MM-DD HH:MM:SS +HH", args, args))?;
                Ok(Task::Once(datetime, None))
            }
            _ => Err(format!(
                "Unknown task type '{}'. Supported types: wait, interval, at, once",
                function_name
            )),
        }
    }
}

impl From<&str> for Task {
    /// Parse a task from a string, panicking on parse errors.
    ///
    /// For better error handling, consider using `Task::parse()` instead.
    ///
    /// # Panics
    ///
    /// Panics if the string cannot be parsed as a valid task.
    fn from(s: &str) -> Self {
        Task::parse(s).unwrap_or_else(|err| {
            panic!("Failed to parse task from string '{}': {}", s, err);
        })
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
        let task = Task::from("wait(10)");
        assert_eq!(task, Task::Wait(10, None));
        let task = Task::from("wait(10)".to_string());
        assert_eq!(task, Task::Wait(10, None));
        let task = Task::from(&"wait(10)".to_string());
        assert_eq!(task, Task::Wait(10, None));
    }

    #[test]
    fn test_from_string_interval() {
        let task = Task::from("interval(10)");
        assert_eq!(task, Task::Interval(10, None));
    }

    #[test]
    fn test_from_string_at() {
        let task = Task::from("at(10:00)");
        assert_eq!(task, Task::At(Time::from_hms(10, 0, 0).unwrap(), None));
    }

    #[test]
    fn test_from_string_once() {
        let task = Task::from("once(2024-01-01 10:00:00 +08)");
        assert_eq!(
            task,
            Task::Once(
                OffsetDateTime::from_unix_timestamp(1704074400).unwrap(),
                None
            )
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
            Task::Once(time, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "once: {} {}", time, skip)
            }
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
    timezone_minutes: i16,
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
        let schedule = task.get_schedule();
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

fn get_now(timezone_minutes: i16) -> Result<OffsetDateTime, time::error::ComponentRange> {
    let hours = timezone_minutes / 60;
    let minutes = timezone_minutes % 60;
    let offset = UtcOffset::from_hms(hours as i8, minutes as i8, 0)?;
    Ok(OffsetDateTime::now_utc().to_offset(offset))
}

impl Scheduler {
    /// run wait task
    #[instrument(skip(cancel))]
    async fn run_wait<T: Notifiable + 'static>(
        task: T,
        cancel: CancellationToken,
        timezone_minutes: i16,
    ) {
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
        if let Task::At(time, skip) = task.get_schedule() {
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
        if let Task::Once(next, skip) = task.get_schedule() {
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
