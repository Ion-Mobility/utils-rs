use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use tokio_gpiod::{Chip, Active, Input, Lines, Options};
use std::io;
use tokio::time::{sleep, Duration};
pub struct IonSpiConn {
    spidev: Spidev,
    ready: Lines<Input>, // Correct type for GPIO lines
}

impl IonSpiConn {
    pub async fn new_async(spidevpath: &str, ready_pin: u32) -> Self {
        let mut spidev = Spidev::open(&spidevpath).expect("Failed to open SPI device");

        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(500_000)
            .mode(SpiModeFlags::SPI_MODE_1)
            .build();

        spidev.configure(&options).expect("Failed to configure SPI device");

        let chip = Chip::new("gpiochip0").await.expect("Failed to open GPIO chip");

        let opts = Options::input([ready_pin]) // Configure GPIO pin
            .active(Active::High)
            .consumer("spi-rdy"); // Set consumer label
    
        let ready = chip.request_lines(opts).await.expect("Failed to request GPIO lines");

        IonSpiConn { spidev, ready }
    }

    pub async fn xfer(&mut self, tx_buf: &[u8]) -> io::Result<Vec<u8>> {
        let mut rx_buf = Vec::with_capacity(tx_buf.len());

        loop {
            // Check if the ready line (first line) is high
            match self.ready.get_values([true;1]).await {
                Ok(_value) => {
                    println!("Rdy status {:?}", _value);
                    if _value[0] == true {
                        for &byte in tx_buf {
                            let tx_buf_single = [byte];
                            let mut rx_buf_single = [0];
                
                            let mut transfer = SpidevTransfer::read_write(&tx_buf_single, &mut rx_buf_single);
                            self.spidev.transfer(&mut transfer)?;
                
                            rx_buf.push(rx_buf_single[0]); // Collect received byte
                        }
                        break;
                    }
                }
                Err(_) => {

                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        Ok(rx_buf)
    }
}
