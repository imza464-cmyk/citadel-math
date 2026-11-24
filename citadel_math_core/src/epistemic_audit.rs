// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — 4-Katmanlı Epistemik Öz-Doğrulama Kalkanı:
//   Kapı 17: Altın Sayısal Referans Çapaları (Reference Anchors — Boyutsuz Katsayı Kalkanı)
//   Kapı 18: Çapraz-Yasa Cebirsel Kapanış Özdeşlikleri (Cross-Law Physics Closures)
//   Kapı 19: Asimptotik Klasik Limit & Dual64 Fiziksel Monotonluk Süzgeci
//   Kapı 20: Malzeme Ashby Kategori Fiziksel Zarfları, Isıl İşlem/Kırılma Modu & 3-Kaynak Uzlaşma Kalkanı

use crate::constants::PhysicalConstants;
use crate::graph::KnowledgeGraph;
use crate::materials::{Material, MaterialsDatabase};
use serde::Deserialize;
use std::collections::HashMap;
use std::f64::consts::PI;

const EMBEDDED_LAWS_JSON: &str = include_str!("../../database/canonical_laws.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ReferenceAnchor {
    pub label: String,
    pub inputs: HashMap<String, f64>,
    pub expected: f64,
    #[serde(default = "default_rel_tol")]
    pub rel_tol: f64,
    #[serde(default)]
    pub source: String,
}

fn default_rel_tol() -> f64 {
    1e-4
}

#[derive(Debug, Clone, Deserialize)]
struct LawWithAnchors {
    id: String,
    #[serde(default)]
    reference_checks: Vec<ReferenceAnchor>,
}

#[derive(Debug, Clone)]
pub struct EpistemicAuditReport {
    pub reference_anchors_checked: usize,
    pub cross_law_closures_checked: usize,
    pub asymptotic_and_monotonicity_checked: usize,
    pub materials_ashby_checked: usize,
}

/// Kapı 17: Kanonik yasalardaki tüm altın sayısal referans çapalarını (`reference_checks`)
/// değerlendirerek boyutsuz katsayı hatalarını (örn. 6π vs 4π, 1/4 vs π, 1/3 vs 1/4) yakalar.
pub fn verify_reference_anchors(kg: &KnowledgeGraph) -> Result<usize, String> {
    let manifest_db = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../database/canonical_laws.json");
    let cwd_db = std::path::Path::new("database/canonical_laws.json");
    let json_content = if manifest_db.exists() {
        std::fs::read_to_string(&manifest_db).unwrap_or_else(|_| EMBEDDED_LAWS_JSON.to_string())
    } else if cwd_db.exists() {
        std::fs::read_to_string(cwd_db).unwrap_or_else(|_| EMBEDDED_LAWS_JSON.to_string())
    } else {
        EMBEDDED_LAWS_JSON.to_string()
    };

    let raw_laws: Vec<LawWithAnchors> = serde_json::from_str(&json_content)
        .map_err(|e| format!("reference_checks JSON ayrıştırma hatası: {}", e))?;

    let mut checked = 0usize;
    for item in &raw_laws {
        if item.reference_checks.is_empty() {
            continue;
        }
        let law = kg
            .laws
            .get(&item.id)
            .ok_or_else(|| format!("Referans çapalı yasa graf içinde bulunamadı: '{}'", item.id))?;
        for anchor in &item.reference_checks {
            let env: HashMap<&str, f64> = anchor
                .inputs
                .iter()
                .map(|(k, &v)| (k.as_str(), v))
                .collect();
            let actual = law.rhs.evaluate(&env).map_err(|e| {
                format!(
                    "Referans çapa değerlendirme hatası [{} / {}]: {}",
                    item.id, anchor.label, e
                )
            })?;
            let denom = anchor.expected.abs().max(1e-30);
            let rel_err = (actual - anchor.expected).abs() / denom;
            if !actual.is_finite() || rel_err > anchor.rel_tol {
                return Err(format!(
                    "ALTIN REFERANS ÇAPA İHLALİ [{} | {}]: hesaplanan={:.12e}, beklenen={:.12e}, bağıl_hata={:.3e} > tol={:.3e} (kaynak: {})",
                    item.id, anchor.label, actual, anchor.expected, rel_err, anchor.rel_tol, anchor.source
                ));
            }
            checked += 1;
        }
    }

    if checked < 35 {
        return Err(format!(
            "Yetersiz altın referans çapa kapsamı: {} (en az 35 bekleniyor)",
            checked
        ));
    }
    Ok(checked)
}

fn eval_law(kg: &KnowledgeGraph, id: &str, pairs: &[(&str, f64)]) -> Result<f64, String> {
    let law = kg
        .laws
        .get(id)
        .ok_or_else(|| format!("Çapraz-yasa denetimi için yasa bulunamadı: '{}'", id))?;
    let env: HashMap<&str, f64> = pairs.iter().copied().collect();
    law.rhs
        .evaluate(&env)
        .map_err(|e| format!("Yasa '{}' değerlendirme hatası: {}", id, e))
}

fn check_rel(name: &str, lhs: f64, rhs: f64, tol: f64) -> Result<(), String> {
    let denom = rhs.abs().max(lhs.abs()).max(1e-30);
    let err = (lhs - rhs).abs() / denom;
    if !lhs.is_finite() || !rhs.is_finite() || err > tol {
        return Err(format!(
            "ÇAPRAZ-YASA KAPANIŞ İHLALİ [{}]: lhs={:.12e}, rhs={:.12e}, bağıl_hata={:.3e} > tol={:.3e}",
            name, lhs, rhs, err, tol
        ));
    }
    Ok(())
}

/// Kapı 18: 14 bağımsız fiziksel çapraz-yasa cebirsel kapanış özdeşliğini doğrular.
/// Tek bir yasada boyutsuz katsayı (örn. 8π, 1/4, 1/3, 1/48, 2/3, 4/3) bozulursa kapanış anında kırılır.
pub fn verify_cross_law_closures(kg: &KnowledgeGraph) -> Result<usize, String> {
    let c = PhysicalConstants::C;
    let g = PhysicalConstants::G;
    let h = PhysicalConstants::H;
    let hbar = PhysicalConstants::HBAR;
    let kb = PhysicalConstants::K_B;
    let qe = PhysicalConstants::E;
    let mu0 = PhysicalConstants::MU_0;
    let eps0 = PhysicalConstants::EPS_0;
    let g0 = PhysicalConstants::G0;
    let r_univ = PhysicalConstants::R_GAS;

    // 1. Kara Delik Termodinamiğinin 1. Yasası: T_H * (dS_BH / dM) == c^2
    //    (r_s = 2GM/c^2, A = 4*pi*r_s^2 => dA/dM = 32*pi*G^2*M/c^4 => dS/dM = 8*pi*G*M*k_B/(hbar*c))
    let m_bh = 1.98847e30; // 1 Güneş kütlesi
    let r_s = eval_law(kg, "law_schwarzschild_radius", &[("G_grav", g), ("M_body", m_bh), ("c", c)])?;
    let a_bh = 4.0 * PI * r_s * r_s;
    let s_bh = eval_law(
        kg,
        "law_bekenstein_hawking_entropy",
        &[("k_B", kb), ("c", c), ("A", a_bh), ("G", g), ("hbar", hbar)],
    )?;
    let t_h = eval_law(
        kg,
        "law_hawking_black_hole_temperature",
        &[("hbar", hbar), ("c", c), ("G", g), ("M_body", m_bh), ("k_B", kb)],
    )?;
    // S_bh ~ M^2 olduğundan dS_bh/dM = 2 * S_bh / M; dolayısıyla T_H * (2 * S_bh / M) == c^2
    check_rel("BH_FirstLaw_T_H*dS/dM==c^2", t_h * (2.0 * s_bh / m_bh), c * c, 1e-11)?;

    // 2. İzotropik Elastisite Üçgeni: K_bulk == lambda_lame + (2/3) * G_shear
    let e_mod = 210.0e9;
    let nu = 0.29;
    let rho_s = 7850.0;
    let k_bulk = eval_law(kg, "law_bulk_modulus_isotropic", &[("E_mod", e_mod), ("nu_pois", nu)])?;
    let lam = eval_law(kg, "law_lame_first_parameter", &[("E_mod", e_mod), ("nu_pois", nu)])?;
    let g_sh = eval_law(kg, "law_isotropic_shear_modulus", &[("E_mod", e_mod), ("nu_pois", nu)])?;
    check_rel("Elasticity_K==lambda+2/3*G", k_bulk, lam + (2.0 / 3.0) * g_sh, 1e-12)?;

    // 3. Sismik Dalga Özdeşliği: v_P^2 == (K + 4/3*G)/rho ve v_P > sqrt(2)*v_S
    let v_p = eval_law(
        kg,
        "law_elastic_p_wave_velocity",
        &[("E_mod", e_mod), ("nu_pois", nu), ("rho", rho_s)],
    )?;
    let v_s = eval_law(kg, "law_elastic_s_wave_velocity", &[("G_shear", g_sh), ("rho", rho_s)])?;
    check_rel("Seismic_vP^2==(K+4/3*G)/rho", v_p * v_p, (k_bulk + (4.0 / 3.0) * g_sh) / rho_s, 1e-12)?;
    if v_p <= 2.0_f64.sqrt() * v_s {
        return Err(format!("Sismik hız eşitsizliği ihlali: v_P ({}) <= sqrt(2)*v_S ({})", v_p, 2.0_f64.sqrt() * v_s));
    }

    // 4. Carnot Isı Pompası / Soğutucu / Verim Kapanışı: COP_hp - COP_ref == 1 ve eta * COP_hp == 1
    let t_h_c = 600.0;
    let t_c_c = 300.0;
    let eta_c = eval_law(kg, "law_carnot_thermal_efficiency", &[("T_h", t_h_c), ("T_c", t_c_c)])?;
    let cop_r = eval_law(kg, "law_carnot_refrigerator_cop", &[("T_h", t_h_c), ("T_c", t_c_c)])?;
    let cop_hp = eval_law(kg, "law_carnot_heat_pump_cop", &[("T_h", t_h_c), ("T_c", t_c_c)])?;
    check_rel("Carnot_COPhp-COPr==1", cop_hp - cop_r, 1.0, 1e-12)?;
    check_rel("Carnot_eta*COPhp==1", eta_c * cop_hp, 1.0, 1e-12)?;

    // 5. Maxwell Vakum Elektromanyetik Üçlüsü: Z_0 * c_em * eps_0 == 1 ve c_em == c
    let z0 = eval_law(kg, "law_free_space_wave_impedance", &[("mu_0", mu0), ("eps_0", eps0)])?;
    let c_em = eval_law(kg, "law_speed_of_light_maxwell", &[("mu_0", mu0), ("eps_0", eps0)])?;
    check_rel("Maxwell_Z0*c*eps0==1", z0 * c_em * eps0, 1.0, 1e-12)?;
    check_rel("Maxwell_c_em==CODATA_c", c_em, c, 2e-10)?;

    // 6. Planck Ölçek Üçlüsü: l_P / t_P == c ve m_P * c^2 * t_P == hbar
    let lp = eval_law(kg, "law_planck_length", &[("hbar", hbar), ("G", g), ("c", c)])?;
    let mp = eval_law(kg, "law_planck_mass", &[("hbar", hbar), ("c", c), ("G", g)])?;
    let tp = eval_law(kg, "law_planck_time", &[("hbar", hbar), ("G", g), ("c", c)])?;
    check_rel("Planck_lP/tP==c", lp / tp, c, 1e-12)?;
    check_rel("Planck_mP*c^2*tP==hbar", mp * c * c * tp, hbar, 1e-12)?;

    // 7. Maxwell-Boltzmann Hız Oranı: v_rms / v_mp == sqrt(3/2)
    let v_rms = eval_law(
        kg,
        "law_maxwell_boltzmann_rms_speed",
        &[("R_univ", r_univ), ("T", 300.0), ("M_mol", 0.0289647)],
    )?;
    let v_mp = eval_law(
        kg,
        "law_maxwell_boltzmann_most_probable_speed",
        &[("R_univ", r_univ), ("T", 300.0), ("M_mol", 0.0289647)],
    )?;
    check_rel("MaxwellBoltzmann_vrms/vmp==sqrt(1.5)", v_rms / v_mp, 1.5_f64.sqrt(), 1e-12)?;

    // 8. Boyutsuz Akışkan Sayıları Kapanışı: Pe == Re * Pr ve We == Ca * Re
    let (rho_f, v_f, l_f, mu_f, gamma_f, cp_f, k_f) = (1000.0, 2.5, 0.2, 1.0e-3, 0.0728, 4184.0, 0.6);
    let re = eval_law(kg, "law_reynolds_number", &[("rho", rho_f), ("v", v_f), ("L_char", l_f), ("mu_visc", mu_f)])?;
    let pr = eval_law(kg, "law_prandtl_number", &[("c_p", cp_f), ("mu_visc", mu_f), ("k_th", k_f)])?;
    let alpha_th = eval_law(kg, "law_thermal_diffusivity", &[("k_th", k_f), ("rho", rho_f), ("c_p", cp_f)])?;
    let pe = eval_law(kg, "law_peclet_number_thermal", &[("v", v_f), ("L_char", l_f), ("alpha_th", alpha_th)])?;
    let we = eval_law(kg, "law_weber_number", &[("rho", rho_f), ("v", v_f), ("L_char", l_f), ("gamma_surf", gamma_f)])?;
    let ca = eval_law(kg, "law_capillary_number", &[("mu_visc", mu_f), ("v", v_f), ("gamma_surf", gamma_f)])?;
    check_rel("Fluids_Pe==Re*Pr", pe, re * pr, 1e-12)?;
    check_rel("Fluids_We==Ca*Re", we, ca * re, 1e-12)?;

    // 9. Darcy-Weisbach Basınç-Yük Eşliği: delta_P_dw == rho * g0 * h_loss
    let dp_dw = eval_law(
        kg,
        "law_darcy_weisbach_pressure_drop",
        &[("f_darcy", 0.02), ("L", 50.0), ("D_pipe", 0.1), ("rho", rho_f), ("v", v_f)],
    )?;
    let h_dw = eval_law(
        kg,
        "law_darcy_weisbach_head_loss",
        &[("f_darcy", 0.02), ("L", 50.0), ("D_pipe", 0.1), ("v", v_f), ("g_0", g0)],
    )?;
    check_rel("DarcyWeisbach_dP==rho*g0*h", dp_dw, rho_f * g0 * h_dw, 1e-12)?;

    // 10. Hagen-Poiseuille Basınç-Debi İkili Özdeşliği: Q(dP(Q)) == Q
    let q_in = 0.0025;
    let dp_hp = eval_law(
        kg,
        "law_hagen_poiseuille_laminar_pressure_drop",
        &[("mu_visc", mu_f), ("L_char", 10.0), ("Q_vol", q_in), ("r", 0.02)],
    )?;
    let q_back = eval_law(
        kg,
        "law_poiseuille_volumetric_flow",
        &[("r", 0.02), ("delta_P_pipe", dp_hp), ("mu_visc", mu_f), ("L_char", 10.0)],
    )?;
    check_rel("Poiseuille_Q(dP(Q))==Q", q_back, q_in, 1e-12)?;

    // 11. Akustik Empedans-Şiddet Özdeşliği: I_ac * Z_ac == P_rms^2
    let p_rms = 20.0;
    let z_ac = eval_law(kg, "law_specific_acoustic_impedance", &[("rho", 1.225), ("c_s", 343.0)])?;
    let i_ac = eval_law(kg, "law_acoustic_plane_wave_intensity", &[("P_rms", p_rms), ("rho", 1.225), ("c_s", 343.0)])?;
    check_rel("Acoustics_I*Z==P_rms^2", i_ac * z_ac, p_rms * p_rms, 1e-12)?;

    // 12. Wiedemann-Franz + Sommerfeld Lorenz Kapanışı
    let l_lor = eval_law(kg, "law_lorenz_sommerfeld_number", &[("k_B", kb), ("q_e", qe)])?;
    let k_wf = eval_law(
        kg,
        "law_wiedemann_franz_thermal_cond",
        &[("L_lorenz", l_lor), ("sigma_elec", 5.96e7), ("T", 300.0)],
    )?;
    let expected_kwf = (PI * PI / 3.0) * (kb / qe).powi(2) * 5.96e7 * 300.0;
    check_rel("WiedemannFranz_Sommerfeld", k_wf, expected_kwf, 1e-12)?;

    // 13. Euler-Bernoulli Konsol vs Basit Kiriş Sehim Oranı: delta_tip / delta_mid == 16
    let d_tip = eval_law(
        kg,
        "law_cantilever_tip_deflection",
        &[("F", 1000.0), ("L", 2.0), ("E_mod", e_mod), ("I_area", 1e-5)],
    )?;
    let d_mid = eval_law(
        kg,
        "law_simply_supported_midspan_deflection",
        &[("F", 1000.0), ("L", 2.0), ("E_mod", e_mod), ("I_area", 1e-5)],
    )?;
    check_rel("BeamDeflection_Cantilever/SimplySupported==16", d_tip / d_mid, 16.0, 1e-12)?;

    // 14. Foton Dalga-Parçacık Kapanışı: E_phot == p_phot * c
    let lam_ph = 500.0e-9;
    let f_ph = c / lam_ph;
    let e_ph = eval_law(kg, "law_planck_photon_energy", &[("h_planck", h), ("f_em", f_ph)])?;
    let p_ph = eval_law(kg, "law_photon_momentum", &[("h", h), ("lambda_em", lam_ph)])?;
    check_rel("Photon_E==p*c", e_ph, p_ph * c, 1e-12)?;

    Ok(14)
}

/// Kapı 19: Asimptotik klasik limit yakınsaması (5 limit) ve Dual64 kısmi türev işaret/monotonluk (12 işaret) denetimi.
pub fn verify_asymptotic_limits_and_monotonicity(kg: &KnowledgeGraph) -> Result<usize, String> {
    let c = PhysicalConstants::C;
    let g = PhysicalConstants::G;
    let hbar = PhysicalConstants::HBAR;
    let kb = PhysicalConstants::K_B;

    // --- A. 5 Asimptotik Limit ---
    // A1: Göreli kinetik enerji (v << c) -> 0.5 * m * v^2
    let v_slow = 1.0e-4 * c;
    let m_test = 10.0;
    let gamma_slow = eval_law(kg, "law_lorentz_factor", &[("v", v_slow), ("c", c)])?;
    let ek_rel = eval_law(
        kg,
        "law_relativistic_kinetic_energy",
        &[("gamma_rel", gamma_slow), ("m", m_test), ("c", c)],
    )?;
    let ek_cls = eval_law(kg, "law_kinetic_energy", &[("m", m_test), ("v", v_slow)])?;
    check_rel("Limit_RelKinetic->Classical", ek_rel, ek_cls, 1e-6)?;

    // A2: Ackeret göreli roket (v_e << c) -> Tsiolkovsky
    let ve_slow = 1.0e-4 * c;
    let v_ack = eval_law(
        kg,
        "law_relativistic_rocket_dv",
        &[("c", c), ("m_0", 10.0), ("m_f", 2.0), ("v_e", ve_slow)],
    )?;
    let v_tsi = eval_law(kg, "law_tsiolkovsky", &[("v_e", ve_slow), ("m_0", 10.0), ("m_f", 2.0)])?;
    check_rel("Limit_RelRocket->Tsiolkovsky", v_ack, v_tsi, 1e-6)?;

    // A3: Kütleçekimsel zaman genişlemesi (r -> inf) -> t_proper == t
    let t_pr = eval_law(
        kg,
        "law_gravitational_time_dilation",
        &[("t", 100.0), ("G", g), ("M_body", 5.972e24), ("r", 1e15), ("c", c)],
    )?;
    check_rel("Limit_GravTimeDilation_r->inf", t_pr, 100.0, 1e-12)?;

    // A4: Sönümlü doğal frekans (zeta -> 0) -> omega_n
    let wd_lim = eval_law(kg, "law_damped_natural_frequency", &[("omega_n", 120.0), ("zeta_damp", 1e-8)])?;
    check_rel("Limit_DampedFreq_zeta->0", wd_lim, 120.0, 1e-12)?;

    // A5: Sığ su dalgası vs Torricelli oranı: v_efflux / v_sw == sqrt(2)
    let v_ef = eval_law(kg, "law_torricelli_efflux_velocity", &[("g_0", PhysicalConstants::G0), ("h_col", 10.0)])?;
    let v_sw = eval_law(kg, "law_shallow_water_wave_speed", &[("g_0", PhysicalConstants::G0), ("h_col", 10.0)])?;
    check_rel("Limit_Torricelli/ShallowWater==sqrt(2)", v_ef / v_sw, 2.0_f64.sqrt(), 1e-12)?;

    // --- B. 12 Dual64 Kısmi Türev İşaret (Fiziksel Monotonluk) Denetimi ---
    type MonotonicityCase<'a> = (&'a str, &'a str, i8, &'a [(&'a str, f64)]);
    let mono_cases: &[MonotonicityCase<'_>] = &[
        ("law_hawking_black_hole_temperature", "M_body", -1, &[("hbar", hbar), ("c", c), ("G", g), ("M_body", 1e30), ("k_B", kb)]),
        ("law_carnot_thermal_efficiency", "T_c", -1, &[("T_h", 1000.0), ("T_c", 300.0)]),
        ("law_carnot_thermal_efficiency", "T_h", 1, &[("T_h", 1000.0), ("T_c", 300.0)]),
        ("law_cantilever_tip_deflection", "E_mod", -1, &[("F", 1000.0), ("L", 2.0), ("E_mod", 200e9), ("I_area", 1e-5)]),
        ("law_cantilever_tip_deflection", "L", 1, &[("F", 1000.0), ("L", 2.0), ("E_mod", 200e9), ("I_area", 1e-5)]),
        ("law_johnson_nyquist_noise_voltage", "T", 1, &[("k_B", kb), ("T", 300.0), ("R_elec", 1000.0), ("B_bw", 1e6)]),
        ("law_wien_displacement_wavelength", "T", -1, &[("b_wien", 2.897771955e-3), ("T", 5778.0)]),
        ("law_euler_critical_buckling_load", "L_eff", -1, &[("E_mod", 200e9), ("I_area", 1e-5), ("L_eff", 3.0)]),
        ("law_elastic_p_wave_velocity", "rho", -1, &[("E_mod", 200e9), ("nu_pois", 0.3), ("rho", 7850.0)]),
        ("law_resonator_quality_factor", "zeta_damp", -1, &[("zeta_damp", 0.05)]),
        ("law_planetary_equilibrium_temperature", "A_bond", -1, &[("T_star", 5778.0), ("A_bond", 0.3), ("R_star", 6.957e8), ("a_orb", 1.496e11)]),
        ("law_gravitational_time_dilation", "M_body", -1, &[("t", 100.0), ("G", g), ("M_body", 5e32), ("r", 8e6), ("c", c)]),
    ];

    for &(law_id, var_name, expected_sign, pts) in mono_cases {
        let law = kg
            .laws
            .get(law_id)
            .ok_or_else(|| format!("Monotonluk denetimi için yasa bulunamadı: '{}'", law_id))?;
        let env: HashMap<&str, f64> = pts.iter().copied().collect();
        let d = law
            .rhs
            .eval_dual(&env, var_name)
            .map_err(|e| format!("Dual64 monotonluk hatası [{} / {}]: {}", law_id, var_name, e))?;
        let ok = if expected_sign > 0 { d.der > 0.0 } else { d.der < 0.0 };
        if !d.der.is_finite() || !ok {
            return Err(format!(
                "FİZİKSEL MONOTONLUK İHLALİ [{}]: d({})/d({}) = {:.6e} (beklenen işaret: {})",
                law_id,
                law.target,
                var_name,
                d.der,
                if expected_sign > 0 { "> 0" } else { "< 0" }
            ));
        }
    }

    Ok(5 + mono_cases.len())
}

struct AshbyEnvelope {
    rho: (f64, f64),       // kg/m^3
    e_gpa: (f64, f64),     // GPa
    ys_mpa: (f64, f64),    // MPa
    t_max_k: (f64, f64),   // K
    k_wmk: (f64, f64),     // W/(m*K)
}

fn category_envelope(category: &str) -> Option<AshbyEnvelope> {
    match category {
        "metal_alloy" => Some(AshbyEnvelope {
            rho: (1400.0, 12000.0),
            e_gpa: (35.0, 350.0),
            ys_mpa: (80.0, 2600.0),
            t_max_k: (350.0, 1600.0),
            k_wmk: (5.0, 260.0),
        }),
        "superalloy" => Some(AshbyEnvelope {
            rho: (7500.0, 10000.0),
            e_gpa: (150.0, 260.0),
            ys_mpa: (250.0, 1500.0),
            t_max_k: (650.0, 1650.0),
            k_wmk: (6.0, 35.0),
        }),
        "copper_superalloy" => Some(AshbyEnvelope {
            rho: (8000.0, 9200.0),
            e_gpa: (90.0, 160.0),
            ys_mpa: (100.0, 600.0),
            t_max_k: (450.0, 1250.0),
            k_wmk: (180.0, 420.0),
        }),
        "refractory_metal" => Some(AshbyEnvelope {
            rho: (8000.0, 22500.0),
            e_gpa: (80.0, 520.0),
            ys_mpa: (150.0, 1600.0),
            t_max_k: (1400.0, 3800.0),
            k_wmk: (25.0, 200.0),
        }),
        "composite" => Some(AshbyEnvelope {
            rho: (1100.0, 2400.0),
            e_gpa: (30.0, 400.0),
            ys_mpa: (400.0, 6000.0),
            t_max_k: (360.0, 1100.0),
            k_wmk: (0.02, 50.0),
        }),
        "nanomaterial" => Some(AshbyEnvelope {
            rho: (900.0, 3000.0),
            e_gpa: (500.0, 1400.0),
            ys_mpa: (10000.0, 150000.0),
            t_max_k: (600.0, 2500.0),
            k_wmk: (500.0, 6000.0),
        }),
        "ceramic_composite" => Some(AshbyEnvelope {
            rho: (1400.0, 3200.0),
            e_gpa: (40.0, 350.0),
            ys_mpa: (60.0, 700.0),
            t_max_k: (1400.0, 3300.0),
            k_wmk: (5.0, 100.0),
        }),
        "uhtc_ceramic" => Some(AshbyEnvelope {
            rho: (4500.0, 13500.0),
            e_gpa: (300.0, 600.0),
            ys_mpa: (150.0, 800.0),
            t_max_k: (2200.0, 4400.0),
            k_wmk: (10.0, 150.0),
        }),
        "technical_ceramic" => Some(AshbyEnvelope {
            rho: (2200.0, 6800.0),
            e_gpa: (150.0, 520.0),
            ys_mpa: (80.0, 1200.0),
            t_max_k: (1100.0, 2400.0),
            k_wmk: (1.5, 340.0),
        }),
        "semiconductor_optical" => Some(AshbyEnvelope {
            rho: (2000.0, 4500.0),
            e_gpa: (60.0, 420.0),
            ys_mpa: (30.0, 600.0),
            t_max_k: (900.0, 2300.0),
            k_wmk: (0.8, 220.0),
        }),
        "polymer" => Some(AshbyEnvelope {
            rho: (900.0, 2300.0),
            e_gpa: (0.5, 15.0),
            ys_mpa: (15.0, 200.0),
            t_max_k: (350.0, 750.0),
            k_wmk: (0.05, 1.0),
        }),
        "ablative_tps" => Some(AshbyEnvelope {
            rho: (150.0, 900.0),
            e_gpa: (0.05, 5.0),
            ys_mpa: (0.5, 40.0),
            t_max_k: (1500.0, 3600.0),
            k_wmk: (0.02, 1.0),
        }),
        "insulator" => Some(AshbyEnvelope {
            rho: (1.0, 500.0),
            e_gpa: (0.0005, 5.0),
            ys_mpa: (0.005, 20.0),
            t_max_k: (600.0, 1600.0),
            k_wmk: (0.005, 0.2),
        }),
        _ => None,
    }
}

fn in_band(val: f64, band: (f64, f64)) -> bool {
    val.is_finite() && val >= band.0 && val <= band.1
}

/// Kapı 20: Tüm malzemeleri kategori bazlı Ashby fiziksel zarflarına, ısıl işlem/kırılma moduna,
/// çok-kaynaklı dağılım aralığına ve >=3 bağımsız kaynak zorunluluğuna karşı denetler.
pub fn verify_materials_ashby_envelopes(mdb: &MaterialsDatabase) -> Result<usize, String> {
    for m in &mdb.list {
        verify_single_material_envelope(m)?;
    }
    Ok(mdb.list.len())
}

pub fn verify_single_material_envelope(m: &Material) -> Result<(), String> {
    let env = category_envelope(&m.category)
        .ok_or_else(|| format!("Bilinmeyen malzeme kategorisi [{}]: '{}'", m.id, m.category))?;

    let ys_mpa = m.yield_strength / 1e6;
    let uts_mpa = m.tensile_strength / 1e6;
    let e_gpa = m.youngs_modulus / 1e9;

    if !in_band(m.density, env.rho) {
        return Err(format!(
            "ASHBY ZARF İHLALİ [{} | {}]: yoğunluk {} kg/m³ ∉ [{}, {}]",
            m.id, m.category, m.density, env.rho.0, env.rho.1
        ));
    }
    if !in_band(e_gpa, env.e_gpa) {
        return Err(format!(
            "ASHBY ZARF İHLALİ [{} | {}]: E = {} GPa ∉ [{}, {}]",
            m.id, m.category, e_gpa, env.e_gpa.0, env.e_gpa.1
        ));
    }
    if !in_band(ys_mpa, env.ys_mpa) {
        return Err(format!(
            "ASHBY ZARF İHLALİ [{} | {}]: akma/kırılma = {} MPa ∉ [{}, {}]",
            m.id, m.category, ys_mpa, env.ys_mpa.0, env.ys_mpa.1
        ));
    }
    if uts_mpa < ys_mpa {
        return Err(format!(
            "MALZEME FİZİK İHLALİ [{}]: çekme ({} MPa) < akma ({} MPa)",
            m.id, uts_mpa, ys_mpa
        ));
    }
    if !in_band(m.max_temperature_k, env.t_max_k) {
        return Err(format!(
            "ASHBY ZARF İHLALİ [{} | {}]: T_max = {} K ∉ [{}, {}]",
            m.id, m.category, m.max_temperature_k, env.t_max_k.0, env.t_max_k.1
        ));
    }
    if !in_band(m.thermal_conductivity, env.k_wmk) {
        return Err(format!(
            "ASHBY ZARF İHLALİ [{} | {}]: k = {} W/(m·K) ∉ [{}, {}]",
            m.id, m.category, m.thermal_conductivity, env.k_wmk.0, env.k_wmk.1
        ));
    }
    if m.condition.trim().is_empty() {
        return Err(format!("MALZEME METADATA EKSİĞİ [{}]: 'condition' (ısıl işlem/durum) boş!", m.id));
    }
    let valid_modes = ["ductile", "brittle", "composite_laminate", "ablative", "foam_insulator"];
    if !valid_modes.contains(&m.failure_mode.as_str()) {
        return Err(format!(
            "MALZEME METADATA HATASI [{}]: geçersiz failure_mode '{}'",
            m.id, m.failure_mode
        ));
    }
    if m.sources.len() < 3 {
        return Err(format!(
            "MALZEME KAYNAK İHLALİ [{}]: en az 3 bağımsız kaynak zorunlu (bulunan: {})",
            m.id,
            m.sources.len()
        ));
    }
    if let Some([lo, hi]) = m.yield_strength_range_mpa {
        if lo <= 0.0 || lo > hi || ys_mpa < lo - 1e-6 || ys_mpa > hi + 1e-6 {
            return Err(format!(
                "MALZEME ARALIK İHLALİ [{}]: nominal {} MPa, kaynak aralığı [{}, {}] dışında!",
                m.id, ys_mpa, lo, hi
            ));
        }
    } else {
        return Err(format!(
            "MALZEME ARALIK EKSİĞİ [{}]: 'yield_strength_range_mpa' tanımlı değil!",
            m.id
        ));
    }
    Ok(())
}

pub fn run_full_epistemic_audit(
    kg: &KnowledgeGraph,
    mdb: &MaterialsDatabase,
) -> Result<EpistemicAuditReport, String> {
    let reference_anchors_checked = verify_reference_anchors(kg)?;
    let cross_law_closures_checked = verify_cross_law_closures(kg)?;
    let asymptotic_and_monotonicity_checked = verify_asymptotic_limits_and_monotonicity(kg)?;
    let materials_ashby_checked = verify_materials_ashby_envelopes(mdb)?;
    Ok(EpistemicAuditReport {
        reference_anchors_checked,
        cross_law_closures_checked,
        asymptotic_and_monotonicity_checked,
        materials_ashby_checked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_four_epistemic_shields_pass_on_canonical_ssot() {
        let kg = KnowledgeGraph::new();
        let mdb = MaterialsDatabase::load_canonical();
        let rep = run_full_epistemic_audit(&kg, &mdb).expect("4-katmanlı epistemik denetim geçmeli!");
        assert!(rep.reference_anchors_checked >= 40);
        assert_eq!(rep.cross_law_closures_checked, 14);
        assert_eq!(rep.asymptotic_and_monotonicity_checked, 17);
        assert_eq!(rep.materials_ashby_checked, 60);
    }

    #[test]
    fn test_adversarial_dimensionless_prefactor_drift_is_caught() {
        // Bekenstein-Hawking'de 1/4 yerine yanlışlıkla pi konulduğunda kapanış kapısı anında reddetmelidir!
        let mut kg = KnowledgeGraph::new();
        let law = kg.laws.get_mut("law_bekenstein_hawking_entropy").unwrap();
        law.rhs = crate::ast::Expr::parse_str("(3.141592653589793 * k_B * (c ** 3) * A) / (G * hbar)").unwrap();
        assert!(
            verify_reference_anchors(&kg).is_err(),
            "Altın çapa hatalı 4*pi katsayısını yakalamalıydı!"
        );
        assert!(
            verify_cross_law_closures(&kg).is_err(),
            "Kara delik 1. yasa kapanışı hatalı 4*pi katsayısını yakalamalıydı!"
        );
    }

    #[test]
    fn test_adversarial_material_unit_confusion_is_caught_by_ashby_envelope() {
        let mdb = MaterialsDatabase::load_canonical();
        let mut bad_polymer = mdb.get("mat_peek_unfilled").unwrap().clone();
        // Polimere yanlışlıkla GPa yerine MPa (3700 GPa) veya °C (250 K yerine) girildiğini simüle et
        bad_polymer.youngs_modulus = 3700.0e9;
        assert!(
            verify_single_material_envelope(&bad_polymer).is_err(),
            "Ashby zarfı polimer için 3700 GPa modül hatasını reddetmeliydi!"
        );
    }
}
