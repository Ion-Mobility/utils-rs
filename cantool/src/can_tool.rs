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
    can_socket: CANSocket,
}

impl CanUtils {
    const DEFAULT_DBC_PATH: &'static str = "/usr/share/can-dbcs/consolidated.dbc";

    /// Creates a new CanUtils instance asynchronously
    pub async fn new(
        ifname: &str, 
        dbc_path: Option<&Path>, 
        ids_filter: Vec<u32>
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {  // Add Send + Sync
        let dbc_path = dbc_path.unwrap_or_else(|| Path::new(Self::DEFAULT_DBC_PATH));

        loop {
            if !dbc_path.exists() {
                warn!("DBC file not found, retrying in 1 second...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            match PgnLibrary::from_dbc_file(dbc_path) {
                Ok(can_info) => {
                    let socket_can = match CANSocket::open(ifname) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to open CAN socket on {}: {}. Retrying...", ifname, e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    let filters: Vec<CANFilter> = ids_filter
                        .into_iter()
                        .map(|id| CANFilter::new(id, 0x1FFFFFFF)) // 0x1FFFFFFF for full mask
                        .collect::<Result<Vec<CANFilter>, _>>()?;

                    // Set filters if available
                    if !filters.is_empty() {
                        if let Err(e) = socket_can.set_filter(&filters) {
                            error!("Failed to set CAN filters: {}", e);
                            return Err(Box::new(e));
                        }
                    }

                    let id_and_signal = can_info
                        .hash_of_canid_signals()
                        .into_iter()
                        .map(|(k, v)| (k, v.into_iter().map(String::from).collect()))
                        .collect::<HashMap<u32, Vec<String>>>();

                    return Ok(CanUtils {
                        canport: ifname.to_string(),
                        filters,
                        can_info,
                        id_and_signal,
                        can_socket: socket_can,
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

    /// Asynchronously fetches signals from CAN frames
    pub async fn get_signals(&mut self) -> Result<HashMap<String, f32>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result: HashMap<String, f32> = HashMap::new();

        // Asynchronously receive the next CAN frame
        if let Some(Ok(frame)) = self.can_socket.next().await {
            let frame_id = frame.id() | 0x80000000;

            if let Some(signals) = self.id_and_signal.get(&frame_id) {
                for signal in signals {
                    if let Some(signal_info) = self.can_info.get_spn(signal) {
                        let mut can_padded_msg = [0u8; 8];
                        can_padded_msg[..frame.data().len()].copy_from_slice(&frame.data());

                        if let Some(value) = signal_info.parse_message(&can_padded_msg) {
                            result.insert(signal.clone(), value);
                        } else {
                            error!("Failed to parse message for signal: {}", signal);
                            return Err("Failed to parse message, please check the DBC file.".into());
                        }
                    } else {
                        error!("Signal not found in DBC: {}", signal);
                        return Err("Signal not found in DBC.".into());
                    }
                }
            } else {
                error!("Message ID {:x} not found in DBC", frame.id());
                return Err("Message ID not found in DBC.".into());
            }
        } else {
            error!("Failed to receive CAN frame");
            return Err("Failed to receive CAN frame.".into());
        }

        Ok(result)
    }
}
