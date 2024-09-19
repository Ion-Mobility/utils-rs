use rusty_network_manager::{
    AccessPointProxy, NetworkManagerProxy, SettingsConnectionProxy, SettingsProxy, WirelessProxy, DeviceProxy
};
// use zbus::zvariant::{OwnedValue, Value as ZValue};
use std::collections::HashMap;
use tokio::time::{sleep, Duration, Instant};
use zbus::zvariant::Value;
use zbus::{Connection, Proxy};
use zvariant::{ObjectPath, OwnedValue, Str};

pub async fn get_ota_message() -> Vec<u8> {
    // Create a connection to the system bus
    let mut _result: Vec<u8> = Vec::new();
    if let Ok(connection) = Connection::system().await {
        if let Ok(proxy) = Proxy::new(
            &connection,
            "org.ion.IComGateway",  // D-Bus destination (service name)
            "/org/ion/IComGateway", // Object path
            "org.ion.IComGateway",  // Introspection interface
        )
        .await {
            if let Ok(received_pack) = proxy.call("RecvOtaMessage", &()).await {
                _result = received_pack;
            }

        }
    }

    // Call the `Introspect` method to retrieve introspection XML
    _result
}

// pub async fn send_ota_message(send_pack: Vec<u8>) -> bool {
//     let mut result = false;
//     // Create a connection to the system bus
//     if let Ok(connection) = Connection::system().await {
//         // Create a proxy for interacting with the D-Bus service
//         if let Ok(proxy) = Proxy::new(
//             &connection,
//             "org.ion.IComGateway",  // D-Bus destination (service name)
//             "/org/ion/IComGateway", // Object path
//             "org.ion.IComGateway",  // Introspection interface
//         )
//         .await {
//             // Call the D-Bus method and check for a successful response
//             proxy.call("SendOtaMessage", &(send_pack)).await;
//         }
//     }
//     result
// }
