use gwclient::{get_notice_info};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    loop {
        let notice_info = get_notice_info().await;
        println!("noticeInfo: {:?}", notice_info);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
