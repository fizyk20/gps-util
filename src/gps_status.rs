use std::{collections::HashMap, time::SystemTime};

use nalgebra::Vector3;

use crate::ublox::GpsSubframe;

#[derive(Debug, Clone)]
pub struct GpsStatus {
    gps_time_correction: f64,
    satellites: HashMap<u8, SatelliteStatus>,
}

impl GpsStatus {
    pub fn new() -> Self {
        Self {
            gps_time_correction: 0.0,
            satellites: Default::default(),
        }
    }

    fn unix_time() -> f64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    pub fn set_time_correction(&mut self, current_gps_time: f64) {
        self.gps_time_correction = current_gps_time - Self::unix_time();
    }

    pub fn gps_time(&self) -> f64 {
        Self::unix_time() + self.gps_time_correction
    }

    pub fn consume_subframe(&mut self, sv_id: u8, subframe: GpsSubframe) {
        self.satellites
            .entry(sv_id)
            .or_default()
            .consume_subframe(subframe);
    }

    pub fn complete_satellites(&self) -> impl Iterator<Item = (u8, SatelliteOrbitalElements)> + '_ {
        self.satellites
            .iter()
            .filter_map(|(sv_id, status)| status.current_orbital_elements.map(|oe| (*sv_id, oe)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SatelliteStatus {
    current_orbital_elements: Option<SatelliteOrbitalElements>,
    partial_subframe: Option<GpsSubframe>,
}

impl SatelliteStatus {
    fn consume_subframe(&mut self, subframe: GpsSubframe) {
        match (self.partial_subframe.take(), subframe) {
            (
                Some(subframe2 @ GpsSubframe::Subframe2 { .. }),
                subframe3 @ GpsSubframe::Subframe3 { .. },
            )
            | (
                Some(subframe3 @ GpsSubframe::Subframe3 { .. }),
                subframe2 @ GpsSubframe::Subframe2 { .. },
            ) => {
                if subframe2.iode() == subframe3.iode() {
                    let new_elements =
                        SatelliteOrbitalElements::from_subframes(subframe2, subframe3);
                    self.current_orbital_elements = Some(new_elements);
                }
            }
            (_, subframe @ GpsSubframe::Subframe2 { .. })
            | (_, subframe @ GpsSubframe::Subframe3 { .. }) => {
                self.partial_subframe = Some(subframe);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SatelliteOrbitalElements {
    m0: f64,
    delta_n: f64,
    e: f64,
    sqrt_a: f64,
    omega0: f64,
    i0: f64,
    omega_small: f64,
    omega_dot: f64,
    i_dot: f64,
    c_uc: f64,
    c_us: f64,
    c_rc: f64,
    c_rs: f64,
    c_ic: f64,
    c_is: f64,
    t_oe: u32,
}

impl SatelliteOrbitalElements {
    fn from_subframes(subframe2: GpsSubframe, subframe3: GpsSubframe) -> Self {
        match (subframe2, subframe3) {
            (
                GpsSubframe::Subframe2 {
                    aodo: _,
                    iode: iode2,
                    c_rs,
                    delta_n,
                    m0,
                    c_uc,
                    e,
                    sqrt_a,
                    c_us,
                    t_oe,
                },
                GpsSubframe::Subframe3 {
                    iode: iode3,
                    c_ic,
                    omega0,
                    c_is,
                    i0,
                    c_rc,
                    omega_small,
                    omega_dot,
                    i_dot,
                },
            ) if iode2 == iode3 => SatelliteOrbitalElements {
                m0,
                delta_n,
                e,
                sqrt_a,
                omega0,
                i0,
                omega_small,
                omega_dot,
                i_dot,
                c_uc,
                c_us,
                c_rc,
                c_rs,
                c_ic,
                c_is,
                t_oe,
            },
            (subframe2, subframe3) => panic!(
                "wrong subframes passed to SatelliteOrbitalElements::from_subframes!\n\
                subframe2 = {:#?}\nsubframe3 = {:#?}\n",
                subframe2, subframe3
            ),
        }
    }

    pub fn position(&self, t: f64) -> Vector3<f64> {
        let tow = t % 604800.0;
        let mu = 3.986005e14;
        let omega_e = 7.2921151467e-5;
        let a = self.sqrt_a * self.sqrt_a;
        let n0 = (mu / a.powi(3)).sqrt();
        let mut tk = tow - self.t_oe as f64;
        if tk > 302400.0 {
            tk -= 604800.0;
        } else if tk < -302400.0 {
            tk += 604800.0;
        }
        let n = n0 + self.delta_n;
        let mk = self.m0 + n * tk;
        let mut ecc_anomaly = mk;
        loop {
            let new_ecc_anomaly = ecc_anomaly
                + (mk - ecc_anomaly - self.e * ecc_anomaly.sin())
                    / (1.0 - self.e * ecc_anomaly.cos());
            if ((new_ecc_anomaly - ecc_anomaly) / ecc_anomaly).abs() < 1e-9 {
                break;
            }
            ecc_anomaly = new_ecc_anomaly;
        }
        let true_anomaly =
            2.0 * (((1.0 + self.e) / (1.0 - self.e)).sqrt() * (ecc_anomaly / 2.0).tan()).atan();

        let phi_k = true_anomaly + self.omega_small;
        let delta_uk = self.c_us * (2.0 * phi_k).sin() + self.c_uc * (2.0 * phi_k).cos();
        let delta_rk = self.c_rs * (2.0 * phi_k).sin() + self.c_rc * (2.0 * phi_k).cos();
        let delta_ik = self.c_is * (2.0 * phi_k).sin() + self.c_ic * (2.0 * phi_k).cos();

        let uk = phi_k + delta_uk;
        let rk = a * (1.0 - self.e * ecc_anomaly.cos()) + delta_rk;
        let ik = self.i0 + delta_ik + self.i_dot * tk;
        let xkprim = rk * uk.cos();
        let ykprim = rk * uk.sin();

        let omega_k = self.omega0 + (self.omega_dot - omega_e) * tk - omega_e * self.t_oe as f64;

        let xk = xkprim * omega_k.cos() - ykprim * omega_k.sin() * ik.cos();
        let yk = xkprim * omega_k.sin() + ykprim * omega_k.cos() * ik.cos();
        let zk = ykprim * ik.sin();

        Vector3::new(xk, yk, zk)
    }
}
