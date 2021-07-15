use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UbxCfgMsg {
    Get { class: u8, id: u8 },
    SetRate { class: u8, id: u8, rate: u8 },
    SetRatePorts { class: u8, id: u8, rate: [u8; 6] },
}

impl From<UbxCfgMsg> for Vec<u8> {
    fn from(msg: UbxCfgMsg) -> Vec<u8> {
        match msg {
            UbxCfgMsg::Get { class, id } => vec![class, id],
            UbxCfgMsg::SetRate { class, id, rate } => vec![class, id, rate],
            UbxCfgMsg::SetRatePorts { class, id, rate } => {
                let mut result = vec![class, id];
                result.extend(&rate[..]);
                result
            }
        }
    }
}

impl TryFrom<Vec<u8>> for UbxCfgMsg {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        match bytes.len() {
            2 => Ok(UbxCfgMsg::Get {
                class: bytes[0],
                id: bytes[1],
            }),
            3 => Ok(UbxCfgMsg::SetRate {
                class: bytes[0],
                id: bytes[1],
                rate: bytes[2],
            }),
            8 => {
                let mut rate = [0; 6];
                rate.copy_from_slice(&bytes[2..]);
                Ok(UbxCfgMsg::SetRatePorts {
                    class: bytes[0],
                    id: bytes[1],
                    rate,
                })
            }
            x => Err(format!("unexpected len for a UbxCfgMsg: {}", x)),
        }
    }
}
