use gwclient::{
    get_ota_pub_message,
    get_ota_sub_message,
    send_ota_pub_message,
    send_ota_sub_message,
    get_isys_info
};
use tokio::time::{sleep, Duration};

async fn stub_send_ota_sub_message() {
    println!("Send Thread");
    loop {
        match send_ota_sub_message("Stub something to send on MQTT Sub Channel".as_bytes().to_vec()).await {
            Ok(_result) => {
                println!("Send Success");
            }
            Err(e) => {
                eprintln!("Can't send ota package {}", e);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

async fn stub_recv_ota_sub_message() {
    println!("Recv Thread");
    loop {
        match get_ota_sub_message().await {
            Ok(_result) => {
                println!("Recv success {:?}", String::from_utf8(_result));
            }
            _ => {

            }
        }
        sleep(Duration::from_millis(500)).await;
    }
}

async fn stub_send_ota_pub_message() {
    println!("Send Thread");
    loop {
        match send_ota_sub_message("Stub something to send on MQTT Pub Channel".as_bytes().to_vec()).await {
            Ok(_result) => {
                println!("Send Success");
            }
            Err(e) => {
                eprintln!("Can't send ota package {}", e);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

async fn stub_recv_ota_pub_message() {
    println!("Recv Thread");
    loop {
        match get_ota_sub_message().await {
            Ok(_result) => {
                println!("Recv success {:?}", String::from_utf8(_result));
            }
            _ => {

            }
        }
        sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // tokio::spawn(async move {
    //     tokio::join!(
    //         stub_send_ota_sub_message(),
    //         stub_recv_ota_sub_message(),
    //         stub_send_ota_pub_message(),
    //         stub_recv_ota_pub_message(),
    //     );
    // });
    loop {
        let result = get_isys_info().await;
        println!("Result: {:?}", result);
        sleep(Duration::from_millis(1000)).await;
    }
}
