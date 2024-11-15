use zvariant::Type;
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
pub struct WifiInfo {
    pub ssid: String,
    pub mac: [u8; 6],
    pub signal: f32,
    pub ipv4: [u8; 4],
    pub ipv6: [u8; 8],
    pub sec: u8, // Security level
    pub internetable: bool,
}

impl WifiInfo {
    pub fn new() -> Self {
        WifiInfo {
            ssid: String::new(),
            mac: [0u8; 6],
            signal: 0.0,
            ipv4: [0u8; 4],
            ipv6: [0u8; 8],
            sec: 0, // Initialize security level
            internetable: false,
        }
    }
}

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
pub struct LteInfo {
    pub ops: String,
    pub ipv4: [u8; 4],
    pub ipv6: [u8; 8],
    pub internetable: bool,
    pub signal: f32,
    pub gpslocked: bool,
    pub timezone: [u8; 20]
}

impl LteInfo {
    pub fn new() -> Self {
        LteInfo {
            ops: String::new(),
            ipv4: [0u8; 4],
            ipv6: [0u8; 8],
            internetable: false,
            signal: 0.0,
            gpslocked: false,
            timezone: [0u8; 20]
        }
    }
}

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
pub struct SysInfo {
    pub req: u32,
    pub wifi_enable: u8,
    pub lte_enable: u8,
    pub gps_enable: u8,
    pub track_enable: u8,
    pub bike_state: u8,
    pub bike_locked: u8,
    pub bike_cmd: u8,
    pub reversed: [u32; 8],
    pub wifi_info: WifiInfo,
    pub lte_info: LteInfo,
}

impl SysInfo {
    pub fn new() -> Self {
        SysInfo {
            req: 0,
            wifi_enable: 1,
            lte_enable: 1,
            gps_enable: 1,
            track_enable: 1,
            bike_state: 0,
            bike_locked: 1,
            bike_cmd: 0,
            reversed: [0u32; 8],
            wifi_info: WifiInfo::new(),
            lte_info: LteInfo::new(),
        }
    }

    pub fn get_wifi_cfg(&self) -> u8 {
        self.wifi_enable
    }

    pub fn get_lte_cfg(&self) -> u8 {
        self.lte_enable
    }

    pub fn get_gps_cfg(&self) -> u8 {
        self.gps_enable
    }

    pub fn set_wifi_cfg(&mut self, val: f32) {
        self.wifi_enable = if val != 0.0 { 1 } else { 0 };
    }

    pub fn set_lte_cfg(&mut self, val: f32) {
        self.lte_enable = if val != 0.0 { 1 } else { 0 };
    }

    pub fn set_gps_cfg(&mut self, val: f32) {
        self.gps_enable = if val != 0.0 { 1 } else { 0 };
    }

    pub fn update_lte_info(&mut self, new_info: LteInfo) {
        self.lte_info = new_info;
    }

    pub fn update_wifi_info(&mut self, new_info: WifiInfo) {
        self.wifi_info = new_info;
    }

    pub fn get_wifi_info(&self) -> WifiInfo {
        self.wifi_info.clone()
    }

    pub fn get_lte_info(&self) -> LteInfo {
        self.lte_info.clone()
    }

    pub fn is_wifi_internet_access(&self) -> bool {
        self.wifi_info.internetable
    }

    pub fn is_lte_internet_access(&self) -> bool {
        self.lte_info.internetable
    }
    
}
