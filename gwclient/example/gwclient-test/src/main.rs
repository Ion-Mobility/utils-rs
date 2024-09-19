use gwclient::{get_ota_message, send_ota_message};
use tokio::time::{sleep, Duration};

async fn stub_send_ota_message() {
    println!("Send Thread");
    loop {
        match send_ota_message("Stub something to send".as_bytes().to_vec()).await {
            Ok(_result) => {
                println!("Success");
            }
            Err(e) => {
                eprintln!("Can't send ota package {}", e);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

async fn stub_recv_ota_message() {
    println!("Recv Thread");
    loop {
        match get_ota_message().await {
            Ok(_result) => {
                println!("Recv success {:?}", String::from_utf8(_result));
            }
            Err(e) => {
                eprintln!("Can't send ota package {}", e);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tokio::spawn(async move {
        tokio::join!(
            stub_send_ota_message(),
            stub_recv_ota_message(),
        );
    });
    loop {

        sleep(Duration::from_millis(1000)).await;
    }
}
