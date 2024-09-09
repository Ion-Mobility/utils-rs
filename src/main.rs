use log::{trace, info};
use mmcli::mmcli::IonModemCli; // Assuming IonModemCli is imported from mmcli crate
use cantool::can_tool::CanUtils; // Assuming CanUtils is imported from cantool crate
use logging::logging::MyLogging; // Assuming MyLogging is imported from logging crate
use spiconn::spi_conn::*;
use tokio::io;
use tokio::time::{sleep, Duration};
use wifitools::{scan_wifi, get_stored_wifi, get_wificmd_pack, send_wificmd_pack};
use icommsg::icom_msg::IONICOMPacketType;
#[tokio::main]
async fn main() -> io::Result<()> {
    loop {
        if let Ok(_wifi_cmd) = get_wificmd_pack().await {
            println!("Found wifi command package: {:?}", _wifi_cmd);
            let _ = send_wificmd_pack(_wifi_cmd).await;
        }
        sleep(Duration::from_millis(100)).await;
    }
}
