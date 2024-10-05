use tokio::{io, time};
use tokio::time::Duration;
use log::{debug, error, info, warn, trace};
use tokio::process::Command;
use logging::logging::*;
use mmcli::mmcli::IonModemCli; // Assuming IonModemCli is imported from mmcli crate

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize modem client
    let mut modem_cli = IonModemCli::default();

    // init logger
    let console_log = MyLogging::default();
    console_log.init_logger();

    loop {
        // info!("LTE Modem Enabled, try to setup Network");
        if let Ok(_imei) = modem_cli.get_imei() {
            info!("Imei: {}", _imei);
        }
        if let Ok(_ops) = modem_cli.get_operator_name() {
            info!("Ops: {}", _ops);
        }
        // if modem_cli.waiting_for_ready() == false {
        //     info!("waiting for LTE Modem Probed");
        // } else {
        //     if modem_cli.is_modem_enabled() == false {
        //         info!("LTE Modem hasn't enabled, try to enable it!");
        //         if let Err(err) = modem_cli.setup_modem_enable(true) {
        //             error!("Failed to perform enable lte modem: {:?}", err);
        //         }
        //     } else {

        //     }
        // }
        // Perform other non-blocking tasks or simply loop
        let _ = time::sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}
