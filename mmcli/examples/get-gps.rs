use mmcli::mmcli::IonModemCli;
use std::io;
fn main() -> io::Result<()> {
    let mut modem_cli = IonModemCli::default();
    match modem_cli.is_gps_lock() {
        Ok(true) => {
            println!("GPS Locked");
        }
        Ok(false) => {
            println!("GPS UnLocked");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    Ok(())
}
