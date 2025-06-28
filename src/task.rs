use std::fmt::{self, Debug};
use time::{Date, OffsetDateTime, Time, macros::format_description};

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
            Skip::Date(date) => write!(f, "date: {date}"),
            Skip::DateRange(start, end) => write!(f, "date range: {start} - {end}"),
            Skip::Day(day) => write!(f, "day: {day:?}"),
            Skip::DayRange(start, end) => write!(f, "day range: {start} - {end}"),
            Skip::Time(time) => write!(f, "time: {time}"),
            Skip::TimeRange(start, end) => write!(f, "time range: {start} - {end}"),
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
            format!("Invalid task format: '{s}'. Expected format like 'wait(10)'")
        })?;

        let close_paren = s
            .rfind(')')
            .ok_or_else(|| format!("Missing closing parenthesis in: '{s}'"))?;

        if close_paren <= open_paren {
            return Err(format!("Invalid parentheses in: '{s}'"));
        }

        let function_name = s[..open_paren].trim();
        let args = s[open_paren + 1..close_paren].trim();

        match function_name {
            "wait" => {
                let seconds = args
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid seconds value '{args}' in wait({args})"))?;
                Ok(Task::Wait(seconds, None))
            }
            "interval" => {
                let seconds = args
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid seconds value '{args}' in interval({args})"))?;
                Ok(Task::Interval(seconds, None))
            }
            "at" => {
                let format = format_description!("[hour]:[minute]");
                let time = Time::parse(args, &format).map_err(|_| {
                    format!("Invalid time format '{args}' in at({args}). Expected format: HH:MM")
                })?;
                Ok(Task::At(time, None))
            }
            "once" => {
                let format = format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
                );
                let datetime = OffsetDateTime::parse(args, &format)
                    .map_err(|_| format!("Invalid datetime format '{args}' in once({args}). Expected format: YYYY-MM-DD HH:MM:SS +HH"))?;
                Ok(Task::Once(datetime, None))
            }
            _ => Err(format!(
                "Unknown task type '{function_name}'. Supported types: wait, interval, at, once"
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
            panic!("Failed to parse task from string '{s}': {err}");
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

#[macro_export]
macro_rules! task {
    // 基础任务，无skip
    (wait $seconds:tt) => {
        $crate::Task::Wait($seconds, None)
    };
    (interval $seconds:tt) => {
        $crate::Task::Interval($seconds, None)
    };
    (at $hour:tt : $minute:tt) => {
        $crate::Task::At(
            time::Time::from_hms($hour, $minute, 0).unwrap(),
            None
        )
    };

    // 带单个skip条件
    (wait $seconds:tt, weekday $day:tt) => {
        $crate::Task::Wait($seconds, Some(vec![$crate::Skip::Day(vec![$day])]))
    };
    (wait $seconds:tt, date $year:tt - $month:tt - $day:tt) => {
        $crate::Task::Wait($seconds, Some(vec![$crate::Skip::Date(
            time::Date::from_calendar_date($year, time::Month::try_from($month).unwrap(), $day).unwrap()
        )]))
    };
    (wait $seconds:tt, time $start_h:tt : $start_m:tt .. $end_h:tt : $end_m:tt) => {
        $crate::Task::Wait($seconds, Some(vec![$crate::Skip::TimeRange(
            time::Time::from_hms($start_h, $start_m, 0).unwrap(),
            time::Time::from_hms($end_h, $end_m, 0).unwrap()
        )]))
    };

    (interval $seconds:tt, weekday $day:tt) => {
        $crate::Task::Interval($seconds, Some(vec![$crate::Skip::Day(vec![$day])]))
    };
    (interval $seconds:tt, date $year:tt - $month:tt - $day:tt) => {
        $crate::Task::Interval($seconds, Some(vec![$crate::Skip::Date(
            time::Date::from_calendar_date($year, time::Month::try_from($month).unwrap(), $day).unwrap()
        )]))
    };
    (interval $seconds:tt, time $start_h:tt : $start_m:tt .. $end_h:tt : $end_m:tt) => {
        $crate::Task::Interval($seconds, Some(vec![$crate::Skip::TimeRange(
            time::Time::from_hms($start_h, $start_m, 0).unwrap(),
            time::Time::from_hms($end_h, $end_m, 0).unwrap()
        )]))
    };

    (at $hour:tt : $minute:tt, weekday $day:tt) => {
        $crate::Task::At(
            time::Time::from_hms($hour, $minute, 0).unwrap(),
            Some(vec![$crate::Skip::Day(vec![$day])])
        )
    };
    (at $hour:tt : $minute:tt, date $year:tt - $month:tt - $day:tt) => {
        $crate::Task::At(
            time::Time::from_hms($hour, $minute, 0).unwrap(),
            Some(vec![$crate::Skip::Date(
                time::Date::from_calendar_date($year, time::Month::try_from($month).unwrap(), $day).unwrap()
            )])
        )
    };
    (at $hour:tt : $minute:tt, time $start_h:tt : $start_m:tt .. $end_h:tt : $end_m:tt) => {
        $crate::Task::At(
            time::Time::from_hms($hour, $minute, 0).unwrap(),
            Some(vec![$crate::Skip::TimeRange(
                time::Time::from_hms($start_h, $start_m, 0).unwrap(),
                time::Time::from_hms($end_h, $end_m, 0).unwrap()
            )])
        )
    };

    // 带多个skip条件列表
    (wait $seconds:tt, [$($skip:tt)*]) => {
        $crate::Task::Wait($seconds, Some($crate::task!(@build_skips $($skip)*)))
    };
    (interval $seconds:tt, [$($skip:tt)*]) => {
        $crate::Task::Interval($seconds, Some($crate::task!(@build_skips $($skip)*)))
    };
    (at $hour:tt : $minute:tt, [$($skip:tt)*]) => {
        $crate::Task::At(
            time::Time::from_hms($hour, $minute, 0).unwrap(),
            Some($crate::task!(@build_skips $($skip)*))
        )
    };

    // 辅助宏：构建skip列表
    (@build_skips) => { vec![] };
    (@build_skips weekday $day:tt $(, $($rest:tt)*)?) => {
        {
            let mut skips = vec![$crate::Skip::Day(vec![$day])];
            $(skips.extend($crate::task!(@build_skips $($rest)*));)?
            skips
        }
    };
    (@build_skips date $year:tt - $month:tt - $day:tt $(, $($rest:tt)*)?) => {
        {
            let mut skips = vec![$crate::Skip::Date(
                time::Date::from_calendar_date($year, time::Month::try_from($month).unwrap(), $day).unwrap()
            )];
            $(skips.extend($crate::task!(@build_skips $($rest)*));)?
            skips
        }
    };
    (@build_skips time $start_h:tt : $start_m:tt .. $end_h:tt : $end_m:tt $(, $($rest:tt)*)?) => {
        {
            let mut skips = vec![$crate::Skip::TimeRange(
                time::Time::from_hms($start_h, $start_m, 0).unwrap(),
                time::Time::from_hms($end_h, $end_m, 0).unwrap()
            )];
            $(skips.extend($crate::task!(@build_skips $($rest)*));)?
            skips
        }
    };
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
                write!(f, "wait: {wait} {skip}")
            }
            Task::Interval(interval, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "interval: {interval} {skip}")
            }
            Task::At(time, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "at: {time} {skip}")
            }
            Task::Once(time, skip) => {
                let skip = skip
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "once: {time} {skip}")
            }
        }
    }
}
