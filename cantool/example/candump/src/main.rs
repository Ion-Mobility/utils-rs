use cantool::can_tool::*;
use std::path::Path;
use logging::logging::MyLogging;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // init logger
    let console_log = MyLogging::default();
    console_log.init_logger();
    if let Ok(mut _cantool) = CanUtils::new("can0", None, vec![]).await {
        // Start a task to handle incoming signals
        let mut signals_handle = tokio::spawn(async move {
            while let frame_result = _cantool.get_signals().await {
                println!("Result: {:?}", frame_result);
            }
            println!("Signal stream ended.");
        });

        // Wait for the signal handling task to finish
        if let Err(e) = signals_handle.await {
            eprintln!("Signal handling task failed: {}", e);
        }
    }

    // Keeping the main function alive
    loop {
        let _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
