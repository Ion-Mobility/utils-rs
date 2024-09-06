use std::thread;
use std::time::Duration;
use log::{trace, info};
use mmcli::mmcli::IonModemCli; // Assuming IonModemCli is imported from mmcli crate
use cantool::can_tool::CanUtils; // Assuming CanUtils is imported from cantool crate
use logging::logging::MyLogging; // Assuming MyLogging is imported from logging crate
use spiconn::spi_conn::*;
use tokio::io;
use wifitools::{scan_wifi, get_stored_wifi};
#[tokio::main]
async fn main() -> io::Result<()> {
    match scan_wifi("wlp0s20f3").await {
        Ok(results) => {
            println!("{:?}", results);
        }
        Err(e) => {

        }
    }
    match get_stored_wifi().await {
        Ok(results) => {
            println!("{:?}", results);
        }
        Err(e) => {

        }
    }
    // // Initialize IonSpiConn asynchronously
    // let mut spi_conn = IonSpiConn::new_async("/dev/spidev1.0", 29).await;

    // // Example data to send over SPI
    // let tx_data = [0xAA; 130];

    // // Perform the SPI transfer and handle the result
    // match spi_conn.xfer(&tx_data).await {
    //     Ok(rx_data) => {
    //         // println!("Received data: {:?}", rx_data);
    //     }
    //     Err(e) => {
    //         eprintln!("SPI transfer failed: {:?}", e);
    //     }
    // }

    Ok(())
}