use wifitools::scan_wifi;

#[tokio::main]
async fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // You can access specific arguments, e.g., the first one after the program name
    if args.len() < 2 {
        eprintln!("Lack of wifi interface name");
    } else {
        match scan_wifi(&args[1]).await {
            Ok(_results) => {
                for (_ssid, _info) in _results {
                    println!("SSID: {}, INFO: {:?}", _ssid, _info);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}