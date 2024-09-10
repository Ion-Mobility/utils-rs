use rusty_network_manager::{SettingsProxy, AccessPointProxy, NetworkManagerProxy, WirelessProxy, SettingsConnectionProxy};
// use zbus::zvariant::{OwnedValue, Value as ZValue};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use zbus::{Connection, Proxy};
use zvariant::OwnedValue;
// use std::collections::HashMap;

pub async fn get_telematic_pack() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
    let received_pack: Vec<u8> = proxy.call("GetLatestReceived", &(1u8)).await?;
    Ok(received_pack)
}

pub async fn send_telematic_pack(send_pack: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a connection to the system bus
    let connection = Connection::system().await?;

    // Create a proxy for interacting with the D-Bus service
    let proxy = Proxy::new(
        &connection,
        "org.ion.IComGateway",           // D-Bus destination (service name)
        "/org/ion/IComGateway",          // Object path
        "org.ion.IComGateway" // Introspection interface
    ).await?;

    let _ = proxy.call("SendPackg", &(send_pack, 1u8)).await?;
    // println!("Received: {:?}", send_pack);
    
    Ok(())
}