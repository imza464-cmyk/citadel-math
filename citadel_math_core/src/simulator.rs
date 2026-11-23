// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Gerçek 4-Aşamalı Runge-Kutta (RK4), Kepler Enberi (Perigee) ve Canlı Sutton-Graves Entegratörü

use crate::constants::PhysicalConstants;
use std::f64::consts::PI;

pub struct AtmosphereModel;

impl AtmosphereModel {
    /// US Standard Atmosphere 1976 (USSA-1976) Jeopotansiyel ve Termodinamik Sabitleri
    pub const R0_GEOPOTENTIAL: f64 = 6_356_766.0;
    pub const R_STAR: f64 = 8.31432;
    pub const M0_AIR: f64 = 0.0289644;
    pub const R_AIR: f64 = Self::R_STAR / Self::M0_AIR; // 287.052874247 J/(kg*K)
    pub const GAMMA_AIR: f64 = 1.4;
    pub const GMR: f64 = (PhysicalConstants::G0 * Self::M0_AIR) / Self::R_STAR; // 0.0341631947363 K/m
    pub const RHO_0: f64 = 1.225;
    pub const P_0: f64 = 101_325.0;
    pub const T_0: f64 = 288.15;
    pub const SUTTON_GRAVES_K: f64 = 1.7415e-4; // [kg^0.5 / m] Dünya atmosferi sabiti

    /// USSA-1976 7-Katmanlı Hidrostatik Tablo: (Z_b [m], T_b [K], L_b [K/m], P_b [Pa])
    const USSA76_LAYERS: [(f64, f64, f64, f64); 7] = [
        (0.0, 288.15, -0.0065, 101_325.0),
        (11_000.0, 216.65, 0.0, 22_632.063_973_462_92),
        (20_000.0, 216.65, 0.0010, 5_474.888_669_677_774_5),
        (32_000.0, 228.65, 0.0028, 868.018_684_755_227_9),
        (47_000.0, 270.65, 0.0, 110.906_305_554_966_08),
        (51_000.0, 270.65, -0.0028, 66.938_873_118_687_3),
        (71_000.0, 214.65, -0.0020, 3.956_420_428_040_732),
    ];

    /// Geometrik irtifadan (h [m]) USSA-1976 tam atmosfer durumunu döndürür:
    /// `(yoğunluk_kg_m3, basınç_pa, sıcaklık_k, ses_hızı_ms)`
    #[inline]
    pub fn state_at(altitude_m: f64) -> (f64, f64, f64, f64) {
        let h = altitude_m.max(0.0);
        if h > 150_000.0 {
            let t_exo = 474.946_f64;
            let a_exo = (Self::GAMMA_AIR * Self::R_AIR * t_exo).sqrt();
            return (0.0, 0.0, t_exo, a_exo);
        }

        // Jeopotansiyel irtifa Z = r0 * h / (r0 + h)
        let z = (Self::R0_GEOPOTENTIAL * h) / (Self::R0_GEOPOTENTIAL + h);
        if z <= 84_852.0 {
            let idx = if z < 11_000.0 {
                0
            } else if z < 20_000.0 {
                1
            } else if z < 32_000.0 {
                2
            } else if z < 47_000.0 {
                3
            } else if z < 51_000.0 {
                4
            } else if z < 71_000.0 {
                5
            } else {
                6
            };
            let (zb, tb, lb, pb) = Self::USSA76_LAYERS[idx];
            let dz = z - zb;
            let (temp_k, press_pa) = if lb.abs() < 1e-12 {
                let p = pb * ((-Self::GMR * dz) / tb).exp();
                (tb, p)
            } else {
                let t = tb + lb * dz;
                let p = pb * (tb / t).powf(Self::GMR / lb);
                (t, p)
            };
            let rho = press_pa / (Self::R_AIR * temp_k);
            let a_sound = (Self::GAMMA_AIR * Self::R_AIR * temp_k).sqrt();
            (rho, press_pa, temp_k, a_sound)
        } else {
            // Alt termosfer (86 km < h <= 150 km): USSA-1976 üst sınırından sürekli üstel geçiş
            let z_top = 84_852.0;
            let (zb, tb, lb, pb) = Self::USSA76_LAYERS[6];
            let t_86 = tb + lb * (z_top - zb); // 186.946 K
            let p_86 = pb * (tb / t_86).powf(Self::GMR / lb); // 0.3733836 Pa
            let dh = (h - 86_000.0).max(0.0);
            let temp_k = t_86 + 0.0045 * dh;
            let press_pa = p_86 * (-dh / 6_500.0).exp();
            let rho = press_pa / (Self::R_AIR * temp_k);
            let a_sound = (Self::GAMMA_AIR * Self::R_AIR * temp_k).sqrt();
            (rho, press_pa, temp_k, a_sound)
        }
    }

    #[inline(always)]
    pub fn density(altitude_m: f64) -> f64 {
        Self::state_at(altitude_m).0
    }

    #[inline(always)]
    pub fn pressure(altitude_m: f64) -> f64 {
        Self::state_at(altitude_m).1
    }

    #[inline(always)]
    pub fn temperature(altitude_m: f64) -> f64 {
        Self::state_at(altitude_m).2
    }

    #[inline(always)]
    pub fn speed_of_sound(altitude_m: f64) -> f64 {
        Self::state_at(altitude_m).3
    }

    #[inline(always)]
    pub fn gravity(altitude_m: f64) -> f64 {
        let r = PhysicalConstants::R_EARTH + altitude_m.max(0.0);
        PhysicalConstants::MU_EARTH / (r * r)
    }
}

pub struct VehicleConfig {
    pub name: &'static str,
    pub dry_mass: f64,
    pub propellant_mass: f64,
    pub thrust_vac: f64,
    pub isp: f64,
    pub area: f64,
    pub cd: f64,
    pub initial_velocity: f64,
    pub initial_altitude: f64,
    pub has_stage_2: bool,
    pub s2_dry_mass: f64,
    pub s2_propellant: f64,
    pub s2_thrust: f64,
    pub s2_isp: f64,
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub vehicle_name: &'static str,
    pub flight_duration_s: f64,
    pub final_altitude_km: f64,
    pub max_altitude_km: f64,
    pub perigee_altitude_km: f64,
    pub apogee_altitude_km: f64,
    pub eccentricity: f64,
    pub final_speed_ms: f64,
    pub final_vertical_speed_ms: f64,
    pub orbit_achieved: bool,
    pub remaining_fuel_kg: f64,
    pub max_q_kpa: f64,
    pub max_g_force: f64,
    pub peak_heat_flux_kw_m2: f64,
    pub total_heat_load_mj_m2: f64,
    pub peak_wall_temp_k: f64,
    pub execution_time_micros: u128,
}

/// Durum Vektörü: y = [x, h, vx, vh, m]
type StateVec = [f64; 5];

/// Türev Vektörü ve Anlık Telemetri: ([dx, dh, dvx, dvh, dm], q_dyn_pa, g_load, q_conv_w_m2, t_wall_k)
type DerivStep = ([f64; 5], f64, f64, f64, f64);

#[derive(Clone, Copy)]
struct StageFlightCtx<'a> {
    vehicle: &'a VehicleConfig,
    stage: u8,
    s1_burnout_mass: f64,
    m_dot_s1: f64,
    m_dot_s2: f64,
    v_e_s1: f64,
    v_e_s2: f64,
    nose_radius_m: f64,
    req_perigee_m: f64,
    target_alt_m: f64,
    max_g_throttle: Option<f64>,
}

pub struct RK4Simulator {
    pub dt: f64,
}

impl RK4Simulator {
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }

    /// Kepler Yörünge Elemanları Hesabı:
    /// Durum (h, vx, vh) -> (perigee_m, apogee_m, eccentricity)
    #[inline(always)]
    pub fn compute_orbital_elements(h: f64, vx: f64, vh: f64) -> (f64, f64, f64) {
        let r = PhysicalConstants::R_EARTH + h.max(0.0);
        let mu = PhysicalConstants::MU_EARTH;
        let v_sq = vx * vx + vh * vh;

        // Özgül mekanik enerji: epsilon = v^2 / 2 - mu / r
        let eps = 0.5 * v_sq - (mu / r);
        // Özgül açısal momentum: h_ang = r * vx
        let h_ang = r * vx;

        // Eksantriklik: e = sqrt(max(0, 1 + (2 * eps * h_ang^2) / mu^2))
        let e_sq = (1.0 + (2.0 * eps * h_ang * h_ang) / (mu * mu)).max(0.0);
        let e = e_sq.sqrt();

        if eps >= -1e-6 {
            // Parabolik / Hiperbolik kaçış yörüngesi
            let r_p = (h_ang * h_ang / mu) / (1.0 + e);
            return (r_p - PhysicalConstants::R_EARTH, 1e9, e);
        }

        let a = -mu / (2.0 * eps);
        let r_perigee = a * (1.0 - e);
        let r_apogee = a * (1.0 + e);
        (
            r_perigee - PhysicalConstants::R_EARTH,
            r_apogee - PhysicalConstants::R_EARTH,
            e,
        )
    }

    /// Kapalı Döngü Faz-Düzlemi (Phase-Plane) + Hohmann Enöte Daireselleştirme Güdümü:
    /// - 0 .. 1.5 km: Dikey kalkış (90°)
    /// - 1.5 .. 55 km: Atmosferik ağırlık dönüşü (Gravity Turn)
    /// - `>= 55 km`: Hedef enöteye tırmanış (frenleme zarfı `vh_brake = sqrt(2 * a_dec * dh)`)
    ///   ve hedef enöte yakalandığında enöte çevresinde yatay daireselleştirme (`vh -> 0`).
    #[inline(always)]
    fn guidance_pitch_angle(
        alt: f64,
        vx: f64,
        vh: f64,
        accel_thrust: f64,
        req_perigee_m: f64,
        target_alt_m: f64,
        cur_apogee_m: f64,
    ) -> f64 {
        if alt < 1500.0 {
            return std::f64::consts::FRAC_PI_2;
        }
        if alt < 55_000.0 {
            let frac = ((alt - 1500.0) / (55_000.0 - 1500.0)).clamp(0.0, 1.0);
            let deg = 90.0 * (1.0 - frac.powf(0.58));
            return deg.to_radians().clamp(14.0_f64.to_radians(), std::f64::consts::FRAC_PI_2);
        }

        let r = PhysicalConstants::R_EARTH + alt.max(0.0);
        let g = AtmosphereModel::gravity(alt);
        let centrifugal = (vx * vx) / r;
        let g_eff = g - centrifugal;
        let a_eff = accel_thrust.max(1.0);

        // Enöte daireselleştirme fazı: enöte hedef yörüngeye ulaştıysa dikey hızı sıfırlayıp
        // itkiyi yatay yörünge hızına (vx -> v_circ) yönlendir.
        if cur_apogee_m >= req_perigee_m + 2_000.0 && alt >= req_perigee_m - 25_000.0 {
            let req_ah_thrust = g_eff - 0.18 * vh;
            let sin_theta = (req_ah_thrust / a_eff).clamp(-0.50, 0.40);
            return sin_theta.asin();
        }

        let target_alt = target_alt_m.max(185_000.0);
        let dh = target_alt - alt;
        let a_dec = 3.5_f64;
        let vh_brake = if dh >= 0.0 {
            (2.0 * a_dec * dh).sqrt()
        } else {
            -((2.0 * a_dec * (-dh)).sqrt())
        };
        let vh_cap = if target_alt > 260_000.0 { 1500.0 } else { 950.0 };
        let vh_des = if dh > 0.0 {
            vh_brake.min(vh_cap)
        } else {
            vh_brake.max(-180.0)
        };

        let vh_err = vh_des - vh;
        let req_ah_thrust = g_eff + 0.35 * vh_err;
        let sin_theta = (req_ah_thrust / a_eff).clamp(-0.75, 0.80);
        sin_theta.asin()
    }

    #[inline(always)]
    fn derivatives(_t: f64, state: &StateVec, ctx: &StageFlightCtx<'_>) -> DerivStep {
        let (_x, h, vx, vh, m) = (state[0], state[1], state[2], state[3], state[4]);
        let eff_m = m.max(1.0);
        let vehicle = ctx.vehicle;

        let (raw_thrust, raw_mdot, stage_ve) = if ctx.stage == 1 {
            let is_burning = eff_m > ctx.s1_burnout_mass + 1e-6 && vehicle.thrust_vac > 0.0;
            if is_burning {
                (vehicle.thrust_vac, ctx.m_dot_s1, ctx.v_e_s1)
            } else {
                (0.0, 0.0, ctx.v_e_s1)
            }
        } else {
            let is_burning = eff_m > vehicle.s2_dry_mass + 1e-6 && vehicle.s2_thrust > 0.0;
            if is_burning {
                (vehicle.s2_thrust, ctx.m_dot_s2, ctx.v_e_s2)
            } else {
                (0.0, 0.0, ctx.v_e_s2)
            }
        };

        let (avail_thrust, avail_mdot) = match ctx.max_g_throttle {
            Some(g_cap) if g_cap > 0.0 && raw_thrust > 0.0 => {
                let max_t = eff_m * g_cap * PhysicalConstants::G0;
                if raw_thrust > max_t {
                    let mdot_throttled = if stage_ve > 0.0 { max_t / stage_ve } else { 0.0 };
                    (max_t, mdot_throttled)
                } else {
                    (raw_thrust, raw_mdot)
                }
            }
            _ => (raw_thrust, raw_mdot),
        };

        let (rho, _p_amb, t_amb, a_sound) = AtmosphereModel::state_at(h);
        let g = AtmosphereModel::gravity(h);
        let v_total = vx.hypot(vh);

        let q = 0.5 * rho * v_total * v_total;
        let drag = q * vehicle.cd * vehicle.area;

        let r_curr = PhysicalConstants::R_EARTH + h.max(0.0);
        let centrifugal_lift = (vx * vx) / r_curr;
        // Polar koordinatlarda açısal momentumun (L = r * vx) ve mekanik enerjinin korunumu için
        // zorunlu Coriolis/teğetsel kuplaj terimi: - (vx * vh) / r
        let coriolis_tangential = (vx * vh) / r_curr;

        // Hohmann Enöteye Süzülme (Coast-to-Apogee): Enöte hedef yörüngeye ulaştığında
        // daireselleştirme yanmasını enöte çevresinde ortalamak için motoru beklet.
        let (_cur_perigee, cur_apogee, cur_ecc) = if h >= 90_000.0 {
            Self::compute_orbital_elements(h, vx, vh)
        } else {
            (-PhysicalConstants::R_EARTH, 0.0, 1.0)
        };
        let v_circ_req =
            (PhysicalConstants::MU_EARTH / (PhysicalConstants::R_EARTH + ctx.req_perigee_m)).sqrt();
        let a_avail = (avail_thrust / eff_m).max(1.0);
        let t_circ = (v_circ_req - vx).max(0.0) / a_avail;
        let g_eff_pos = (g - centrifugal_lift).max(0.3);
        let t_apo = if vh > 0.0 { vh / g_eff_pos } else { 0.0 };
        let coasting = avail_thrust > 0.0
            && h >= 90_000.0
            && cur_apogee >= ctx.req_perigee_m + 6_000.0
            && vh > 15.0
            && t_apo > 0.55 * t_circ + 4.0
            && cur_ecc < 0.85;

        let (thrust, mass_loss) = if coasting {
            (0.0, 0.0)
        } else {
            (avail_thrust, avail_mdot)
        };

        let accel_thrust = thrust / eff_m;
        let theta = Self::guidance_pitch_angle(
            h,
            vx,
            vh,
            accel_thrust,
            ctx.req_perigee_m,
            ctx.target_alt_m,
            cur_apogee,
        );
        let fx_thrust = thrust * theta.cos();
        let fh_thrust = thrust * theta.sin();

        let (fx_drag, fh_drag) = if v_total > 0.0 {
            (drag * (vx / v_total), drag * (vh / v_total))
        } else {
            (0.0, 0.0)
        };

        // Öz ivme (Proper Acceleration / İvmeölçer Okuması): yalnızca temas kuvvetleri (İtki + Sürükleme) / m.
        // Serbest düşüş yörüngesinde (F_thrust = 0, F_drag = 0) Einstein Eşdeğerlik İlkesi gereği tam 0.0 G verir!
        let a_proper_x = (fx_thrust - fx_drag) / eff_m;
        let a_proper_h = (fh_thrust - fh_drag) / eff_m;

        let ax = a_proper_x - coriolis_tangential;
        let ah = a_proper_h - g + centrifugal_lift;

        let g_load = a_proper_x.hypot(a_proper_h) / PhysicalConstants::G0;

        // Canlı Sutton-Graves Konvektif Isı Akısı (W/m^2) ve USSA-1976 Gerçek Ortam Sıcaklıklı Radyatif Denge (K)
        let q_conv = if rho > 1e-9 && v_total > 100.0 {
            AtmosphereModel::SUTTON_GRAVES_K * (rho / ctx.nose_radius_m).sqrt() * v_total.powi(3)
        } else {
            0.0
        };
        let mach = v_total / a_sound.max(1.0);
        let t_stag = t_amb * (1.0 + 0.2 * mach * mach);
        let emissivity = 0.85_f64;
        let t_rad = ((q_conv / (emissivity * PhysicalConstants::SIGMA_SB)) + t_amb.powi(4)).powf(0.25);
        let t_wall = t_stag.min(t_rad);

        ([vx, vh, ax, ah, -mass_loss], q, g_load, q_conv, t_wall)
    }

    /// Gerçek 4. Derece Klasik Runge-Kutta (RK4) Entegrasyonu + Canlı Aerotermal Entegrasyon
    pub fn simulate(&self, vehicle: &VehicleConfig, max_time_s: f64) -> SimulationResult {
        self.simulate_for_orbit(vehicle, 172.0, None, max_time_s)
    }

    /// Hedef yörünge irtifası (`target_orbit_km`) ve opsiyonel azami G-kısması (`max_g_throttle`) ile RK4 yörünge entegrasyonu.
    /// SECO ancak gerçek Kepler enberisi `perigee_m >= target_orbit_km * 1000.0` olduğunda onaylanır.
    pub fn simulate_for_orbit(
        &self,
        vehicle: &VehicleConfig,
        target_orbit_km: f64,
        max_g_throttle: Option<f64>,
        max_time_s: f64,
    ) -> SimulationResult {
        let start_time = std::time::Instant::now();
        let req_perigee_m = (target_orbit_km * 1000.0).max(160_000.0);
        let target_alt_m = req_perigee_m + 12_000.0;
        let v_circ_target = (PhysicalConstants::MU_EARTH / (PhysicalConstants::R_EARTH + req_perigee_m)).sqrt();

        let mut t = 0.0;
        let s2_total_mass = if vehicle.has_stage_2 {
            vehicle.s2_dry_mass + vehicle.s2_propellant
        } else {
            0.0
        };
        let s1_burnout_mass = vehicle.dry_mass + s2_total_mass;
        let m_initial = s1_burnout_mass + vehicle.propellant_mass;

        let mut y: StateVec = [
            0.0,
            vehicle.initial_altitude,
            vehicle.initial_velocity * 0.80,
            vehicle.initial_velocity * 0.60,
            m_initial,
        ];

        let hull_r = (vehicle.area / PI).sqrt().max(0.25);
        let nose_radius_m = (hull_r * 0.45).max(0.15);

        let v_e = vehicle.isp * PhysicalConstants::G0;
        let m_dot_s1 = if v_e > 0.0 { vehicle.thrust_vac / v_e } else { 0.0 };

        let s2_ve = vehicle.s2_isp * PhysicalConstants::G0;
        let m_dot_s2 = if s2_ve > 0.0 { vehicle.s2_thrust / s2_ve } else { 0.0 };

        let mut current_stage: u8 = 1;
        let mut staged = false;

        let mut max_q = 0.0;
        let mut max_g = 0.0;
        let mut max_alt = y[1];
        let mut peak_q_conv = 0.0_f64;
        let mut total_heat_j_m2 = 0.0_f64;
        let mut peak_wall_temp_k = 290.0_f64;
        let mut orbit_achieved = false;
        let dt = self.dt;

        while t < max_time_s && (y[1] >= 0.0 || t < 5.0) {
            if current_stage == 1 {
                let is_burning = y[4] > s1_burnout_mass + 1e-6 && vehicle.thrust_vac > 0.0;
                if !is_burning && vehicle.has_stage_2 && !staged {
                    current_stage = 2;
                    staged = true;
                    y[4] = s2_total_mass;
                }
            }

            let ctx = StageFlightCtx {
                vehicle,
                stage: current_stage,
                s1_burnout_mass,
                m_dot_s1,
                m_dot_s2,
                v_e_s1: v_e,
                v_e_s2: s2_ve,
                nose_radius_m,
                req_perigee_m,
                target_alt_m,
                max_g_throttle,
            };

            let (k1, q, g_load, q_conv, t_wall) = Self::derivatives(t, &y, &ctx);

            let mut y1 = [0.0; 5];
            for i in 0..5 {
                y1[i] = y[i] + 0.5 * dt * k1[i];
            }
            let (k2, _, _, q_conv2, _) = Self::derivatives(t + 0.5 * dt, &y1, &ctx);

            let mut y2 = [0.0; 5];
            for i in 0..5 {
                y2[i] = y[i] + 0.5 * dt * k2[i];
            }
            let (k3, _, _, q_conv3, _) = Self::derivatives(t + 0.5 * dt, &y2, &ctx);

            let mut y3 = [0.0; 5];
            for i in 0..5 {
                y3[i] = y[i] + dt * k3[i];
            }
            let (k4, _, _, q_conv4, _) = Self::derivatives(t + dt, &y3, &ctx);

            for i in 0..5 {
                y[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            }
            total_heat_j_m2 += (dt / 6.0) * (q_conv + 2.0 * q_conv2 + 2.0 * q_conv3 + q_conv4);

            let min_mass = if current_stage == 1 {
                s1_burnout_mass
            } else {
                vehicle.s2_dry_mass
            };
            y[4] = y[4].max(min_mass);
            t += dt;

            if q > max_q {
                max_q = q;
            }
            if g_load > max_g {
                max_g = g_load;
            }
            if y[1] > max_alt {
                max_alt = y[1];
            }
            if q_conv > peak_q_conv {
                peak_q_conv = q_conv;
            }
            if t_wall > peak_wall_temp_k {
                peak_wall_temp_k = t_wall;
            }

            // Gerçek Kepler Yörünge Kararlılık Kriteri (SECO):
            // Anlık irtifa >= hedef enberi VE Enberi >= hedef enberi VE Eksantriklik e < 0.08
            if y[1] >= req_perigee_m && y[2] >= v_circ_target * 0.96 {
                let (perigee_m, _apogee_m, ecc) = Self::compute_orbital_elements(y[1], y[2], y[3]);
                if perigee_m >= req_perigee_m && ecc < 0.08 {
                    orbit_achieved = true;
                    break;
                }
            }
        }

        let elapsed = start_time.elapsed().as_micros();
        let final_speed = (y[2] * y[2] + y[3] * y[3]).sqrt();
        let (perigee_m, apogee_m, ecc) = Self::compute_orbital_elements(y[1], y[2], y[3]);
        let rem_fuel = if current_stage == 1 {
            y[4] - s1_burnout_mass
        } else {
            y[4] - vehicle.s2_dry_mass
        };

        SimulationResult {
            vehicle_name: vehicle.name,
            flight_duration_s: t,
            final_altitude_km: y[1] / 1000.0,
            max_altitude_km: max_alt / 1000.0,
            perigee_altitude_km: perigee_m / 1000.0,
            apogee_altitude_km: (apogee_m / 1000.0).min(99_999.0),
            eccentricity: ecc,
            final_speed_ms: final_speed,
            final_vertical_speed_ms: y[3],
            orbit_achieved,
            remaining_fuel_kg: rem_fuel.max(0.0),
            max_q_kpa: max_q / 1000.0,
            max_g_force: max_g,
            peak_heat_flux_kw_m2: peak_q_conv / 1000.0,
            total_heat_load_mj_m2: total_heat_j_m2 / 1e6,
            peak_wall_temp_k,
            execution_time_micros: elapsed,
        }
    }

    /// Faz 3: Gömülü 7-Aşamalı Dormand-Prince RK4(5) (DOPRI5 - O(dt^5)) Adaptif Adım Boylu Çözücü.
    /// Yerel kesme hatasını ||e||_∞ <= tol içinde tutar ve kademe ayrılma süreksizlik noktasına (burn_time_s1) tam kilitlenir.
    pub fn simulate_rk45(&self, vehicle: &VehicleConfig, max_time_s: f64, rel_tol: f64) -> SimulationResult {
        let start_time = std::time::Instant::now();
        let tol = rel_tol.clamp(1e-13, 1e-3);

        let mut t = 0.0;
        let s2_total_mass = if vehicle.has_stage_2 {
            vehicle.s2_dry_mass + vehicle.s2_propellant
        } else {
            0.0
        };
        let s1_burnout_mass = vehicle.dry_mass + s2_total_mass;
        let m_initial = s1_burnout_mass + vehicle.propellant_mass;

        let mut y: StateVec = [
            0.0,
            vehicle.initial_altitude,
            vehicle.initial_velocity * 0.80,
            vehicle.initial_velocity * 0.60,
            m_initial,
        ];

        let hull_r = (vehicle.area / PI).sqrt().max(0.25);
        let nose_radius_m = (hull_r * 0.45).max(0.15);

        let v_e = vehicle.isp * PhysicalConstants::G0;
        let m_dot_s1 = if v_e > 0.0 { vehicle.thrust_vac / v_e } else { 0.0 };
        let burn_time_s1 = if m_dot_s1 > 0.0 { vehicle.propellant_mass / m_dot_s1 } else { 0.0 };

        let s2_ve = vehicle.s2_isp * PhysicalConstants::G0;
        let m_dot_s2 = if s2_ve > 0.0 { vehicle.s2_thrust / s2_ve } else { 0.0 };

        let mut current_stage: u8 = 1;
        let mut staged = false;

        let mut max_q = 0.0;
        let mut max_g = 0.0;
        let mut max_alt = y[1];
        let mut peak_q_conv = 0.0_f64;
        let mut total_heat_j_m2 = 0.0_f64;
        let mut peak_wall_temp_k = 290.0_f64;
        let mut orbit_achieved = false;
        let mut dt = self.dt.clamp(0.01, 2.0);

        // Dormand-Prince (1980) 5. derece ağırlıkları b_i ve hata katsayıları e_i = b_i^(5) - b_i^(4)
        const B1: f64 = 35.0 / 384.0;
        const B3: f64 = 500.0 / 1113.0;
        const B4: f64 = 125.0 / 192.0;
        const B5: f64 = -2187.0 / 6784.0;
        const B6: f64 = 11.0 / 84.0;

        const E1: f64 = 71.0 / 57600.0;
        const E3: f64 = -71.0 / 16695.0;
        const E4: f64 = 71.0 / 1920.0;
        const E5: f64 = -17253.0 / 339200.0;
        const E6: f64 = 22.0 / 525.0;
        const E7: f64 = -1.0 / 40.0;

        while t < max_time_s && (y[1] >= 0.0 || t < 5.0) {
            if current_stage == 1 {
                let is_burning = t < burn_time_s1 - 1e-9 && y[4] > s1_burnout_mass + 1e-6;
                if !is_burning && vehicle.has_stage_2 && !staged {
                    current_stage = 2;
                    staged = true;
                    y[4] = s2_total_mass;
                }
            }

            // Kademe ayrılma süreksizliği veya bitiş zamanı aşımını tam sınırda yakala
            let mut h = dt.min(max_time_s - t);
            if current_stage == 1 && burn_time_s1 > 0.0 && t < burn_time_s1 && t + h > burn_time_s1 {
                h = (burn_time_s1 - t).max(1e-6);
            }

            let ctx = StageFlightCtx {
                vehicle,
                stage: current_stage,
                s1_burnout_mass,
                m_dot_s1,
                m_dot_s2,
                v_e_s1: v_e,
                v_e_s2: s2_ve,
                nose_radius_m,
                req_perigee_m: 172_000.0,
                target_alt_m: 190_000.0,
                max_g_throttle: None,
            };

            let (k1, dyn_q1, g1, q1, tw1) = Self::derivatives(t, &y, &ctx);

            let mut y2 = [0.0; 5];
            for i in 0..5 {
                y2[i] = y[i] + h * (0.2 * k1[i]);
            }
            let (k2, dyn_q2, g2, q2, tw2) = Self::derivatives(t + 0.2 * h, &y2, &ctx);

            let mut y3 = [0.0; 5];
            for i in 0..5 {
                y3[i] = y[i] + h * ((3.0 / 40.0) * k1[i] + (9.0 / 40.0) * k2[i]);
            }
            let (k3, dyn_q3, g3, q3, tw3) = Self::derivatives(t + 0.3 * h, &y3, &ctx);

            let mut y4 = [0.0; 5];
            for i in 0..5 {
                y4[i] = y[i] + h * ((44.0 / 45.0) * k1[i] - (56.0 / 15.0) * k2[i] + (32.0 / 9.0) * k3[i]);
            }
            let (k4, dyn_q4, g4, q4, tw4) = Self::derivatives(t + 0.8 * h, &y4, &ctx);

            let mut y5 = [0.0; 5];
            for i in 0..5 {
                y5[i] = y[i]
                    + h * ((19372.0 / 6561.0) * k1[i] - (25360.0 / 2187.0) * k2[i]
                        + (64448.0 / 6561.0) * k3[i]
                        - (212.0 / 729.0) * k4[i]);
            }
            let (k5, dyn_q5, g5, q5, tw5) = Self::derivatives(t + (8.0 / 9.0) * h, &y5, &ctx);

            let mut y6 = [0.0; 5];
            for i in 0..5 {
                y6[i] = y[i]
                    + h * ((9017.0 / 3168.0) * k1[i] - (355.0 / 33.0) * k2[i]
                        + (46732.0 / 5247.0) * k3[i]
                        + (49.0 / 176.0) * k4[i]
                        - (5103.0 / 18656.0) * k5[i]);
            }
            let (k6, dyn_q6, g6, q6, tw6) = Self::derivatives(t + h, &y6, &ctx);

            // 5. derece çözüm adayı y_next
            let mut y_next = [0.0; 5];
            for i in 0..5 {
                y_next[i] = y[i] + h * (B1 * k1[i] + B3 * k3[i] + B4 * k4[i] + B5 * k5[i] + B6 * k6[i]);
            }
            let (k7, dyn_q7, g7, q7, tw7) = Self::derivatives(t + h, &y_next, &ctx);

            // Bağıl/mutlak ölçekli yerel kesme hatası normu (5 durum + 6. durum: aerotermal ısı integrali Q)
            let mut err_norm = 0.0_f64;
            for i in 0..5 {
                let err_i = h * (E1 * k1[i] + E3 * k3[i] + E4 * k4[i] + E5 * k5[i] + E6 * k6[i] + E7 * k7[i]);
                let scale = 1e-6 + tol * y[i].abs().max(y_next[i].abs()).max(1.0);
                err_norm = err_norm.max((err_i / scale).abs());
            }
            let dq_step = h * (B1 * q1 + B3 * q3 + B4 * q4 + B5 * q5 + B6 * q6).max(0.0);
            let err_q = h * (E1 * q1 + E3 * q3 + E4 * q4 + E5 * q5 + E6 * q6 + E7 * q7);
            let scale_q = 1.0 + tol * (total_heat_j_m2 + dq_step).max(1000.0);
            err_norm = err_norm.max((err_q / scale_q).abs());

            if err_norm <= 1.0 || h <= 1e-5 {
                // Adım kabul edildi
                y = y_next;
                total_heat_j_m2 += dq_step;

                let min_mass = if current_stage == 1 {
                    s1_burnout_mass
                } else {
                    vehicle.s2_dry_mass
                };
                y[4] = y[4].max(min_mass);
                t += h;

                let step_max_q = dyn_q1.max(dyn_q2).max(dyn_q3).max(dyn_q4).max(dyn_q5).max(dyn_q6).max(dyn_q7);
                let step_max_g = g1.max(g2).max(g3).max(g4).max(g5).max(g6).max(g7);
                let step_peak_qconv = q1.max(q2).max(q3).max(q4).max(q5).max(q6).max(q7);
                let step_peak_twall = tw1.max(tw2).max(tw3).max(tw4).max(tw5).max(tw6).max(tw7);

                if step_max_q > max_q { max_q = step_max_q; }
                if step_max_g > max_g { max_g = step_max_g; }
                if y[1] > max_alt { max_alt = y[1]; }
                if step_peak_qconv > peak_q_conv { peak_q_conv = step_peak_qconv; }
                if step_peak_twall > peak_wall_temp_k { peak_wall_temp_k = step_peak_twall; }

                if y[1] >= 172_000.0 && y[2] >= 7650.0 {
                    let (perigee_m, _apogee_m, ecc) = Self::compute_orbital_elements(y[1], y[2], y[3]);
                    if perigee_m >= 172_000.0 && ecc < 0.065 {
                        orbit_achieved = true;
                        break;
                    }
                }
            }

            let factor = if err_norm == 0.0 {
                2.5
            } else {
                (0.9 * err_norm.powf(-0.2)).clamp(0.2, 2.5)
            };
            dt = (h * factor).clamp(1e-4, 2.0);
        }

        let elapsed = start_time.elapsed().as_micros();
        let final_speed = (y[2] * y[2] + y[3] * y[3]).sqrt();
        let (perigee_m, apogee_m, ecc) = Self::compute_orbital_elements(y[1], y[2], y[3]);
        let rem_fuel = if current_stage == 1 {
            y[4] - s1_burnout_mass
        } else {
            y[4] - vehicle.s2_dry_mass
        };

        SimulationResult {
            vehicle_name: vehicle.name,
            flight_duration_s: t,
            final_altitude_km: y[1] / 1000.0,
            max_altitude_km: max_alt / 1000.0,
            perigee_altitude_km: perigee_m / 1000.0,
            apogee_altitude_km: (apogee_m / 1000.0).min(99_999.0),
            eccentricity: ecc,
            final_speed_ms: final_speed,
            final_vertical_speed_ms: y[3],
            orbit_achieved,
            remaining_fuel_kg: rem_fuel.max(0.0),
            max_q_kpa: max_q / 1000.0,
            max_g_force: max_g,
            peak_heat_flux_kw_m2: peak_q_conv / 1000.0,
            total_heat_load_mj_m2: total_heat_j_m2 / 1e6,
            peak_wall_temp_k,
            execution_time_micros: elapsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hohmann_circular_orbit_400km_45g_and_beamed_200km() {
        let sim = RK4Simulator::new(0.2);
        let v_lh2 = VehicleConfig {
            name: "Test-LOX-LH2-400km",
            dry_mass: 2200.0,
            propellant_mass: 17280.0,
            thrust_vac: 480_000.0,
            isp: 425.0,
            area: PI,
            cd: 0.26,
            initial_velocity: 0.0,
            initial_altitude: 0.0,
            has_stage_2: true,
            s2_dry_mass: 1850.0,
            s2_propellant: 6720.0,
            s2_thrust: 110_000.0,
            s2_isp: 474.6,
        };
        let r = sim.simulate_for_orbit(&v_lh2, 400.0, Some(4.5), 1400.0);
        assert!(r.orbit_achieved, "400 km yörüngeye oturmalı: {:?}", r);
        assert!(r.perigee_altitude_km >= 400.0, "Enberi >= 400 km olmalı: {}", r.perigee_altitude_km);
        assert!(r.max_g_force <= 4.55, "Max-G <= 4.55 olmalı: {}", r.max_g_force);
        assert!(r.eccentricity < 0.08, "Dairesel yörünge e < 0.08 olmalı: {}", r.eccentricity);

        let v_beamed = VehicleConfig {
            name: "Test-Beamed-SSTO-200km",
            dry_mass: 1_200.0,
            propellant_mass: 6_000.0,
            thrust_vac: 120_000.0,
            isp: 950.0,
            area: 1.5,
            cd: 0.25,
            initial_velocity: 0.0,
            initial_altitude: 0.0,
            has_stage_2: false,
            s2_dry_mass: 0.0,
            s2_propellant: 0.0,
            s2_thrust: 0.0,
            s2_isp: 0.0,
        };
        let rb = sim.simulate_for_orbit(&v_beamed, 200.0, Some(4.5), 1200.0);
        assert!(rb.orbit_achieved, "Beamed SSTO 200 km yörüngeye oturmalı: {:?}", rb);
        assert!(rb.perigee_altitude_km >= 200.0, "Beamed enberi >= 200 km olmalı: {}", rb.perigee_altitude_km);
        assert!(rb.max_g_force <= 4.55, "Beamed Max-G <= 4.55 olmalı: {}", rb.max_g_force);
        assert!(rb.eccentricity < 0.08, "Beamed e < 0.08 olmalı: {}", rb.eccentricity);
    }
}
