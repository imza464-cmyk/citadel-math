// Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
// Matematik Çekirdeği — Yüksek Performanslı Saf Rust Çekirdeği (Citadel Math Core)

#![forbid(unsafe_code)]

pub mod dimensions;
pub mod constants;
pub mod interval;
pub mod ast;
pub mod materials;
pub mod simulator;
pub mod graph;
pub mod proof_kernel;
pub mod physical_bounds;
pub mod buckingham;
pub mod pareto;
pub mod designer;
pub mod codegen;
pub mod astrodynamics;
pub mod cad;

pub use dimensions::{Dimension, ScaledUnit, check_dimensional_consistency};
pub use constants::PhysicalConstants;
pub use interval::{Interval64, Dual64};
pub use ast::Expr;
pub use astrodynamics::{Vector3, J2OrbitalModel, LambertResult, LambertSolver};
pub use materials::{
    CylinderGeometry, Material, MaterialsDatabase, StructuralStressChecker, TpsShieldResult,
};
pub use simulator::{AtmosphereModel, VehicleConfig, SimulationResult, RK4Simulator};
pub use graph::{VarBound, are_regimes_compatible, Law, SynthesizedFormula, KnowledgeGraph};
pub use proof_kernel::{
    MathKernelError, PolyZ, GuardedPoly, Monomial, MultiPolyZ, ProofRule, ProofRecord, ProofVerifier,
    ReviewStage, AuditVerdict, EpistemicKind,
};
pub use physical_bounds::{PhysicalLawViolation, PhysicalSystemCandidate, PhysicalBoundsShield};
pub use buckingham::{PiGroup, EmpiricalScalingFit, BuckinghamDiscoveryReport, BuckinghamPiSolver};
pub use pareto::{
    ObjectiveDirection, ObjectiveSpec, ParetoPointMetrics, ParetoFrontSummary,
    NamedObjectiveCandidate, ParetoSynthesizer,
};
pub use designer::{MissionConstraints, VerifiedVehicleDesign, InverseDesignReport, InverseDesignEngine};
pub use codegen::VerifiedCodegen;
pub use cad::{
    BeamEngineeringReport, BoundingBox2D, CadDocument, CadEntity, CadUnit, Point2D,
    SectionProperties, SectionSign,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z7_dimensional_homogeneity_and_rejection() {
        let rhs = Dimension::mass().mul_dim(&Dimension::acceleration());
        assert!(check_dimensional_consistency(Dimension::force(), rhs).is_ok());
        assert!(check_dimensional_consistency(Dimension::force(), Dimension::mass()).is_err());
    }

    #[test]
    fn test_interval64_outward_rounding_and_zero_div_guard() {
        let one = Interval64::point(1.0).unwrap();
        let three = Interval64::point(3.0).unwrap();
        let third = one.div(&three).unwrap();
        assert!(third.width() > 0.0);
        assert!(third.mul(&three).contains(1.0));

        let zero_span = Interval64::new(-1.0, 1.0).unwrap();
        assert!(Interval64::new(1.0, 2.0).unwrap().div(&zero_span).is_err());
    }

    #[test]
    fn test_nist_codata_2022_vacuum_identity() {
        let prod = PhysicalConstants::MU_0 * PhysicalConstants::EPS_0 * PhysicalConstants::C.powi(2);
        assert!((prod - 1.0).abs() < 2e-10, "Maxwell vacuum identity drift: {}", prod);
    }

    #[test]
    fn test_buckingham_pi_taylor_sedov_nullspace() {
        let vars = vec![
            ("r".to_string(), Dimension::length()),
            ("E".to_string(), Dimension::energy()),
            ("rho".to_string(), Dimension::density()),
            ("t".to_string(), Dimension::time()),
        ];
        let report = BuckinghamPiSolver::discover(&vars, Some("r"), &[], &[]).unwrap();
        assert_eq!(report.dimensional_rank, 3);
        assert_eq!(report.num_pi_groups, 1);
        assert!(report.target_identifiable);
    }

    #[test]
    fn test_wgs84_j2_critical_molniya_inclination_and_lambert() {
        let i_crit = J2OrbitalModel::CRITICAL_INCLINATION_RAD;
        assert!((i_crit.to_degrees() - 63.4349488).abs() < 1e-4);
        assert!(J2OrbitalModel::is_critical_inclination(i_crit, 0.01));
        let rate = J2OrbitalModel::perigee_precession_rate_rad_s(26_560_000.0, 0.72, i_crit).unwrap();
        assert!(rate.abs() < 1e-16);

        let mu = PhysicalConstants::MU_EARTH;
        let r1 = Vector3::new(7_000_000.0, 0.0, 0.0);
        let r2 = Vector3::new(0.0, 7_000_000.0, 0.0);
        let res = LambertSolver::solve(r1, r2, 1_450.0, true, mu).unwrap();
        let eps1 = 0.5 * res.v1.norm_sq() - mu / r1.norm();
        let eps2 = 0.5 * res.v2.norm_sq() - mu / r2.norm();
        assert!((eps1 - eps2).abs() / eps1.abs().max(1.0) < 1e-9);
    }
}
