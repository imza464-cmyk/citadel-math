// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
//! Standalone pure-Rust example demonstrating Z^7 dimensional shield, GuardedPoly domain-guard
//! preservation, IEEE 1788-2015 outward-rounded Interval64, and Carnot/causality bounds.

#![forbid(unsafe_code)]

use citadel_math_core::{
    check_dimensional_consistency, Dimension, GuardedPoly, Interval64, MaterialsDatabase,
    ObjectiveSpec, ParetoSynthesizer, PhysicalBoundsShield, PhysicalSystemCandidate, PolyZ,
    StructuralStressChecker,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Z^7 Dimensional Homogeneity Check: F == m * a
    let rhs = Dimension::mass().mul_dim(&Dimension::acceleration());
    check_dimensional_consistency(Dimension::force(), rhs)?;

    // 2. GuardedPoly Singular Cancellation: (x^2 - 1) / (x - 1) -> x + 1 with excluded {1}
    let numerator = PolyZ::new(vec![-1, 0, 1]); // -1 + 0*x + 1*x^2
    let guarded = GuardedPoly::cancel_linear_factor(&numerator, 1, 16)
        .map_err(|e| format!("GuardedPoly error: {:?}", e))?;
    assert!(guarded.excluded_points.contains(&1));

    // 3. Outward-Rounded Interval64 Hoop Stress: (P * r) / t
    let p = Interval64::new(390_000.0, 410_000.0)?;
    let r = Interval64::new(1.795, 1.805)?;
    let t = Interval64::new(0.0049, 0.0051)?;
    let hoop_pa = p.mul(&r).div(&t)?;

    // 4. Structural Stress Check on Al-Li 2195
    let mdb = MaterialsDatabase::load_canonical();
    let alli = mdb
        .list
        .iter()
        .find(|m| m.id == "mat_alli_2195")
        .ok_or("mat_alli_2195 missing")?;
    let (sigma_vm, sf, passed) = StructuralStressChecker::check_hull_at_temp_with_pressure(
        alli, 45_000.0, 4.5, 35_000.0, 1.8, 5.0, 293.15, 4.0,
    );
    assert!(passed);

    // 5. Physical Bounds Shield (Carnot & Special Relativity)
    let cand = PhysicalSystemCandidate {
        mass_kg: Some(45_000.0),
        velocity_ms: Some(2_400.0),
        temp_hot_k: Some(3_200.0),
        temp_cold_k: Some(300.0),
        efficiency: Some(0.68),
        ..Default::default()
    };
    let violations = PhysicalBoundsShield::collect_violations(&cand);
    assert!(violations.is_empty());

    // 6. Exact Multi-Objective Pareto Skyline
    let specs = vec![
        ObjectiveSpec::minimize("mass_kg", "kg"),
        ObjectiveSpec::maximize("safety_factor", "x"),
    ];
    let pareto = ParetoSynthesizer::analyze(
        &specs,
        &[vec![980.0, 1.45], vec![1_120.0, 2.05], vec![1_300.0, 1.40]],
    )?;

    println!("=== Citadel-Math Pure-Rust Core Example ===");
    println!("  [PASS] Z^7 F == m*a verified");
    println!("  [PASS] GuardedPoly excluded roots: {:?}", guarded.excluded_points);
    println!(
        "  [PASS] Hoop stress Interval64: [{:.2}, {:.2}] MPa",
        hoop_pa.lo / 1e6,
        hoop_pa.hi / 1e6
    );
    println!(
        "  [PASS] Al-Li 2195 Von Mises: {:.2} MPa (SF = {:.3}x)",
        sigma_vm / 1e6,
        sf
    );
    println!("  [PASS] Pareto skyline count: {}/3", pareto.skyline_count);
    Ok(())
}
