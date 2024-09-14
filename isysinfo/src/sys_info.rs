use zvariant::SerializeDict;
use zbus::zvariant::Type;

#[derive(Debug, Clone, SerializeDict, Type)]
pub struct SysInfo {
    req: u32,
    wifi_enable: u8,
    lte_enable: u8,
    gps_enable: u8,
    track_enable: u8,

}

impl SysInfo {
    pub fn new() -> Self {
        SysInfo{
            req: 0,
            wifi_enable: 0,
            lte_enable: 0,
            gps_enable: 0,
            track_enable: 0
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
}