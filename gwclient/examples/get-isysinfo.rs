use gwclient::get_isys_info;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        let result = get_isys_info().await;
        println!("Result: {:?}", result);
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
