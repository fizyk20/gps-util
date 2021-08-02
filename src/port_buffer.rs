use std::{
    convert::TryInto,
    io::{Read, Write},
    mem,
};

use serialport::{self, SerialPort, TTYPort};

use crate::ublox::UbloxMsg;

#[derive(Debug, Clone)]
pub enum Message {
    Ublox(UbloxMsg),
    Nmea(String),
}

#[derive(Debug)]
pub struct PortBuffer {
    port: TTYPort,
    buf: Vec<u8>,
}

impl PortBuffer {
    pub fn new(port: TTYPort) -> PortBuffer {
        PortBuffer { port, buf: vec![] }
    }

    pub fn send(&mut self, msg: Message) {
        let bytes = match msg {
            Message::Ublox(ubmsg) => ubmsg.into(),
            Message::Nmea(nmea) => nmea.into_bytes(),
        };
        self.port.write_all(&bytes).unwrap();
        self.port.flush().unwrap();
    }

    pub fn read(&mut self) -> Result<(), String> {
        let num_bytes = self
            .port
            .bytes_to_read()
            .map_err(|err| format!("{}", err))?;
        if num_bytes == 0 {
            return Ok(());
        }
        let mut bytes = vec![0; num_bytes as usize];
        self.port.read_exact(&mut bytes).unwrap();
        self.buf.extend(bytes);
        Ok(())
    }

    pub fn sync(&mut self) -> bool {
        let mut i = 0;
        if self.buf.len() < 2 {
            return false;
        }
        while i < self.buf.len() - 1 {
            if &self.buf[i..i + 2] == &[0xb5, 0x62]
                || (self.buf[i] == b'$' && self.buf[i + 1] >= b'A' && self.buf[i + 1] <= b'Z')
            {
                let rest = self.buf.split_off(i);
                self.buf = rest;
                return true;
            }
            i += 1;
        }
        false
    }

    pub fn read_msg(&mut self) -> Option<Message> {
        if !self.sync() {
            return None;
        }
        if self.buf[0] == b'$' && self.buf[1] >= b'A' && self.buf[1] <= b'Z' {
            let (end, _) = self.buf.iter().enumerate().find(|(_, c)| **c == b'\n')?;
            let rest = self.buf.split_off(end + 1);
            let msg = String::from_utf8(mem::replace(&mut self.buf, rest)).unwrap();
            return Some(Message::Nmea(msg));
        }
        if &self.buf[0..2] == &[0xb5, 0x62] {
            if self.buf.len() < 8 {
                return None;
            }
            let length = u16::from_le_bytes(self.buf[4..6].try_into().unwrap()) as usize;
            if self.buf.len() < 8 + length {
                return None;
            }
            let rest = self.buf.split_off(8 + length);
            let msg = mem::replace(&mut self.buf, rest);
            return Some(Message::Ublox(msg.try_into().unwrap()));
        }
        None
    }
}
