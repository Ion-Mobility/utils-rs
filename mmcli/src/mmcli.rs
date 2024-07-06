use dbus::{blocking::Connection, Message};
use dbus::arg::RefArg;
use std::time::Duration;
use std::collections::HashMap;
use log::{trace, info};
use dbus::arg::messageitem::MessageItem;
use dbus::blocking::BlockingSender;
use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;

#[derive(Debug)]
pub enum IonModemCliError {
    ModemError(String),
    ConnectionError(String),
    MethodCallError(String),
    SendError(String),
    ResponseError(String),
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
}

impl Default for IonModemCli {
    fn default() -> Self {
        IonModemCli {
            destination: "org.freedesktop.ModemManager1".to_owned(),
            object: "/org/freedesktop/ModemManager1".to_owned(),
            modem: String::new(),
            ready: false,
        }
    }
}

impl IonModemCli {
    pub fn new(destination: String, object: String, modem: String, ready: bool) -> Self {
        IonModemCli {
            destination,
            object,
            modem,
            ready,
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
}