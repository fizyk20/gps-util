use std::convert::{TryFrom, TryInto};

use super::UbloxRawMsg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UbloxMsg {
    Other(UbloxRawMsg),
}

impl TryFrom<UbloxRawMsg> for UbloxMsg {
    type Error = String;

    fn try_from(raw_msg: UbloxRawMsg) -> Result<UbloxMsg, String> {
        Ok(UbloxMsg::Other(raw_msg))
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
