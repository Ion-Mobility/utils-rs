use log::{trace, info};
use mmcli::mmcli::IonModemCli; // Assuming IonModemCli is imported from mmcli crate
use cantool::can_tool::CanUtils; // Assuming CanUtils is imported from cantool crate
use logging::logging::MyLogging; // Assuming MyLogging is imported from logging crate
use spiconn::spi_conn::*;
use tokio::io;
use tokio::time::{sleep, Duration};
use telconn::{get_telematic_pack, send_telematic_pack};
use icommsg::icom_msg::IONICOMPacketType;
#[tokio::main]
async fn main() -> io::Result<()> {
    loop {
        if let Ok(_tel_pack_rx) = get_telematic_pack().await {
            println!("Found wifi command package: {:?}", _tel_pack_rx);
            let _ = send_telematic_pack(_tel_pack_rx).await;
        }
        sleep(Duration::from_millis(100)).await;
    }
}
