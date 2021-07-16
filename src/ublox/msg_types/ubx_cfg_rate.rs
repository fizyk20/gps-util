use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UbxCfgRateTimeRef {
    Utc = 0,
    Gps = 1,
    Glonass = 2,
    BeiDou = 3,
    Galileo = 4,
}

impl TryFrom<u16> for UbxCfgRateTimeRef {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, String> {
        match val {
            0 => Ok(UbxCfgRateTimeRef::Utc),
            1 => Ok(UbxCfgRateTimeRef::Gps),
            2 => Ok(UbxCfgRateTimeRef::Glonass),
            3 => Ok(UbxCfgRateTimeRef::BeiDou),
            4 => Ok(UbxCfgRateTimeRef::Galileo),
            x => Err(format!("unexpected value for UbxCfgRateTimeRef: {}", x)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UbxCfgRate {
    pub meas_rate_ms: u16,
    pub nav_rate_cycles: u16,
    pub time_ref: UbxCfgRateTimeRef,
}

impl From<UbxCfgRate> for Vec<u8> {
    fn from(msg: UbxCfgRate) -> Vec<u8> {
        let [mr0, mr1] = msg.meas_rate_ms.to_le_bytes();
        let [nr0, nr1] = msg.nav_rate_cycles.to_le_bytes();
        let [tr0, tr1] = (msg.time_ref as u16).to_le_bytes();
        vec![mr0, mr1, nr0, nr1, tr0, tr1]
    }
}

impl TryFrom<Vec<u8>> for UbxCfgRate {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        match bytes.len() {
            6 => {
                let meas_rate_ms = u16::from_le_bytes([bytes[0], bytes[1]]);
                let nav_rate_cycles = u16::from_le_bytes([bytes[2], bytes[3]]);
                let time_ref_u16 = u16::from_le_bytes([bytes[4], bytes[5]]);
                let time_ref = UbxCfgRateTimeRef::try_from(time_ref_u16)?;
                Ok(UbxCfgRate {
                    meas_rate_ms,
                    nav_rate_cycles,
                    time_ref,
                })
            }
            x => Err(format!("unexpected len for a UbxCfgRate: {}", x)),
        }
    }
}
