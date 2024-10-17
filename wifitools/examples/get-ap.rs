use wifitools::get_ap_info;

#[tokio::main]
async fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // You can access specific arguments, e.g., the first one after the program name
    if args.len() < 2 {
        eprintln!("Lack of wifi interface name");
    } else {
        match get_ap_info(&args[1]).await {
            Ok(_ap_info) => {
                if let Ok((_ssid, _info)) = _ap_info.try_into() {
                    println!("SSID {}, Infor {:?}", _ssid, _info);                    
                }
                println!("Wifi turned on");
            }
            Err(e) => {
                eprintln!("Can't turn on wifi {}", e);
            }
        }
    }
}