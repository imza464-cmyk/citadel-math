// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Tekil Kaynaklı (SSOT JSON) Malzeme Bilimi, NASA SP-8007 + Euler Burkulma ve Ablatif TPS Motoru

use crate::constants::PhysicalConstants;
use crate::simulator::AtmosphereModel;
use serde::Deserialize;
use std::f64::consts::PI;
use std::fs;
use std::path::Path;

const EMBEDDED_MATERIALS_JSON: &str = include_str!("../../database/materials.json");

#[derive(Debug, Clone, Deserialize)]
struct RawMaterialJson {
    id: String,
    name: String,
    category: String,
    density_kg_m3: f64,
    tensile_strength_mpa: f64,
    yield_strength_mpa: f64,
    youngs_modulus_gpa: f64,
    max_temperature_k: f64,
    thermal_conductivity_w_mk: f64,
    description: String,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub id: String,
    pub name: String,
    pub category: String,
    pub density: f64,              // kg/m^3
    pub tensile_strength: f64,     // Pa
    pub yield_strength: f64,       // Pa
    pub youngs_modulus: f64,       // Pa
    pub max_temperature_k: f64,    // K
    pub thermal_conductivity: f64, // W/(m*K)
    pub description: String,
}

impl Material {
    pub fn specific_strength(&self) -> f64 {
        if self.density > 0.0 {
            self.yield_strength / self.density
        } else {
            0.0
        }
    }

    pub fn breaking_length_km(&self) -> f64 {
        if self.density > 0.0 {
            (self.yield_strength / (self.density * PhysicalConstants::G0)) / 1000.0
        } else {
            0.0
        }
    }

    /// Faz 4 (Öz-Eleştiri Düzeltmesi #6): Sıcaklığa bağlı termal yumuşama (Thermal Derating - MMPDS / MIL-HDBK-5):
    /// Gövde sıcaklığı 300 K (27 °C) üzerine çıktıkça `max_temperature_k` sınırına doğru
    /// akma dayanımı σ_y(T) (%35'e kadar) ve Young modülü E(T) (%25'e kadar) doğrusal olmayan biçimde düşer;
    /// `temp_k > max_temperature_k` durumunda malzeme yapısal bütünlüğünü kaybeder (0.0).
    pub fn derated_properties_at_temp_k(&self, temp_k: f64) -> (f64, f64) {
        if temp_k > self.max_temperature_k {
            return (0.0, 0.0);
        }
        if temp_k <= 300.0 || self.max_temperature_k <= 305.0 {
            return (self.yield_strength, self.youngs_modulus);
        }
        let theta = ((temp_k - 300.0) / (self.max_temperature_k - 300.0)).clamp(0.0, 1.0);
        let yield_factor = 1.0 - 0.35 * theta.powf(1.5);
        let modulus_factor = 1.0 - 0.25 * theta.powf(1.5);
        (
            self.yield_strength * yield_factor,
            self.youngs_modulus * modulus_factor,
        )
    }
}

pub struct MaterialsDatabase {
    pub list: Vec<Material>,
}

impl MaterialsDatabase {
    pub fn load_canonical() -> Self {
        let manifest_db = Path::new(env!("CARGO_MANIFEST_DIR")).join("../database/materials.json");
        let cwd_db = Path::new("database/materials.json");
        let json_content = if manifest_db.exists() {
            fs::read_to_string(&manifest_db).unwrap_or_else(|_| EMBEDDED_MATERIALS_JSON.to_string())
        } else if cwd_db.exists() {
            fs::read_to_string(cwd_db).unwrap_or_else(|_| EMBEDDED_MATERIALS_JSON.to_string())
        } else {
            EMBEDDED_MATERIALS_JSON.to_string()
        };
        let raw_list: Vec<RawMaterialJson> =
            serde_json::from_str(&json_content).expect("materials.json ayrıştırma hatası");

        let list = raw_list
            .into_iter()
            .map(|r| Material {
                id: r.id,
                name: r.name,
                category: r.category,
                density: r.density_kg_m3,
                tensile_strength: r.tensile_strength_mpa * 1e6,
                yield_strength: r.yield_strength_mpa * 1e6,
                youngs_modulus: r.youngs_modulus_gpa * 1e9,
                max_temperature_k: r.max_temperature_k,
                thermal_conductivity: r.thermal_conductivity_w_mk,
                description: r.description,
            })
            .collect();
        Self { list }
    }

    pub fn get(&self, id: &str) -> Option<&Material> {
        self.list.iter().find(|m| m.id == id)
    }

    pub fn rank_by_breaking_length(&self) -> Vec<&Material> {
        let mut sorted: Vec<&Material> = self.list.iter().collect();
        sorted.sort_by(|a, b| {
            b.breaking_length_km()
                .total_cmp(&a.breaking_length_km())
        });
        sorted
    }
}

/// Ablatif Isı Kalkanı (TPS — PICA-X sınıfı Fenolik Karbon) boyutlandırma sonucu
#[derive(Debug, Clone)]
pub struct TpsShieldResult {
    pub bare_wall_temp_k: f64,
    pub bare_survives: bool,
    pub convective_flux_kw_m2: f64,
    pub total_heat_load_mj_m2: f64,
    pub tps_required: bool,
    pub tps_thickness_mm: f64,
    pub tps_mass_kg: f64,
    pub protected_bondline_temp_k: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct CylinderGeometry {
    pub radius_m: f64,
    pub length_m: f64,
    pub wall_thickness_mm: f64,
}

pub struct StructuralStressChecker;

impl StructuralStressChecker {
    pub const SAFETY_FACTOR: f64 = 1.35;
    pub const SUTTON_GRAVES_K: f64 = 1.7415e-4; // [kg^0.5 / m] Dünya atmosferi sabiti

    /// İnce cidarlı silindirik roket gövdesinde (Thin-Walled Cylinder):
    /// - Eksenel basma kuvveti: F_axial = m * g_load * g0 + q_max * C_d * (pi * r^2)
    /// - Eksenel gerilme (basma, negatif): sigma_z = - F_axial / (2 * pi * r * t)
    /// - Çevresel (Hoop) gerilme (çekme, pozitif): sigma_theta = (P_int * r) / t
    /// - Düzlem gerilme Von Mises: sigma_VM = sqrt(sigma_z^2 - sigma_z * sigma_theta + sigma_theta^2)
    pub fn check_hull(
        material: &Material,
        total_mass_kg: f64,
        g_force: f64,
        max_q_pa: f64,
        hull_radius_m: f64,
        wall_thickness_mm: f64,
    ) -> (f64, f64, bool) {
        Self::check_hull_at_temp(
            material,
            total_mass_kg,
            g_force,
            max_q_pa,
            hull_radius_m,
            wall_thickness_mm,
            293.15,
        )
    }

    /// Termal yumuşama (σ_y(T)) etkisini hesaba katan iki eksenli Von Mises ince cidar denetimi.
    pub fn check_hull_at_temp(
        material: &Material,
        total_mass_kg: f64,
        g_force: f64,
        max_q_pa: f64,
        hull_radius_m: f64,
        wall_thickness_mm: f64,
        wall_temp_k: f64,
    ) -> (f64, f64, bool) {
        let t_m = wall_thickness_mm / 1000.0;
        let r_m = hull_radius_m;

        let a_shell = 2.0 * PI * r_m * t_m;
        let a_frontal = PI * r_m * r_m;
        let cd = 0.30;

        let inertial_force = total_mass_kg * g_force * PhysicalConstants::G0;
        let aero_drag_force = max_q_pa * cd * a_frontal;
        let total_axial_compression = inertial_force + aero_drag_force;

        let sigma_z = if a_shell > 0.0 {
            -(total_axial_compression / a_shell)
        } else {
            0.0
        };

        let p_int = 3.0 * 100_000.0; // 3 bar standart tank basıncı
        let sigma_theta = if t_m > 0.0 { (p_int * r_m) / t_m } else { 0.0 };

        // İki eksenli Von Mises Eşdeğer Gerilmesi
        let sigma_vm = (sigma_z.powi(2) - sigma_z * sigma_theta + sigma_theta.powi(2)).sqrt();

        let (derated_yield, _) = material.derated_properties_at_temp_k(wall_temp_k);
        let allowable = derated_yield / Self::SAFETY_FACTOR;
        let safety_achieved = if sigma_vm > 0.0 {
            derated_yield / sigma_vm
        } else {
            999.0
        };
        let survives = derated_yield > 0.0 && sigma_vm <= allowable;

        (sigma_vm, safety_achieved, survives)
    }

    /// Birleşik NASA SP-8007 Yerel Kabuk Burkulması + Küresel Euler Kolon Burkulması + Aerodinamik Eğilme Momenti:
    /// 1. Eksenel basma + Max-Q hücum açısı (alpha = 2.5°) eğilme momenti gerilmesi:
    ///    M_bend = F_normal * (0.25 * L), I = pi * r^3 * t, sigma_bend = M_bend * r / I
    ///    sigma_comp_peak = sigma_axial + sigma_bend
    /// 2. NASA SP-8007 Yerel Kabuk Burkulması:
    ///    gamma = 1 - 0.901 * (1 - exp(-(1/16) * sqrt(r / t)))
    ///    sigma_cr_shell = 1.45 * gamma * (E / sqrt(3*(1 - nu^2))) * (t / r) + 0.50 * (P_int * r) / (2 * t)
    ///    (1.45x ortogrid/halka takviye katsayısı, %50 konservatif iç basınç stabilizasyonu)
    /// 3. Küresel Euler Kolon Burkulması (Boy L etkisini doğrudan denetler):
    ///    sigma_cr_euler = (pi^2 * E * r^2) / (2 * (K * L)^2)  (K = 0.80)
    /// 4. Narinlik Oranı Sınırı (Slenderness L / D <= 18.0): Aşıldığında aeroelastik kırbaçlanma nedeniyle reddedilir.
    pub fn check_cylinder_buckling(
        material: &Material,
        total_mass_kg: f64,
        g_force: f64,
        max_q_pa: f64,
        hull_radius_m: f64,
        hull_length_m: f64,
        wall_thickness_mm: f64,
    ) -> (f64, f64, bool) {
        Self::check_cylinder_buckling_at_temp(
            material,
            total_mass_kg,
            g_force,
            max_q_pa,
            CylinderGeometry {
                radius_m: hull_radius_m,
                length_m: hull_length_m,
                wall_thickness_mm,
            },
            293.15,
        )
    }

    /// Termal yumuşama (E(T)) etkisini hesaba katan birleşik NASA SP-8007 + Euler kolon burkulma denetimi.
    pub fn check_cylinder_buckling_at_temp(
        material: &Material,
        total_mass_kg: f64,
        g_force: f64,
        max_q_pa: f64,
        geom: CylinderGeometry,
        wall_temp_k: f64,
    ) -> (f64, f64, bool) {
        let t_m = (geom.wall_thickness_mm / 1000.0).max(1e-5);
        let r_m = geom.radius_m.max(0.05);
        let l_m = geom.length_m.max(1.0);
        let nu = 0.30_f64;

        let (_, derated_e) = material.derated_properties_at_temp_k(wall_temp_k);
        if derated_e <= 0.0 {
            return (0.0, 0.0, false);
        }

        // Narinlik (L / D) kontrolü
        let aspect_ratio = l_m / (2.0 * r_m);
        if aspect_ratio > 18.0 {
            return (0.0, 0.10, false);
        }

        let a_shell = 2.0 * PI * r_m * t_m;
        let a_frontal = PI * r_m * r_m;
        let cd = 0.30;

        let inertial_force = total_mass_kg * g_force * PhysicalConstants::G0;
        let aero_drag_force = max_q_pa * cd * a_frontal;
        let sigma_axial = (inertial_force + aero_drag_force) / a_shell;

        // Max-Q aerodinamik yanal yük ve eğilme momenti (alpha ~ 2.5°, C_N_alpha ~ 2.0 -> C_N ~ 0.087)
        let f_normal = max_q_pa * 0.087 * a_frontal;
        let m_bend = f_normal * (0.25 * l_m);
        let i_moment = PI * r_m.powi(3) * t_m;
        let sigma_bend = (m_bend * r_m) / i_moment;

        let sigma_comp_peak = sigma_axial + sigma_bend;

        // 1. NASA SP-8007 (1968 / 2020 REV2) Yerel Kabuk Burkulması:
        //    - Eksenel basma kusur düşürme (Eq. 11): gamma_axial = 1 - 0.901 * (1 - exp(-phi))
        //    - Saf eğilme kusur düşürme (Eq. 13):    gamma_bend  = 1 - 0.731 * (1 - exp(-phi)),  phi = sqrt(R/t)/16
        //    - Birleşik etkileşim (Eq. 17):          R_comb = sigma_axial / sigma_cr_axial + sigma_bend / sigma_cr_bend <= 1
        let phi = (r_m / t_m).sqrt() / 16.0;
        let exp_neg_phi = (-phi).exp();
        let gamma_axial = 1.0 - 0.901 * (1.0 - exp_neg_phi);
        let gamma_bend = 1.0 - 0.731 * (1.0 - exp_neg_phi);
        let denom = (3.0 * (1.0 - nu * nu)).sqrt();
        let sigma_cl_orthogrid = 1.45 * (derated_e / denom) * (t_m / r_m);

        let p_int = 3.0 * 100_000.0; // 3 bar iç basınç (%50 konservatif eksenel rahatlama)
        let sigma_press_relief = 0.50 * (p_int * r_m) / (2.0 * t_m);
        let sigma_cr_axial = gamma_axial * sigma_cl_orthogrid + sigma_press_relief;
        let sigma_cr_bend = gamma_bend * sigma_cl_orthogrid + sigma_press_relief;

        let sf_local_sp8007 = if sigma_comp_peak > 0.0 {
            let interaction = (sigma_axial / sigma_cr_axial) + (sigma_bend / sigma_cr_bend);
            if interaction > 0.0 {
                1.0 / interaction
            } else {
                999.0
            }
        } else {
            999.0
        };

        // 2. Küresel Euler Kolon Burkulması: F_cr = pi^2 * E(T) * I / (K * L)^2
        let k_eff = 0.80_f64;
        let sigma_cr_euler = (PI * PI * derated_e * r_m * r_m)
            / (2.0 * (k_eff * l_m).powi(2));
        let sf_euler = if sigma_comp_peak > 0.0 {
            sigma_cr_euler / sigma_comp_peak
        } else {
            999.0
        };

        let buckling_sf = sf_local_sp8007.min(sf_euler);
        let sigma_cr_governing = if sigma_comp_peak > 0.0 {
            buckling_sf * sigma_comp_peak
        } else {
            sigma_cr_axial.min(sigma_cr_euler)
        };
        let survives = buckling_sf >= Self::SAFETY_FACTOR;
        (sigma_cr_governing, buckling_sf, survives)
    }

    /// Sutton-Graves konvektif ısı akısı ve Stefan-Boltzmann radyatif denge hesabı (Tekil hız testi için).
    pub fn check_heating(material: &Material, max_speed_ms: f64) -> (f64, bool) {
        let res = Self::size_ablative_tps(material, max_speed_ms, 0.5, 45.0);
        (res.bare_wall_temp_k, res.bare_survives)
    }

    /// RK4 yörünge entegrasyonundan gelen gerçek zaman-integralli ısı yükü (Q_total = ∫ q_conv(t) dt [MJ/m^2])
    /// ve gerçek pik şok sıcaklığı (peak_wall_temp_k) ile PICA-X Ablatif Isı Kalkanı boyutlandırır.
    pub fn size_ablative_tps_from_trajectory(
        material: &Material,
        hull_radius_m: f64,
        peak_wall_temp_k: f64,
        peak_heat_flux_kw_m2: f64,
        total_heat_load_mj_m2: f64,
        heating_duration_s: f64,
    ) -> TpsShieldResult {
        let bare_survives = peak_wall_temp_k < material.max_temperature_k;
        if bare_survives {
            return TpsShieldResult {
                bare_wall_temp_k: peak_wall_temp_k,
                bare_survives: true,
                convective_flux_kw_m2: peak_heat_flux_kw_m2,
                total_heat_load_mj_m2,
                tps_required: false,
                tps_thickness_mm: 0.0,
                tps_mass_kg: 0.0,
                protected_bondline_temp_k: peak_wall_temp_k,
            };
        }

        // PICA-X (Phenolic Impregnated Carbon Ablator) Fiziksel Sabitleri:
        let rho_tps = 270.0_f64; // kg/m^3
        let h_eff = 2.5e7_f64;   // J/kg efektif ablasyon entalpisi
        let k_tps = 0.060_f64;   // W/(m*K) ısıl iletkenlik
        let cp_tps = 1260.0_f64; // J/(kg*K) özgül ısı
        let alpha_tps = k_tps / (rho_tps * cp_tps);

        // 1. Ablasyon kalınlığı: doğrudan RK4 ısı integrali Q_total (J/m^2) / (rho_tps * H_eff)
        let q_total_j_m2 = total_heat_load_mj_m2 * 1e6;
        let d_ablation_m = q_total_j_m2 / (rho_tps * h_eff);

        // 2. İletim (conduction soak) yalıtım kalınlığı
        let target_bondline_k = (material.max_temperature_k - 40.0).max(310.0);
        let delta_t_surface = (peak_wall_temp_k - 290.0).max(80.0);
        let delta_t_allow = (target_bondline_k - 290.0).max(20.0);
        let temp_ratio = (delta_t_surface / delta_t_allow).max(1.15).ln();
        let eff_soak_time = heating_duration_s.clamp(30.0, 140.0);
        let d_insul_m = 1.65 * (alpha_tps * eff_soak_time).sqrt() * temp_ratio;

        let total_tps_m = ((d_ablation_m + d_insul_m) * 1.25).max(0.004);
        let shield_area_m2 = 2.4 * PI * hull_radius_m * hull_radius_m;
        let tps_mass_kg = shield_area_m2 * total_tps_m * rho_tps;

        TpsShieldResult {
            bare_wall_temp_k: peak_wall_temp_k,
            bare_survives: false,
            convective_flux_kw_m2: peak_heat_flux_kw_m2,
            total_heat_load_mj_m2,
            tps_required: true,
            tps_thickness_mm: total_tps_m * 1000.0,
            tps_mass_kg,
            protected_bondline_temp_k: target_bondline_k,
        }
    }

    /// Tekil hız değerinden USSA-1976 hidrostatik atmosfer katmanı üzerinden TPS boyutlandırıcı (statik stres testi komutu için).
    pub fn size_ablative_tps(
        material: &Material,
        max_speed_ms: f64,
        hull_radius_m: f64,
        heating_duration_s: f64,
    ) -> TpsShieldResult {
        let (v_eff, ref_alt_m) = if max_speed_ms > 4000.0 {
            (max_speed_ms * 0.42, 45_000.0)
        } else {
            (max_speed_ms, 15_000.0)
        };
        let (rho_eff, _p_pa, ambient_temp, speed_of_sound) = AtmosphereModel::state_at(ref_alt_m);

        let mach = v_eff / speed_of_sound;
        let t_stag = ambient_temp * (1.0 + 0.2 * mach.powi(2));
        let nose_radius_m = (hull_radius_m * 0.45).max(0.15);
        let emissivity = 0.85_f64;
        let q_conv = Self::SUTTON_GRAVES_K * (rho_eff / nose_radius_m).sqrt() * v_eff.powi(3);
        let t_rad_eq = ((q_conv / (emissivity * PhysicalConstants::SIGMA_SB)) + ambient_temp.powi(4)).powf(0.25);
        let bare_wall_temp_k = t_stag.min(t_rad_eq);
        let total_heat_mj = (q_conv * 0.55 * heating_duration_s) / 1e6;

        Self::size_ablative_tps_from_trajectory(
            material,
            hull_radius_m,
            bare_wall_temp_k,
            q_conv / 1000.0,
            total_heat_mj,
            heating_duration_s,
        )
    }
}
