use wifitools::{connect_wifi,remove_stored_wifi};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    // connect_wifi("wlan0", "VIETTEL_AP_8CE000", Some("1234567890a"), Duration::from_secs(10)).await.unwrap();
    connect_wifi("wlan0", "Ion Mobility VN", Some("Imv@104"), Duration::from_secs(10)).await.unwrap();
    // remove_stored_wifi("VIETTEL_AP_8CE000".to_string()).await.unwrap();
}