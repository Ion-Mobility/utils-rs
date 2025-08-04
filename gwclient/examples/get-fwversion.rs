use gwclient::get_fwversion_info;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        if let Ok(isysinfo) = get_fwversion_info().await {
            println!("Result: {:?}", isysinfo);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

}
