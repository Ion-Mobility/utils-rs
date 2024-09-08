use std::thread;
use std::time::Duration;
use log::{trace, info};
use mmcli::mmcli::IonModemCli; // Assuming IonModemCli is imported from mmcli crate
use cantool::can_tool::CanUtils; // Assuming CanUtils is imported from cantool crate
use logging::logging::MyLogging; // Assuming MyLogging is imported from logging crate
use spiconn::spi_conn::*;
use tokio::io;
use wifitools::{scan_wifi, get_stored_wifi};
use icomconn::icom_conn::IONICOMPacketType;
#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize IonSpiConn asynchronously
    let mut spi_conn = IonSpiConn::new_async("/dev/spidev1.0", 29).await;
    loop {
        // Example data to send over SPI
        let tx_data = [0xAA; 128];
        let txbuf = IONICOMPacketType::new_from(tx_data.to_vec());
        // Perform the SPI transfer and handle the result
        match spi_conn.xfer(&txbuf.to_byte_array()).await {
            Ok(rx_data) => {
                // println!("Received data: {:?}", rx_data);
                match IONICOMPacketType::from_byte_array(rx_data) {
                    Ok(_icom_rx) => {
                        println!("{:?}", _icom_rx);
                    }
                    Err(e) => {
                        eprintln!("error: e");
                    }
                }
            }
            Err(e) => {
                eprintln!("SPI transfer failed: {:?}", e);
            }
        }
    }


    Ok(())
}