use zvariant::Type;
use zbus::zvariant::{SerializeDict, DeserializeDict, Value, Structure};
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, SerializeDict, DeserializeDict, Type)]
pub struct WifiInfo {
    pub ssid: String,
    pub mac: [u8; 6],
    pub signal: u8,
    pub ipv4: [u8; 4],
    pub ipv6: [u8; 8],
    pub sec: u8, // Security level
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
        }
    }

    pub fn from_vec(bytes: &[u8]) -> Self {
        let ssid_len = bytes[0] as usize;
        let ssid = String::from_utf8_lossy(&bytes[1..1 + ssid_len]).to_string();
        let mac = <[u8; 6]>::try_from(&bytes[1 + ssid_len..7 + ssid_len]).unwrap();
        let signal = bytes[7 + ssid_len];
        let ipv4 = <[u8; 4]>::try_from(&bytes[8 + ssid_len..12 + ssid_len]).unwrap();
        let ipv6 = <[u8; 8]>::try_from(&bytes[12 + ssid_len..20 + ssid_len]).unwrap();
        let sec = bytes[20 + ssid_len];  // Added sec

        WifiInfo {
            ssid,
            mac,
            signal,
            ipv4,
            ipv6,
            sec,  // Assign sec value
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.ssid.len() as u8); // Length of SSID
        bytes.extend(self.ssid.as_bytes());
        bytes.extend(&self.mac);
        bytes.push(self.signal);
        bytes.extend(&self.ipv4);
        bytes.extend(&self.ipv6);
        bytes.push(self.sec);  // Append sec value
        bytes
    }
}

#[derive(Debug, Clone, SerializeDict, DeserializeDict, Type)]
pub struct LteInfo {
    pub ops: String,
    pub ipv4: [u8; 4],
    pub ipv6: [u8; 8],
}

impl LteInfo {
    pub fn new() -> Self {
        LteInfo {
            ops: String::new(),
            ipv4: [0u8; 4],
            ipv6: [0u8; 8],
        }
    }

    pub fn from_vec(bytes: &[u8]) -> Self {
        let ops_len = bytes[0] as usize;
        let ops = String::from_utf8_lossy(&bytes[1..1 + ops_len]).to_string();
        let ipv4 = <[u8; 4]>::try_from(&bytes[1 + ops_len..5 + ops_len]).unwrap();
        let ipv6 = <[u8; 8]>::try_from(&bytes[5 + ops_len..13 + ops_len]).unwrap();

        LteInfo { ops, ipv4, ipv6 }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.ops.len() as u8); // Length of operator string
        bytes.extend(self.ops.as_bytes());
        bytes.extend(&self.ipv4);
        bytes.extend(&self.ipv6);
        bytes
    }
}

#[derive(Debug, Clone, SerializeDict, DeserializeDict, Type)]
pub struct SysInfo {
    req: u32,
    wifi_enable: u8,
    lte_enable: u8,
    gps_enable: u8,
    track_enable: u8,
    wifi_info: WifiInfo,
    lte_info: LteInfo,
}

impl SysInfo {
    pub fn new() -> Self {
        SysInfo {
            req: 0,
            wifi_enable: 0,
            lte_enable: 0,
            gps_enable: 0,
            track_enable: 0,
            wifi_info: WifiInfo::new(),
            lte_info: LteInfo::new(),
        }
    }

    pub fn from_vec(bytes: &[u8]) -> Self {
        let mut req = [0u8; 4];
        let mut wifi_enable = [0u8; 1];
        let mut lte_enable = [0u8; 1];
        let mut gps_enable = [0u8; 1];
        let mut track_enable = [0u8; 1];

        req.copy_from_slice(&bytes[0..4]);
        wifi_enable.copy_from_slice(&bytes[4..5]);
        lte_enable.copy_from_slice(&bytes[5..6]);
        gps_enable.copy_from_slice(&bytes[6..7]);
        track_enable.copy_from_slice(&bytes[7..8]);

        let wifi_info = WifiInfo::from_vec(&bytes[8..]);
        let lte_info = LteInfo::from_vec(&bytes[8 + wifi_info.to_vec().len()..]);

        SysInfo {
            req: LittleEndian::read_u32(&req),
            wifi_enable: wifi_enable[0],
            lte_enable: lte_enable[0],
            gps_enable: gps_enable[0],
            track_enable: track_enable[0],
            wifi_info,
            lte_info,
        }
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
}
