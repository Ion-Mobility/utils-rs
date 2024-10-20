use rusty_network_manager::{
    AccessPointProxy, NetworkManagerProxy, SettingsConnectionProxy, SettingsProxy, WirelessProxy, DeviceProxy, IP4ConfigProxy
};
// use zbus::zvariant::{OwnedValue, Value as ZValue};
use std::collections::HashMap;
use tokio::time::{sleep, Duration, Instant};
use zbus::zvariant::Value;
use zbus::{Connection, Proxy};
use zvariant::{ObjectPath, OwnedValue, Str};
use std::net::Ipv4Addr;
use tokio::sync::Mutex;
use std::sync::Arc;
// use std::collections::HashMap;
const WIFI_MAC_LEN: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Unknown = 0,
    Unmanaged = 10,
    Unavailable = 20,
    Disconnected = 30,
    Prepare = 40,
    Config = 50,
    NeedAuth = 60,
    IPConfig = 70,
    IPCheck = 80,
    Secondaries = 90,
    Activated = 100,
    Deactivating = 110,
    Failed = 120,
}

impl DeviceState {
    /// Convert from a `u32` value to `DeviceState` enum.
    pub fn from_u32(value: u32) -> Option<DeviceState> {
        match value {
            0 => Some(DeviceState::Unknown),
            10 => Some(DeviceState::Unmanaged),
            20 => Some(DeviceState::Unavailable),
            30 => Some(DeviceState::Disconnected),
            40 => Some(DeviceState::Prepare),
            50 => Some(DeviceState::Config),
            60 => Some(DeviceState::NeedAuth),
            70 => Some(DeviceState::IPConfig),
            80 => Some(DeviceState::IPCheck),
            90 => Some(DeviceState::Secondaries),
            100 => Some(DeviceState::Activated),
            110 => Some(DeviceState::Deactivating),
            120 => Some(DeviceState::Failed),
            _ => None,  // Return `None` for unknown values.
        }
    }

    /// Convert the `DeviceState` back to a `u32` value.
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum WifiSecurity {
    WifiSecOpen = 0,
    WifiSecWep,
    WifiSecWpa,
    WifiSecWpa23,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct WifiInfo {
    pub mac: [u8; WIFI_MAC_LEN],
    pub freq: u32,
    pub rssi: u8,
    pub security: WifiSecurity,
    pub ip4_addr: [u8; 4],
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WifiStoredInfo {
    pub created: String,
    pub security: WifiSecurity,
    pub psk: String,
    pub seen_bssid: Vec<String>
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
    let wireless_proxy = WirelessProxy::new_from_path(wireless_path.clone(), &connection).await?;

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

                let mut ip4_address = [0, 0, 0, 0];
                let check_connection = check_connection_success(interface, &String::from_utf8(ssid.clone()).unwrap()).await?;
                if check_connection.0 {
                    let device_proxy = DeviceProxy::new_from_path(wireless_path.clone(), &connection).await?;
                    let ip4_str = get_ip4_str_address(&device_proxy, &connection).await;
                    ip4_address = ip_to_bytes(&ip4_str);
                }

                let ssid_str = String::from_utf8(ssid).unwrap();
                let wifi_info = WifiInfo {
                    mac: mac_str_to_array(&hw_address)?,
                    freq: frequency,
                    rssi: signal_strength,
                    security: security_type,
                    ip4_addr: ip4_address
                };
                scan_results.insert(
                    ssid_str,
                    wifi_info,
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

fn extract_string_array(value: &OwnedValue) -> Option<Vec<String>> {
    if let Ok(array) = value.downcast_ref::<zvariant::Array>() {
        Some(
            array
                .iter()
                .filter_map(|v| v.downcast_ref::<zvariant::Str>().map(|s| s.to_string()).ok())
                .collect(),
        )
    } else {
        None
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

pub async fn get_stored_wifi() -> Result<Arc<Mutex<HashMap<String, WifiStoredInfo>>>, Box<dyn std::error::Error + Send + Sync>>
{
    let stored_results: Arc<Mutex<HashMap<String, WifiStoredInfo>>> = Arc::new(Mutex::new(HashMap::new()));

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
            let security: WifiSecurity;

            // Access the security settings
            let security_settings = setcfgs.get("802-11-wireless-security");
            if let Some(security_settings) = security_settings {
                let key_mgmt = security_settings.get("key-mgmt");
                // let mut psk = String::new();
                // Extract the PSK (if available)
                // if let Some(psk_value) = security_settings.get("psk") {
                //     psk = extract_string(psk_value).unwrap_or_default();
                // }
                // println!("PSK: {}", psk);
                // Map key-mgmt to security type
                security = match key_mgmt.and_then(extract_string).as_deref() {
                    Some("wpa-psk") => WifiSecurity::WifiSecWpa,
                    Some("wpa2-psk") => WifiSecurity::WifiSecWpa23,
                    Some("none") => WifiSecurity::WifiSecOpen,
                    _ => WifiSecurity::WifiSecWep, // Use WEP for unknown types
                };
            } else {
                security = WifiSecurity::WifiSecOpen;
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

            // Extract the BSSIDs (optional)
            let bssids = setcfgs
            .get("802-11-wireless")
            .and_then(|w| w.get("seen-bssids"))
            .and_then(extract_string_array)
            .unwrap_or_default(); // Default to empty list if not found

            // Handle default values
            let default_ssid = "No ID found".to_string();
            let ap_name = ssid.as_deref().unwrap_or(&default_ssid);
            let ap_created = timestamp.map_or("No timestamp found".to_string(), |t| t.to_string());
            // Store results in the shared HashMap
            let mut results = stored_results.lock().await;
            results.insert(
                ap_name.to_string(),
                WifiStoredInfo {
                    created: ap_created,
                    security: security,
                    psk: "".to_string(), // Placeholder for PSK
                    seen_bssid: bssids,
                },
            );
        }
    }

    Ok(stored_results)
}

fn convert_hashmap<'a>(
    input: &'a HashMap<String, HashMap<String, OwnedValue>>,
) -> HashMap<&'a str, HashMap<&'a str, &Value<'a>>> {
    // Create the output HashMap
    let mut output: HashMap<&str, HashMap<&str, &Value>> = HashMap::new();

    // Iterate through the outer HashMap
    for (outer_key, inner_map) in input {
        // Create a new HashMap for the inner values
        let mut inner_output_map: HashMap<&str, &Value> = HashMap::new();

        // Iterate through the inner HashMap
        for (inner_key, inner_value) in inner_map {
            // Convert OwnedValue to zvariant::Value
            let value = inner_value.downcast_ref().unwrap(); // Assuming OwnedValue implements conversion to Value

            // Insert the borrowed keys and converted values into the new inner HashMap
            inner_output_map.insert(inner_key.as_str(), value);
        }

        // Insert the borrowed outer key and the new inner HashMap into the output HashMap
        output.insert(outer_key.as_str(), inner_output_map);
    }

    output
}
// Function to check if the connection was successful
async fn check_connection_success(
    interface: &str,
    ssid: &str,
) -> Result<(bool, WifiInfo), Box<dyn std::error::Error>> {
    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;
    let devices = nm.devices().await?;
    for device_path in devices {
        let device_proxy = DeviceProxy::new_from_path(device_path.clone(), &connection).await?;
        let device_interface = device_proxy.interface().await?;

        // Check if this is the desired interface
        if device_interface == interface {
            // println!("{:?}", DeviceState::from_u32(device_proxy.state().await?));
            match DeviceState::from_u32(device_proxy.state().await?) {
                Some(DeviceState::Activated) => {
                    let settings_proxy = SettingsProxy::new(&connection).await?;
                    let connections: Vec<zvariant::OwnedObjectPath> = settings_proxy.list_connections().await?;
                
                    // Try to find an existing connection with the same SSID or interface name
                    for connection_path in connections {
                        let setting_connection_proxy =
                            SettingsConnectionProxy::new_from_path(connection_path.clone(), &connection).await?;
                
                        let settings = setting_connection_proxy.get_settings().await?;
                        // println!("Setting: {:?}", settings);
                        if let Some(connection_props) = settings.get("connection") {
                            let id = connection_props
                                .get("id")
                                .and_then(|v| Some(v.downcast_ref::<Str>()));
                            let interface_name = connection_props
                                .get("interface-name")
                                .and_then(|v| Some(v.downcast_ref::<Str>()));
                
                                let _id = {
                                if let Some(Ok(_id)) = id {
                                    _id
                                } else {
                                    Str::from("")
                                }
                            };
                            let _interface = {
                                if let Some(Ok(_interface)) = interface_name {
                                    _interface
                                } else {
                                    Str::from("")
                                }
                            };
                            if _id == ssid && _interface == interface {
                                let ip4_str = get_ip4_str_address(&device_proxy, &connection).await;                              
                                let ip4_address = ip_to_bytes(&ip4_str);

                                let wireless_path = nm.get_device_by_ip_iface(interface).await?;
                                let wireless_proxy = WirelessProxy::new_from_path(wireless_path.clone(), &connection).await?;
                                let access_point_path = wireless_proxy.active_access_point().await?;
                                let access_point = AccessPointProxy::new_from_path(access_point_path, &connection).await?;
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

                                let wifi_info = WifiInfo {
                                    mac: mac_str_to_array(&hw_address)?,
                                    freq: frequency,
                                    rssi: signal_strength,
                                    security: security_type,
                                    ip4_addr: ip4_address
                                };
                                return Ok((true, wifi_info));
                            }

                        }
                    }
                }
                _ => {

                }
            }
        }
    }
    return Ok((false, WifiInfo {
        mac:  [0u8; WIFI_MAC_LEN],
        freq: 0,
        rssi: 0,
        security: WifiSecurity::WifiSecOpen,
        ip4_addr: [0u8; 4]} ));
}

fn ip_to_bytes(ip_str: &str) -> [u8; 4] {
    if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
        ip.octets() // If successfully parsed, return the 4-byte array
    } else {
        [0, 0, 0, 0] // If parsing fails, return an array of zeroes
    }
}
async fn get_ip4_str_address(device_proxy: &DeviceProxy<'_>, connection: &Connection) -> String {
    let ip4config_path = device_proxy.ip4_config().await;
    let ip4config = IP4ConfigProxy::new_from_path(ip4config_path.unwrap(), connection).await;

    let Ok(config) = ip4config else {
        return String::from("Unknown");
    }; // Assuming ip4config is a Result type
    let Ok(address_data) = config.address_data().await else {
        return String::from("Unknown");
    };
    let Some(address) = address_data.first().and_then(|addr| addr.get("address")) else {
        return String::from("Unknown");
    };

    address.downcast_ref().unwrap()
}

pub async fn connect_wifi(
    interface: &str,
    ssid: &str,
    password: Option<&str>,
    timeout: Duration
) -> Result<(bool, WifiInfo), Box<dyn std::error::Error>> {
    if ssid.len() > 32 {
        return Err("SSID Invalid".into());
    }
    if password.expect("Invalid Password").len() < 8 {
        return Err("Password invalid!".into());
    }
    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;
    let settings_proxy = SettingsProxy::new(&connection).await?;

    let connections: Vec<zvariant::OwnedObjectPath> = settings_proxy.list_connections().await?;

    // Try to find an existing connection with the same SSID or interface name
    let mut existing_connection_path: Option<zvariant::OwnedObjectPath> = None;
    for connection_path in connections {
        let setting_connection_proxy =
            SettingsConnectionProxy::new_from_path(connection_path.clone(), &connection).await?;

        let settings = setting_connection_proxy.get_settings().await?;
        // println!("Setting: {:?}", settings);
        if let Some(connection_props) = settings.get("connection") {
            let id = connection_props
                .get("id")
                .and_then(|v| Some(v.downcast_ref::<Str>()));
            let interface_name = connection_props
                .get("interface-name")
                .and_then(|v| Some(v.downcast_ref::<Str>()));

                let _id = {
                if let Some(Ok(_id)) = id {
                    _id
                } else {
                    Str::from("")
                }
            };
            let _interface = {
                if let Some(Ok(_interface)) = interface_name {
                    _interface
                } else {
                    Str::from("")
                }
            };
            if _id == ssid && _interface == interface {
                existing_connection_path = Some(connection_path);
                break;
            }
        }
    }

    if let Some(_connection_path) = existing_connection_path {
        println!("Updated and connected to Wi-Fi network '{}'", ssid);

        let settings_connection =
            SettingsConnectionProxy::new_from_path(_connection_path.clone(), &connection).await?;
        let mut settings: HashMap<String, HashMap<String, OwnedValue>> =
            settings_connection.get_settings().await?;

        // Update Wi-Fi security settings if password is provided
        if let Some(pass) = password {
            if let Some(wireless_security) = settings.get_mut("802-11-wireless-security") {
                wireless_security.insert(
                    "key-mgmt".to_owned(),
                    OwnedValue::from(Str::from("wpa-psk")),
                );
                wireless_security.insert("psk".to_owned(), OwnedValue::from(Str::from(pass)));
            } else {
                // Add wireless security settings if missing
                let mut security_props: HashMap<String, OwnedValue> = HashMap::new();
                security_props.insert(
                    "key-mgmt".to_owned(),
                    OwnedValue::from(Str::from("wpa-psk")),
                );
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
        settings_connection
            .update(convert_hashmap(&settings))
            .await?;

        // Activate the updated connection
        let device_path = nm.get_device_by_ip_iface(interface).await?;
        let base_path = ObjectPath::try_from("/")?;
        nm.activate_connection(&_connection_path, &device_path, &base_path)
            .await?;

    } else {
        println!("Creating and Connect to new AP");
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
        nm.activate_connection(&connection_path, &device_path, &base_path).await?;
    }
    let start = Instant::now();

    let mut check_result = check_connection_success(interface, ssid).await?;
    if check_result.0 == false {
        while start.elapsed() < timeout {
            check_result = check_connection_success(interface, ssid).await?;
            if check_result.0 {
                println!("Connected to Wi-Fi network '{}'", ssid);
                return Ok(check_result); // Successfully connected to the correct SSID
            }

            // Sleep for a short duration between checks (e.g., 1 second)
            sleep(Duration::from_secs(1)).await;
        }
    }
    println!("Cannot Connect to Wi-Fi network '{}'", ssid);
    Ok(check_result)
}

pub async fn remove_stored_wifi(remove_apname: String) -> Result<bool, Box<dyn std::error::Error>> {
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

        for (keystr, value) in &setcfgs {
            if keystr == "connection" {
                if let Some(type_value) = value.get("type") {
                    if let Some(type_str) = extract_string(type_value) {
                        if type_str == "802-11-wireless" {
                            let ssid: Option<String> = setcfgs
                            .get("connection")
                            .and_then(|c| c.get("id"))
                            .and_then(extract_string);
                    
                            // Handle default values
                            let default_ssid = "No ID found".to_string();
                            let ap_name = ssid.as_deref().unwrap_or(&default_ssid);
                            if ap_name == remove_apname {
                                println!("Found stored {}, begin remove it", remove_apname);
                                setting_connection_proxy.delete().await?;
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }

    }
    println!("Not found {} to remove", remove_apname);
    Ok(false)
}

pub async fn get_ap_info(
    interface: &str
) -> Result<(String, WifiInfo), Box<dyn std::error::Error + Send + Sync>> {

    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;

    match nm.get_device_by_ip_iface(interface).await {
        Ok(wireless_path) => {
            if let Ok(wireless_proxy) = WirelessProxy::new_from_path(wireless_path, &connection)
            .await {
                if let Ok(access_point_path) = wireless_proxy.active_access_point().await {
                    if let Ok(access_point) = AccessPointProxy::new_from_path(access_point_path, &connection)
                    .await {
                        if let Ok(ssid_option) = access_point.ssid().await {
                            if !ssid_option.is_empty() {
                                let ssid = String::from_utf8_lossy(&ssid_option);
                                // println!("SSID: {:?}", ssid);
                                match check_connection_success(interface, &ssid.to_string()).await {
                                    Ok(_result) => {
                                        return Ok((ssid.to_string(),_result.1));
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
    
                    }
                }
            }
        }

        Err(_) => {
            println!("Wireless device not found!");
        }
    }
    return Ok(("".to_string(), WifiInfo {
        mac:  [0u8; WIFI_MAC_LEN],
        freq: 0,
        rssi: 0,
        security: WifiSecurity::WifiSecOpen,
        ip4_addr: [0u8; 4]} ));
}

fn mac_str_to_array(mac: &str) -> Result<[u8; 6], Box<dyn std::error::Error>> {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() != 6 {
        return Err("Invalid MAC address format".into());
    }
    
    let mut mac_bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        mac_bytes[i] = u8::from_str_radix(part, 16)?;
    }
    
    Ok(mac_bytes)
}

pub async fn turn_off_wifi(interface: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;
    
    // Get the device path using the interface name
    let device_path = nm.get_device_by_ip_iface(interface).await?;
    
    // Create a DeviceProxy for the specific device
    let device_proxy = DeviceProxy::new_from_path(device_path, &connection).await?;
    if device_proxy.managed().await? {
        // Assuming 'managed' controls the device's management state
        device_proxy.set_managed(false).await?;
        println!("Wi-Fi radio turned off for interface '{}'", interface);
    } else {
        println!("Wifi already off");
    }

    Ok(())
}

pub async fn turn_on_wifi(interface: &str) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&connection).await?;
    
    // Get the device path using the interface name
    let device_path = nm.get_device_by_ip_iface(interface).await?;
    
    // Create a DeviceProxy for the specific device
    let device_proxy = DeviceProxy::new_from_path(device_path, &connection).await?;
    println!("Device State {}", device_proxy.state().await?);
    // Assuming 'managed' controls the device's management state
    if !device_proxy.managed().await? {
        // Assuming 'managed' controls the device's management state
        device_proxy.set_managed(true).await?;
        println!("Wi-Fi turned on for interface '{}'", interface);
    } else {
        println!("Wifi already on");
    }
    Ok(())
}