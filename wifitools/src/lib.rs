use rusty_network_manager::{
    AccessPointProxy, NetworkManagerProxy, SettingsConnectionProxy, SettingsProxy, WirelessProxy,
};
// use zbus::zvariant::{OwnedValue, Value as ZValue};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use zbus::zvariant::Value;
use zbus::{Connection, Proxy};
use zvariant::{ObjectPath, OwnedValue, Str};
// use std::collections::HashMap;

#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum WifiSecurity {
    WifiSecOpen = 0,
    WifiSecWep,
    WifiSecWpa,
    WifiSecWpa23,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WifiInfo {
    pub freq: u32,
    pub bssid: String,
    pub signal: u8,
    pub security: WifiSecurity,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WifiStoredInfo {
    pub created: String,
    pub security: WifiSecurity,
    pub psk: String,
}

pub async fn get_wificmd_pack() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Create a proxy for interacting with the D-Bus service
    let proxy = Proxy::new(
        &connection,
        "org.ion.IComGateway",  // D-Bus destination (service name)
        "/org/ion/IComGateway", // Object path
        "org.ion.IComGateway",  // Introspection interface
    )
    .await?;

    // Call the `Introspect` method to retrieve introspection XML
    let received_pack: Vec<u8> = proxy.call("GetLatestReceived", &(0u8)).await?;
    Ok(received_pack)
}

pub async fn send_wificmd_pack(send_pack: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Create a proxy for interacting with the D-Bus service
    let proxy = Proxy::new(
        &connection,
        "org.ion.IComGateway",  // D-Bus destination (service name)
        "/org/ion/IComGateway", // Object path
        "org.ion.IComGateway",  // Introspection interface
    )
    .await?;

    proxy.call("SendPackg", &(send_pack, 0u8)).await?;
    // println!("Received: {:?}", send_pack);

    Ok(())
}

pub async fn scan_wifi(
    interface: &str,
) -> Result<HashMap<String, WifiInfo>, Box<dyn std::error::Error>> {
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

                let security_type: WifiSecurity = if rsn_flags != 0 {
                    // "WPA2/WPA3"
                    WifiSecurity::WifiSecWpa23
                } else if wpa_flags != 0 {
                    // "WPA"
                    WifiSecurity::WifiSecWpa
                } else if flags & 0x01 != 0 {
                    // "WEP"
                    WifiSecurity::WifiSecWep
                } else {
                    // "Open"
                    WifiSecurity::WifiSecOpen
                };
                scan_results.insert(
                    String::from_utf8(ssid).unwrap(),
                    WifiInfo {
                        freq: frequency,
                        bssid: hw_address,
                        signal: signal_strength,
                        security: security_type,
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

pub async fn get_stored_wifi() -> Result<HashMap<String, WifiStoredInfo>, Box<dyn std::error::Error>>
{
    let mut stored_results: HashMap<String, WifiStoredInfo> = HashMap::new();

    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Initialize the SettingsProxy instance
    let settings = SettingsProxy::new(&connection).await?;

    // Retrieve the list of stored connections
    let connections = settings.list_connections().await?;

    // Iterate over each connection path to get more details
    for conn_path in connections {
        let setting_connection_proxy =
            SettingsConnectionProxy::new_from_path(conn_path, &connection).await?;

        // Retrieve connection settings
        let setcfgs: HashMap<String, HashMap<String, OwnedValue>> =
            setting_connection_proxy.get_settings().await?;
        let _pathcfg = setting_connection_proxy.filename().await?;
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
            let security: Option<String>;

            // Access the security settings
            let security_settings = setcfgs.get("802-11-wireless-security");

            if let Some(security_settings) = security_settings {
                let key_mgmt = security_settings.get("key-mgmt");
                security = key_mgmt.and_then(extract_string);
            } else {
                security = None;
                println!("No wireless-security settings found.");
            }

            // Extract connection settings
            let ssid: Option<String> = setcfgs
                .get("connection")
                .and_then(|c| c.get("id"))
                .and_then(extract_string);

            let timestamp: Option<u64> = setcfgs
                .get("connection")
                .and_then(|c| c.get("timestamp"))
                .and_then(extract_u64);

            // Handle default values
            let default_ssid = "No ID found".to_string();
            let ap_name = ssid.as_deref().unwrap_or(&default_ssid);
            let ap_created = timestamp.map_or("No timestamp found".to_string(), |t| t.to_string());
            let default_sec = "None".to_string();
            let _ap_sec = security.as_deref().unwrap_or(&default_sec);

            // Print extracted details
            // println!("SSID: {}", ap_name);
            // println!("Timestamp: {}", ap_created);
            // println!("Security: {}", ap_sec);
            // println!("Config Path: {}", pathcfg);

            stored_results.insert(
                ap_name.to_string(),
                WifiStoredInfo {
                    created: ap_created,
                    security: WifiSecurity::WifiSecOpen,
                    psk: "".to_string(), // Placeholder for PSK
                },
            );
        }
    }
    Ok(stored_results)
}

fn convert_hashmap(
    original: HashMap<String, HashMap<String, OwnedValue>>,
) -> HashMap<String, HashMap<String, Value<'static>>> {
    let mut converted: HashMap<String, HashMap<String, Value<'_>>> = HashMap::new();

    for (outer_key, inner_map) in original {
        let mut new_inner_map: HashMap<String, Value<'_>> = HashMap::new();
        for (inner_key, value) in inner_map {
            // Insert the owned value directly
            new_inner_map.insert(inner_key, value.into());
        }
        converted.insert(outer_key, new_inner_map);
    }

    converted
}
pub async fn connect_wifi(
    interface: &str,
    ssid: &str,
    password: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;
    let settings_proxy = SettingsProxy::new(&connection).await?;

    let connections = settings_proxy.list_connections().await?;

    // Try to find an existing connection with the same SSID or interface name
    let mut existing_connection_path: Option<zvariant::OwnedObjectPath> = None;
    for connection_path in connections {
        let setting_connection_proxy =
        SettingsConnectionProxy::new_from_path(connection_path.clone(), &connection).await?;

        let settings = setting_connection_proxy.get_settings().await?;
        if let Some(connection_props) = settings.get("connection") {
            let id = connection_props.get("id").and_then(|v| Some(v.downcast_ref::<Str>()));
            let interface_name = connection_props.get("interface-name").and_then(|v| Some(v.downcast_ref::<Str>()));
            if id.expect("REASON").ok() == Some(Str::from(ssid)) || interface_name.expect("REASON").ok() == Some(Str::from(interface)) {
                println!("{} Exitsed!", ssid);
                existing_connection_path = Some(connection_path.clone());
                break;
            }
        }
    }
    if let Some(connection_path) = existing_connection_path {
        let settings_connection = SettingsConnectionProxy::new_from_path(connection_path, &connection).await?;
        let mut settings: HashMap<String, HashMap<String, OwnedValue>> = settings_connection.get_settings().await?;

        // Update Wi-Fi security settings if password is provided
        if let Some(pass) = password {
            if let Some(wireless_security) = settings.get_mut("802-11-wireless-security") {
                wireless_security.insert("key-mgmt".to_owned(), OwnedValue::from(Str::from("wpa-psk")));
                wireless_security.insert("psk".to_owned(), OwnedValue::from(Str::from(pass)));
            } else {
                // Add wireless security settings if missing
                let mut security_props: HashMap<String, OwnedValue> = HashMap::new();
                security_props.insert("key-mgmt".to_owned(), OwnedValue::from(Str::from("wpa-psk")));
                security_props.insert("psk".to_owned(), OwnedValue::from(Str::from(pass)));
                settings.insert("802-11-wireless-security".to_owned(), security_props);
            }
        }

        // Update DNS settings
        if let Some(ipv4_props) = settings.get_mut("ipv4") {
            ipv4_props.insert("method".to_owned(), OwnedValue::from(Str::from("auto")));
        } else {
            let mut ipv4_props: HashMap<String, OwnedValue> = HashMap::new();
            ipv4_props.insert("method".to_owned(), OwnedValue::from(Str::from("auto")));
            settings.insert("ipv4".to_owned(), ipv4_props);
        }

        // Save the updated settings
        settings_connection.update(convert_hashmap(settings)).await?;

        // Activate the updated connection
        let device_path = nm.get_device_by_ip_iface(interface).await?;
        let base_path = ObjectPath::try_from("/")?;
        nm.activate_connection(&connection_path, &device_path, &base_path)
            .await?;

        println!("Updated and connected to Wi-Fi network '{}'", ssid);
    } else {
        // Create connection properties
        let mut connection_properties: HashMap<&str, HashMap<&str, zbus::zvariant::Value<'_>>> =
            HashMap::new();

        // Create properties for the "connection" section
        let mut conn_props: HashMap<&str, Value> = HashMap::new();
        conn_props.insert("id", Value::from(ssid));
        conn_props.insert("type", Value::from("802-11-wireless"));
        conn_props.insert("interface-name", Value::from(interface));
        connection_properties.insert("connection", conn_props);

        // Create properties for the "802-11-wireless" section
        let mut wireless_props: HashMap<&str, Value> = HashMap::new();
        wireless_props.insert("ssid", Value::from(ssid.as_bytes().to_vec()));
        wireless_props.insert("mode", Value::from("infrastructure"));
        connection_properties.insert("802-11-wireless", wireless_props);

        // Create properties for "802-11-wireless-security" if a password is provided
        if let Some(pass) = password {
            let mut security_props: HashMap<&str, Value> = HashMap::new();
            security_props.insert("key-mgmt", Value::new(Str::from("wpa-psk")));
            security_props.insert("psk", Value::new(Str::from(pass)));
            connection_properties.insert("802-11-wireless-security", security_props);
        }

        // Create properties for the "ipv4" section to configure DNS
        let mut ipv4_props: HashMap<&str, Value> = HashMap::new();
        ipv4_props.insert("method", Value::from("auto"));
        connection_properties.insert("ipv4", ipv4_props);

        // Add the new connection
        let settings_proxy = SettingsProxy::new(&connection).await?;
        let connection_path: zvariant::OwnedObjectPath =
            settings_proxy.add_connection(connection_properties).await?;
        
        // Activate the new connection
        let device_path = nm.get_device_by_ip_iface(interface).await?;
        let base_path = ObjectPath::try_from("/")?;
        nm.activate_connection(&connection_path, &device_path, &base_path)
            .await?;

    }

    println!("Connected to Wi-Fi network '{}'", ssid);

    Ok(())
}