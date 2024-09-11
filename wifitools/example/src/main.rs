use wifitools::connect_wifi;

#[tokio::main]
async fn main() {
    println!("hello");
    connect_wifi("wlo1", "VIETTEL_AP_8CE000", Some("123456789a")).await.unwrap();
}