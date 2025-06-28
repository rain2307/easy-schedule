use easy_schedule::{prelude::*, task};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use time::{OffsetDateTime, macros::offset};

fn print_time(name: &str) {
    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
    let format = time::macros::format_description!("[hour]:[minute]:[second]");
    println!("[{}] {}: executed", now.format(&format).unwrap(), name);
}

#[derive(Debug)]
struct MacroTask {
    name: String,
    task: Task,
    counter: Arc<AtomicU32>,
}

impl MacroTask {
    fn new(name: &str, task: Task) -> Self {
        Self {
            name: name.to_string(),
            task,
            counter: Arc::new(AtomicU32::new(0)),
        }
    }
}

#[async_trait]
impl Notifiable for MacroTask {
    fn get_schedule(&self) -> Task {
        self.task.clone()
    }

    async fn on_time(&self, cancel: CancellationToken) {
        let count = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        print_time(&format!("{} #{count}", self.name));

        // Cancel after 3 executions
        if count >= 3 {
            cancel.cancel();
        }
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        print_time(&format!("{} SKIPPED", self.name));
    }
}

#[tokio::main]
async fn main() {
    println!("Easy Schedule Macro Example");
    println!("===========================");

    // Basic task macros (no skip conditions)
    println!("📝 Creating tasks with macros:");
    let wait_task = task!(wait 2);
    let interval_task = task!(interval 3);
    let at_task = task!(at 23:59);

    println!("  task!(wait 2)        -> {wait_task}");
    println!("  task!(interval 3)    -> {interval_task}");
    println!("  task!(at 23:59)      -> {at_task}");

    // Tasks with single skip condition
    println!("\n🚫 Tasks with single skip conditions:");
    let skip_weekday = task!(wait 2, weekday 6);
    let skip_date = task!(interval 3, date 2024-12-25);
    let skip_time = task!(at 9:30, time 12:00..13:00);

    println!("  task!(wait 2, weekday 6)           -> {skip_weekday}");
    println!("  task!(interval 3, date 2024-12-25) -> {skip_date}");
    println!("  task!(at 9:30, time 12:00..13:00)  -> {skip_time}");

    // Tasks with multiple skip conditions
    println!("\n🚫🚫 Tasks with multiple skip conditions:");
    let multi_skip1 = task!(wait 2, [weekday 6, weekday 7]);
    let multi_skip2 = task!(interval 3, [date 2024-12-25, time 12:00..13:00]);

    println!("  task!(wait 2, [weekday 6, weekday 7])");
    println!("    -> {multi_skip1}");
    println!("  task!(interval 3, [date 2024-12-25, time 12:00..13:00])");
    println!("    -> {multi_skip2}");

    // Compare with traditional syntax
    println!("\n🔄 Traditional vs Macro syntax:");
    let traditional = Task::Wait(5, Some(vec![Skip::Day(vec![1, 2])]));
    let macro_style = task!(wait 5, [weekday 1, weekday 2]);

    println!("  Traditional: Task::Wait(5, Some(vec![Skip::Day(vec![1, 2])]))");
    println!("  Macro:       task!(wait 5, [weekday 1, weekday 2])");
    println!("  Both create: {traditional}");
    println!("  Result equal: {}", traditional == macro_style);

    println!("\n🚀 Running macro-created tasks...");

    let scheduler = Scheduler::new();

    // Create tasks using macros
    let tasks = vec![
        MacroTask::new("BasicWait", task!(wait 2)),
        MacroTask::new("BasicInterval", task!(interval 4)),
        MacroTask::new("SkipWeekend", task!(wait 1, [weekday 6, weekday 7])),
    ];

    println!("Starting {} tasks created with macros...", tasks.len());

    for task in tasks {
        scheduler.run(task).await;
    }

    // Let tasks run for a while
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    println!("Stopping scheduler...");
    scheduler.stop();

    println!("\n✅ Macro example completed!");
    println!("💡 Key benefits of using macros:");
    println!("   - Shorter, more readable syntax");
    println!("   - Compile-time validation");
    println!("   - Better IDE support with syntax highlighting");
    println!("   - Type-safe construction");
}
