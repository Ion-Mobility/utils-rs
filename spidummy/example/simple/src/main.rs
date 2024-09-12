use tokio::io;
use spidummy::spi_dummy::SpiDummy;
#[tokio::main]
async fn main() -> io::Result<()> {
    // Create an instance of SpiDummy
    let mut spi_dummy = SpiDummy::new("/dev/ion-conn").await?;
    
    loop {
        // Receive data from the device
        let received_data = spi_dummy.recv(259).await?;
        println!("Received data: {:?}", received_data);

        match spi_dummy.send(received_data.to_vec()).await {
            Ok(_) => {
                println!("Send Success");
            }
            Err(e) => {
                eprintln!("Send failed: {:?}", e);
            }
        }
    }

    Ok(())
}