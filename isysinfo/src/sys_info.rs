use zvariant::Type;
use zbus::zvariant::{SerializeDict, DeserializeDict};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, SerializeDict, DeserializeDict, Type)]
pub struct WifiInfo {
    pub ssid: String,
    pub mac: [u8; 6],
    pub signal: u8,
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
            signal: 0,
            ipv4: [0u8; 4],
            ipv6: [0u8; 8],
            sec: 0, // Initialize security level
            internetable: false,
        }
    }

    pub fn from_vec(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Input bytes are empty".to_string());
        }

        let ssid_len = bytes[0] as usize;
        let total_len = 1 + ssid_len + 6 + 1 + 4 + 8 + 1 + 1; // Calculate total required size

        if bytes.len() < total_len {
            return Err("Invalid input byte length".to_string());
        }

        let ssid = String::from_utf8_lossy(&bytes[1..1 + ssid_len]).to_string();
        let mac = <[u8; 6]>::try_from(&bytes[1 + ssid_len..7 + ssid_len])
            .map_err(|_| "Failed to parse MAC address".to_string())?;
        let signal = bytes[7 + ssid_len];
        let ipv4 = <[u8; 4]>::try_from(&bytes[8 + ssid_len..12 + ssid_len])
            .map_err(|_| "Failed to parse IPv4 address".to_string())?;
        let ipv6 = <[u8; 8]>::try_from(&bytes[12 + ssid_len..20 + ssid_len])
            .map_err(|_| "Failed to parse IPv6 address".to_string())?;
        let sec = bytes[20 + ssid_len];
        let internetable = bytes[21 + ssid_len] != 0;

        Ok(WifiInfo {
            ssid,
            mac,
            signal,
            ipv4,
            ipv6,
            sec,
            internetable,
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.ssid.len() as u8); // Length of SSID
        bytes.extend(self.ssid.as_bytes());
        bytes.extend(&self.mac);
        bytes.push(self.signal);
        bytes.extend(&self.ipv4);
        bytes.extend(&self.ipv6);
        bytes.push(self.sec);
        bytes.push(self.internetable as u8); // Convert bool to byte
        bytes
    }
}

#[derive(Debug, Clone, SerializeDict, DeserializeDict, Type)]
pub struct LteInfo {
    pub ops: String,
    pub ipv4: [u8; 4],
    pub ipv6: [u8; 8],
    pub internetable: bool,
    pub signal: u8,
}

impl LteInfo {
    pub fn new() -> Self {
        LteInfo {
            ops: String::new(),
            ipv4: [0u8; 4],
            ipv6: [0u8; 8],
            internetable: false,
            signal: 0,
        }
    }

    pub fn from_vec(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Input bytes are empty".to_string());
        }

        let ops_len = bytes[0] as usize;
        let total_len = 1 + ops_len + 4 + 8 + 1 + 1; // Full size including all fields
        if bytes.len() < total_len {
            return Err("Invalid input byte length".to_string());
        }

        let ops = String::from_utf8_lossy(&bytes[1..1 + ops_len]).to_string();
        let ipv4 = <[u8; 4]>::try_from(&bytes[1 + ops_len..5 + ops_len])
            .map_err(|_| "Failed to parse IPv4 address".to_string())?;
        let ipv6 = <[u8; 8]>::try_from(&bytes[5 + ops_len..13 + ops_len])
            .map_err(|_| "Failed to parse IPv6 address".to_string())?;
        let internetable = bytes[13 + ops_len] != 0;
        let signal = bytes[14 + ops_len];

        Ok(LteInfo {
            ops,
            ipv4,
            ipv6,
            internetable,
            signal,
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.ops.len() as u8); // Length of operator string
        bytes.extend(self.ops.as_bytes());
        bytes.extend(&self.ipv4);
        bytes.extend(&self.ipv6);
        bytes.push(self.internetable as u8); // Convert bool to byte
        bytes.push(self.signal); // Add signal value
        bytes
    }
}

// #[derive(Debug, Clone, SerializeDict, DeserializeDict, Type)]
// pub struct SysInfo {
//     req: u32,
//     wifi_enable: u8,
//     lte_enable: u8,
//     gps_enable: u8,
//     track_enable: u8,
//     wifi_info: WifiInfo,
//     lte_info: LteInfo,
// }

#[derive(Debug, Clone)]
pub struct SysInfo {
    pub req: u32,
    pub wifi_enable: u8,
    pub lte_enable: u8,
    pub gps_enable: u8,
    pub track_enable: u8,
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
            wifi_info: WifiInfo::new(),
            lte_info: LteInfo::new(),
        }
    }

    pub fn from_vec(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Input byte slice is too short".to_string());
        }

        let req = LittleEndian::read_u32(&bytes[0..4]);
        let wifi_enable = bytes[4];
        let lte_enable = bytes[5];
        let gps_enable = bytes[6];
        let track_enable = bytes[7];

        let wifi_info_start = 8;
        let wifi_info = WifiInfo::from_vec(&bytes[wifi_info_start..])
            .map_err(|e| format!("Failed to parse WifiInfo: {}", e))?;

        let lte_info_start = wifi_info_start + wifi_info.to_vec().len();
        let lte_info = LteInfo::from_vec(&bytes[lte_info_start..])
            .map_err(|e| format!("Failed to parse LteInfo: {}", e))?;

        Ok(SysInfo {
            req,
            wifi_enable,
            lte_enable,
            gps_enable,
            track_enable,
            wifi_info,
            lte_info,
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.req.to_le_bytes());
        bytes.push(self.wifi_enable);
        bytes.push(self.lte_enable);
        bytes.push(self.gps_enable);
        bytes.push(self.track_enable);
        bytes.extend(self.wifi_info.to_vec());
        bytes.extend(self.lte_info.to_vec());
        bytes
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
