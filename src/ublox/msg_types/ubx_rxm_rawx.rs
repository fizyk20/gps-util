use std::convert::{TryFrom, TryInto};

use bitflags::bitflags;

bitflags! {
    pub struct UbxRxmRawxRecvStatus: u8 {
        const LEAP_SEC = 0x01;
        const CLK_RESET = 0x02;
    }
}

bitflags! {
    pub struct UbxRxmRawxMeasurementTrkStatus: u8 {
        const PR_VALID = 0x01;
        const CP_VALID = 0x02;
        const HALF_CYC = 0x04;
        const SUB_HALF_CYC = 0x08;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UbxRxmRawxMeasurement {
    pub pseudorange: f64,
    pub carrier_phase: f64,
    pub doppler: f32,
    pub gnss_id: u8,
    pub sv_id: u8,
    pub freq_id: u8,
    pub locktime: u16,
    pub cno: u8,
    pub pseudorange_stdev: f32,
    pub carrier_phase_stdev: Option<f32>,
    pub doppler_stdev: f32,
    pub trk_status: UbxRxmRawxMeasurementTrkStatus,
}

impl From<UbxRxmRawxMeasurement> for Vec<u8> {
    fn from(measurement: UbxRxmRawxMeasurement) -> Vec<u8> {
        let mut result = vec![];
        result.extend(&measurement.pseudorange.to_le_bytes()[..]);
        result.extend(&measurement.carrier_phase.to_le_bytes()[..]);
        result.extend(&measurement.doppler.to_le_bytes()[..]);
        result.push(measurement.gnss_id);
        result.push(measurement.sv_id);
        result.push(0);
        result.push(measurement.freq_id);
        result.extend(&measurement.locktime.to_le_bytes()[..]);
        result.push(measurement.cno);
        result.push((measurement.pseudorange_stdev / 0.01) as u8);
        result.push(match measurement.carrier_phase_stdev {
            None => 15,
            Some(x) => (x / 0.004) as u8,
        });
        result.push((measurement.doppler_stdev / 0.002) as u8);
        result.push(measurement.trk_status.bits());
        result.push(0);
        result
    }
}

impl TryFrom<Vec<u8>> for UbxRxmRawxMeasurement {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<UbxRxmRawxMeasurement, String> {
        if bytes.len() != 32 {
            return Err(format!(
                "wrong length for UbxRxmRawxMeasurement: expected 32, got {}",
                bytes.len()
            ));
        }

        let pseudorange =
            f64::from_le_bytes(bytes[0..8].try_into().map_err(|err| format!("{}", err))?);
        let carrier_phase =
            f64::from_le_bytes(bytes[8..16].try_into().map_err(|err| format!("{}", err))?);
        let doppler =
            f32::from_le_bytes(bytes[16..20].try_into().map_err(|err| format!("{}", err))?);
        let gnss_id = bytes[20];
        let sv_id = bytes[21];
        let freq_id = bytes[23];
        let locktime =
            u16::from_le_bytes(bytes[24..26].try_into().map_err(|err| format!("{}", err))?);
        let cno = bytes[26];

        let pseudorange_stdev = match bytes[27] {
            x if x < 16 => 2.0f32.powi(x as i32) * 0.01,
            x => {
                return Err(format!("invalid pseudorange stdev: {}", x));
            }
        };
        let carrier_phase_stdev = match bytes[28] {
            x if x < 15 => Some(x as f32 * 0.004),
            15 => None,
            x => {
                return Err(format!("invalid carrier phase stdev: {}", x));
            }
        };
        let doppler_stdev = match bytes[29] {
            x if x < 16 => 2.0f32.powi(x as i32) * 0.002,
            x => {
                return Err(format!("invalid Doppler stdev: {}", x));
            }
        };

        let trk_status = UbxRxmRawxMeasurementTrkStatus::from_bits(bytes[30])
            .ok_or_else(|| format!("invalid UbxRxmRawxMeasurementTrkStatus: {}", bytes[30]))?;

        Ok(UbxRxmRawxMeasurement {
            pseudorange,
            carrier_phase,
            doppler,
            gnss_id,
            sv_id,
            freq_id,
            locktime,
            cno,
            pseudorange_stdev,
            carrier_phase_stdev,
            doppler_stdev,
            trk_status,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UbxRxmRawx {
    pub rcv_tow: f64,
    pub week: u16,
    pub leap_sec: i8,
    pub recv_status: UbxRxmRawxRecvStatus,
    pub measurements: Vec<UbxRxmRawxMeasurement>,
}

impl From<UbxRxmRawx> for Vec<u8> {
    fn from(msg: UbxRxmRawx) -> Vec<u8> {
        let mut result = vec![];
        result.extend(&msg.rcv_tow.to_le_bytes()[..]);
        result.extend(&msg.week.to_le_bytes()[..]);
        result.push(msg.leap_sec as u8);
        result.push(msg.measurements.len() as u8);
        result.push(msg.recv_status.bits());
        result.extend(&[0, 0, 0][..]);
        for measurement in msg.measurements {
            result.extend(Vec::<u8>::from(measurement));
        }
        result
    }
}

impl TryFrom<Vec<u8>> for UbxRxmRawx {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err(format!(
                "UbxRxmRawx too short - expected min. 16 bytes, got {}",
                bytes.len()
            ));
        }

        if (bytes.len() - 16) % 32 != 0 {
            return Err(format!(
                "uneven number of bytes for UbxRxmRawx - got {}",
                bytes.len()
            ));
        }

        let length = bytes[11] as usize;
        if bytes.len() != 16 + 32 * length {
            return Err(format!(
                "wrong message length for UbxRxmRawx - expected {}, got {}",
                16 + 32 * length,
                bytes.len()
            ));
        }

        let rcv_tow = f64::from_le_bytes(bytes[0..8].try_into().map_err(|err| format!("{}", err))?);
        let week = u16::from_le_bytes(bytes[8..10].try_into().map_err(|err| format!("{}", err))?);
        let leap_sec = bytes[10] as i8;
        let recv_status = UbxRxmRawxRecvStatus::from_bits(bytes[12])
            .ok_or_else(|| format!("wrong UbxRxmRawxRecvStatus: {}", bytes[12]))?;

        let mut measurements = vec![];
        for i in 0..length {
            let measurement = bytes[16 + 32 * i..16 + 32 * (i + 1)].to_vec().try_into()?;
            measurements.push(measurement);
        }

        Ok(UbxRxmRawx {
            rcv_tow,
            week,
            leap_sec,
            recv_status,
            measurements,
        })
    }
}
