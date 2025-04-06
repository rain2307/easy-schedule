use time::OffsetDateTime;

fn main() {
    let local_time = OffsetDateTime::now_local().unwrap();

    println!("年: {}", local_time.year());
    println!("月: {}", local_time.month());
    println!("日: {}", local_time.day());
    println!("时: {}", local_time.hour());
    println!("分: {}", local_time.minute());
    println!("秒: {}", local_time.second());
}
