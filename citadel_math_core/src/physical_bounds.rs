// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Değişmez 2: Termodinamik, Korunum ve Nedensellik Budama Kalkanı
// Referans: AGENTS.md Değişmez 2 (Termodinamik ve Doğa Yasalarının Aşılamazlığı)

use crate::constants::PhysicalConstants;
use std::fmt;

/// Doğa yasası ihlali türleri (Karantina ve Budama Sınıflandırması)
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicalLawViolation {
    /// Negatif veya sonlu olmayan kütle / mutlak sıcaklık
    NonPhysicalState(String),
    /// Özel Görelilik İhlali: Kütleli cisim veya egzoz hızı v >= c
    SuperluminalCausalityViolation { velocity_ms: f64, speed_of_light_ms: f64 },
    /// Termodinamik 1. Yasa İhlali: Çıkış gücü > Giriş gücü (1. Tür Devridaim)
    FirstLawEnergyViolation { power_out_w: f64, power_in_w: f64 },
    /// Termodinamik 2. Yasa İhlali: Verim eta > 1.0 veya eta < 0.0 veya dS < 0
    SecondLawEntropyViolation { detail: String },
    /// Carnot Sınırı İhlali: Isı makinesi verimi eta > 1 - T_cold / T_hot (2. Tür Devridaim)
    CarnotBoundViolation { claimed_eta: f64, carnot_max_eta: f64, t_hot_k: f64, t_cold_k: f64 },
    /// Momentum Korunumu İhlali: Kütle debisi m_dot = 0 iken foton momentum sınırını (2*P/c) aşan tepkisiz itki
    ReactionlessMomentumViolation { thrust_n: f64, max_photon_thrust_n: f64 },
}

impl fmt::Display for PhysicalLawViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhysicalLawViolation::NonPhysicalState(msg) => {
                write!(f, "Fiziksel Olmayan Durum: {}", msg)
            }
            PhysicalLawViolation::SuperluminalCausalityViolation {
                velocity_ms,
                speed_of_light_ms,
            } => write!(
                f,
                "Özel Görelilik Nedensellik İhlali: |v| = {:.3e} m/s >= c ({:.0} m/s)",
                velocity_ms, speed_of_light_ms
            ),
            PhysicalLawViolation::FirstLawEnergyViolation {
                power_out_w,
                power_in_w,
            } => write!(
                f,
                "Termodinamik 1. Yasa İhlali (1. Tür Devridaim): P_out ({:.3e} W) > P_in ({:.3e} W)",
                power_out_w, power_in_w
            ),
            PhysicalLawViolation::SecondLawEntropyViolation { detail } => {
                write!(f, "Termodinamik 2. Yasa İhlali: {}", detail)
            }
            PhysicalLawViolation::CarnotBoundViolation {
                claimed_eta,
                carnot_max_eta,
                t_hot_k,
                t_cold_k,
            } => write!(
                f,
                "Carnot Sınırı İhlali (2. Tür Devridaim): eta = {:.6} > eta_Carnot = {:.6} (T_h = {:.1} K, T_c = {:.1} K)",
                claimed_eta, carnot_max_eta, t_hot_k, t_cold_k
            ),
            PhysicalLawViolation::ReactionlessMomentumViolation {
                thrust_n,
                max_photon_thrust_n,
            } => write!(
                f,
                "Momentum Korunumu İhlali (Tepkisiz İtki): m_dot = 0 iken F = {:.6e} N > 2*P_in/c ({:.6e} N)",
                thrust_n, max_photon_thrust_n
            ),
        }
    }
}

/// İtki, enerji veya termodinamik bir sistem adayının fiziksel gerçeklenebilirlik parametreleri
#[derive(Debug, Clone, Default)]
pub struct PhysicalSystemCandidate {
    pub mass_kg: Option<f64>,
    pub velocity_ms: Option<f64>,
    pub exhaust_velocity_ms: Option<f64>,
    pub mass_flow_kg_s: Option<f64>,
    pub thrust_n: Option<f64>,
    pub power_in_w: Option<f64>,
    pub power_out_w: Option<f64>,
    pub efficiency: Option<f64>,
    pub temp_hot_k: Option<f64>,
    pub temp_cold_k: Option<f64>,
    pub entropy_delta_j_k: Option<f64>,
}

pub struct PhysicalBoundsShield;

impl PhysicalBoundsShield {
    /// Kütleli bir parçacık veya aracın hızının nedensellik sınırına (|v| < c) uyduğunu doğrular
    /// ve Lorentz faktörü gamma = 1 / sqrt(1 - (v/c)^2) değerini döndürür.
    pub fn check_subluminal_velocity(v_ms: f64) -> Result<f64, PhysicalLawViolation> {
        if !v_ms.is_finite() {
            return Err(PhysicalLawViolation::NonPhysicalState(
                "Hız sonlu bir reel sayı olmalıdır".to_string(),
            ));
        }
        let abs_v = v_ms.abs();
        if abs_v >= PhysicalConstants::C {
            return Err(PhysicalLawViolation::SuperluminalCausalityViolation {
                velocity_ms: abs_v,
                speed_of_light_ms: PhysicalConstants::C,
            });
        }
        let beta = abs_v / PhysicalConstants::C;
        Ok(1.0 / (1.0 - beta * beta).sqrt())
    }

    /// Bir sistem adayını AGENTS.md Değişmez 2 kapsamındaki 4 temel doğa yasasına karşı
    /// erken çıkış yapmadan denetler ve tespit edilen TÜM ihlalleri toplar.
    pub fn collect_violations(cand: &PhysicalSystemCandidate) -> Vec<PhysicalLawViolation> {
        const EPS: f64 = 1e-9;
        let mut violations: Vec<PhysicalLawViolation> = Vec::new();

        if let Some(m) = cand.mass_kg {
            if !m.is_finite() || m < 0.0 {
                violations.push(PhysicalLawViolation::NonPhysicalState(format!(
                    "Kütle negatif veya tanımsız olamaz: {} kg",
                    m
                )));
            }
        }

        // 1. Özel Görelilik Nedensellik Sınırı (v < c, v_e < c)
        if let Some(v) = cand.velocity_ms {
            if (v.is_finite() && v.abs() > 0.0) || !v.is_finite() {
                let check_massive = cand.mass_kg.map(|m| m > 0.0).unwrap_or(true);
                if check_massive {
                    if let Err(err) = Self::check_subluminal_velocity(v) {
                        violations.push(err);
                    }
                }
            }
        }
        if let Some(ve) = cand.exhaust_velocity_ms {
            if let Err(err) = Self::check_subluminal_velocity(ve) {
                violations.push(err);
            }
        }

        // 2. Termodinamik 2. Yasa: Entropi (dS >= 0), Verim (0 <= eta <= 1) ve Carnot Sınırı
        if let Some(ds) = cand.entropy_delta_j_k {
            if !ds.is_finite() || ds < -1e-12 {
                violations.push(PhysicalLawViolation::SecondLawEntropyViolation {
                    detail: format!("İzole sistem entropisi azalamaz: dS = {:.3e} J/K < 0", ds),
                });
            }
        }

        if let Some(eta) = cand.efficiency {
            if !eta.is_finite() || !(0.0..=1.0 + EPS).contains(&eta) {
                violations.push(PhysicalLawViolation::SecondLawEntropyViolation {
                    detail: format!("Termodinamik dönüşüm verimi [0, 1] aralığında olmalıdır: eta = {}", eta),
                });
            }
            if let (Some(th), Some(tc)) = (cand.temp_hot_k, cand.temp_cold_k) {
                if !th.is_finite() || !tc.is_finite() || th <= 0.0 || tc <= 0.0 {
                    violations.push(PhysicalLawViolation::NonPhysicalState(format!(
                        "Mutlak sıcaklık 0 K üzerinde olmalıdır (3. Yasa): T_h = {} K, T_c = {} K",
                        th, tc
                    )));
                } else if tc > th {
                    violations.push(PhysicalLawViolation::NonPhysicalState(format!(
                        "Soğuk rezervuar sıcaklığı ({} K) sıcak rezervuardan ({} K) büyük olamaz",
                        tc, th
                    )));
                } else {
                    let carnot_max = 1.0 - (tc / th);
                    if eta.is_finite() && eta > carnot_max + EPS {
                        violations.push(PhysicalLawViolation::CarnotBoundViolation {
                            claimed_eta: eta,
                            carnot_max_eta: carnot_max,
                            t_hot_k: th,
                            t_cold_k: tc,
                        });
                    }
                }
            }
        }

        // 3. Termodinamik 1. Yasa: Enerji Korunumu (P_out <= P_in ve P_jet <= eta * P_in)
        if let (Some(p_in), Some(p_out)) = (cand.power_in_w, cand.power_out_w) {
            if !p_in.is_finite() || !p_out.is_finite() || p_in < 0.0 || p_out < 0.0 {
                violations.push(PhysicalLawViolation::NonPhysicalState(
                    "Güç değerleri negatif olmayan sonlu sayılar olmalıdır".to_string(),
                ));
            } else if p_out > p_in * (1.0 + EPS) + 1e-9 {
                violations.push(PhysicalLawViolation::FirstLawEnergyViolation {
                    power_out_w: p_out,
                    power_in_w: p_in,
                });
            }
        }

        if let (Some(mdot), Some(ve), Some(p_in)) =
            (cand.mass_flow_kg_s, cand.exhaust_velocity_ms, cand.power_in_w)
        {
            if !mdot.is_finite() || mdot < 0.0 {
                violations.push(PhysicalLawViolation::NonPhysicalState(
                    "Kütlesel debi negatif veya tanımsız olamaz".to_string(),
                ));
            } else if ve.is_finite() && p_in.is_finite() && p_in >= 0.0 {
                let p_jet = 0.5 * mdot * ve * ve;
                let eff_limit = cand.efficiency.unwrap_or(1.0).clamp(0.0, 1.0);
                let max_allowed_jet = p_in * eff_limit;
                if p_jet > max_allowed_jet * (1.0 + 1e-6) + 1e-6 {
                    violations.push(PhysicalLawViolation::FirstLawEnergyViolation {
                        power_out_w: p_jet,
                        power_in_w: max_allowed_jet,
                    });
                }
            }
        }

        // 4. Çizgisel Momentum Korunumu: Tepkisiz İtki (Reactionless Drive) Budaması
        if let (Some(mdot), Some(thrust)) = (cand.mass_flow_kg_s, cand.thrust_n) {
            if mdot.is_finite() && thrust.is_finite() {
                let abs_f = thrust.abs();
                if mdot.abs() <= 1e-15 && abs_f > 0.0 {
                    let p_in = cand.power_in_w.unwrap_or(0.0).max(0.0);
                    let max_photon_thrust = (2.0 * p_in) / PhysicalConstants::C;
                    if abs_f > max_photon_thrust * (1.0 + 1e-6) + 1e-15 {
                        violations.push(PhysicalLawViolation::ReactionlessMomentumViolation {
                            thrust_n: abs_f,
                            max_photon_thrust_n: max_photon_thrust,
                        });
                    }
                }
            }
        }

        violations
    }

    /// Bir sistem adayını AGENTS.md Değişmez 2 kapsamındaki 4 temel doğa yasasına karşı denetler.
    pub fn verify_candidate(cand: &PhysicalSystemCandidate) -> Result<(), PhysicalLawViolation> {
        let mut violations = Self::collect_violations(cand);
        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subluminal_lorentz_exact_and_causality_rejection() {
        let g06 = PhysicalBoundsShield::check_subluminal_velocity(0.6 * PhysicalConstants::C).unwrap();
        assert!((g06 - 1.25).abs() < 1e-12);
        let g08 = PhysicalBoundsShield::check_subluminal_velocity(0.8 * PhysicalConstants::C).unwrap();
        assert!((g08 - (5.0 / 3.0)).abs() < 1e-12);
        assert!(matches!(
            PhysicalBoundsShield::check_subluminal_velocity(PhysicalConstants::C),
            Err(PhysicalLawViolation::SuperluminalCausalityViolation { .. })
        ));
        assert!(matches!(
            PhysicalBoundsShield::check_subluminal_velocity(f64::NAN),
            Err(PhysicalLawViolation::NonPhysicalState(_))
        ));
    }

    #[test]
    fn test_carnot_bound_and_first_law_enforcement() {
        let ok_engine = PhysicalSystemCandidate {
            temp_hot_k: Some(600.0),
            temp_cold_k: Some(300.0),
            efficiency: Some(0.49),
            power_in_w: Some(1000.0),
            power_out_w: Some(490.0),
            ..Default::default()
        };
        assert!(PhysicalBoundsShield::verify_candidate(&ok_engine).is_ok());

        let bad_carnot = PhysicalSystemCandidate {
            temp_hot_k: Some(600.0),
            temp_cold_k: Some(300.0),
            efficiency: Some(0.55), // > 0.50 Carnot
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&bad_carnot),
            Err(PhysicalLawViolation::CarnotBoundViolation { .. })
        ));

        let perpetual = PhysicalSystemCandidate {
            power_in_w: Some(1000.0),
            power_out_w: Some(1200.0),
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&perpetual),
            Err(PhysicalLawViolation::FirstLawEnergyViolation { .. })
        ));
    }

    #[test]
    fn test_reactionless_drive_rejected_but_ideal_photon_mirror_passes() {
        let p_in = 1.0e6; // 1 MW
        let f_photon_max = (2.0 * p_in) / PhysicalConstants::C;
        let photon_sail = PhysicalSystemCandidate {
            mass_flow_kg_s: Some(0.0),
            power_in_w: Some(p_in),
            thrust_n: Some(f_photon_max * 0.99),
            ..Default::default()
        };
        assert!(PhysicalBoundsShield::verify_candidate(&photon_sail).is_ok());

        let emdrive = PhysicalSystemCandidate {
            mass_flow_kg_s: Some(0.0),
            power_in_w: Some(p_in),
            thrust_n: Some(1.0), // 1 N >> 6.67 mN photon limit!
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&emdrive),
            Err(PhysicalLawViolation::ReactionlessMomentumViolation { .. })
        ));
    }

    #[test]
    fn test_collect_violations_reports_all_simultaneous_breaches() {
        let impossible = PhysicalSystemCandidate {
            mass_kg: Some(10.0),
            velocity_ms: Some(1.2 * PhysicalConstants::C),
            entropy_delta_j_k: Some(-5.0),
            temp_hot_k: Some(600.0),
            temp_cold_k: Some(300.0),
            efficiency: Some(0.80),
            power_in_w: Some(100.0),
            power_out_w: Some(500.0),
            ..Default::default()
        };
        let v = PhysicalBoundsShield::collect_violations(&impossible);
        assert_eq!(v.len(), 4, "Tüm 4 eşzamanlı doğa yasası ihlali toplanmalıydı: {:?}", v);
    }
}
