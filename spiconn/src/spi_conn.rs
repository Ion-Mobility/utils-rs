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
    pub fn hexdump(&mut self, data: &[u8], len: usize) {
        // Ensure the length doesn't exceed the actual data size
        let len = len.min(data.len());
    
        for (i, chunk) in data[..len].chunks(16).enumerate() {
            // Print the offset
            print!("{:08x}: ", i * 16);
    
            // Print each byte in hex
            for byte in chunk {
                print!("{:02x} ", byte);
            }
    
            // Print spacing for incomplete chunks
            if chunk.len() < 16 {
                for _ in 0..(16 - chunk.len()) {
                    print!("   ");
                }
            }
    
            // Print the ASCII representation
            print!("|");
            for byte in chunk {
                let ascii_char = if byte.is_ascii_graphic() || *byte == b' ' {
                    *byte as char
                } else {
                    '.'
                };
                print!("{}", ascii_char);
            }
            println!("|");
        }
    }
    pub async fn xfer(&mut self, tx_buf: &[u8]) -> io::Result<Vec<u8>> {
        let mut rx_buf = Vec::with_capacity(tx_buf.len());

        loop {
            // Check if the ready line (first line) is high
            match self.ready.get_values([true;1]).await {
                Ok(_value) => {
                    if _value[0] == true {
                        for &byte in tx_buf {
                            let tx_buf_single = [byte];
                            let mut rx_buf_single = [0];
                
                            let mut transfer = SpidevTransfer::read_write(&tx_buf_single, &mut rx_buf_single);
                            self.spidev.transfer(&mut transfer)?;
                
                            rx_buf.push(rx_buf_single[0]); // Collect received byte
                        }
                        self.hexdump(&rx_buf, rx_buf[0] as usize);
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
