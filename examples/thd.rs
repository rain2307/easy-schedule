#[allow(unused_imports)]
use easy_schedule::{CancellationToken, ScheduledTask, Scheduler, Task};

#[derive(Debug, Clone)]
struct WaitTask;

impl ScheduledTask for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(3, None)
    }

    fn on_time(&self, cancel: CancellationToken) {
        println!("on_time {}", time::OffsetDateTime::now_local().unwrap());
        cancel.cancel();
    }

    fn on_skip(&self, _cancel: CancellationToken) {
        println!("WaitTask on_skip");
    }
}

#[tokio::main]
async fn main() {
    println!("start {}", time::OffsetDateTime::now_local().unwrap());
    let cancel = Scheduler::start(WaitTask).await;

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        cancel.cancel();
        println!("cancel {}", time::OffsetDateTime::now_local().unwrap());
    })
    .await
    .unwrap();

    tokio::signal::ctrl_c().await.unwrap();
}
