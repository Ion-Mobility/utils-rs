use gwclient::{get_isys_info, get_wifi_info, get_lte_info, get_gps_info};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        // let isysinfo = get_isys_info().await;
        // println!("Result: {:?}", isysinfo);
        let wifiInfo = get_wifi_info().await;
        println!("wifiInfo: {:?}", wifiInfo);
        let lteInfo = get_lte_info().await;
        println!("lteInfo: {:?}", lteInfo);
        let gpsInfo = get_gps_info().await;
        println!("gpsInfo: {:?}", gpsInfo);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
