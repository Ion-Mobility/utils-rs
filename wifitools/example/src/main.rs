use wifitools::connect_wifi;

#[tokio::main]
async fn main() {
    println!("hello");
    connect_wifi("wlan0", "VIETTEL_AP_8CE000", Some("1234567890a")).await.unwrap();
}