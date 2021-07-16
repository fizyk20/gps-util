use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GnssId {
    Gps = 0,
    Sbas = 1,
    Galileo = 2,
    BeiDou = 3,
    Imes = 4,
    Qzss = 5,
    Glonass = 6,
}

impl TryFrom<u8> for GnssId {
    type Error = String;

    fn try_from(val: u8) -> Result<GnssId, String> {
        match val {
            0 => Ok(GnssId::Gps),
            1 => Ok(GnssId::Sbas),
            2 => Ok(GnssId::Galileo),
            3 => Ok(GnssId::BeiDou),
            4 => Ok(GnssId::Imes),
            5 => Ok(GnssId::Qzss),
            6 => Ok(GnssId::Glonass),
            x => Err(format!("invalid GNSS ID: {}", x)),
        }
    }
}
