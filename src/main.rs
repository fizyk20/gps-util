mod ublox;

use std::{
    convert::TryInto,
    io::{Read, Write},
    mem,
    time::Duration,
};

use serialport::{self, SerialPort, TTYPort};

use ublox::*;

#[derive(Debug, Clone)]
enum Message {
    Ublox(UbloxMsg),
    Nmea(String),
}

#[derive(Debug)]
struct PortBuffer {
    port: TTYPort,
    buf: Vec<u8>,
}

impl PortBuffer {
    fn new(port: TTYPort) -> PortBuffer {
        PortBuffer { port, buf: vec![] }
    }

    fn send(&mut self, msg: Message) {
        let bytes = match msg {
            Message::Ublox(ubmsg) => ubmsg.into(),
            Message::Nmea(nmea) => nmea.into_bytes(),
        };
        self.port.write_all(&bytes).unwrap();
        self.port.flush().unwrap();
    }

    fn read(&mut self) -> Result<(), String> {
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

    fn sync(&mut self) -> bool {
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

    fn read_msg(&mut self) -> Option<Message> {
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

fn main() {
    let serial = serialport::new("/dev/ttyACM0", 9600)
        .timeout(Duration::from_secs(10))
        .open_native()
        .unwrap();

    let mut port = PortBuffer::new(serial);

    port.send(Message::Ublox(UbloxMsg::CfgPrt(UbxCfgPrt::SetUsb {
        in_mask: UbxCfgPrtUsbInMask::UBX,
        out_mask: UbxCfgPrtUsbOutMask::UBX,
    })));

    port.send(Message::Ublox(UbloxMsg::CfgRate(UbxCfgRate {
        meas_rate_ms: 1000,
        nav_rate_cycles: 1,
        time_ref: UbxCfgRateTimeRef::Gps,
    })));

    port.send(Message::Ublox(UbloxMsg::CfgMsg(UbxCfgMsg::SetRate {
        class: 0x02,
        id: 0x13,
        rate: 1,
    })));

    port.send(Message::Ublox(UbloxMsg::CfgMsg(UbxCfgMsg::SetRate {
        class: 0x02,
        id: 0x15,
        rate: 1,
    })));

    loop {
        if let Err(err) = port.read() {
            println!("Error! {}\n", err);
            continue;
        }
        let msg = port.read_msg();
        match msg {
            None => {}
            Some(msg) => {
                println!("{:#?}\n", msg);
            }
        }
    }
}
