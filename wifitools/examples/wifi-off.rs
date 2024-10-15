use wifitools::turn_off_wifi;

#[tokio::main]
async fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // You can access specific arguments, e.g., the first one after the program name
    if args.len() < 2 {
        eprintln!("Lack of wifi interface name");
    } else {
        match turn_off_wifi(&args[1]).await {
            Ok(()) => {
                println!("Wifi turned off");
            }
            Err(e) => {
                eprintln!("Can't turn off wifi {}", e);
            }
        }
    }
}