extern crate chrono;
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::Path;
use canparse::pgn::{ParseMessage, PgnLibrary};
use tokio::time::Duration;
use tokio_socketcan::{CANFilter, CANSocket, CANFrame};
use futures_util::{stream::StreamExt, TryStreamExt};
use std::io::Error;


#[derive(Debug)]
pub struct CanUtils {
    canport: String,
    filters: Vec<CANFilter>,
    can_info: PgnLibrary,
    id_and_signal: HashMap<u32, Vec<String>>,
    can_socket: CANSocket
}

impl CanUtils {
    const DEFAULT_DBC_PATH: &'static str = "/usr/share/can-dbcs/consolidated.dbc";

    pub async fn new(ifname: &str, dbc_path: Option<&Path>, ids_filter: Vec<u32>) -> Result<Self, Box<dyn std::error::Error>> {
        // Default path for DBC file
        let dbc_path = dbc_path.unwrap_or_else(|| Path::new(Self::DEFAULT_DBC_PATH));

        loop {
            if !dbc_path.exists() {
                warn!("DBC file not found, retrying in 1 second...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            // Try to load the DBC file into PgnLibrary
            match PgnLibrary::from_dbc_file(dbc_path) {
                Ok(can_info) => {
                    // Open the CAN socket asynchronously
                    let socket_can = match CANSocket::open(ifname) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to open CAN socket on {}: {}. Retrying...", ifname, e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    // Create CAN filters from the provided ID list
                    let filters: Vec<CANFilter> = ids_filter
                        .into_iter()
                        .map(|id| CANFilter::new(id, 0x1FFFFFFF)) // Exact ID match filter (0x7FF is full mask)
                        .collect::<Result<Vec<CANFilter>, _>>()?; // Handle Result<_, ConstructionError>

                    // Apply the filters to the CAN socket if there are any filters
                    if !filters.is_empty() {
                        match socket_can.set_filter(&filters) {
                            Ok(_) => {

                            }
                            Err(e) => {
                                eprintln!("Can't set fillter {}", e);
                            }
                        }
                    }
                    // Extract CAN signals from the PgnLibrary
                    let id_and_signal = can_info
                        .hash_of_canid_signals()
                        .into_iter()
                        .map(|(k, v)| (k, v.into_iter().map(String::from).collect()))
                        .collect::<HashMap<u32, Vec<String>>>();

                    // Return the CanUtils struct
                    return Ok(CanUtils {
                        canport: ifname.to_string(),
                        filters,
                        can_info,
                        id_and_signal,
                        can_socket: socket_can
                    });
                }
                Err(e) => {
                    error!("Failed to load DBC file {}: {}. Retrying...", dbc_path.display(), e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }

    // The updated get_signals method that returns frames
    pub async fn get_signals(&mut self) -> Result<HashMap<String, f32>, Box<dyn std::error::Error>> {
        let mut _result: HashMap<String, f32> = HashMap::new();
        if let Some(Ok(_frame)) = self.can_socket.next().await {
            if let Some(signals) = self.id_and_signal.get(&(&_frame.id() | 0x80000000)) {
                for signal in signals {
                    if let Some(signal_info) = self.can_info.get_spn(signal) {
                        let mut can_padded_msg = [0u8; 8];
                        can_padded_msg[.._frame.data().len()].copy_from_slice(&_frame.data());
                        if let Some(value) = signal_info.parse_message(&can_padded_msg) {
                            _result.insert((&signal).to_string(), value);
                        }
                    } else {
                        error!("Can't parse signal for {}", _frame.id());
                        return Err("Can't parse signal, please re-check dbc file".into());
                    }
                }
            } else {
                error!("Not found msgid {} in dbc file", _frame.id());
                return Err("Can't parse income msg, please re-check filtter and dbc file".into());
            }
        }
        Ok(_result)
    }
}
