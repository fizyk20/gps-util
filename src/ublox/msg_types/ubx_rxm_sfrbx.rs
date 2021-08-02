use std::convert::TryFrom;

use super::GnssId;

#[derive(Debug, Clone, PartialEq)]
pub enum GpsSubframe {
    Subframe1 {
        week_number: u16,
        ura_index: u8,
        sv_health: u8,
        tgd: f64,
        iodc: u16,
        toc: u32,
        af2: f64,
        af1: f64,
        af0: f64,
    },
    Subframe2 {
        aodo: u16,
        iode: u8,
        c_rs: f64,
        delta_n: f64,
        m0: f64,
        c_uc: f64,
        e: f64,
        sqrt_a: f64,
        c_us: f64,
        t_oe: u32,
    },
    Subframe3 {
        iode: u8,
        c_ic: f64,
        omega0: f64,
        c_is: f64,
        i0: f64,
        c_rc: f64,
        omega_small: f64,
        omega_dot: f64,
        i_dot: f64,
    },
    Subframe4,
    Subframe5,
}

impl GpsSubframe {
    pub fn iode(&self) -> u8 {
        match *self {
            GpsSubframe::Subframe2 { iode, .. } | GpsSubframe::Subframe3 { iode, .. } => iode,
            _ => panic!("wrong subframe for IODE! {:#?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UbxRxmSfrbxDataGps {
    pub tlm_message: u16,
    pub integrity_bit: bool,
    pub tow: u32,
    pub anti_spoof: bool,
    pub alert: bool,
    pub subframe: GpsSubframe,
}

impl From<UbxRxmSfrbxDataGps> for Vec<u8> {
    fn from(data: UbxRxmSfrbxDataGps) -> Vec<u8> {
        let tlm_word: u32 = ((0x8b0000 | data.tlm_message as u32) << 2
            | if data.integrity_bit { 2 } else { 0 })
            << 6;

        let subframe_id: u32 = match data.subframe {
            GpsSubframe::Subframe1 { .. } => 1,
            GpsSubframe::Subframe2 { .. } => 2,
            GpsSubframe::Subframe3 { .. } => 3,
            GpsSubframe::Subframe4 => 4,
            GpsSubframe::Subframe5 => 5,
        };

        let how_word: u32 = (data.tow << 7
            | subframe_id << 2
            | if data.anti_spoof { 32 } else { 0 }
            | if data.alert { 64 } else { 0 })
            << 6;

        let mut result = vec![];

        result.extend(&tlm_word.to_le_bytes()[..]);
        result.extend(&how_word.to_le_bytes()[..]);

        for _ in 0..32 {
            result.push(0);
        }

        result
    }
}

fn to_i32(val: u32, bits: u8) -> i32 {
    let sign = val & (1 << (bits - 1));
    let mut result = val;
    for i in 1..32 - bits + 1 {
        result |= sign << i;
    }
    result as i32
}

fn to_f64_signed(val: u32, bits: u8, scale_exp: i32) -> f64 {
    to_i32(val, bits) as f64 * 2.0_f64.powi(scale_exp)
}

fn to_f64_unsigned(val: u32, scale_exp: i32) -> f64 {
    val as f64 * 2.0_f64.powi(scale_exp)
}

fn decode_subframe1(words: &[u32]) -> GpsSubframe {
    let week_number = (words[0] >> 14) as u16;
    let ura_index = ((words[0] >> 8) & 15) as u8;
    let sv_health = ((words[0] >> 2) & 63) as u8;
    let iodc = (((words[0] & 3) << 8) | (words[5] >> 16)) as u16;

    let tgd = to_f64_signed(words[4] & 255, 8, -31);
    let toc = (words[5] & 65535) * 16;
    let af0 = to_f64_signed(words[7] >> 2, 22, -31);
    let af1 = to_f64_signed(words[6] & 65535, 16, -43);
    let af2 = to_f64_signed(words[6] >> 16, 8, -55);

    GpsSubframe::Subframe1 {
        week_number,
        ura_index,
        sv_health,
        iodc,
        tgd,
        toc,
        af0,
        af1,
        af2,
    }
}

fn decode_subframe2(words: &[u32]) -> GpsSubframe {
    let iode = (words[0] >> 16) as u8;
    let c_rs = to_f64_signed(words[0] & 65535, 16, -5);
    let delta_n = to_f64_signed(words[1] >> 8, 16, -43);
    let m0 = to_f64_signed(((words[1] & 255) << 24) | words[2], 32, -31);
    let c_uc = to_f64_signed(words[3] >> 8, 16, -29);
    let e = to_f64_unsigned(((words[3] & 255) << 24) | words[4], -33);
    let c_us = to_f64_signed(words[5] >> 8, 16, -29);
    let sqrt_a = to_f64_unsigned(((words[5] & 255) << 24) | words[6], -19);
    let t_oe = (words[7] >> 8) * 16;
    let aodo = (words[7] & 31) as u16 * 900;

    GpsSubframe::Subframe2 {
        iode,
        c_rs,
        delta_n,
        m0,
        c_uc,
        e,
        c_us,
        sqrt_a,
        t_oe,
        aodo,
    }
}

fn decode_subframe3(words: &[u32]) -> GpsSubframe {
    let c_ic = to_f64_signed(words[0] >> 8, 16, -29);
    let omega0 = to_f64_signed(((words[0] & 255) << 24) | words[1], 32, -31);
    let c_is = to_f64_signed(words[2] >> 8, 16, -29);
    let i0 = to_f64_signed(((words[2] & 255) << 24) | words[3], 32, -31);
    let c_rc = to_f64_signed(words[4] >> 8, 16, -5);
    let omega_small = to_f64_signed(((words[4] & 255) << 24) | words[5], 32, -31);
    let omega_dot = to_f64_signed(words[6], 24, -43);
    let iode = (words[7] >> 16) as u8;
    let i_dot = to_f64_signed((words[7] >> 2) & 16383, 14, -43);

    GpsSubframe::Subframe3 {
        c_ic,
        omega0,
        c_is,
        i0,
        c_rc,
        omega_small,
        omega_dot,
        iode,
        i_dot,
    }
}

impl TryFrom<Vec<u8>> for UbxRxmSfrbxDataGps {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() != 40 {
            return Err(format!(
                "UbxRxmSfrbxDataGps: expected 40 bytes, got {}",
                bytes.len()
            ));
        }

        let mut words = vec![];
        for word in bytes.chunks(4) {
            let word_32 =
                u32::from_le_bytes(<[u8; 4]>::try_from(word).map_err(|err| format!("{}", err))?);
            // TODO: validate parity
            words.push((word_32 >> 6) & 0x00FFFFFF);
        }

        if words[0] >> 16 != 0x8b {
            return Err(format!(
                "UbxRxmSfrbxDataGps: wrong preamble, expected {}, got {}",
                0x8b,
                words[0] >> 16
            ));
        }

        let tlm_message = ((words[0] >> 2) & 0x3FFF) as u16;
        let integrity_bit = (words[0] >> 1) & 1 == 1;

        let tow = words[1] >> 7;
        let subframe = match (words[1] >> 2) & 7 {
            1 => decode_subframe1(&words[2..]),
            2 => decode_subframe2(&words[2..]),
            3 => decode_subframe3(&words[2..]),
            4 => GpsSubframe::Subframe4,
            5 => GpsSubframe::Subframe5,
            x => {
                return Err(format!("UbxRxmSfrbxDataGps: invalid subframe ID: {}", x));
            }
        };

        let anti_spoof = (words[1] & (1 << 5)) != 0;
        let alert = (words[1] & (1 << 6)) != 0;

        Ok(UbxRxmSfrbxDataGps {
            tlm_message,
            integrity_bit,
            tow,
            anti_spoof,
            alert,
            subframe,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UbxRxmSfrbxData {
    Gps(UbxRxmSfrbxDataGps),
    Other(Vec<u8>),
}

impl UbxRxmSfrbxData {
    fn words(&self) -> u8 {
        match self {
            UbxRxmSfrbxData::Gps(_) => 10,
            UbxRxmSfrbxData::Other(data) => (data.len() / 4) as u8,
        }
    }
}

impl From<UbxRxmSfrbxData> for Vec<u8> {
    fn from(data: UbxRxmSfrbxData) -> Vec<u8> {
        match data {
            UbxRxmSfrbxData::Gps(data) => data.into(),
            UbxRxmSfrbxData::Other(data) => data,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UbxRxmSfrbx {
    pub gnss_id: GnssId,
    pub sv_id: u8,
    pub freq_id: u8,
    pub version: u8,
    pub data: UbxRxmSfrbxData,
}

impl From<UbxRxmSfrbx> for Vec<u8> {
    fn from(msg: UbxRxmSfrbx) -> Vec<u8> {
        let mut result = vec![];
        result.push(msg.gnss_id as u8);
        result.push(msg.sv_id);
        result.push(0);
        result.push(msg.freq_id);
        result.push(msg.data.words());
        result.push(0);
        result.push(msg.version);
        result.push(0);
        result.extend(Vec::<u8>::from(msg.data));
        result
    }
}

impl TryFrom<Vec<u8>> for UbxRxmSfrbx {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err(format!(
                "UbxRxmSfrbx: data too short; expected min 8 bytes, got {}",
                bytes.len()
            ));
        }

        let gnss_id = GnssId::try_from(bytes[0])?;
        let sv_id = bytes[1];
        let freq_id = bytes[3];
        let length = bytes[4] as usize;

        if bytes.len() != 8 + 4 * length {
            return Err(format!(
                "UbxRxmSfrbx: wrong length, expected {}, got {}",
                8 + 4 * length,
                bytes.len()
            ));
        }

        let version = bytes[6];

        let data = match gnss_id {
            GnssId::Gps => UbxRxmSfrbxData::Gps(UbxRxmSfrbxDataGps::try_from(bytes[8..].to_vec())?),
            _ => UbxRxmSfrbxData::Other(bytes[8..].to_vec()),
        };

        Ok(UbxRxmSfrbx {
            gnss_id,
            sv_id,
            freq_id,
            version,
            data,
        })
    }
}
