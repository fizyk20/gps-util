mod port_buffer;
mod ublox;

use std::{thread, time::Duration};

use serialport::{self};

use port_buffer::*;
use ublox::{
    GnssId, UbloxMsg, UbxCfgGnss, UbxCfgMsg, UbxCfgPrt, UbxCfgPrtUsbInMask, UbxCfgPrtUsbOutMask,
    UbxCfgRate, UbxCfgRateTimeRef,
};

fn port_thread() {
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

    port.send(Message::Ublox(UbloxMsg::CfgGnss(UbxCfgGnss::Poll)));

    loop {
        if let Err(err) = port.read() {
            println!("Error! {}\n", err);
            continue;
        }
        let msg = port.read_msg();
        match msg {
            None => {}
            Some(Message::Ublox(UbloxMsg::CfgGnss(UbxCfgGnss::Settings {
                version,
                num_trk_ch_hw,
                num_trk_ch_use,
                config_blocks,
            }))) => {
                println!(
                    "{:#?}\n",
                    Message::Ublox(UbloxMsg::CfgGnss(UbxCfgGnss::Settings {
                        version,
                        num_trk_ch_hw,
                        num_trk_ch_use,
                        config_blocks: config_blocks.clone()
                    }))
                );
                let config_blocks = config_blocks
                    .into_iter()
                    .map(|mut block| {
                        if block.gnss_id != GnssId::Gps {
                            block.enabled = false;
                        } else {
                            block.res_trk_ch = block.max_trk_ch;
                        }
                        block
                    })
                    .collect();
                let msg = UbloxMsg::CfgGnss(UbxCfgGnss::Settings {
                    version,
                    num_trk_ch_hw,
                    num_trk_ch_use,
                    config_blocks,
                });
                println!("Sending {:#?}\n", msg);
                port.send(Message::Ublox(msg));
            }
            Some(msg) => {
                println!("{:#?}\n", msg);
            }
        }
    }
}

fn main() {
    let port_thread = thread::spawn(port_thread);
    port_thread.join().unwrap();
}
