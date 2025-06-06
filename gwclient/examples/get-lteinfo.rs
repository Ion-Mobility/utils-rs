use gwclient::{get_lte_info};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        let lteInfo = get_lte_info().await;
        println!("lteInfo: {:?}", lteInfo);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
