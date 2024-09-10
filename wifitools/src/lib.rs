use rusty_network_manager::{SettingsProxy, AccessPointProxy, NetworkManagerProxy, WirelessProxy, SettingsConnectionProxy};
use zbus::zvariant::{OwnedValue, Value as ZValue};
use std::{collections::HashMap, str::FromStr};
use tokio::time::{sleep, Duration};
use zbus::{Connection, Proxy};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WifiInfo {
    pub freq: u32,
    pub bssid: String,
    pub signal: u8,
    pub security: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WifiStoredInfo {
    pub created: String,
    pub security: String,
    pub psk: String,
}

pub async fn get_wificmd_pack() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Create a proxy for interacting with the D-Bus service
    let proxy = Proxy::new(
        &connection,
        "org.ion.IComGateway",           // D-Bus destination (service name)
        "/org/ion/IComGateway",          // Object path
        "org.ion.IComGateway" // Introspection interface
    ).await?;

    // Call the `Introspect` method to retrieve introspection XML
    let received_pack: Vec<u8> = proxy.call("GetLatestReceived", &()).await?;
    Ok(received_pack)
}

pub async fn send_wificmd_pack(send_pack: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Create a proxy for interacting with the D-Bus service
    let proxy = Proxy::new(
        &connection,
        "org.ion.IComGateway",           // D-Bus destination (service name)
        "/org/ion/IComGateway",          // Object path
        "org.ion.IComGateway" // Introspection interface
    ).await?;

    let _ = proxy.call("SendPackg", &(send_pack)).await?;
    println!("Received: {:?}", send_pack);
    
    Ok(())
}

pub async fn scan_wifi(interface: &str) -> Result<HashMap<String, WifiInfo>, Box<dyn std::error::Error>> {
    let mut scan_results: HashMap<String, WifiInfo> = HashMap::new();
    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;

    let wireless_path = nm.get_device_by_ip_iface(interface).await?;
    let wireless_proxy = WirelessProxy::new_from_path(wireless_path, &connection).await?;

    // Create an empty HashMap for scan options
    let scan_options: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();

    // Request a Wi-Fi scan
    wireless_proxy.request_scan(scan_options).await?;

    // Poll for scan results
    loop {
        let access_points = wireless_proxy.get_access_points().await?;
        if !access_points.is_empty() {
            for ap_path in access_points {
                let access_point = AccessPointProxy::new_from_path(ap_path, &connection).await?;
                let ssid = access_point.ssid().await.unwrap();
                let frequency = access_point.frequency().await.unwrap();
                let hw_address = access_point.hw_address().await.unwrap();
                let signal_strength = access_point.strength().await.unwrap(); // Signal strength in dBm

                let flags = access_point.flags().await.unwrap();
                let wpa_flags = access_point.wpa_flags().await.unwrap();
                let rsn_flags = access_point.rsn_flags().await.unwrap();

                let security_type = if rsn_flags != 0 {
                    "WPA2/WPA3"
                } else if wpa_flags != 0 {
                    "WPA"
                } else if flags & 0x01 != 0 {
                    "WEP"
                } else {
                    "Open"
                };
                scan_results.insert(
                    String::from_utf8(ssid).unwrap(),
                    WifiInfo {
                        freq: frequency,
                        bssid: hw_address,
                        signal: signal_strength,
                        security: security_type.to_string(),
                    },
                );
            }
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    Ok(scan_results)
}

// Function to extract a string from OwnedValue
fn extract_string(value: &OwnedValue) -> Option<String> {
    if let Ok(s) = <&str>::try_from(value) {
        Some(s.to_string()) // Convert &str to String
    } else {
        None // Handle cases where the conversion isn't possible
    }
}

// Function to extract a u64 from OwnedValue
fn extract_u64(value: &OwnedValue) -> Option<u64> {
    if let Ok(v) = <u64>::try_from(value) {
        Some(v) // Return the u64 value if conversion is successful
    } else {
        None // Handle cases where the conversion isn't possible
    }
}

pub async fn get_stored_wifi() -> Result<HashMap<String, WifiStoredInfo>, Box<dyn std::error::Error>> {
    let mut stored_results: HashMap<String, WifiStoredInfo> = HashMap::new();

    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Initialize the SettingsProxy instance
    let settings = SettingsProxy::new(&connection).await?;

    // Retrieve the list of stored connections
    let connections = settings.list_connections().await?;

    // Iterate over each connection path to get more details
    for conn_path in connections {
        let setting_connection_proxy = SettingsConnectionProxy::new_from_path(conn_path, &connection).await?;

        // Retrieve connection settings
        let setcfgs: HashMap<String, HashMap<String, OwnedValue>> = setting_connection_proxy.get_settings().await?;
        let pathcfg = setting_connection_proxy.filename().await?;
        let mut wireless_cfg_found = false;
        for (keystr, value) in &setcfgs {
            if keystr == "connection" {
                if let Some(type_value) = value.get("type") {
                    if let Some(type_str) = extract_string(type_value) {
                        if type_str == "802-11-wireless" {
                            wireless_cfg_found = true;
                            break;
                        }
                    }
                }
            }
        }

        if wireless_cfg_found {
            let ssid: Option<String>;
            let timestamp: Option<u64>;
            let security: Option<String>;

            // Access the security settings
            let security_settings = setcfgs.get("802-11-wireless-security");

            if let Some(security_settings) = security_settings {
                let key_mgmt = security_settings.get("key-mgmt");
                security = key_mgmt.and_then(|s| extract_string(s));
            } else {
                security = None;
                println!("No wireless-security settings found.");
            }

            // Extract connection settings
            ssid = setcfgs.get("connection")
                .and_then(|c| c.get("id"))
                .and_then(|id| extract_string(id));

            timestamp = setcfgs.get("connection")
                .and_then(|c| c.get("timestamp"))
                .and_then(|ts| extract_u64(ts));

            // Handle default values
            let default_ssid = "No ID found".to_string();
            let ap_name = ssid.as_deref().unwrap_or(&default_ssid);
            let ap_created = timestamp.map_or("No timestamp found".to_string(), |t| t.to_string());
            let default_sec = "None".to_string();
            let ap_sec = security.as_deref().unwrap_or(&default_sec);

            // Print extracted details
            // println!("SSID: {}", ap_name);
            // println!("Timestamp: {}", ap_created);
            // println!("Security: {}", ap_sec);
            // println!("Config Path: {}", pathcfg);

            stored_results.insert(
                ap_name.to_string(),
                WifiStoredInfo {
                    created: ap_created,
                    security: ap_sec.to_string(),
                    psk: "".to_string(), // Placeholder for PSK
                },
            );
        }
    }
    Ok(stored_results)
}

