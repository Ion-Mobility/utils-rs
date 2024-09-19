use rusty_network_manager::{
    AccessPointProxy, NetworkManagerProxy, SettingsConnectionProxy, SettingsProxy, WirelessProxy, DeviceProxy
};
// use zbus::zvariant::{OwnedValue, Value as ZValue};
use std::collections::HashMap;
use tokio::time::{sleep, Duration, Instant};
use zbus::zvariant::Value;
use zbus::{Connection, Proxy};
use zvariant::{ObjectPath, OwnedValue, Str};

pub async fn get_ota_message() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
    let received_pack: Vec<u8> = proxy.call("RecvOtaMessage", &()).await?;
    Ok(received_pack)
}

pub async fn send_ota_message(send_pack: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
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

    proxy.call("SendOtaMessage", &(send_pack)).await?;

    Ok(())
}
