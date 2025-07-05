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

        // Parse arguments - check if there are skip conditions
        let (primary_arg, skip_conditions) = Self::parse_arguments(args)?;

        match function_name {
            "wait" => {
                let seconds = primary_arg
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid seconds value '{primary_arg}' in wait({primary_arg})"))?;
                Ok(Task::Wait(seconds, skip_conditions))
            }
            "interval" => {
                let seconds = primary_arg
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid seconds value '{primary_arg}' in interval({primary_arg})"))?;
                Ok(Task::Interval(seconds, skip_conditions))
            }
            "at" => {
                let format = format_description!("[hour]:[minute]");
                let time = Time::parse(&primary_arg, &format).map_err(|_| {
                    format!("Invalid time format '{primary_arg}' in at({primary_arg}). Expected format: HH:MM")
                })?;
                Ok(Task::At(time, skip_conditions))
            }
            "once" => {
                let format = format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]"
                );
                let datetime = OffsetDateTime::parse(&primary_arg, &format)
                    .map_err(|_| format!("Invalid datetime format '{primary_arg}' in once({primary_arg}). Expected format: YYYY-MM-DD HH:MM:SS +HH"))?;
                Ok(Task::Once(datetime, skip_conditions))
            }
            _ => Err(format!(
                "Unknown task type '{function_name}'. Supported types: wait, interval, at, once"
            )),
        }
    }

    fn parse_arguments(args: &str) -> Result<(String, Option<Vec<Skip>>), String> {
        let args = args.trim();
        
        // Check if there's a comma, indicating skip conditions
        if let Some(comma_pos) = args.find(',') {
            let primary_arg = args[..comma_pos].trim().to_string();
            let skip_part = args[comma_pos + 1..].trim();
            
            let skip_conditions = Self::parse_skip_conditions(skip_part)?;
            Ok((primary_arg, Some(skip_conditions)))
        } else {
            Ok((args.to_string(), None))
        }
    }

    fn parse_skip_conditions(skip_str: &str) -> Result<Vec<Skip>, String> {
        let skip_str = skip_str.trim();
        
        // Check if it's a list format [...]
        if skip_str.starts_with('[') && skip_str.ends_with(']') {
            let list_content = &skip_str[1..skip_str.len()-1];
            Self::parse_skip_list(list_content)
        } else {
            // Single skip condition
            let skip = Self::parse_single_skip(skip_str)?;
            Ok(vec![skip])
        }
    }

    fn parse_skip_list(list_str: &str) -> Result<Vec<Skip>, String> {
        let mut skips = Vec::new();
        let list_str = list_str.trim();
        
        if list_str.is_empty() {
            return Ok(skips);
        }
        
        // Split by comma and parse each skip condition
        for part in list_str.split(',') {
            let part = part.trim();
            if !part.is_empty() {
                let skip = Self::parse_single_skip(part)?;
                skips.push(skip);
            }
        }
        
        Ok(skips)
    }

    fn parse_single_skip(skip_str: &str) -> Result<Skip, String> {
        let skip_str = skip_str.trim();
        let parts: Vec<&str> = skip_str.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err("Empty skip condition".to_string());
        }
        
        match parts[0] {
            "weekday" => {
                if parts.len() != 2 {
                    return Err(format!("Invalid weekday format: '{}'. Expected 'weekday N'", skip_str));
                }
                let day = parts[1].parse::<u8>()
                    .map_err(|_| format!("Invalid weekday number: '{}'", parts[1]))?;
                if day < 1 || day > 7 {
                    return Err(format!("Weekday must be between 1-7, got: {}", day));
                }
                Ok(Skip::Day(vec![day]))
            }
            "date" => {
                if parts.len() != 2 {
                    return Err(format!("Invalid date format: '{}'. Expected 'date YYYY-MM-DD'", skip_str));
                }
                let date_str = parts[1];
                let date_parts: Vec<&str> = date_str.split('-').collect();
                if date_parts.len() != 3 {
                    return Err(format!("Invalid date format: '{}'. Expected 'YYYY-MM-DD'", date_str));
                }
                
                let year = date_parts[0].parse::<i32>()
                    .map_err(|_| format!("Invalid year: '{}'", date_parts[0]))?;
                let month = date_parts[1].parse::<u8>()
                    .map_err(|_| format!("Invalid month: '{}'", date_parts[1]))?;
                let day = date_parts[2].parse::<u8>()
                    .map_err(|_| format!("Invalid day: '{}'", date_parts[2]))?;
                
                let month_enum = time::Month::try_from(month)
                    .map_err(|_| format!("Invalid month: {}", month))?;
                let date = time::Date::from_calendar_date(year, month_enum, day)
                    .map_err(|_| format!("Invalid date: {}-{}-{}", year, month, day))?;
                
                Ok(Skip::Date(date))
            }
            "time" => {
                if parts.len() != 2 {
                    return Err(format!("Invalid time format: '{}'. Expected 'time HH:MM..HH:MM'", skip_str));
                }
                let time_range = parts[1];
                if let Some(range_pos) = time_range.find("..") {
                    let start_str = &time_range[..range_pos];
                    let end_str = &time_range[range_pos + 2..];
                    
                    let format = format_description!("[hour]:[minute]");
                    let start_time = Time::parse(start_str, &format)
                        .map_err(|_| format!("Invalid start time: '{}'", start_str))?;
                    let end_time = Time::parse(end_str, &format)
                        .map_err(|_| format!("Invalid end time: '{}'", end_str))?;
                    
                    Ok(Skip::TimeRange(start_time, end_time))
                } else {
                    // Single time
                    let format = format_description!("[hour]:[minute]");
                    let time = Time::parse(time_range, &format)
                        .map_err(|_| format!("Invalid time: '{}'", time_range))?;
                    Ok(Skip::Time(time))
                }
            }
            _ => Err(format!("Unknown skip type: '{}'. Supported types: weekday, date, time", parts[0])),
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
