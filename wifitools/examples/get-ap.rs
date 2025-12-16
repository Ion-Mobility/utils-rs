use wifitools::get_ap_info;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <interface> <ssid>", args[0]);
        eprintln!(r#"Example: {} wlan0 "My Home Wifi""#, args[0]);
        return;
    }

    let interface = &args[1];

    // Join remaining args into SSID (allows spaces)
    let ssid = args[2..].join(" ");

    match get_ap_info(interface).await {
        Ok(ap_info) => {
            if let Ok((found_ssid, info)) = ap_info.try_into() {
                println!("SSID: {}", found_ssid);
                println!("Info: {:?}", info);

                if found_ssid == ssid {
                    println!("Connected to desired SSID ✅");
                } else {
                    println!("Connected, but SSID does not match ❌");
                }
            }
        }
        Err(e) => {
            eprintln!("Can't get AP info: {}", e);
        }
    }
}
