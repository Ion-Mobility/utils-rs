use tokio::fs::{OpenOptions, File};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

pub struct SpiDummy {
    file: File,
}

impl SpiDummy {
    pub async fn new(device_path: &str) -> io::Result<Self> {
        // Open the device file for both reading and writing
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)
            .await?;

        Ok(SpiDummy { file })
    }

    pub async fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
        println!("Sending {} Bytes", data.len());
        // Write data to the device file
        self.file.write_all(&data).await
    }

    pub async fn recv(&mut self, buffer_size: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; buffer_size];
        let bytes_read = self.file.read(&mut buffer).await?;
        buffer.truncate(bytes_read); // Trim buffer to actual read bytes
        Ok(buffer)
    }
}
