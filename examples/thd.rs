use async_trait::async_trait;
use easy_schedule::prelude::*;

#[derive(Debug, Clone)]
struct WaitTask;

#[async_trait]
impl Notifiable for WaitTask {
    fn get_schedule(&self) -> Task {
        Task::Wait(3, None)
    }

    async fn on_time(&self, cancel: CancellationToken) {
        println!("on_time {}", time::OffsetDateTime::now_local().unwrap());
        cancel.cancel();
    }

    async fn on_skip(&self, _cancel: CancellationToken) {
        println!("WaitTask on_skip");
    }
}

#[tokio::main]
async fn main() {
    println!("start {}", time::OffsetDateTime::now_local().unwrap());
    let scheduler = Scheduler::new();
    scheduler.run(WaitTask).await;

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        scheduler.stop();
        println!("cancel {}", time::OffsetDateTime::now_local().unwrap());
    })
    .await
    .unwrap();
}
