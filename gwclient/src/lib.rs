use zbus::{Connection, Proxy};
use zbus::fdo::Result;
// use isysinfo::sys_info::SysInfo;
use isysinfo::sys_info::{LteInfo, SysInfo, WifiInfo, BikeNotice, GpsInfo};
use tokio::time::timeout;
use std::time::Duration;
// use zbus::{Error, fdo};
// use zbus::names::OwnedErrorName;
// use std::convert::TryInto; // Required for `.try_into()`

pub async fn get_ota_pub_message() -> Result<Vec<u8>> {
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
            if let Ok(received_pack) = proxy.call("RecvOtaPubMessage", &()).await {
                _result = received_pack;
            }

        }
    }

    // Call the `Introspect` method to retrieve introspection XML
    Ok(_result)
}

pub async fn get_ota_sub_message() -> Result<Vec<u8>> {
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
            if let Ok(received_pack) = proxy.call("RecvOtaSubMessage", &()).await {
                _result = received_pack;
            }

        }
    }

    // Call the `Introspect` method to retrieve introspection XML
    Ok(_result)
}

pub async fn send_ota_pub_message(data: Vec<u8>) -> Result<bool> {
    // Create a connection to the system bus
    let mut _result = false;
    if let Ok(connection) = Connection::system().await {
        if let Ok(proxy) = Proxy::new(
            &connection,
            "org.ion.IComGateway",  // D-Bus destination (service name)
            "/org/ion/IComGateway", // Object path
            "org.ion.IComGateway",  // Introspection interface
        )
        .await {
            proxy.call("SendOtaPubMessage", &(data)).await?;
        }
    }

    // Call the `Introspect` method to retrieve introspection XML
    Ok(_result)
}

pub async fn send_ota_sub_message(data: Vec<u8>) -> Result<bool> {
    // Create a connection to the system bus
    let mut _result = false;
    if let Ok(connection) = Connection::system().await {
        if let Ok(proxy) = Proxy::new(
            &connection,
            "org.ion.IComGateway",  // D-Bus destination (service name)
            "/org/ion/IComGateway", // Object path
            "org.ion.IComGateway",  // Introspection interface
        )
        .await {
            proxy.call("SendOtaSubMessage", &(data)).await?;
        }
    }

    // Call the `Introspect` method to retrieve introspection XML
    Ok(_result)
}

pub async fn get_isys_info() -> Result<SysInfo> {
    // Create a connection to the system bus
    let mut _result: SysInfo = SysInfo::new();
    if let Ok(connection) = Connection::system().await {
        if let Ok(proxy) = Proxy::new(
            &connection,
            "org.ion.IComGateway",  // D-Bus destination (service name)
            "/org/ion/IComGateway", // Object path
            "org.ion.IComGateway",  // Introspection interface
        )
        .await {
            // Call the D-Bus method to get system info (returns Vec<u8>)
            _result = proxy.call("GetSystemInfo", &()).await?;
        } 
    }
    Ok(_result)
}

pub async fn get_wifi_info() -> WifiInfo {
    let timeout_duration = Duration::from_millis(5000);

    let operation = async {
        let connection = Connection::system().await.ok()?;
        let proxy = Proxy::new(
            &connection,
            "org.ion.IComGateway",
            "/org/ion/IComGateway",
            "org.ion.IComGateway",
        ).await.ok()?;

        let gps_info: WifiInfo = proxy.call("GetWifiInfo", &()).await.ok()?;

        Some(gps_info)
    };

    match timeout(timeout_duration, operation).await {
        Ok(Some(gps_info)) => gps_info,
        _ => WifiInfo::new(), // default fallback on any failure
    }
}

pub async fn get_lte_info() -> LteInfo {
    let timeout_duration = Duration::from_millis(5000);

    let operation = async {
        let connection = Connection::system().await.ok()?;
        let proxy = Proxy::new(
            &connection,
            "org.ion.IComGateway",
            "/org/ion/IComGateway",
            "org.ion.IComGateway",
        ).await.ok()?;

        let gps_info: LteInfo = proxy.call("GetLteInfo", &()).await.ok()?;

        Some(gps_info)
    };

    match timeout(timeout_duration, operation).await {
        Ok(Some(gps_info)) => gps_info,
        _ => LteInfo::new(), // default fallback on any failure
    }
}


pub async fn get_gps_info() -> GpsInfo {
    let timeout_duration = Duration::from_millis(5000);

    let operation = async {
        let connection = Connection::system().await.ok()?;
        let proxy = Proxy::new(
            &connection,
            "org.ion.IComGateway",
            "/org/ion/IComGateway",
            "org.ion.IComGateway",
        ).await.ok()?;

        let gps_info: GpsInfo = proxy.call("GetGpsInfo", &()).await.ok()?;

        Some(gps_info)
    };

    match timeout(timeout_duration, operation).await {
        Ok(Some(gps_info)) => gps_info,
        _ => GpsInfo::new(), // default fallback on any failure
    }
}

pub async fn get_notice_info() -> BikeNotice {
    let timeout_duration = Duration::from_millis(5000);

    let operation = async {
        let connection = Connection::system().await.ok()?;
        let proxy = Proxy::new(
            &connection,
            "org.ion.IComGateway",
            "/org/ion/IComGateway",
            "org.ion.IComGateway",
        ).await.ok()?;

        let notice_info: BikeNotice = proxy.call("GetNoticeInfo", &()).await.ok()?;
        Some(notice_info)
    };

    match timeout(timeout_duration, operation).await {
        Ok(Some(notice_info)) => notice_info,
        _ => BikeNotice::new(), // default fallback on any failure
    }
}