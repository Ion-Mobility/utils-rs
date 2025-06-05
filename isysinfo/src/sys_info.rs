use zvariant::Type;
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
pub struct LocationData {
    pub lat_deg: f64,
    pub lng_deg: f64,
    pub altitude: f64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
pub struct MovementData {
    pub speed: f64,
    pub heading_deg: f32,
    pub bearing: f64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
pub struct AccuracyData {
    pub accuracy: f64,
    pub epx_m: f64,
    pub epy_m: f64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Type, PartialEq, Debug, Clone)]
// Original struct is now split
pub struct GpsInfo {
    pub time: i64,
    pub fixmode: u8,
    pub location: LocationData,
    pub movement: MovementData,
    pub accuracy: AccuracyData,
}

impl GpsInfo {
    pub fn new() -> Self {
        GpsInfo {
            time: 0,
            fixmode: 0,
            location: LocationData {
                lat_deg: 0.0,
                lng_deg: 0.0,
                altitude: 0.0
            },
            movement: MovementData {
                speed: 0.0,
                heading_deg: 0.0,
                bearing: 0.0
            },
            accuracy: AccuracyData {
                accuracy: 0.0,
                epx_m: 0.0,
                epy_m: 0.0
            }
        }
    }
}

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
pub struct BikeNotice {
    pub cooling: u8,
    pub falling: u8,
    pub tyre_low: u8,
    pub bms_low: u8,
    pub charge_int: u8,
    pub charge_comp: u8,
    pub keyf_low: u8,
    pub thref_det: u8,
    pub tamp_det: u8,
    pub reversed: u8
}

impl BikeNotice {
    pub fn new() -> Self {
        BikeNotice {
            cooling: 0u8,
            falling: 0u8,
            tyre_low: 0u8,
            bms_low: 0u8,
            charge_int: 0u8,
            charge_comp: 0u8,
            keyf_low: 0u8,
            thref_det: 0u8,
            tamp_det: 0u8,
            reversed: 0u8
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
    pub bike_noti: BikeNotice,
    pub ridemode: u8,
    pub range_km: u32,
    pub soc_pct: u8,
    pub steering: u8,
    pub odo_m: u32,
    pub front_tire: f32,
    pub rear_tire: f32,
    pub wifi_info: WifiInfo,
    pub lte_info: LteInfo,
    pub gps_info: GpsInfo
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
            bike_cmd: BikeNotice::new(),
            ridemode: 0,
            range_km: 0,
            soc_pct: 0,
            steering: 0,
            odo_m: 0,
            front_tire: 0.0,
            rear_tire: 0.0,
            wifi_info: WifiInfo::new(),
            lte_info: LteInfo::new(),
            gps_info: GpsInfo::new()
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

    pub fn get_fronttire_info(&self) -> f32 {
        self.front_tire.clone()
    }
    pub fn set_fronttire_info(&mut self, new_value: f32) {
        self.front_tire = new_value;
    }

    pub fn get_reartire_info(&self) -> f32 {
        self.rear_tire.clone()
    }
    pub fn set_reartire_info(&mut self, new_value: f32) {
        self.rear_tire = new_value;
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
    
    pub fn get_gps_info(&self) -> GpsInfo {
        self.gps_info.clone()
    }
}
