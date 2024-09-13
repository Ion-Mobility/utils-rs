use zvariant::SerializeDict;
use zbus::zvariant::Type;

#[derive(Debug, Clone, SerializeDict, Type)]
pub struct SysInfo {
    req: u32,
    wifi_enable: bool,
    lte_enable: bool,
    gps_enable: bool,
    track_enable: bool,

}

impl SysInfo {
    pub fn new() -> Self {
        SysInfo{
            req: 0,
            wifi_enable: true,
            lte_enable: true,
            gps_enable: true,
            track_enable: true
        }
    }
}