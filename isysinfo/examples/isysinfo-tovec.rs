use std::time::Duration;
use isysinfo::sys_info::SysInfo;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut _isysinfo = SysInfo::new();
    loop {
        let result = _isysinfo.to_vec();
        println!("Result: {:?}", result);
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
