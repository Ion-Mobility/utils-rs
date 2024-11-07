use isysinfo::sys_info::SysInfo;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut _isysinfo = SysInfo::new();
    let invalid = [0u8, 10];
    let validbuf = [0u8, 55];
    let result = _isysinfo.from_vec(invalid);
    println!("Result: {:?}", result);
    let result = _isysinfo.from_vec(validbuf);
    println!("Result: {:?}", result);
    loop {

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
