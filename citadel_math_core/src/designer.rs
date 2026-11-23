// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Gerçekçi Kütle, Kepler Enberi (Perigee) ve Canlı Isı İntegralli Tersine Tasarım Motoru

use crate::ast::Expr;
use crate::constants::PhysicalConstants;
use crate::graph::{KnowledgeGraph, SynthesizedFormula};
use crate::interval::Interval64;
use crate::materials::{CylinderGeometry, MaterialsDatabase, StructuralStressChecker};
use crate::pareto::{ObjectiveSpec, ParetoFrontSummary, ParetoSynthesizer};
use crate::simulator::{RK4Simulator, VehicleConfig};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct MissionConstraints {
    pub payload_kg: f64,
    pub target_orbit_km: f64,
    pub max_g_limit: f64,
    pub min_safety_factor: f64,
}

impl Default for MissionConstraints {
    fn default() -> Self {
        Self {
            payload_kg: 500.0,
            target_orbit_km: 185.0,
            max_g_limit: 8.5,
            min_safety_factor: 1.35,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerifiedVehicleDesign {
    pub propulsion_name: &'static str,
    pub material_id: String,
    pub material_name: String,
    pub stages: u8,
    pub hull_radius_m: f64,
    pub hull_length_m: f64,
    pub slenderness_l_d: f64,
    pub wall_thickness_mm: f64,
    pub structural_shell_mass_kg: f64,
    pub tps_thickness_mm: f64,
    pub tps_mass_kg: f64,
    pub payload_mass_kg: f64,
    pub total_dry_mass_kg: f64,
    pub fuel_mass_kg: f64,
    pub gross_liftoff_mass_kg: f64,
    pub structural_fraction_pct: f64,
    pub payload_fraction_pct: f64,
    pub thrust_kn: f64,
    pub isp_s: f64,
    pub beamed_power_mw: Option<f64>,
    pub final_altitude_km: f64,
    pub perigee_altitude_km: f64,
    pub apogee_altitude_km: f64,
    pub eccentricity: f64,
    pub final_speed_ms: f64,
    pub max_q_kpa: f64,
    pub max_g_force: f64,
    pub total_heat_load_mj_m2: f64,
    pub von_mises_mpa: f64,
    pub yield_safety_factor: f64,
    pub buckling_crit_mpa: f64,
    pub buckling_safety_factor: f64,
    pub min_structural_safety_factor: f64,
    pub bare_shock_temp_c: f64,
    pub protected_bondline_temp_c: f64,
    pub material_max_temp_c: f64,
    pub ideal_dv_interval_ms: Interval64,
    pub dv_sensitivity_isp_ms_per_s: f64,
    pub dv_sensitivity_dry_mass_ms_per_kg: f64,
    pub pareto_rank: usize,
    pub crowding_distance: f64,
    pub utopia_distance: f64,
    pub is_knee_point: bool,
    pub dominated_count: usize,
}

#[derive(Debug, Clone)]
pub struct InverseDesignReport {
    pub constraints: MissionConstraints,
    pub synthesized_laws: Vec<SynthesizedFormula>,
    pub total_candidates_evaluated: usize,
    pub orbit_passed_count: usize,
    pub g_limit_passed_count: usize,
    pub structural_passed_count: usize,
    pub pareto_summary: ParetoFrontSummary,
    pub exact_skyline_designs: Vec<VerifiedVehicleDesign>,
    pub pareto_optimal_designs: Vec<VerifiedVehicleDesign>,
}

struct PropulsionArchetype {
    name: &'static str,
    isp_s: f64,
    propellant_bulk_density: f64, // kg/m^3 (Gerçek kriyojenik yoğunluk!)
    cryo_insulation_kg_m2: f64,   // Kriyojenik SOFI köpük yalıtımı (kg/m^2)
    engine_mass_factor: f64,      // kg / N thrust
    plumbing_copv_fraction: f64,  // Boru, valf, COPV helyum ve kademe adaptörü oranı
    initial_velocity: f64,
    initial_altitude: f64,
    cd: f64,
    has_stage_2: bool,
    is_beamed: bool,
}

pub struct InverseDesignEngine;

impl InverseDesignEngine {
    pub fn optimize(constraints: MissionConstraints) -> InverseDesignReport {
        let kg = KnowledgeGraph::new();
        let mdb = MaterialsDatabase::load_canonical();
        let sim = RK4Simulator::new(0.25);

        // 1. Hipergraf üzerinden kanonik itki, güç ve ters-çözüm (m_0) denklemlerini sentezle
        let mut inputs_dv = HashSet::new();
        inputs_dv.insert("I_sp");
        inputs_dv.insert("g_0");
        inputs_dv.insert("m_0");
        inputs_dv.insert("m_f");
        let mut synthesized_laws = kg.synthesize_formulas(&inputs_dv, "delta_v", 4);
        let dv_expr = synthesized_laws[0].symbolic_expr.clone();
        // 2-Kademeli roketler için gerçek 2-kademeli Tsiolkovsky + ilk hız (katapult) sembolik ifadesi:
        let dv_2stage_expr = Expr::parse_str(
            "v_init + (0.94 * g_0 * I_sp) * ln((m_d1 + m_p1 + m_d2 + m_p2) / (m_d1 + m_d2 + m_p2)) + (1.05 * g_0 * I_sp) * ln((m_d2 + m_p2) / m_d2)",
        )
        .expect("2-kademeli Tsiolkovsky AST ayrıştırma hatası");

        let mut inputs_beam = HashSet::new();
        inputs_beam.insert("I_sp");
        inputs_beam.insert("g_0");
        inputs_beam.insert("m_dot");
        inputs_beam.insert("eta");
        synthesized_laws.extend(kg.synthesize_formulas(&inputs_beam, "P_beam", 4));

        // Faz 3: Çift-yönlü cebirsel tersleme ile m_0 = m_f * exp(delta_v / (g_0 * I_sp)) sentezle
        let mut inv_kg = KnowledgeGraph::new();
        inv_kg.register_algebraic_inversions();
        let mut inputs_inv_m0 = HashSet::new();
        inputs_inv_m0.insert("delta_v");
        inputs_inv_m0.insert("g_0");
        inputs_inv_m0.insert("I_sp");
        inputs_inv_m0.insert("m_f");
        if let Some(inv_m0_law) = inv_kg
            .synthesize_formulas(&inputs_inv_m0, "m_0", 2)
            .into_iter()
            .next()
        {
            synthesized_laws.push(inv_m0_law);
        }

        // 2. Gerçek Fiziksel Yoğunluklu İtki Mimarileri:
        // - Saf Sıvı Hidrojen (LH2 @ 20K): 70.85 kg/m^3
        // - LOX/LH2 (6:1 O/F karışım): 360.0 kg/m^3
        // - LOX/CH4 (3.6:1 O/F sıvı metan): 820.0 kg/m^3
        // - LOX/RP-1 (2.56:1 O/F kerosen): 1030.0 kg/m^3
        let archetypes = [
            PropulsionArchetype {
                name: "Beamed-Laser Termal Saf-LH2 SSTO",
                isp_s: 920.0,
                propellant_bulk_density: 70.85, // Gerçek saf LH2 yoğunluğu!
                cryo_insulation_kg_m2: 1.80,
                engine_mass_factor: 0.0018,     // Lazer ısı eşanjörü + nozul (TWR ~ 56)
                plumbing_copv_fraction: 0.032,
                initial_velocity: 0.0,
                initial_altitude: 0.0,
                cd: 0.25,
                has_stage_2: false,
                is_beamed: true,
            },
            PropulsionArchetype {
                name: "Elektromanyetik Katapult (1.5 km/s) + LOX/CH4",
                isp_s: 375.0,
                propellant_bulk_density: 820.0,
                cryo_insulation_kg_m2: 0.60,
                engine_mass_factor: 0.0014,
                plumbing_copv_fraction: 0.035,
                initial_velocity: 1500.0,
                initial_altitude: 4_500.0, // Yüksek irtifa dağ yamacı vakum tüpü çıkışı
                cd: 0.22,
                has_stage_2: true,
                is_beamed: false,
            },
            PropulsionArchetype {
                name: "2-Kademeli Kriyojenik LOX/LH2 Roket",
                isp_s: 452.0,
                propellant_bulk_density: 360.0,
                cryo_insulation_kg_m2: 1.20,
                engine_mass_factor: 0.0016,
                plumbing_copv_fraction: 0.038,
                initial_velocity: 0.0,
                initial_altitude: 0.0,
                cd: 0.26,
                has_stage_2: true,
                is_beamed: false,
            },
            PropulsionArchetype {
                name: "2-Kademeli LOX/RP-1 Kerosen Roket",
                isp_s: 335.0,
                propellant_bulk_density: 1030.0,
                cryo_insulation_kg_m2: 0.35,
                engine_mass_factor: 0.0013,
                plumbing_copv_fraction: 0.035,
                initial_velocity: 0.0,
                initial_altitude: 0.0,
                cd: 0.28,
                has_stage_2: true,
                is_beamed: false,
            },
        ];

        let candidate_material_ids = [
            "mat_carbon_fiber_t1000g",
            "mat_im7_8552_cfrp",
            "mat_alli_2195",
            "mat_titanium_ti6al4v",
            "mat_ss_304l_cryo",
            "mat_inconel_718",
        ];

        let radii_m = [0.75, 1.00, 1.30, 1.65];
        let walls_mm = [2.2, 3.2, 4.5, 6.5];
        let fuel_multipliers = [5.5, 8.0, 11.5, 16.5, 24.0, 34.0];
        let twr_options = [1.45, 1.68, 1.95];

        let mut total_evaluated = 0usize;
        let mut orbit_passed = 0usize;
        let mut g_passed = 0usize;
        let mut structural_passed = 0usize;
        let mut valid_designs = Vec::new();

        for arch in &archetypes {
            for &mat_id in &candidate_material_ids {
                let Some(mat) = mdb.get(mat_id) else { continue };
                for &r_m in &radii_m {
                    for &t_mm in &walls_mm {
                        for &fuel_mult in &fuel_multipliers {
                            for &twr in &twr_options {
                                total_evaluated += 1;

                                let total_fuel_kg = constraints.payload_kg * fuel_mult;
                                // %10 ullage + kademe arası hacim payı
                                let tank_vol_m3 = (total_fuel_kg / arch.propellant_bulk_density) * 1.10;
                                let cross_area = PI * r_m * r_m;
                                let hull_length_m = (tank_vol_m3 / cross_area).max(3.5);
                                let slenderness_l_d = hull_length_m / (2.0 * r_m);

                                // Silindirik gövde + kubbeler + ortogrid halka takviyeleri (%30) + kriyo yalıtım
                                let t_m = t_mm / 1000.0;
                                let shell_surface_m2 = 2.0 * PI * r_m * hull_length_m + 4.0 * PI * r_m * r_m;
                                let raw_shell_kg = shell_surface_m2 * t_m * mat.density * 1.30;
                                let cryo_insul_kg = shell_surface_m2 * arch.cryo_insulation_kg_m2;
                                let plumbing_copv_kg = total_fuel_kg * arch.plumbing_copv_fraction;
                                let avionics_kg = 75.0_f64;

                                // Ön TPS tahmini (~30 kg) ile ilk kalkış kütlesi ve motor boyutu
                                let approx_dry = constraints.payload_kg
                                    + raw_shell_kg
                                    + cryo_insul_kg
                                    + plumbing_copv_kg
                                    + avionics_kg
                                    + 30.0;
                                let approx_glow = approx_dry + total_fuel_kg;
                                let thrust_n = approx_glow * PhysicalConstants::G0 * twr;
                                let engine_mass_kg = thrust_n * arch.engine_mass_factor;

                                let base_structural_kg =
                                    raw_shell_kg + cryo_insul_kg + plumbing_copv_kg + avionics_kg + engine_mass_kg;
                                let pre_dry_mass_kg = constraints.payload_kg + base_structural_kg + 25.0;

                                let vehicle = if arch.has_stage_2 {
                                    let s1_fuel = total_fuel_kg * 0.76;
                                    let s2_fuel = total_fuel_kg * 0.24;
                                    let s1_dry = base_structural_kg * 0.75;
                                    let s2_dry = constraints.payload_kg + 25.0 + base_structural_kg * 0.25;
                                    let s2_thrust = (s2_dry + s2_fuel) * PhysicalConstants::G0 * 1.18;
                                    VehicleConfig {
                                        name: arch.name,
                                        dry_mass: s1_dry,
                                        propellant_mass: s1_fuel,
                                        thrust_vac: thrust_n,
                                        isp: arch.isp_s * 0.94,
                                        area: cross_area,
                                        cd: arch.cd,
                                        initial_velocity: arch.initial_velocity,
                                        initial_altitude: arch.initial_altitude,
                                        has_stage_2: true,
                                        s2_dry_mass: s2_dry,
                                        s2_propellant: s2_fuel,
                                        s2_thrust,
                                        s2_isp: arch.isp_s * 1.05,
                                    }
                                } else {
                                    VehicleConfig {
                                        name: arch.name,
                                        dry_mass: pre_dry_mass_kg,
                                        propellant_mass: total_fuel_kg,
                                        thrust_vac: thrust_n,
                                        isp: arch.isp_s,
                                        area: cross_area,
                                        cd: arch.cd,
                                        initial_velocity: arch.initial_velocity,
                                        initial_altitude: arch.initial_altitude,
                                        has_stage_2: false,
                                        s2_dry_mass: 0.0,
                                        s2_propellant: 0.0,
                                        s2_thrust: 0.0,
                                        s2_isp: 0.0,
                                    }
                                };

                                let sim_res = sim.simulate(&vehicle, 1200.0);
                                if !sim_res.orbit_achieved
                                    || sim_res.perigee_altitude_km < constraints.target_orbit_km * 0.85
                                {
                                    continue;
                                }
                                orbit_passed += 1;

                                if sim_res.max_g_force > constraints.max_g_limit {
                                    continue;
                                }
                                g_passed += 1;

                                // Canlı RK4 yörüngesinden gelen gerçek ısı integrali (∫ q_conv dt) ile TPS boyutlandır
                                let tps = StructuralStressChecker::size_ablative_tps_from_trajectory(
                                    mat,
                                    r_m,
                                    sim_res.peak_wall_temp_k,
                                    sim_res.peak_heat_flux_kw_m2,
                                    sim_res.total_heat_load_mj_m2,
                                    sim_res.flight_duration_s * 0.35,
                                );

                                let total_dry_mass_kg =
                                    constraints.payload_kg + base_structural_kg + tps.tps_mass_kg;
                                let glow_kg = total_dry_mass_kg + total_fuel_kg;

                                // Max-Q eksenel + eğilme + yerel kabuk + Euler kolon burkulma denetimi (Termal yumuşama σ_y(T), E(T) dahil!)
                                let max_q_g_load = (sim_res.max_g_force * 0.58).max(twr * 1.35);
                                let max_q_pa = sim_res.max_q_kpa * 1000.0;
                                let bondline_temp_k = tps.protected_bondline_temp_k;

                                let (sigma_vm_pa, yield_sf, ok_yield) =
                                    StructuralStressChecker::check_hull_at_temp(
                                        mat,
                                        glow_kg,
                                        max_q_g_load,
                                        max_q_pa,
                                        r_m,
                                        t_mm,
                                        bondline_temp_k,
                                    );
                                if !ok_yield || yield_sf < constraints.min_safety_factor {
                                    continue;
                                }

                                let (sigma_buckle_pa, buckle_sf, ok_buckle) =
                                    StructuralStressChecker::check_cylinder_buckling_at_temp(
                                        mat,
                                        glow_kg,
                                        max_q_g_load,
                                        max_q_pa,
                                        CylinderGeometry {
                                            radius_m: r_m,
                                            length_m: hull_length_m,
                                            wall_thickness_mm: t_mm,
                                        },
                                        bondline_temp_k,
                                    );
                                if !ok_buckle || buckle_sf < constraints.min_safety_factor {
                                    continue;
                                }

                                structural_passed += 1;

                                let beamed_power_mw = if arch.is_beamed {
                                    let ve = arch.isp_s * PhysicalConstants::G0;
                                    let eta = 0.65;
                                    Some(((thrust_n * ve) / (2.0 * eta)) / 1e6)
                                } else {
                                    None
                                };

                                let struct_plus_tps = base_structural_kg + tps.tps_mass_kg;
                                let structural_fraction_pct =
                                    (struct_plus_tps / (struct_plus_tps + total_fuel_kg)) * 100.0;

                                // Faz 3 (Öz-Eleştiri Düzeltmesi #4):
                                // 1-Kademeli (SSTO) araçlarda tek kademeli Tsiolkovsky,
                                // 2-Kademeli araçlarda gerçek 2-kademeli Tsiolkovsky + katapult ilk hızı ile Interval64 & Dual64 hesabı
                                let (
                                    ideal_dv_interval_ms,
                                    dv_sensitivity_isp_ms_per_s,
                                    dv_sensitivity_dry_mass_ms_per_kg,
                                ) = if arch.has_stage_2 {
                                    let s1_fuel = total_fuel_kg * 0.76;
                                    let s2_fuel = total_fuel_kg * 0.24;
                                    let s1_dry = base_structural_kg * 0.75;
                                    let s2_dry =
                                        constraints.payload_kg + tps.tps_mass_kg + base_structural_kg * 0.25;

                                    let mut iv_env = HashMap::new();
                                    iv_env.insert("v_init", Interval64::point(arch.initial_velocity).unwrap());
                                    iv_env.insert("g_0", Interval64::point(PhysicalConstants::G0).unwrap());
                                    iv_env.insert(
                                        "I_sp",
                                        Interval64::from_rel_uncertainty(arch.isp_s, 0.005).unwrap(),
                                    );
                                    iv_env.insert(
                                        "m_d1",
                                        Interval64::from_rel_uncertainty(s1_dry, 0.0025).unwrap(),
                                    );
                                    iv_env.insert(
                                        "m_p1",
                                        Interval64::from_rel_uncertainty(s1_fuel, 0.001).unwrap(),
                                    );
                                    iv_env.insert(
                                        "m_d2",
                                        Interval64::from_rel_uncertainty(s2_dry, 0.0025).unwrap(),
                                    );
                                    iv_env.insert(
                                        "m_p2",
                                        Interval64::from_rel_uncertainty(s2_fuel, 0.001).unwrap(),
                                    );
                                    let iv = dv_2stage_expr
                                        .eval_taylor_interval(&iv_env)
                                        .unwrap_or_else(|_| Interval64::point(0.0).unwrap());

                                    let mut dual_env = HashMap::new();
                                    dual_env.insert("v_init", arch.initial_velocity);
                                    dual_env.insert("g_0", PhysicalConstants::G0);
                                    dual_env.insert("I_sp", arch.isp_s);
                                    dual_env.insert("m_d1", s1_dry);
                                    dual_env.insert("m_p1", s1_fuel);
                                    dual_env.insert("m_d2", s2_dry);
                                    dual_env.insert("m_p2", s2_fuel);
                                    let d_isp = dv_2stage_expr
                                        .eval_dual(&dual_env, "I_sp")
                                        .map(|d| d.der)
                                        .unwrap_or(0.0);
                                    // Üst kademe kuru kütlesine (m_d2) göre duyarlılık (en kritik kütle cezası)
                                    let d_md2 = dv_2stage_expr
                                        .eval_dual(&dual_env, "m_d2")
                                        .map(|d| d.der)
                                        .unwrap_or(0.0);
                                    (iv, d_isp, d_md2)
                                } else {
                                    let mut iv_env = HashMap::new();
                                    iv_env.insert("g_0", Interval64::point(PhysicalConstants::G0).unwrap());
                                    iv_env.insert(
                                        "I_sp",
                                        Interval64::from_rel_uncertainty(arch.isp_s, 0.005).unwrap(),
                                    );
                                    iv_env.insert(
                                        "m_0",
                                        Interval64::from_rel_uncertainty(glow_kg, 0.001).unwrap(),
                                    );
                                    iv_env.insert(
                                        "m_f",
                                        Interval64::from_rel_uncertainty(total_dry_mass_kg, 0.0025).unwrap(),
                                    );
                                    let iv = dv_expr
                                        .eval_taylor_interval(&iv_env)
                                        .unwrap_or_else(|_| Interval64::point(0.0).unwrap());

                                    let mut dual_env = HashMap::new();
                                    dual_env.insert("g_0", PhysicalConstants::G0);
                                    dual_env.insert("I_sp", arch.isp_s);
                                    dual_env.insert("m_0", glow_kg);
                                    dual_env.insert("m_f", total_dry_mass_kg);
                                    let d_isp = dv_expr
                                        .eval_dual(&dual_env, "I_sp")
                                        .map(|d| d.der)
                                        .unwrap_or(0.0);
                                    let d_mf = dv_expr
                                        .eval_dual(&dual_env, "m_f")
                                        .map(|d| d.der)
                                        .unwrap_or(0.0);
                                    (iv, d_isp, d_mf)
                                };

                                valid_designs.push(VerifiedVehicleDesign {
                                    propulsion_name: arch.name,
                                    material_id: mat.id.clone(),
                                    material_name: mat.name.clone(),
                                    stages: if arch.has_stage_2 { 2 } else { 1 },
                                    hull_radius_m: r_m,
                                    hull_length_m,
                                    slenderness_l_d,
                                    wall_thickness_mm: t_mm,
                                    structural_shell_mass_kg: base_structural_kg,
                                    tps_thickness_mm: tps.tps_thickness_mm,
                                    tps_mass_kg: tps.tps_mass_kg,
                                    payload_mass_kg: constraints.payload_kg,
                                    total_dry_mass_kg,
                                    fuel_mass_kg: total_fuel_kg,
                                    gross_liftoff_mass_kg: glow_kg,
                                    structural_fraction_pct,
                                    payload_fraction_pct: (constraints.payload_kg / glow_kg) * 100.0,
                                    thrust_kn: thrust_n / 1000.0,
                                    isp_s: arch.isp_s,
                                    beamed_power_mw,
                                    final_altitude_km: sim_res.final_altitude_km,
                                    perigee_altitude_km: sim_res.perigee_altitude_km,
                                    apogee_altitude_km: sim_res.apogee_altitude_km,
                                    eccentricity: sim_res.eccentricity,
                                    final_speed_ms: sim_res.final_speed_ms,
                                    max_q_kpa: sim_res.max_q_kpa,
                                    max_g_force: sim_res.max_g_force,
                                    total_heat_load_mj_m2: sim_res.total_heat_load_mj_m2,
                                    von_mises_mpa: sigma_vm_pa / 1e6,
                                    yield_safety_factor: yield_sf,
                                    buckling_crit_mpa: sigma_buckle_pa / 1e6,
                                    buckling_safety_factor: buckle_sf,
                                    min_structural_safety_factor: yield_sf.min(buckle_sf),
                                    bare_shock_temp_c: tps.bare_wall_temp_k - 273.15,
                                    protected_bondline_temp_c: tps.protected_bondline_temp_k - 273.15,
                                    material_max_temp_c: mat.max_temperature_k - 273.15,
                                    ideal_dv_interval_ms,
                                    dv_sensitivity_isp_ms_per_s,
                                    dv_sensitivity_dry_mass_ms_per_kg,
                                    pareto_rank: 0,
                                    crowding_distance: 0.0,
                                    utopia_distance: 0.0,
                                    is_knee_point: false,
                                    dominated_count: 0,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Çok amaçlı kesin Pareto cephesi (Exact Skyline / NSGA-II Rank 0) sentezi
        let obj_specs = vec![
            ObjectiveSpec::minimize("gross_liftoff_mass_kg", "kg"),
            ObjectiveSpec::minimize("max_g_force", "G"),
            ObjectiveSpec::minimize("total_heat_load_mj_m2", "MJ/m^2"),
            ObjectiveSpec::maximize("min_structural_safety_factor", "x"),
        ];
        let raw_obj_matrix: Vec<Vec<f64>> = valid_designs
            .iter()
            .map(|d| {
                vec![
                    d.gross_liftoff_mass_kg,
                    d.max_g_force,
                    d.total_heat_load_mj_m2,
                    d.min_structural_safety_factor,
                ]
            })
            .collect();

        let pareto_summary = ParetoSynthesizer::analyze(&obj_specs, &raw_obj_matrix)
            .expect("Pareto cephesi NSGA-II analizi başarısız!");
        ParetoSynthesizer::verify_skyline_invariants(&obj_specs, &raw_obj_matrix, &pareto_summary)
            .expect("Pareto Skyline matematiksel değişmez doğrulaması başarısız!");

        for m in &pareto_summary.point_metrics {
            let d = &mut valid_designs[m.candidate_index];
            d.pareto_rank = m.pareto_rank;
            d.crowding_distance = m.crowding_distance;
            d.utopia_distance = m.utopia_distance;
            d.is_knee_point = m.is_knee_point;
            d.dominated_count = m.dominated_count;
        }

        let exact_skyline_designs: Vec<VerifiedVehicleDesign> = pareto_summary
            .skyline_indices
            .iter()
            .map(|&idx| valid_designs[idx].clone())
            .collect();

        // Seçilen vitrin Pareto-optimal tasarımları yalnızca Rank-0 kesin Skyline kümesinden üretilir:
        // 1) Ütopya uzlaşma diz-noktası (Knee), 2) En hafif çapa, 3) Farklı itki/malzeme aileleri, 4) En yüksek NSGA-II crowding.
        let mut selected_sky_pos: Vec<usize> = Vec::new();
        if let Some(knee_pos) = exact_skyline_designs.iter().position(|d| d.is_knee_point) {
            selected_sky_pos.push(knee_pos);
        }
        if let Some((light_pos, _)) = exact_skyline_designs
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.gross_liftoff_mass_kg.total_cmp(&b.gross_liftoff_mass_kg))
        {
            if !selected_sky_pos.contains(&light_pos) {
                selected_sky_pos.push(light_pos);
            }
        }

        let mut seen_prop: HashSet<&'static str> = selected_sky_pos
            .iter()
            .map(|&p| exact_skyline_designs[p].propulsion_name)
            .collect();
        for (pos, d) in exact_skyline_designs.iter().enumerate() {
            if !seen_prop.contains(d.propulsion_name) {
                seen_prop.insert(d.propulsion_name);
                if !selected_sky_pos.contains(&pos) {
                    selected_sky_pos.push(pos);
                }
            }
        }

        let mut seen_families: HashSet<String> = selected_sky_pos
            .iter()
            .map(|&p| {
                format!(
                    "{}::{}",
                    exact_skyline_designs[p].propulsion_name, exact_skyline_designs[p].material_id
                )
            })
            .collect();
        for (pos, d) in exact_skyline_designs.iter().enumerate() {
            if selected_sky_pos.len() >= 6 {
                break;
            }
            let key = format!("{}::{}", d.propulsion_name, d.material_id);
            if !seen_families.contains(&key) && !selected_sky_pos.contains(&pos) {
                seen_families.insert(key);
                selected_sky_pos.push(pos);
            }
        }
        for pos in 0..exact_skyline_designs.len() {
            if selected_sky_pos.len() >= 6 {
                break;
            }
            if !selected_sky_pos.contains(&pos) {
                selected_sky_pos.push(pos);
            }
        }

        let mut pareto: Vec<VerifiedVehicleDesign> = selected_sky_pos
            .into_iter()
            .map(|p| exact_skyline_designs[p].clone())
            .collect();
        pareto.sort_by(|a, b| a.gross_liftoff_mass_kg.total_cmp(&b.gross_liftoff_mass_kg));

        InverseDesignReport {
            constraints,
            synthesized_laws,
            total_candidates_evaluated: total_evaluated,
            orbit_passed_count: orbit_passed,
            g_limit_passed_count: g_passed,
            structural_passed_count: structural_passed,
            pareto_summary,
            exact_skyline_designs,
            pareto_optimal_designs: pareto,
        }
    }
}
