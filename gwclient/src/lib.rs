use zbus::{Connection, Proxy};
use zbus::fdo::Result;
use isysinfo::sys_info::SysInfo;
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
    let mut _result: Vec<u8> = Vec::new();
    if let Ok(connection) = Connection::system().await {
        if let Ok(proxy) = Proxy::new(
            &connection,
            "org.ion.IComGateway",  // D-Bus destination (service name)
            "/org/ion/IComGateway", // Object path
            "org.ion.IComGateway",  // Introspection interface
        )
        .await {
            _result = proxy.call("GetSystemInfo", &()).await?;
            if let Ok(_result_isysinfo) = SysInfo::from_vec(&_result) {
                return Ok(_result_isysinfo);
            } else {
                return Err(zbus::fdo::Error::Failed("Can't parse isysinfo".into()).into());
            }

        }
    }
    return Err(zbus::fdo::Error::Failed("Can't get isysinfo".into()).into());
}