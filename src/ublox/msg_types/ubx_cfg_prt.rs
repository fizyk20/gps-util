use std::convert::TryFrom;

use bitflags::bitflags;

bitflags! {
    pub struct UbxCfgPrtUsbInMask: u16 {
        const UBX = 0x01;
        const NMEA = 0x02;
        const RTCM = 0x04;
        const RTCM3 = 0x20;
    }
}

bitflags! {
    pub struct UbxCfgPrtUsbOutMask: u16 {
        const UBX = 0x01;
        const NMEA = 0x02;
        const RTCM3 = 0x20;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UbxCfgPrt {
    Get {
        port_id: u8,
    },
    SetUsb {
        in_mask: UbxCfgPrtUsbInMask,
        out_mask: UbxCfgPrtUsbOutMask,
    },
}

impl TryFrom<Vec<u8>> for UbxCfgPrt {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        match bytes.len() {
            1 => Ok(UbxCfgPrt::Get { port_id: bytes[0] }),
            20 => {
                let port_id = bytes[0];
                match port_id {
                    3 => {
                        let flags = u16::from_le_bytes([bytes[12], bytes[13]]);
                        let in_mask = UbxCfgPrtUsbInMask::from_bits(flags)
                            .ok_or_else(|| format!("invalid UbxCfgPrtInMask: {}", flags))?;
                        let flags = u16::from_le_bytes([bytes[14], bytes[15]]);
                        let out_mask = UbxCfgPrtUsbOutMask::from_bits(flags)
                            .ok_or_else(|| format!("invalid UbxCfgPrtOutMask: {}", flags))?;
                        Ok(UbxCfgPrt::SetUsb { in_mask, out_mask })
                    }
                    _ => unimplemented!(),
                }
            }
            x => Err(format!("unexpected len for a UbxCfgPrt: {}", x)),
        }
    }
}

impl From<UbxCfgPrt> for Vec<u8> {
    fn from(msg: UbxCfgPrt) -> Vec<u8> {
        match msg {
            UbxCfgPrt::Get { port_id } => vec![port_id],
            UbxCfgPrt::SetUsb { in_mask, out_mask } => {
                let [in0, in1] = in_mask.bits().to_le_bytes();
                let [out0, out1] = out_mask.bits().to_le_bytes();
                vec![
                    3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, in0, in1, out0, out1, 0, 0, 0, 0,
                ]
            }
        }
    }
}
