extern crate chrono;
use log::{error, info, warn};
use socketcan::{CanFilter, Frame};  use core::f32;
//NOTE: Adatped to socketcan="3.3.0"
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use socketcan::{CanSocket, EmbeddedFrame, Socket};
use canparse::pgn::{ParseMessage, PgnLibrary};
use std::path::Path;

#[derive(Debug)]
pub struct CanUtils {
    hash_msg: HashMap<String, u32>,
    canport: String,
    socket_can: CanSocket,
    can_info: PgnLibrary,
    id_and_signal: HashMap<u32, Vec<String>>,
    can_filter_names: Vec<CanFilter>,
}

impl CanUtils {
    pub fn new<P: AsRef<Path>>(dbcpath: P, canport: &str, canfilter_names: &[&str]) -> Result<Self, Box<dyn Error>> {
        let dbcpath_str = match dbcpath.as_ref().to_str() {
            Some(s) => s.to_string(),
            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid DBC path"))),
        };

        let mut hash_msg = HashMap::new();
        let can_info = match PgnLibrary::from_dbc_file(&dbcpath_str) {
            Ok(info) => info,
            Err(e) => return Err(Box::new(e)),
        };

        let id_and_signal = can_info
            .hash_of_canid_signals()
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(String::from).collect()))
            .collect::<HashMap<u32, Vec<String>>>();

        // Read the DBC file and populate hash_msg
        let file = match File::open(&dbcpath) {
            Ok(f) => f,
            Err(e) => return Err(Box::new(e)),
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if l.starts_with("BO_ ") {
                        let parts: Vec<&str> = l.split_whitespace().collect();
                        if parts.len() > 2 {
                            if let Ok(id) = parts[1].parse::<u32>() {
                                let name = parts[2].trim_end_matches(':').to_string();
                                hash_msg.insert(name, id);
                            }
                        }
                    }
                },
                Err(e) => return Err(Box::new(e)),
            }
        }

        let socket_can = match CanSocket::open(canport) {
            Ok(s) => s,
            Err(e) => return Err(Box::new(e)),
        };

        let mut can_filters = Vec::new();
        for &name in canfilter_names {
            if let Some(&id) = hash_msg.get(name) {
                let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
                let filter = CanFilter::new(extended_id, 0x1FFFFFFF);
                info!(
                    "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ----> Extended ID: {:#X}",
                    name, id, id, extended_id
                );
                can_filters.push(filter);
            } else {
                error!("Error: CAN name '{}' not found in the database", name);
            }
        }

        if !can_filters.is_empty() {
            if let Err(e) = socket_can.set_filters(can_filters.as_slice()) {
                return Err(Box::new(e));
            }
        }

        Ok(CanUtils {
            hash_msg,
            canport: canport.to_string(),
            socket_can,
            can_info,
            id_and_signal,
            can_filter_names: can_filters
        })
    }

    pub fn get_can_ids_from_can_names(&self, can_names: &[&str]) -> Vec<u32> {
        let mut can_ids = Vec::new();
        for &name in can_names {
            if let Some(&id) = self.hash_msg.get(name) {
                let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
                can_ids.push(extended_id);
                info!(
                    "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ---> Extended ID: {:#X}",
                    name, id, id, extended_id
                );
            } else {
                error!("Error: CAN name '{}' not found in the database", name);
            }
        }

        can_ids
    }

    pub fn get_can_id_from_can_name(&self, can_name: String) -> u32 {
        let can_name_str = can_name.as_str();
        if let Some(&id) = self.hash_msg.get(can_name_str) {
            let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
            info!(
                "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ----> Extended ID: {:#X}",
                can_name, id, id, extended_id
            );
            extended_id // Return the extended ID
        } else {
            error!("Error: CAN name '{}' not found in the database", can_name);
            0 // Return 0 if CAN name not found
        }
    }

    pub fn set_can_filters_from_can_names(&self, can_names: &[&str]) {
        let mut can_filters = Vec::new();
        for &name in can_names {
            if let Some(&id) = self.hash_msg.get(name) {
                let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
                let filter = CanFilter::new(extended_id, 0x1FFFFFFF);
                info!(
                    "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ----> Extended ID: {:#X}",
                    name, id, id, extended_id
                );
                can_filters.push(filter);
            } else {
                error!("Error: CAN name '{}' not found in the database", name);
            }
        }

        if !can_filters.is_empty() {
            if let Err(e) = self.socket_can.set_filters(can_filters.as_slice()) {
                error!("Failed to set CAN filters: {:?}", e);
            }
        }
    }

    pub fn get_signals(&mut self) -> Result<HashMap<String, f32>, Box<dyn Error>> {
        let mut result = HashMap::new();

        match self.socket_can.read_frame() {
            Ok(frame) => {
                let can_id: u32 = frame.id_word();
                log::trace!("Got message CAN ID: {:X}", can_id);
                let mut can_padded_msg = [0u8; 8];

                if let Some(can_msg) = self.id_and_signal.get(&can_id) {
                    for signal in can_msg {
                        if let Some(signal_info) = self.can_info.get_spn(signal) {
                            let can_msg_data = {
                                if frame.data().len() < 8 {
                                    can_padded_msg[..frame.data().len()].copy_from_slice(frame.data());
                                    &can_padded_msg
                                } else {
                                    frame.data()
                                }
                            };

                            if let Some(value) = signal_info.parse_message(can_msg_data) {
                                result.insert(signal.to_string(), value);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read CAN frame: {:?}", e);
                log::info!("Attempting to reopen CanSocket...");

                // Attempt to reopen the CanSocket
                match CanSocket::open(&self.canport) {
                    Ok(new_socket) => {
                        self.socket_can = new_socket;
                        log::info!("Successfully reopened CanSocket.");
                        if !self.can_filter_names.is_empty() {
                            if let Err(e) = self.socket_can.set_filters(self.can_filter_names.as_slice()) {
                                return Err(Box::new(e));
                            }
                        }
                    }
                    Err(reopen_error) => {
                        log::error!("Failed to reopen CanSocket: {:?}", reopen_error);
                        return Err(Box::new(reopen_error));
                    }
                }
            }
        }

        Ok(result)
    }
}