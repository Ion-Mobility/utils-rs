use wifitools::turn_on_wifi;

#[tokio::main]
async fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // You can access specific arguments, e.g., the first one after the program name
    if args.len() < 2 {
        eprintln!("Lack of wifi interface name");
    } else {
        match turn_on_wifi(&args[1]).await {
            Ok(()) => {
                println!("Wifi turned on");
            }
            Err(e) => {
                eprintln!("Can't turn on wifi {}", e);
            }
        }
    }
}