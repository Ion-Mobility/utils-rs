use gwclient::{get_gps_info};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        let gps_info = get_gps_info().await;
        println!("gpsInfo: {:?}", gps_info);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
