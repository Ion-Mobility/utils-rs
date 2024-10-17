use mmcli::mmcli::IonModemCli;
use std::io;
fn main() -> io::Result<()> {
    let mut modem_cli = IonModemCli::default();
    if let Ok(_imei) = modem_cli.get_imei() {
        println!("Imei: {}", _imei);
    }
    if let Ok(_ops) = modem_cli.get_operator_name() {
        println!("Ops: {}", _ops);
    }
    Ok(())
}
