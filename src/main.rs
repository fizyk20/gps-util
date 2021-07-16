mod ublox;

use std::{
    convert::TryInto,
    io::{Read, Write},
    iter,
    time::Duration,
};

use serialport::{self, SerialPort, TTYPort};

use ublox::*;

fn read_msg(port: &mut TTYPort) -> Result<Option<UbloxMsg>, String> {
    let num_bytes = port.bytes_to_read().map_err(|err| format!("{}", err))?;
    if num_bytes == 0 {
        return Ok(None);
    }
    let mut bytes: Vec<u8> = iter::repeat(0).take(num_bytes as usize).collect();
    port.take(num_bytes as u64).read(&mut bytes).unwrap();
    bytes.try_into().map(Some)
}

fn main() {
    let mut serial = serialport::new("/dev/ttyACM0", 9600)
        .timeout(Duration::from_secs(10))
        .open_native()
        .unwrap();

    let msg: Vec<u8> = UbloxMsg::CfgPrt(UbxCfgPrt::SetUsb {
        in_mask: UbxCfgPrtUsbInMask::UBX,
        out_mask: UbxCfgPrtUsbOutMask::UBX,
    })
    .into();
    serial.write_all(&msg).unwrap();

    let msg: Vec<u8> = UbloxMsg::CfgRate(UbxCfgRate {
        meas_rate_ms: 1000,
        nav_rate_cycles: 1,
        time_ref: UbxCfgRateTimeRef::Gps,
    })
    .into();
    serial.write_all(&msg).unwrap();

    let msg: Vec<u8> = UbloxMsg::CfgMsg(UbxCfgMsg::SetRate {
        class: 0x02,
        id: 0x13,
        rate: 1,
    })
    .into();
    serial.write_all(&msg).unwrap();

    let msg: Vec<u8> = UbloxMsg::CfgMsg(UbxCfgMsg::SetRate {
        class: 0x02,
        id: 0x15,
        rate: 1,
    })
    .into();
    serial.write_all(&msg).unwrap();

    serial.flush().unwrap();

    loop {
        let msg = read_msg(&mut serial);
        match msg {
            Ok(None) => {}
            Ok(Some(msg)) => {
                println!("{:#?}\n", msg);
            }
            Err(err) => {
                println!("Error! {}\n", err);
            }
        }
    }
}
