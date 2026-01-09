extern crate chrono;
use can_dbc::DBC;
use futures_util::{stream::StreamExt, TryStreamExt};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt, time::Duration};
use tokio_socketcan::{CANFilter, CANSocket};

const CAN_RECV_TIMEOUT_S: u64 = 10;

#[derive(Debug)]
pub struct CanUtils {
    canport: String,
    filters: Vec<CANFilter>,
    dbc: DBC,
    can_socket: CANSocket,
}

#[derive(Debug, Clone)]
pub struct RawCanFrame {
    pub id: u32,
    pub data: [u8; 8],
    pub dlc: usize,
}

impl CanUtils {
    const DEFAULT_DBC_PATH: &'static str = "/usr/share/can-dbcs/consolidated.dbc";

    /// Creates a new CanUtils instance asynchronously
    pub async fn new(
        ifname: &str,
        dbc_path: Option<&Path>,
        ids_filter: &Vec<u32>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Add Send + Sync
        let dbc_path = dbc_path.unwrap_or_else(|| Path::new(Self::DEFAULT_DBC_PATH));

        loop {
            if !dbc_path.exists() {
                warn!("DBC file not found, retrying in 1 second...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            match Self::parse_dbc(dbc_path).await {
                Ok(dbc) => {
                    let socket_can = match CANSocket::open(ifname) {
                        Ok(s) => s,
                        Err(e) => {
                            error!(
                                "Failed to open CAN socket on {}: {}. Retrying...",
                                ifname, e
                            );
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    };

                    let filters: Vec<CANFilter> = ids_filter
                        .into_iter()
                        .map(|id| CANFilter::new(*id, 0x1FFFFFFF)) // 0x1FFFFFFF for full mask
                        .collect::<Result<Vec<CANFilter>, _>>()?;

                    // Set filters if available
                    if !filters.is_empty() {
                        if let Err(e) = socket_can.set_filter(&filters) {
                            error!("Failed to set CAN filters: {}", e);
                            return Err(Box::new(e));
                        }
                    }

                    return Ok(CanUtils {
                        canport: ifname.to_string(),
                        filters,
                        dbc,
                        can_socket: socket_can,
                    });
                }
                Err(e) => {
                    error!(
                        "Failed to load DBC file {}: {}. Retrying...",
                        dbc_path.display(),
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }

    async fn parse_dbc(dbc_path: &Path) -> Result<DBC, std::io::Error> {
        let mut f = File::open(dbc_path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;

        let dbc = can_dbc::DBC::from_slice(&buffer).expect("Failed to parse dbc file");

        Ok(dbc)
    }

    /// Restarts the CAN socket
    async fn restart_socket(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            match CANSocket::open(&self.canport) {
                Ok(socket) => {
                    self.can_socket = socket;
                    info!("Successfully restarted CAN socket.");

                    // Reapply filters if necessary
                    if !self.filters.is_empty() {
                        if let Err(e) = self.can_socket.set_filter(&self.filters) {
                            error!("Failed to reapply CAN filters: {}", e);
                            return Err(Box::new(e));
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    error!(
                        "Failed to restart CAN socket: {}. Retrying in 1 second...",
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    pub async fn try_get_raw_frame(
        &mut self,
    ) -> Result<RawCanFrame, Box<dyn std::error::Error + Send + Sync>> {
        match self.can_socket.try_next().await {
            Ok(Some(frame)) => {
                let mut data = [0u8; 8];
                let frame_data = frame.data();
                let dlc = frame_data.len().min(8);

                data[..dlc].copy_from_slice(&frame_data[..dlc]);

                Ok(RawCanFrame {
                    id: frame.id(),
                    data,
                    dlc,
                })
            }
            Ok(None) => Err("No more frames available.".into()),
            Err(e) => {
                error!("Failed to receive CAN frame: {}, sleep a bit", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                Err("Failed to receive CAN frame".into())
            }
        }
    }

    /// Asynchronously fetches signals from CAN frames with socket restart logic and timeout
    pub async fn get_signals(
        &mut self,
    ) -> Result<HashMap<String, f32>, Box<dyn std::error::Error + Send + Sync>> {
        loop {
            // Use the `timeout` function with the resolved duration
            let frame_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(CAN_RECV_TIMEOUT_S),
                self.can_socket.next(),
            )
            .await?;

            match frame_result {
                Some(Ok(frame)) => {
                    let frame_id = frame.id() | 0x80000000;
                    for message in self.dbc.messages() {
                        if frame_id == (message.message_id().raw()) {
                            let padding_data = self.pad_to_8_bytes(frame.data());
                            let signal_data = message.parse_from_can(&padding_data);
                            return Ok(signal_data);
                        }
                    }
                }
                Some(Err(e)) => {
                    error!(
                        "Failed to receive CAN frame: {}. Attempting socket restart...",
                        e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    self.restart_socket().await?; // Restart the socket and retry
                }
                None => {
                    error!("No more frames available from the CAN socket.");
                    return Err("No more frames available.".into());
                }
            }
        }
    }

    /// Asynchronously fetches signals from CAN frames with socket restart logic and timeout
    // pub async fn try_get_signals(
    //     &mut self,
    // ) -> Result<HashMap<String, f32>, Box<dyn std::error::Error + Send + Sync>> {
    //     // Use the `timeout` function with the resolved duration
    //     let frame_result = self.can_socket.try_next().await;
    //     match frame_result {
    //         Ok(Some(frame)) => {
    //             let frame_id = frame.id() | 0x80000000;
    //             for message in self.dbc.messages() {
    //                 if frame_id == (message.message_id().raw()) {
    //                     let padding_data = self.pad_to_8_bytes(frame.data());
    //                     let signal_data = message.parse_from_can(&padding_data);
    //                     return Ok(signal_data);
    //                 }
    //             }
    //             error!("Message ID {:x} not found in DBC", frame.id());
    //             Err("Message ID not found in DBC.".into())
    //         }
    //         Ok(None) => Err("No more frames available.".into()),
    //         Err(_e) => {
    //             error!("Failed to receive CAN frame: {}, sleep a bit", _e);
    //             tokio::time::sleep(Duration::from_secs(1)).await;
    //             Err("Failed to receive CAN frame".into())
    //         }
    //     }
    // }
    pub async fn try_get_signals(
        &mut self,
    ) -> Result<HashMap<String, f32>, Box<dyn std::error::Error + Send + Sync>> {
        let raw = self.try_get_raw_frame().await?;

        let frame_id = raw.id | 0x8000_0000;

        for message in self.dbc.messages() {
            if frame_id == message.message_id().raw() {
                let signal_data = message.parse_from_can(&raw.data);
                return Ok(signal_data);
            }
        }

        error!("Message ID {:x} not found in DBC", raw.id);
        Err("Message ID not found in DBC.".into())
    }
    
    fn pad_to_8_bytes(&self, data: &[u8]) -> Vec<u8> {
        // Convert the byte slice to a Vec<u8>
        let mut padded_data = data.to_vec();

        // Calculate the number of padding bytes needed
        let padding_needed = 8usize.saturating_sub(padded_data.len());

        // Extend the vector with zeros (or another byte) to make it 8 bytes long
        padded_data.extend(std::iter::repeat(0).take(padding_needed));

        // Return the padded vector
        padded_data
    }
}
