use wifitools::connect_wifi;

#[tokio::main]
async fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // You can access specific arguments, e.g., the first one after the program name
    if args.len() < 4 {
        eprintln!("Usage: <interface> <ssid> <password>");
    } else {
        match connect_wifi(&args[1], &args[2], Some(&args[3]), tokio::time::Duration::from_secs(10)).await {
            Ok(_) => {
                println!("Successfully connected to SSID: {}", args[2]);
            }
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
            }
        }
    }
}