use std::thread;
use std::time::Duration;
use log::{trace, info};
use mmcli::mmcli::IonModemCli; // Assuming IonModemCli is imported from mmcli crate
use cantool::can_tool::CanUtils; // Assuming CanUtils is imported from cantool crate
use logging::logging::MyLogging; // Assuming MyLogging is imported from logging crate

fn main() {
    // Initialize logging
    let console_log = MyLogging::default();
    console_log.init_logger();
    
    // Initialize CAN communication
    let can_filters: Vec<&str> = vec!["vcu_ble_pkt_1"];
    let mut can_conn = match CanUtils::new("/usr/share/can-dbcs/consolidated.dbc", "can0", &can_filters) {
        Ok(conn) => conn,
        Err(e) => {
            info!("Failed to initialize CAN connection: {}", e);
            return;
        }
    };

    // Initialize modem client
    let mut modem_cli = IonModemCli::default();
    trace!("Modem CLI: {:?}", modem_cli);

    // Variables to track vehicle settings
    let mut vehicle_gps_enable = true;
    let mut vehicle_cell_enable = true;

    // Main loop
    loop {
        // Read CAN signals
        match can_conn.get_signals() {
            Ok(signals) => {
                info!("CAN Signal: {:?}", signals);
                for (signal, value) in signals {
                    match signal {
                        "ble_cellular" => {
                            vehicle_cell_enable = value != 0.0;
                            trace!("Cellular: {}", vehicle_cell_enable);
                        }
                        "ble_gps" => {
                            vehicle_gps_enable = value != 0.0;
                            trace!("GPS: {}", vehicle_gps_enable);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                info!("Error reading CAN Signal: {}", e);
            }
        }

        // Check if modem is ready
        // if modem_cli.waiting_for_ready() {
        //     info!("Location Enabled: {}, Modem Enabled: {}, Signal Strength: {}", 
        //         modem_cli.is_location_enabled(), modem_cli.is_modem_enabled(), modem_cli.get_signal_strength());

        //     // Enable modem if not already enabled
        //     if !modem_cli.is_modem_enabled() {
        //         match modem_cli.setup_modem_enable(true) {
        //             Ok(_) => {
        //                 trace!("Modem enabled successfully");
        //             }
        //             Err(e) => {
        //                 info!("Failed to enable modem: {:?}", e);
        //             }
        //         }
        //     }

        //     // Handle GPS based on user settings
        //     if vehicle_gps_enable {
        //         trace!("Enabling GPS based on user setting");
        //         if !modem_cli.is_location_enabled() {
        //             match modem_cli.setup_location(0x07, true) {
        //                 Ok(_) => {
        //                     trace!("Location enabled successfully");
        //                 }
        //                 Err(e) => {
        //                     info!("Failed to enable location: {:?}", e);
        //                 }
        //             }
        //         }
        //     } else {
        //         if modem_cli.is_location_enabled() {
        //             match modem_cli.setup_location(0x03, true) {
        //                 Ok(_) => {
        //                     trace!("Location disabled successfully");
        //                 }
        //                 Err(e) => {
        //                     info!("Failed to disable location: {:?}", e);
        //                 }
        //             }
        //         }
        //     }

        //     // Handle cellular data based on user settings (currently traced only)
        //     if vehicle_cell_enable {
        //         trace!("Enabling Data LTE based on user setting");
        //     }

        // } else {
        //     info!("Modem is not ready");
        // }

        // Sleep for a short duration before looping again
        thread::sleep(Duration::from_millis(100));
    }
}