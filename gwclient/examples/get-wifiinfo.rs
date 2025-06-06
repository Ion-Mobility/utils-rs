use gwclient::{get_wifi_info};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        let wifi_info = get_wifi_info().await;
        println!("wifiInfo: {:?}", wifi_info);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
