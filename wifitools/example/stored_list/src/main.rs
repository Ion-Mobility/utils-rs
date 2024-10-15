use wifitools::get_stored_wifi;

#[tokio::main]
async fn main() {
    match get_stored_wifi().await {
        Ok(_result) => {
            let results = _result.lock().await;
            for (_ssid, _infor) in results.iter() {
                println!("{} {:?}", _ssid, _infor);
            }
        }
        Err(e) => {
            eprintln!("Can't get stored wifi {}", e);
        }
    }
}