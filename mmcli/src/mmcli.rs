use dbus::{blocking::Connection, Message};
use dbus::arg::RefArg;
use dbus::arg::messageitem::MessageItem;
use dbus::blocking::BlockingSender;
use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
use dbus::arg::messageitem::MessageItemDict;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{trace, info, warn};

#[derive(Debug)]
pub enum IonModemCliError {
    ModemError(String),
    ConnectionError(String),
    MethodCallError(String),
    SendError(String),
    ResponseError(String),
}

#[derive(Default, Debug)]
pub struct LteSignalStrength {
    pub rsrp: Option<i32>, // Reference Signal Received Power
    pub rsrq: Option<i32>, // Reference Signal Received Quality
}

impl std::fmt::Display for IonModemCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IonModemCliError::ModemError(msg) => write!(f, "Modem Error: {}", msg),
            IonModemCliError::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
            IonModemCliError::MethodCallError(msg) => write!(f, "Method Call Error: {}", msg),
            IonModemCliError::SendError(msg) => write!(f, "Send Error: {}", msg),
            IonModemCliError::ResponseError(msg) => write!(f, "Response Error: {}", msg),
        }
    }
}

impl std::error::Error for IonModemCliError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IonModemCli {
    destination: String,
    object: String,
    modem: String,
    ready: bool,
    last_check: Instant, // Field to track the last check time
    check_interval: Duration, // Duration between checks
}

impl Default for IonModemCli {
    fn default() -> Self {
        IonModemCli {
            destination: "org.freedesktop.ModemManager1".to_owned(),
            object: "/org/freedesktop/ModemManager1".to_owned(),
            modem: String::new(),
            ready: false,
            last_check: Instant::now(),
            check_interval: Duration::from_secs(10),
        }
    }
}

impl IonModemCli {
    pub fn new(destination: String, object: String, modem: String) -> Self {
        IonModemCli {
            destination,
            object,
            modem,
            ready: false,
            last_check: Instant::now(),
            check_interval: Duration::from_secs(10),
        }
    }

    fn modem_preparing(&mut self) -> Result<(), IonModemCliError> {
        match self.modem_path_detection() {
            Ok(_modempath) => {
                self.modem = _modempath;
                Ok(())
            }
            Err(e) => {
                info!("Modem preparation failed: {:?}", e);
                Err(IonModemCliError::ModemError(format!("Failed to prepare modem: {:?}", e)))
            }
        }
    }

    fn get_modem_properties(&self, object: &str, prop: &str) -> Result<Vec<MessageItem>, IonModemCliError> {
        // Check if self.modem is empty
        if self.modem.is_empty() {
            return Err(IonModemCliError::ModemError("Modem is not specified".to_owned()));
        }
        
        // Connect to the system bus
        let conn = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;
    
        let interface = "org.freedesktop.DBus.Properties";
    
        // Prepare the D-Bus message to get the property
        let msg = Message::new_method_call(&self.destination, &self.modem, interface, "Get")
            .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?
            .append2(object, prop);
    
        // Send the message and await the response
        let reply = conn.send_with_reply_and_block(msg, Duration::from_secs(2))
            .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;
    
        trace!("{:?}", reply);
        let enabled_variant = reply.get_items();
        trace!("{:?}", enabled_variant);
        
        Ok(enabled_variant)
    }

    fn modem_path_detection(&self) -> Result<String, IonModemCliError> {
        // Connect to the D-Bus system bus
        let connection = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;

        // Get managed objects
        let proxy = connection.with_proxy(&self.destination, &self.object, Duration::from_millis(5000));
        let managed_objects: HashMap<dbus::Path<'_>, HashMap<String, HashMap<String, dbus::arg::Variant<Box<dyn RefArg>>>>>
            = proxy.get_managed_objects()
                .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to get managed objects: {}", e)))?;

        // Iterate over the managed objects and find the modem objects
        for (path, interfaces) in managed_objects {
            if interfaces.contains_key("org.freedesktop.ModemManager1.Modem") {
                return Ok(path.to_string());
            }
        }

        Err(IonModemCliError::ModemError("No modem object found".to_owned()))
    }

    pub fn is_location_enabled(&self) -> bool {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Location", "Enabled") {
            Ok(results) => {
                for result in results.iter() {
                    trace!("{:?}", result);
                    if let MessageItem::Variant(ret_variant) = result {
                        if let MessageItem::UInt32(locationmask) = **ret_variant {
                            trace!("Mask: {}", locationmask);
                            return (locationmask & 4) != 0;
                        }
                    }
                }
            }
            Err(e) => {
                info!("Failed to get location enabled state: {:?}", e);
            }
        }
        false
    }

    pub fn is_modem_enabled(&self) -> bool {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem", "State") {
            Ok(results) => {
                for result in results.iter() {
                    trace!("{:?}", result);
                    if let MessageItem::Variant(ret_variant) = result {
                        if let MessageItem::Int32(modemmask) = **ret_variant {
                            return (modemmask & 8) != 0;
                        }
                    }
                }
            }
            Err(e) => {
                info!("Failed to get modem enabled state: {:?}", e);
            }
        }
        false
    }

    pub fn get_signal_quality(&self) -> u32 {
        // Placeholder method, implement based on your actual requirements
        0
    }

    pub fn get_signal_strength(&self) -> f32 {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Signal", "Lte") {
            Ok(results) => {
                for result in results.iter() {
                    if let MessageItem::Variant(ret_variant) = result {
                        if let MessageItem::Dict(ref dict) = **ret_variant {
                            let a = dict.to_vec();
                            for (x, y) in a {
                                if x == "rsrp".into() {
                                    if let MessageItem::Variant(rsrpval) = y {
                                        if let MessageItem::Double(rsrpret) = *rsrpval {
                                            return rsrpret as f32;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                info!("Failed to get signal strength: {:?}", e);
            }
        }
        0.0
    }

    pub fn get_location(&self) -> String {
        let mut nmea_str = String::new();
        if self.is_location_enabled() {
            // Connect to the system bus
            let c = Connection::new_system().expect("D-Bus connection failed");

            // Specify the interface and method to call for getting location
            let interface = "org.freedesktop.ModemManager1.Modem.Location";
            let gpsmethod = "GetLocation";

            // Prepare the D-Bus message
            let msg = Message::new_method_call(&self.destination, &self.modem, interface, gpsmethod)
                .expect("Failed to create method call");

            // Send the message and await the response
            let reply = c.send_with_reply_and_block(msg, Duration::from_secs(2));
            match reply {
                Ok(result) => {
                    // Parse the response to get the Args
                    let responds: Vec<MessageItem> = result.get_items();
                    for respond in responds.iter() {
                        if let MessageItem::Dict(dict) = respond {
                            let a = dict.to_vec();
                            for (x, y) in a {
                                if let MessageItem::UInt32(id) = x {
                                    if id == 4 {
                                        if let MessageItem::Variant(var) = y {
                                            if let MessageItem::Str(nmea) = *var {
                                                nmea_str = nmea;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to get location: {:?}", e);
                }
            }
        }

        nmea_str
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn waiting_for_ready(&mut self) -> bool {
        if !self.ready {
            if let Err(err) = self.modem_preparing() {
                info!("Failed to prepare modem: {}", err);
                return false;
            }
            self.ready = true;
        } else {
            let now = Instant::now();
            if now.duration_since(self.last_check) >= self.check_interval {
                info!("Recheck modem every {:?} Seconds", self.check_interval);
                self.last_check = now;
                match self.modem_path_detection() {
                    Ok(_modempath) => {
                        if self.modem != _modempath {
                            warn!("Modem already changed, need to reinit everything");
                            self.ready = false;
                            self.modem = _modempath;
                        } else {
                            info!("Modem hasn't changed!");
                            self.ready = true;
                        }
                    }
                    Err(e) => {
                        info!("Modem preparation failed: {:?}", e);
                        self.ready = false;
                        self.modem = "".to_string();
                    }
                }
            }
        }
        self.ready
    }

    pub fn setup_modem_enable(&self, status: bool) -> Result<(), IonModemCliError> {
        // Check if self.modem is empty
        if self.modem.is_empty() {
            return Err(IonModemCliError::ModemError("Modem is not specified".to_owned()));
        }

        let interface = "org.freedesktop.ModemManager1.Modem";
        let method = "Enable";
        let connection = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;
    
        // Prepare the D-Bus message to enable the modem
        let msg = Message::new_method_call("org.freedesktop.ModemManager1", &self.modem, interface, method)
            .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?
            .append1(status);
    
        // Send the message and handle the response
        connection.send_with_reply_and_block(msg, Duration::from_millis(2000))
            .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;
    
        Ok(())
    }

    pub fn setup_location(&self, sources: u32, signal_location: bool) -> Result<(), IonModemCliError> {
        // Check if self.modem is empty
        if self.modem.is_empty() {
            return Err(IonModemCliError::ModemError("Modem is not specified".to_owned()));
        }

        let interface = "org.freedesktop.ModemManager1.Modem.Location";
        let method = "Setup";
    
        // Connect to the system bus
        let connection = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;
    
        // Prepare the D-Bus message to setup location
        let msg = Message::new_method_call("org.freedesktop.ModemManager1", &self.modem, interface, method)
            .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?
            .append2(sources, signal_location);
    
        // Send the message and handle the response
        connection.send_with_reply_and_block(msg, Duration::from_millis(2000))
            .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;
    
        Ok(())
    }

    // Get the current signal refresh rate
    pub fn get_signal_refresh_rate(&self) -> Result<u32, IonModemCliError> {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Signal", "Rate") {
            Ok(results) => {
                for result in results.iter() {
                    if let MessageItem::Variant(ret_variant) = result {
                        if let MessageItem::UInt32(rate) = **ret_variant {
                            return Ok(rate);
                        }
                    }
                }
            }
            Err(e) => {
                info!("Failed to get signal refresh rate: {:?}", e);
            }
        }
        Err(IonModemCliError::ModemError("Failed to retrieve signal refresh rate".to_owned()))
    }

    // Set the signal refresh rate
    pub fn setup_signal_refresh_rate(&self, rate: u32) -> Result<(), IonModemCliError> {
        // Check if self.modem is empty
        if self.modem.is_empty() {
            return Err(IonModemCliError::ModemError("Modem is not specified".to_owned()));
        }

        let interface = "org.freedesktop.ModemManager1.Modem.Signal";
        let method = "Setup";
        let connection = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;

        // Prepare the D-Bus message to set the signal refresh rate
        let msg = Message::new_method_call("org.freedesktop.ModemManager1", &self.modem, interface, method)
            .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?
            .append1(rate);

        // Send the message and handle the response
        connection.send_with_reply_and_block(msg, Duration::from_millis(2000))
            .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    // Get the LTE signal strength
    pub fn get_lte_signal_strength(&self) -> Result<Option<LteSignalStrength>, IonModemCliError> {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Signal", "Lte") {
            Ok(results) => {
                for result in results.iter() {
                    if let MessageItem::Variant(ref ret_variant) = result {
                        if let MessageItem::Dict(ref dict) = **ret_variant {
                            let mut lte_signal = LteSignalStrength::default();
                            for (key, value) in dict.to_vec() {
                                if let MessageItem::Str(ref key_str) = key {
                                    match key_str.as_str() {
                                        "rsrp" => {
                                            if let MessageItem::Variant(ref rsrp_value) = value {
                                                if let MessageItem::Double(rsrp) = **rsrp_value {
                                                    lte_signal.rsrp = Some(rsrp as i32);
                                                }
                                            }
                                        }
                                        "rsrq" => {
                                            if let MessageItem::Variant(ref rsrq_value) = value {
                                                if let MessageItem::Double(rsrq) = **rsrq_value {
                                                    lte_signal.rsrq = Some(rsrq as i32);
                                                }
                                            }
                                        }
                                        // Add other LTE signal-related parameters here
                                        _ => {}
                                    }
                                }
                            }
                            return Ok(Some(lte_signal));
                        }
                    }
                }
            }
            Err(e) => {
                info!("Failed to get LTE signal strength: {:?}", e);
            }
        }
        Err(IonModemCliError::ModemError("Failed to retrieve LTE signal strength".to_owned()))
    }

    pub fn list_firmware(&self) -> Result<(String, Vec<HashMap<String, MessageItem>>), IonModemCliError> {
        if self.modem.is_empty() {
            return Err(IonModemCliError::ResponseError("Modem is not specified".to_owned()));
        }

        let conn = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;
        
        let interface = "org.freedesktop.ModemManager1.Modem.Firmware";
        let method = "List";

        let msg = Message::new_method_call(&self.destination, &self.modem, interface, method)
            .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?;

        let reply = conn.send_with_reply_and_block(msg, Duration::from_secs(2))
            .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;

        let items = reply.get_items();
        let mut installed_firmware = Vec::new();
        let mut selected_firmware = String::new();

        if let Some(MessageItem::Str(selected)) = items.get(0) {
            selected_firmware = selected.to_string();
        }

        if let Some(MessageItem::Array(ref array)) = items.get(1) {
            for item in array.iter() {
                if let MessageItem::Dict(ref dict) = item {
                    let mut hashmap = HashMap::new();
                    for (key, value) in dict.to_vec() {
                        if let MessageItem::Str(ref key_str) = key {
                            hashmap.insert(key_str.to_string(), value);
                        }
                    }
                    installed_firmware.push(hashmap);
                }
            }
        }

        Ok((selected_firmware, installed_firmware))
    }

    pub fn list_profiles(&self) -> Result<Vec<HashMap<String, MessageItem>>, IonModemCliError> {
        if self.modem.is_empty() {
            return Err(IonModemCliError::ResponseError("Modem is not specified".to_owned()));
        }

        let conn = Connection::new_system()
            .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;
        
        let interface = "org.freedesktop.ModemManager1.Modem.Modem3gpp.ProfileManager";
        let method = "List";

        let msg = Message::new_method_call(&self.destination, &self.modem, interface, method)
            .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?;

        let reply = conn.send_with_reply_and_block(msg, Duration::from_secs(2))
            .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;

        let items = reply.get_items();
        if let Some(MessageItem::Array(ref array)) = items.get(0) {
            let mut profiles = Vec::new();
            for item in array.iter() {
                if let MessageItem::Dict(ref dict) = item {
                    let mut hashmap = HashMap::new();
                    for (key, value) in dict.to_vec() {
                        if let MessageItem::Str(ref key_str) = key {
                            hashmap.insert(key_str.to_string(), value);
                        }
                    }
                    profiles.push(hashmap);
                }
            }
            Ok(profiles)
        } else {
            Err(IonModemCliError::ResponseError("Invalid response format".to_owned()))
        }
    }

    // Implement the `set_profile` method
    // pub fn set_profile(&self, requested_properties: HashMap<String, MessageItem>) -> Result<HashMap<String, MessageItem>, IonModemCliError> {
    //     if self.modem.is_empty() {
    //         return Err(IonModemCliError::ResponseError("Modem is not specified".to_owned()));
    //     }

    //     let conn = Connection::new_system()
    //         .map_err(|e| IonModemCliError::ConnectionError(format!("Failed to connect to system bus: {}", e)))?;
        
    //     let interface = "org.freedesktop.ModemManager1.Modem.Modem3gpp.ProfileManager";
    //     let method = "Set";

    //     // Create MessageItemDict manually
    //     let mut dict: MessageItemDict = MessageItemDict::new();

    //     // for (key, value) in requested_properties.into_iter() {
    //     //     // Convert String to MessageItem::Str
    //     //     let key_item = MessageItem::Str(key);
    //     //     dict.insert(key_item, value);
    //     // }
        
    //     // Box the Dict and wrap it in MessageItem::Variant
    //     let variant_properties = MessageItem::Variant(Box::new(MessageItem::Dict(dict)));

    //     let msg = Message::new_method_call(&self.destination, &self.modem, interface, method)
    //         .map_err(|e| IonModemCliError::MethodCallError(format!("Failed to create method call: {}", e)))?
    //         .append1(variant_properties);

    //     let reply = conn.send_with_reply_and_block(msg, Duration::from_secs(2))
    //         .map_err(|e| IonModemCliError::SendError(format!("Failed to send message: {}", e)))?;

    //     let items = reply.get_items();
    //     if let Some(MessageItem::Dict(ref dict)) = items.get(1) {
    //         let mut hashmap = HashMap::new();
    //         for (key, value) in dict.to_vec() {
    //             if let MessageItem::Str(ref key_str) = key {
    //                 hashmap.insert(key_str.to_string(), value);
    //             }
    //         }
    //         Ok(hashmap)
    //     } else {
    //         Err(IonModemCliError::ResponseError("Invalid response format".to_owned()))
    //     }
    // }

}