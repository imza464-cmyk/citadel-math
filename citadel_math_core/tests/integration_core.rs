// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
//! Integration tests covering pure-Rust algebraic, interval, structural, Pareto, and physical-bounds engines.

#![forbid(unsafe_code)]

use citadel_math_core::{
    AtmosphereModel, Dual64, GuardedPoly, Interval64, MaterialsDatabase, ObjectiveSpec,
    ParetoSynthesizer, PhysicalBoundsShield, PhysicalConstants, PhysicalLawViolation,
    PhysicalSystemCandidate, PolyZ, StructuralStressChecker,
};

#[test]
fn integration_guarded_poly_preserves_excluded_singularity() {
    // (x^2 - 9) / (x - 3) == x + 3 ONLY on Z \ {3}
    let num = PolyZ::new(vec![-9, 0, 1]);
    let guarded = GuardedPoly::cancel_linear_factor(&num, 3, 16).expect("exact linear factor");
    assert_eq!(guarded.poly, PolyZ::new(vec![3, 1]));
    assert!(guarded.excluded_points.contains(&3));
    assert_ne!(guarded, GuardedPoly::total(PolyZ::new(vec![3, 1])));
}

#[test]
fn integration_physical_bounds_catches_carnot_superluminal_and_reactionless() {
    let impossible = PhysicalSystemCandidate {
        mass_kg: Some(1_000.0),
        velocity_ms: Some(PhysicalConstants::C * 1.05),
        mass_flow_kg_s: Some(0.0),
        power_in_w: Some(1_000.0),
        thrust_n: Some(10.0), // > 2*P/c photon bound
        temp_hot_k: Some(600.0),
        temp_cold_k: Some(300.0),
        efficiency: Some(0.80), // > Carnot 0.50
        ..Default::default()
    };
    let violations = PhysicalBoundsShield::collect_violations(&impossible);
    assert!(violations
        .iter()
        .any(|v| matches!(v, PhysicalLawViolation::SuperluminalCausalityViolation { .. })));
    assert!(violations
        .iter()
        .any(|v| matches!(v, PhysicalLawViolation::CarnotBoundViolation { .. })));
    assert!(violations
        .iter()
        .any(|v| matches!(v, PhysicalLawViolation::ReactionlessMomentumViolation { .. })));
}

#[test]
fn integration_materials_and_biaxial_von_mises_hull_check() {
    let db = MaterialsDatabase::load_canonical();
    assert!(db.list.len() >= 80);
    let alli = db
        .list
        .iter()
        .find(|m| m.id == "mat_alli_2195")
        .expect("mat_alli_2195 present");
    let (sigma_vm, sf, passed) = StructuralStressChecker::check_hull_at_temp_with_pressure(
        alli, 45_000.0, 4.5, 35_000.0, 1.8, 5.0, 293.15, 4.0,
    );
    assert!(passed);
    assert!(sigma_vm > 1.0e7);
    assert!(sf > StructuralStressChecker::SAFETY_FACTOR);
}

#[test]
fn integration_interval64_and_dual64_hoop_stress_sensitivity() {
    let p = Interval64::new(390_000.0, 410_000.0).unwrap();
    let r = Interval64::new(1.795, 1.805).unwrap();
    let t = Interval64::new(0.0049, 0.0051).unwrap();
    let sigma = p.mul(&r).div(&t).unwrap();
    let exact_mid = (400_000.0 * 1.8) / 0.005;
    assert!(sigma.contains(exact_mid));

    // Dual64 automatic derivative d/dt ((P * r) / t) = -(P * r) / t^2
    let p_d = Dual64::constant(400_000.0);
    let r_d = Dual64::constant(1.8);
    let t_d = Dual64::active_variable(0.005);
    let res_d = (p_d * r_d).checked_div(t_d).unwrap();
    let expected_deriv = -(400_000.0 * 1.8) / (0.005_f64 * 0.005_f64);
    assert!((res_d.der - expected_deriv).abs() / expected_deriv.abs() < 1e-12);
}

#[test]
fn integration_pareto_skyline_and_ussa1976_atmosphere() {
    let specs = vec![
        ObjectiveSpec::minimize("mass_kg", "kg"),
        ObjectiveSpec::maximize("safety_factor", "x"),
    ];
    let summary = ParetoSynthesizer::analyze(
        &specs,
        &[vec![1_000.0, 1.5], vec![1_150.0, 2.2], vec![1_250.0, 1.4]],
    )
    .unwrap();
    assert_eq!(summary.skyline_count, 2);
    assert_eq!(summary.skyline_indices, vec![0, 1]);

    let (_rho0, _p0, t0, a0) = AtmosphereModel::state_at(0.0);
    assert!((t0 - 288.15).abs() < 1e-3);
    assert!((a0 - 340.294).abs() < 0.05);
}
