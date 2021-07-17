use std::convert::TryFrom;

use bitflags::bitflags;

use super::GnssId;

bitflags! {
    pub struct UbxCfgGnssBlockFlagsGps: u8 {
        const L1CA = 0x01;
        const L2C = 0x10;
        const L5 = 0x20;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UbxCfgGnssBlockFlags {
    Gps(UbxCfgGnssBlockFlagsGps),
    Other,
}

impl From<UbxCfgGnssBlockFlags> for Vec<u8> {
    fn from(flags: UbxCfgGnssBlockFlags) -> Vec<u8> {
        match flags {
            UbxCfgGnssBlockFlags::Gps(gps) => vec![gps.bits()],
            UbxCfgGnssBlockFlags::Other => vec![0],
        }
    }
}

impl UbxCfgGnssBlockFlags {
    fn gps_try_from(val: u8) -> Result<UbxCfgGnssBlockFlags, String> {
        UbxCfgGnssBlockFlagsGps::from_bits(val)
            .ok_or_else(|| format!("invalid UbxCfgGnssBlockFlagsGps: {}", val))
            .map(UbxCfgGnssBlockFlags::Gps)
    }

    fn other_try_from(_val: u8) -> Result<UbxCfgGnssBlockFlags, String> {
        Ok(UbxCfgGnssBlockFlags::Other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UbxCfgGnssBlock {
    pub gnss_id: GnssId,
    pub res_trk_ch: u8,
    pub max_trk_ch: u8,
    pub enabled: bool,
    pub flags: UbxCfgGnssBlockFlags,
}

impl From<UbxCfgGnssBlock> for Vec<u8> {
    fn from(block: UbxCfgGnssBlock) -> Vec<u8> {
        let mut result = vec![block.gnss_id as u8, block.res_trk_ch, block.max_trk_ch, 0];
        if block.enabled {
            result.push(1);
        } else {
            result.push(0);
        }
        result.push(0);
        result.extend(Vec::<u8>::from(block.flags));
        result.push(0);
        result
    }
}

impl TryFrom<Vec<u8>> for UbxCfgGnssBlock {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<UbxCfgGnssBlock, String> {
        let gnss_id = GnssId::try_from(bytes[0])?;
        let res_trk_ch = bytes[1];
        let max_trk_ch = bytes[2];
        let enabled = match bytes[4] {
            0 => false,
            1 => true,
            x => {
                return Err(format!("UbxCfgGnssBlock: invalid value for enabled: {}", x));
            }
        };
        let flags = match gnss_id {
            GnssId::Gps => UbxCfgGnssBlockFlags::gps_try_from(bytes[6])?,
            _ => UbxCfgGnssBlockFlags::other_try_from(bytes[6])?,
        };

        Ok(UbxCfgGnssBlock {
            gnss_id,
            res_trk_ch,
            max_trk_ch,
            enabled,
            flags,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UbxCfgGnss {
    Poll,
    Settings {
        version: u8,
        num_trk_ch_hw: u8,
        num_trk_ch_use: u8,
        config_blocks: Vec<UbxCfgGnssBlock>,
    },
}

impl From<UbxCfgGnss> for Vec<u8> {
    fn from(msg: UbxCfgGnss) -> Vec<u8> {
        match msg {
            UbxCfgGnss::Poll => vec![],
            UbxCfgGnss::Settings {
                version,
                num_trk_ch_hw,
                num_trk_ch_use,
                config_blocks,
            } => {
                let mut result = vec![
                    version,
                    num_trk_ch_hw,
                    num_trk_ch_use,
                    config_blocks.len() as u8,
                ];
                for block in config_blocks {
                    result.extend(Vec::<u8>::from(block));
                }
                result
            }
        }
    }
}

impl TryFrom<Vec<u8>> for UbxCfgGnss {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        match bytes.len() {
            0 => Ok(UbxCfgGnss::Poll),
            x if (x - 4) % 8 == 0 => {
                let version = bytes[0];
                let num_trk_ch_hw = bytes[1];
                let num_trk_ch_use = bytes[2];
                let num_blocks = bytes[3] as usize;
                let mut config_blocks = vec![];
                for i in 0..num_blocks {
                    let block = UbxCfgGnssBlock::try_from(bytes[4 + 8 * i..12 + 8 * i].to_vec())?;
                    config_blocks.push(block);
                }
                Ok(UbxCfgGnss::Settings {
                    version,
                    num_trk_ch_hw,
                    num_trk_ch_use,
                    config_blocks,
                })
            }
            x => Err(format!("unexpected len for a UbxCfgGnss: {}", x)),
        }
    }
}
