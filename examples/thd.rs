use std::sync::Arc;

#[allow(unused_imports)]
use easy_schedule::{CancellationToken, ScheduledTask, Scheduler, Task};
use tokio::spawn;

#[derive(Debug, Clone)]
struct WaitTask;

impl ScheduledTask for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(3, None)
    }

    fn on_time(&self, cancel: CancellationToken) {
        println!("WaitTask");
        cancel.cancel();
    }

    fn on_skip(&self, _cancel: CancellationToken) {
        println!("WaitTask on_skip");
    }
}

#[tokio::main]
async fn main() {
    spawn(async move {
        let scheduler = Scheduler::new();
        scheduler.add_task(Arc::new(Box::new(WaitTask))).await;
        scheduler.start().await;
    })
    .await
    .unwrap();
}
