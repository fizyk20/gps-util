use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UbloxRawMsg {
    class: u8,
    id: u8,
    payload: Vec<u8>,
    checksum: [u8; 2],
}

impl UbloxRawMsg {
    pub fn new(class: u8, id: u8, payload: Vec<u8>) -> Self {
        let checksum = Self::calc_checksum(class, id, &payload);
        UbloxRawMsg {
            class,
            id,
            payload,
            checksum,
        }
    }

    pub fn class(&self) -> u8 {
        self.class
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn take_payload(self) -> Vec<u8> {
        self.payload
    }

    pub fn checksum(&self) -> [u8; 2] {
        self.checksum
    }

    fn calc_checksum(class: u8, id: u8, payload: &[u8]) -> [u8; 2] {
        let mut ck_a = 0u8;
        let mut ck_b = 0u8;

        let mut consume_byte = |byte| {
            ck_a = ck_a.wrapping_add(byte);
            ck_b = ck_b.wrapping_add(ck_a);
        };

        consume_byte(class);
        consume_byte(id);
        let [l0, l1] = (payload.len() as u16).to_le_bytes();
        consume_byte(l0);
        consume_byte(l1);

        for byte in payload {
            consume_byte(*byte);
        }

        [ck_a, ck_b]
    }
}

impl TryFrom<Vec<u8>> for UbloxRawMsg {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, String> {
        if &bytes[0..2] != &[0xb5, 0x62] {
            return Err(format!(
                "wrong header: expected [181, 98], got {:?}",
                &bytes[0..2]
            ));
        }

        if bytes.len() < 8 {
            return Err(format!(
                "message too short: expected at least 8 bytes, got {}",
                bytes.len()
            ));
        }

        let class = bytes[2];
        let id = bytes[3];
        let length = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;

        if bytes.len() < 8 + length {
            return Err(format!(
                "message too short: expected {} bytes, got {}",
                8 + length,
                bytes.len()
            ));
        }

        let payload = bytes[6..6 + length].to_vec();

        Ok(Self::new(class, id, payload))
    }
}

impl From<UbloxRawMsg> for Vec<u8> {
    fn from(msg: UbloxRawMsg) -> Vec<u8> {
        let [l0, l1] = (msg.payload.len() as u16).to_le_bytes();
        let mut result = vec![0xb5, 0x62, msg.class, msg.id, l0, l1];
        result.extend(&msg.payload);
        result.extend(&msg.checksum[..]);
        result
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use super::UbloxRawMsg;

    #[test]
    fn simple_test_from_vec() {
        let bytes = vec![
            0xb5, 0x62, 0x01, 0x02, 0x04, 0x00, b't', b'e', b's', b't', 0xc7, 0x87,
        ];
        let msg: Result<UbloxRawMsg, _> = bytes.try_into();
        assert_eq!(
            msg,
            Ok(UbloxRawMsg {
                class: 0x01,
                id: 0x02,
                payload: vec![b't', b'e', b's', b't'],
                checksum: [0xc7, 0x87],
            })
        );
    }

    #[test]
    fn simple_test_to_vec() {
        let payload = vec![b't', b'e', b's', b't'];
        let msg = UbloxRawMsg::new(0x01, 0x02, payload);
        let bytes: Vec<u8> = msg.into();
        assert_eq!(
            bytes,
            vec![0xb5, 0x62, 0x01, 0x02, 0x04, 0x00, b't', b'e', b's', b't', 0xc7, 0x87]
        );
    }
}
