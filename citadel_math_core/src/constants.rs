// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Temel Fiziksel ve Astronomik Sabitler

/// NIST CODATA 2022 ve IAU 2015 Kanonik Sabitleri
pub struct PhysicalConstants;

impl PhysicalConstants {
    pub const C: f64 = 299_792_458.0;                    // Işık hızı (m/s) [2019 SI Tam]
    pub const G: f64 = 6.67430e-11;                       // Çekim Sabiti (m^3 kg^-1 s^-2) [CODATA 2022]
    pub const H: f64 = 6.62607015e-34;                    // Planck sabiti (J s) [2019 SI Tam]
    pub const HBAR: f64 = 1.054_571_817_646_156_5e-34;    // İndirgenmiş Planck h/(2*pi) (J s) [Tam]
    pub const K_B: f64 = 1.380649e-23;                    // Boltzmann (J/K) [2019 SI Tam]
    pub const E: f64 = 1.602176634e-19;                   // Temel yük (C) [2019 SI Tam]
    pub const ALPHA: f64 = 7.297_352_564_3e-3;            // İnce yapı sabiti [CODATA 2022]
    pub const MU_0: f64 = 1.256_637_061_27e-6;            // Vakum manyetik geçirgenliği (N/A^2) [CODATA 2022]
    pub const EPS_0: f64 = 8.854_187_818_8e-12;           // Vakum elektriksel geçirgenliği (F/m) [CODATA 2022]
    pub const SIGMA_SB: f64 = 5.670_374_419_184_429_4e-8; // Stefan-Boltzmann (W m^-2 K^-4) [2019 SI Tam]
    pub const N_A: f64 = 6.02214076e23;                   // Avogadro sayısı (mol^-1) [2019 SI Tam]
    pub const R_GAS: f64 = 8.314_462_618_153_24;          // Evrensel gaz sabiti N_A*k_B (J/(mol*K)) [Tam]
    pub const G0: f64 = 9.80665;                          // Standart yerçekimi (m/s^2) [CGPM Tam]

    // Dünya ve Yörünge Parametreleri
    pub const M_EARTH: f64 = 5.97219e24;                  // Dünya Kütlesi (kg)
    pub const R_EARTH: f64 = 6_371_000.0;                 // Dünya Yarıçapı (m)
    pub const MU_EARTH: f64 = 3.986004418e14;             // Standart Çekim GM (m^3/s^2)
    pub const V_ORBIT_LEO: f64 = 7_800.0;                 // LEO Yörünge Hızı (m/s)
    pub const V_ESCAPE_EARTH: f64 = 11_186.0;             // Kaçış Hızı (m/s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_2019_si_exact_definitions_and_codata_2022_identities() {
        // hbar == h / (2*pi)
        assert!((PhysicalConstants::HBAR - PhysicalConstants::H / (2.0 * PI)).abs() < 1e-49);
        // R_gas == N_A * k_B (2019 SI exact)
        assert!((PhysicalConstants::R_GAS - PhysicalConstants::N_A * PhysicalConstants::K_B).abs() < 1e-14);
        // Stefan-Boltzmann exact 2019 SI identity: pi^2 * k_B^4 / (60 * hbar^3 * c^2)
        let sigma_exact = (PI.powi(2) * PhysicalConstants::K_B.powi(4))
            / (60.0 * PhysicalConstants::HBAR.powi(3) * PhysicalConstants::C.powi(2));
        assert!((PhysicalConstants::SIGMA_SB - sigma_exact).abs() / sigma_exact < 1e-15);
        // Maxwell vacuum identity: mu_0 * eps_0 * c^2 == 1
        let maxwell = PhysicalConstants::MU_0 * PhysicalConstants::EPS_0 * PhysicalConstants::C.powi(2);
        assert!((maxwell - 1.0).abs() < 2e-10);
        // QED fine-structure identity: 2 * alpha * h / (mu_0 * c * e^2) == 1
        let qed = (2.0 * PhysicalConstants::ALPHA * PhysicalConstants::H)
            / (PhysicalConstants::MU_0 * PhysicalConstants::C * PhysicalConstants::E.powi(2));
        assert!((qed - 1.0).abs() < 2e-10);
    }
}
