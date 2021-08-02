mod gps_status;
mod port_buffer;
mod renderer;
mod ublox;

use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use glium::{
    glutin::{
        event::{ElementState, Event, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    Display,
};
use serialport::{self};

use gps_status::GpsStatus;
use port_buffer::*;
use renderer::Renderer;
use ublox::{
    GnssId, UbloxMsg, UbxCfgGnss, UbxCfgMsg, UbxCfgPrt, UbxCfgPrtUsbInMask, UbxCfgPrtUsbOutMask,
    UbxCfgRate, UbxCfgRateTimeRef, UbxRxmRawx, UbxRxmSfrbx, UbxRxmSfrbxData, UbxRxmSfrbxDataGps,
};

fn port_thread(gps_status: Arc<RwLock<GpsStatus>>) {
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
        if let Some(ref msg) = msg {
            println!("{:#?}", msg);
        }
        match msg {
            None => {}
            Some(Message::Ublox(UbloxMsg::CfgGnss(UbxCfgGnss::Settings {
                version,
                num_trk_ch_hw,
                num_trk_ch_use,
                config_blocks,
            }))) => {
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
            Some(Message::Ublox(UbloxMsg::RxmRawx(UbxRxmRawx { rcv_tow, week, .. }))) => {
                gps_status
                    .write()
                    .unwrap()
                    .set_time_correction(week as f64 * 604800.0 + rcv_tow);
            }
            Some(Message::Ublox(UbloxMsg::RxmSfrbx(UbxRxmSfrbx {
                sv_id,
                data: UbxRxmSfrbxData::Gps(UbxRxmSfrbxDataGps { subframe, .. }),
                ..
            }))) => {
                gps_status
                    .write()
                    .unwrap()
                    .consume_subframe(sv_id, subframe);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new().with_title("GPS Visualization");
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let mut renderer = Renderer::new(&display);

    let gps_status = Arc::new(RwLock::new(GpsStatus::new()));
    let gps_status_clone = gps_status.clone();
    let _port_thread = thread::spawn(move || port_thread(gps_status_clone));

    let start = Instant::now();
    let mut old_t = 0.0;

    // event handling loop (in main thread)
    event_loop.run(move |ev, _, control_flow| {
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    match (input.state, input.virtual_keycode) {
                        (ElementState::Pressed, Some(VirtualKeyCode::Space)) => {
                            // TODO
                        }
                        _ => (),
                    }
                }
                _ => return,
            },
            Event::MainEventsCleared => {
                let t = start.elapsed().as_secs_f32();
                if old_t < t.floor() {
                    let satellites: Vec<_> =
                        gps_status.read().unwrap().complete_satellites().collect();
                    let gps_t = gps_status.read().unwrap().gps_time();
                    for (sv_id, orb_elem) in satellites {
                        println!("{}: {:#?}\n", sv_id, orb_elem.position(gps_t));
                    }
                }
                old_t = t;
                renderer.draw(&display, t);
            }
            _ => (),
        }
        *control_flow = ControlFlow::Poll;
    });
}
