use easy_schedule::prelude::*;
use time::{OffsetDateTime, macros::offset};

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
struct StringTask {
    task_string: String,
    name: String,
}

impl StringTask {
    fn new(task_string: &str, name: &str) -> Self {
        Self {
            task_string: task_string.to_string(),
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl Notifiable for StringTask {
    fn get_schedule(&self) -> Task {
        Task::from(self.task_string.as_str())
    }

    async fn on_time(&self, cancel: CancellationToken) {
        print_time(&self.name);
        cancel.cancel();
    }
}

#[tokio::main]
async fn main() {
    println!("Easy Schedule String Parsing Example");
    println!("====================================");

    let scheduler = Scheduler::new();

    let tasks = vec![
        StringTask::new("wait(3)", "Wait 3 seconds"),
        StringTask::new("interval(2)", "Interval 2 seconds"),
        StringTask::new("at(23:59)", "At 23:59"),
        StringTask::new(
            "once(2024-12-31 23:59:59 +08)",
            "Once at 2024-12-31 23:59:59",
        ),
    ];

    println!("Task parsing demonstration:");
    for task in &tasks {
        let parsed = Task::from(task.task_string.as_str());
        println!("'{}' -> {}", task.task_string, parsed);
    }

    println!("\nStarting tasks from strings...");

    for task in tasks {
        scheduler.run(task).await;
    }

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    println!("Stopping scheduler...");
    scheduler.stop();
}
