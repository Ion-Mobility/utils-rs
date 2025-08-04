use gwclient::{get_isys_info};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Ok(isysinfo) = get_isys_info().await {
        println!("Result: {:?}", isysinfo);
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}
