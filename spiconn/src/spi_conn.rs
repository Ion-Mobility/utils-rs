use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use tokio_gpiod::{Chip, Active, Input, Lines, Options};
use std::io;
use tokio::time::{sleep, Duration};
use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum IonSpiConnError {
    IoError(io::Error),
    GpioError(String), // Use a String for GPIO errors
    PacketError(Box<dyn StdError + Send>), // Boxed error with Send
}

impl fmt::Display for IonSpiConnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StdError for IonSpiConnError {}

impl From<io::Error> for IonSpiConnError {
    fn from(err: io::Error) -> IonSpiConnError {
        IonSpiConnError::IoError(err)
    }
}

impl From<String> for IonSpiConnError {
    fn from(err: String) -> IonSpiConnError {
        IonSpiConnError::GpioError(err)
    }
}

impl From<Box<dyn StdError + Send>> for IonSpiConnError {
    fn from(err: Box<dyn StdError + Send>) -> IonSpiConnError {
        IonSpiConnError::PacketError(err)
    }
}

pub struct IonSpiConn {
    spidev: Spidev,
    ready: Lines<Input>, // Correct type for GPIO lines
}

impl IonSpiConn {
    pub async fn new_async(spidevpath: &str, ready_pin: u32) -> Result<Self, IonSpiConnError> {
        let mut spidev = Spidev::open(&spidevpath).map_err(IonSpiConnError::from)?;

        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(500_000)
            .mode(SpiModeFlags::SPI_MODE_1)
            .build();

        spidev.configure(&options).map_err(IonSpiConnError::from)?;

        let chip = Chip::new("gpiochip0").await.map_err(|e| IonSpiConnError::from(e.to_string()))?;

        let opts = Options::input([ready_pin])
            .active(Active::High)
            .consumer("spi-rdy");
    
        let ready = chip.request_lines(opts).await.map_err(|e| IonSpiConnError::from(e.to_string()))?;

        Ok(IonSpiConn { spidev, ready })
    }

    pub fn hexdump(&self, data: &[u8], len: usize) {
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

    pub async fn xfer(&mut self, tx_buf: &[u8]) -> Result<Vec<u8>, IonSpiConnError> {
        let mut rx_buf = Vec::with_capacity(tx_buf.len());

        loop {
            match self.ready.get_values([true; 1]).await.map_err(|e| IonSpiConnError::from(e.to_string())) {
                Ok(_value) => {
                    if _value[0] == true {
                        for &byte in tx_buf {
                            let tx_buf_single = [byte];
                            let mut rx_buf_single = [0];
                
                            let mut transfer = SpidevTransfer::read_write(&tx_buf_single, &mut rx_buf_single);
                            self.spidev.transfer(&mut transfer).map_err(IonSpiConnError::from)?;
                
                            rx_buf.push(rx_buf_single[0]);
                        }
                        self.hexdump(&rx_buf, rx_buf.len() as usize);
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        Ok(rx_buf)
    }
}
