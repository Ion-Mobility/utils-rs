use zvariant::SerializeDict;
use zbus::zvariant::Type;

#[derive(Debug, Clone, SerializeDict, Type)]
pub struct SysInfo {
    req: u32,
    wifi_enable: u8,
    lte_enable: u8,
    gps_enable: u8,
    track_enable: u8,
    wifi_info: WifiInfo,
    lte_info: LteInfo
}

#[derive(Debug, Clone, SerializeDict, Type)]
pub struct WifiInfo {
    pub ssid: String,
    pub mac: [u8; 6],
    pub signal: u8,
    pub ipv4: [u8; 4],
    pub ipv6: [u8; 8]
}

impl WifiInfo {
    pub fn new() -> Self {
        WifiInfo{
            ssid: String::new(),
            mac: [0u8; 6],
            signal: 0,
            ipv4: [0u8; 4],
            ipv6: [0u8; 8]
        }
    }
}

#[derive(Debug, Clone, SerializeDict, Type)]
pub struct LteInfo {
    ops: String,
    ipv4: [u8;4],
    ipv6: [u8;8]
}

impl LteInfo {
    pub fn new() -> Self {
        LteInfo{
            ops: String::new(),
            ipv4: [0u8; 4],
            ipv6: [0u8; 8]
        }
    }
}

impl SysInfo {
    pub fn new() -> Self {
        SysInfo{
            req: 0,
            wifi_enable: 0,
            lte_enable: 0,
            gps_enable: 0,
            track_enable: 0,
            wifi_info: WifiInfo::new(),
            lte_info: LteInfo::new()
        }
    }

    pub fn get_wifi_cfg(&self) -> u8 {
        return self.wifi_enable;
    }
    pub fn get_lte_cfg(&self) -> u8 {
        return self.wifi_enable;
    }
    pub fn get_gps_cfg(&self) -> u8 {
        return self.gps_enable;
    }
    pub fn set_wifi_cfg(&mut self, val: f32) {
        if val != 0.0 {
            self.wifi_enable = 1;
        } else {
            self.wifi_enable = 0;
        }
    }
    pub fn set_lte_cfg(&mut self, val: f32) {
        if val != 0.0 {
            self.lte_enable = 1;
        } else {
            self.lte_enable = 0;
        }
    }
    pub fn set_gps_cfg(&mut self, val: f32) {
        if val != 0.0 {
            self.gps_enable = 1;
        } else {
            self.gps_enable = 0;
        }
    }
    pub fn update_wifi_info(&mut self, _new_info: WifiInfo) {
        self.wifi_info.ssid = _new_info.ssid;
        self.wifi_info.ipv4 = _new_info.ipv4;
        self.wifi_info.signal = _new_info.signal;
        self.wifi_info.mac = _new_info.mac;
    }
}