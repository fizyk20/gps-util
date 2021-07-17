use std::convert::{TryFrom, TryInto};

pub use super::{msg_types::*, UbloxRawMsg};

#[derive(Debug, Clone, PartialEq)]
pub enum UbloxMsg {
    CfgMsg(UbxCfgMsg),
    CfgPrt(UbxCfgPrt),
    CfgRate(UbxCfgRate),
    CfgGnss(UbxCfgGnss),
    RxmSfrbx(UbxRxmSfrbx),
    RxmRawx(UbxRxmRawx),
    //RxmSfrbx(UbxRxmSfrbx),
    Other(UbloxRawMsg),
}

impl TryFrom<UbloxRawMsg> for UbloxMsg {
    type Error = String;

    fn try_from(raw_msg: UbloxRawMsg) -> Result<UbloxMsg, String> {
        match (raw_msg.class(), raw_msg.id()) {
            (0x02, 0x13) => {
                let inner = UbxRxmSfrbx::try_from(raw_msg.take_payload())?;
                Ok(UbloxMsg::RxmSfrbx(inner))
            }
            (0x02, 0x15) => {
                let inner = UbxRxmRawx::try_from(raw_msg.take_payload())?;
                Ok(UbloxMsg::RxmRawx(inner))
            }
            (0x06, 0x00) => {
                let inner = UbxCfgPrt::try_from(raw_msg.take_payload())?;
                Ok(UbloxMsg::CfgPrt(inner))
            }
            (0x06, 0x01) => {
                let inner = UbxCfgMsg::try_from(raw_msg.take_payload())?;
                Ok(UbloxMsg::CfgMsg(inner))
            }
            (0x06, 0x08) => {
                let inner = UbxCfgRate::try_from(raw_msg.take_payload())?;
                Ok(UbloxMsg::CfgRate(inner))
            }
            (0x06, 0x3e) => {
                let inner = UbxCfgGnss::try_from(raw_msg.take_payload())?;
                Ok(UbloxMsg::CfgGnss(inner))
            }
            _ => Ok(UbloxMsg::Other(raw_msg)),
        }
    }
}

impl TryFrom<Vec<u8>> for UbloxMsg {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<UbloxMsg, String> {
        let raw_msg: UbloxRawMsg = bytes.try_into()?;
        raw_msg.try_into()
    }
}

impl From<UbloxMsg> for UbloxRawMsg {
    fn from(msg: UbloxMsg) -> UbloxRawMsg {
        match msg {
            UbloxMsg::RxmSfrbx(inner) => UbloxRawMsg::new(0x02, 0x13, inner.into()),
            UbloxMsg::RxmRawx(inner) => UbloxRawMsg::new(0x02, 0x15, inner.into()),
            UbloxMsg::CfgPrt(inner) => UbloxRawMsg::new(0x06, 0x00, inner.into()),
            UbloxMsg::CfgMsg(inner) => UbloxRawMsg::new(0x06, 0x01, inner.into()),
            UbloxMsg::CfgRate(inner) => UbloxRawMsg::new(0x06, 0x08, inner.into()),
            UbloxMsg::CfgGnss(inner) => UbloxRawMsg::new(0x06, 0x3e, inner.into()),
            UbloxMsg::Other(raw_msg) => raw_msg,
        }
    }
}

impl From<UbloxMsg> for Vec<u8> {
    fn from(msg: UbloxMsg) -> Vec<u8> {
        let raw_msg: UbloxRawMsg = msg.into();
        raw_msg.into()
    }
}
