// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Boyut Analizi ve Ölçekli SI Birim Kalkanı (Rust Dimensional Engine)

use serde::{Deserialize, Serialize};
use std::fmt;

/// 7 Temel SI Boyut Vektörü (Z^7 Serbest Abelyen Grubu).
/// 7 bayt yer kaplar (CPU yazmacında tek çevrimde taşınır, sıfır heap tahsisi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Dimension {
    pub m: i8,     // Kütle [kg]
    pub l: i8,     // Uzunluk [m]
    pub t: i8,     // Zaman [s]
    pub i: i8,     // Akım [A]
    pub theta: i8, // Sıcaklık [K]
    pub n: i8,     // Madde miktarı [mol]
    pub j: i8,     // Işık şiddeti [cd]
}

impl Dimension {
    pub const fn new(m: i8, l: i8, t: i8, i: i8, theta: i8, n: i8, j: i8) -> Self {
        Self { m, l, t, i, theta, n, j }
    }

    pub const fn dimensionless() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0)
    }

    pub const fn mass() -> Self { Self::new(1, 0, 0, 0, 0, 0, 0) }
    pub const fn length() -> Self { Self::new(0, 1, 0, 0, 0, 0, 0) }
    pub const fn time() -> Self { Self::new(0, 0, 1, 0, 0, 0, 0) }
    pub const fn current() -> Self { Self::new(0, 0, 0, 1, 0, 0, 0) }
    pub const fn temperature() -> Self { Self::new(0, 0, 0, 0, 1, 0, 0) }
    pub const fn amount() -> Self { Self::new(0, 0, 0, 0, 0, 1, 0) }
    pub const fn luminous() -> Self { Self::new(0, 0, 0, 0, 0, 0, 1) }

    pub const fn area() -> Self { Self::new(0, 2, 0, 0, 0, 0, 0) }
    pub const fn volume() -> Self { Self::new(0, 3, 0, 0, 0, 0, 0) }
    pub const fn velocity() -> Self { Self::new(0, 1, -1, 0, 0, 0, 0) }
    pub const fn acceleration() -> Self { Self::new(0, 1, -2, 0, 0, 0, 0) }
    pub const fn force() -> Self { Self::new(1, 1, -2, 0, 0, 0, 0) }
    pub const fn momentum() -> Self { Self::new(1, 1, -1, 0, 0, 0, 0) }
    pub const fn energy() -> Self { Self::new(1, 2, -2, 0, 0, 0, 0) }
    pub const fn power() -> Self { Self::new(1, 2, -3, 0, 0, 0, 0) }
    pub const fn pressure() -> Self { Self::new(1, -1, -2, 0, 0, 0, 0) }
    pub const fn density() -> Self { Self::new(1, -3, 0, 0, 0, 0, 0) }
    pub const fn mass_flow_rate() -> Self { Self::new(1, 0, -1, 0, 0, 0, 0) }
    pub const fn angular_velocity() -> Self { Self::new(0, 0, -1, 0, 0, 0, 0) }
    pub const fn moment_of_inertia() -> Self { Self::new(1, 2, 0, 0, 0, 0, 0) }

    pub const fn specific_impulse_s() -> Self { Self::new(0, 0, 1, 0, 0, 0, 0) }
    pub const fn specific_energy() -> Self { Self::new(0, 2, -2, 0, 0, 0, 0) }
    pub const fn gravitational_parameter() -> Self { Self::new(0, 3, -2, 0, 0, 0, 0) }

    pub const fn heat_capacity() -> Self { Self::new(1, 2, -2, 0, -1, 0, 0) }
    pub const fn specific_heat() -> Self { Self::new(0, 2, -2, 0, -1, 0, 0) }
    pub const fn radiation_flux() -> Self { Self::new(1, 0, -3, 0, 0, 0, 0) }
    pub const fn stefan_boltzmann_const() -> Self { Self::new(1, 0, -3, 0, -4, 0, 0) }

    pub const fn electric_charge() -> Self { Self::new(0, 0, 1, 1, 0, 0, 0) }
    pub const fn voltage() -> Self { Self::new(1, 2, -3, -1, 0, 0, 0) }
    pub const fn electric_field() -> Self { Self::new(1, 1, -3, -1, 0, 0, 0) }
    pub const fn magnetic_field() -> Self { Self::new(1, 0, -2, -1, 0, 0, 0) }
    pub const fn permeability() -> Self { Self::new(1, 1, -2, -2, 0, 0, 0) }
    pub const fn permittivity() -> Self { Self::new(-1, -3, 4, 2, 0, 0, 0) }
    pub const fn inductance_gradient() -> Self { Self::new(1, 1, -2, -2, 0, 0, 0) }
    pub const fn magnetic_flux() -> Self { Self::new(1, 2, -2, -1, 0, 0, 0) }
    pub const fn resistance() -> Self { Self::new(1, 2, -3, -2, 0, 0, 0) }
    pub const fn electrical_resistivity() -> Self { Self::new(1, 3, -3, -2, 0, 0, 0) }
    pub const fn electrical_conductivity() -> Self { Self::new(-1, -3, 3, 2, 0, 0, 0) }
    pub const fn current_density() -> Self { Self::new(0, -2, 0, 1, 0, 0, 0) }
    pub const fn charge_density() -> Self { Self::new(0, -3, 1, 1, 0, 0, 0) }
    pub const fn inductance() -> Self { Self::new(1, 2, -2, -2, 0, 0, 0) }
    pub const fn capacitance() -> Self { Self::new(-1, -2, 4, 2, 0, 0, 0) }

    pub const fn frequency() -> Self { Self::new(0, 0, -1, 0, 0, 0, 0) }
    pub const fn thermal_conductivity() -> Self { Self::new(1, 1, -3, 0, -1, 0, 0) }
    pub const fn heat_transfer_coeff() -> Self { Self::new(1, 0, -3, 0, -1, 0, 0) }
    pub const fn thermal_expansion_coeff() -> Self { Self::new(0, 0, 0, 0, -1, 0, 0) }
    pub const fn dynamic_viscosity() -> Self { Self::new(1, -1, -1, 0, 0, 0, 0) }
    pub const fn kinematic_viscosity() -> Self { Self::new(0, 2, -1, 0, 0, 0, 0) }
    pub const fn surface_energy() -> Self { Self::new(1, 0, -2, 0, 0, 0, 0) }
    pub const fn molar_mass() -> Self { Self::new(1, 0, 0, 0, 0, -1, 0) }
    pub const fn molar_gas_constant() -> Self { Self::new(1, 2, -2, 0, -1, -1, 0) }
    pub const fn angular_momentum() -> Self { Self::new(1, 2, -1, 0, 0, 0, 0) }
    pub const fn torque() -> Self { Self::new(1, 2, -2, 0, 0, 0, 0) }
    pub const fn impulse() -> Self { Self::new(1, 1, -1, 0, 0, 0, 0) }
    pub const fn wavenumber() -> Self { Self::new(0, -1, 0, 0, 0, 0, 0) }
    pub const fn number_density() -> Self { Self::new(0, -3, 0, 0, 0, 0, 0) }
    pub const fn volumetric_flow_rate() -> Self { Self::new(0, 3, -1, 0, 0, 0, 0) }
    pub const fn area_moment_of_inertia() -> Self { Self::new(0, 4, 0, 0, 0, 0, 0) }
    pub const fn gravitational_constant() -> Self { Self::new(-1, 3, -2, 0, 0, 0, 0) }
    pub const fn areal_density() -> Self { Self::new(1, -2, 0, 0, 0, 0, 0) }
    pub const fn linear_density() -> Self { Self::new(1, -1, 0, 0, 0, 0, 0) }
    pub const fn compressibility() -> Self { Self::new(-1, 1, 2, 0, 0, 0, 0) }
    pub const fn molar_energy() -> Self { Self::new(1, 2, -2, 0, 0, -1, 0) }
    pub const fn molar_concentration() -> Self { Self::new(0, -3, 0, 0, 0, 1, 0) }
    pub const fn molar_charge() -> Self { Self::new(0, 0, 1, 1, 0, -1, 0) }
    pub const fn carrier_mobility() -> Self { Self::new(-1, 0, 2, 1, 0, 0, 0) }
    pub const fn acoustic_impedance() -> Self { Self::new(1, -2, -1, 0, 0, 0, 0) }
    pub const fn damping_coeff() -> Self { Self::new(1, 0, -1, 0, 0, 0, 0) }
    pub const fn lorenz_number() -> Self { Self::new(2, 4, -6, -2, -2, 0, 0) }
    pub const fn wien_constant() -> Self { Self::new(0, 1, 0, 0, 1, 0, 0) }
    pub const fn molar_flux() -> Self { Self::new(0, -2, -1, 0, 0, 1, 0) }
    pub const fn thermal_resistance() -> Self { Self::new(-1, -2, 3, 0, 1, 0, 0) }
    pub const fn molar_attenuation() -> Self { Self::new(0, 2, 0, 0, 0, -1, 0) }
    pub const fn illuminance() -> Self { Self::new(0, -2, 0, 0, 0, 0, 1) }
    pub const fn luminous_efficacy() -> Self { Self::new(-1, -2, 3, 0, 0, 0, 1) }
    pub const fn magnetic_moment() -> Self { Self::new(0, 2, 0, 1, 0, 0, 0) }
    pub const fn seebeck_coeff() -> Self { Self::new(1, 2, -3, -1, -1, 0, 0) }
    pub const fn triple_product() -> Self { Self::new(0, -3, 1, 0, 1, 0, 0) }
    pub const fn stress_intensity_sq() -> Self { Self::new(2, -1, -4, 0, 0, 0, 0) }
    pub const fn volumetric_heat_gen() -> Self { Self::new(1, -1, -3, 0, 0, 0, 0) }

    pub const fn is_dimensionless(&self) -> bool {
        self.m == 0 && self.l == 0 && self.t == 0 && self.i == 0 &&
        self.theta == 0 && self.n == 0 && self.j == 0
    }

    #[inline(always)]
    pub fn mul_dim(&self, other: &Self) -> Self {
        Self {
            m: self.m.saturating_add(other.m),
            l: self.l.saturating_add(other.l),
            t: self.t.saturating_add(other.t),
            i: self.i.saturating_add(other.i),
            theta: self.theta.saturating_add(other.theta),
            n: self.n.saturating_add(other.n),
            j: self.j.saturating_add(other.j),
        }
    }

    #[inline]
    pub fn checked_mul_dim(&self, other: &Self) -> Result<Self, String> {
        Ok(Self {
            m: self.m.checked_add(other.m).ok_or_else(|| "Z^7 boyut üssü taşması (M)".to_string())?,
            l: self.l.checked_add(other.l).ok_or_else(|| "Z^7 boyut üssü taşması (L)".to_string())?,
            t: self.t.checked_add(other.t).ok_or_else(|| "Z^7 boyut üssü taşması (T)".to_string())?,
            i: self.i.checked_add(other.i).ok_or_else(|| "Z^7 boyut üssü taşması (I)".to_string())?,
            theta: self.theta.checked_add(other.theta).ok_or_else(|| "Z^7 boyut üssü taşması (Theta)".to_string())?,
            n: self.n.checked_add(other.n).ok_or_else(|| "Z^7 boyut üssü taşması (N)".to_string())?,
            j: self.j.checked_add(other.j).ok_or_else(|| "Z^7 boyut üssü taşması (J)".to_string())?,
        })
    }

    #[inline(always)]
    pub fn div_dim(&self, other: &Self) -> Self {
        Self {
            m: self.m.saturating_sub(other.m),
            l: self.l.saturating_sub(other.l),
            t: self.t.saturating_sub(other.t),
            i: self.i.saturating_sub(other.i),
            theta: self.theta.saturating_sub(other.theta),
            n: self.n.saturating_sub(other.n),
            j: self.j.saturating_sub(other.j),
        }
    }

    #[inline]
    pub fn checked_div_dim(&self, other: &Self) -> Result<Self, String> {
        Ok(Self {
            m: self.m.checked_sub(other.m).ok_or_else(|| "Z^7 boyut üssü taşması (M)".to_string())?,
            l: self.l.checked_sub(other.l).ok_or_else(|| "Z^7 boyut üssü taşması (L)".to_string())?,
            t: self.t.checked_sub(other.t).ok_or_else(|| "Z^7 boyut üssü taşması (T)".to_string())?,
            i: self.i.checked_sub(other.i).ok_or_else(|| "Z^7 boyut üssü taşması (I)".to_string())?,
            theta: self.theta.checked_sub(other.theta).ok_or_else(|| "Z^7 boyut üssü taşması (Theta)".to_string())?,
            n: self.n.checked_sub(other.n).ok_or_else(|| "Z^7 boyut üssü taşması (N)".to_string())?,
            j: self.j.checked_sub(other.j).ok_or_else(|| "Z^7 boyut üssü taşması (J)".to_string())?,
        })
    }

    #[inline(always)]
    pub fn pow_dim(&self, p: i8) -> Self {
        Self {
            m: self.m.saturating_mul(p),
            l: self.l.saturating_mul(p),
            t: self.t.saturating_mul(p),
            i: self.i.saturating_mul(p),
            theta: self.theta.saturating_mul(p),
            n: self.n.saturating_mul(p),
            j: self.j.saturating_mul(p),
        }
    }

    /// Rasyonel üs (p / q): Z^7 kafesindeki tüm üslerin q ile tam bölündüğünü ve i8 sınırını aşmadığını doğrular.
    pub fn rational_pow(&self, p: i32, q: i32) -> Result<Self, String> {
        if q == 0 {
            return Err("Üs paydası sıfır olamaz!".to_string());
        }
        let q64 = i64::from(q);
        let p64 = i64::from(p);
        let comps = [
            i64::from(self.m) * p64,
            i64::from(self.l) * p64,
            i64::from(self.t) * p64,
            i64::from(self.i) * p64,
            i64::from(self.theta) * p64,
            i64::from(self.n) * p64,
            i64::from(self.j) * p64,
        ];
        if comps.iter().any(|&c| c % q64 != 0) {
            return Err(format!(
                "Kesirli boyut hatası: [{}] ^ ({}/{}) işlemi Z^7 kafesinde tam bölünmüyor!",
                self, p, q
            ));
        }
        let conv = |val: i64| -> Result<i8, String> {
            i8::try_from(val / q64).map_err(|_| {
                format!(
                    "Z^7 boyut üssü taşması: [{}] ^ ({}/{}) sonucu i8 sınırını aşıyor!",
                    self, p, q
                )
            })
        };
        Ok(Self {
            m: conv(comps[0])?,
            l: conv(comps[1])?,
            t: conv(comps[2])?,
            i: conv(comps[3])?,
            theta: conv(comps[4])?,
            n: conv(comps[5])?,
            j: conv(comps[6])?,
        })
    }

    /// Karekök işleminde Z^7 kafesindeki tüm üslerin çift sayı olduğunu doğrular ve 2'ye böler.
    pub fn sqrt_dim(&self) -> Result<Self, String> {
        self.rational_pow(1, 2)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "dimensionless" => Some(Self::dimensionless()),
            "mass" => Some(Self::mass()),
            "length" => Some(Self::length()),
            "time" | "specific_impulse_s" => Some(Self::time()),
            "current" => Some(Self::current()),
            "temperature" => Some(Self::temperature()),
            "amount" => Some(Self::amount()),
            "luminous" => Some(Self::luminous()),
            "area" => Some(Self::area()),
            "volume" => Some(Self::volume()),
            "velocity" | "specific_impulse_v" => Some(Self::velocity()),
            "acceleration" => Some(Self::acceleration()),
            "force" => Some(Self::force()),
            "momentum" => Some(Self::momentum()),
            "energy" => Some(Self::energy()),
            "power" => Some(Self::power()),
            "pressure" | "stress" | "magnetic_pressure" => Some(Self::pressure()),
            "density" => Some(Self::density()),
            "mass_flow_rate" => Some(Self::mass_flow_rate()),
            "angular_velocity" | "frequency" | "rate" => Some(Self::angular_velocity()),
            "moment_of_inertia" => Some(Self::moment_of_inertia()),
            "specific_energy" | "specific_strength" => Some(Self::specific_energy()),
            "gravitational_parameter" => Some(Self::gravitational_parameter()),
            "heat_capacity" | "entropy" | "boltzmann_const" => Some(Self::heat_capacity()),
            "specific_heat" | "gas_constant" | "specific_entropy" => Some(Self::specific_heat()),
            "radiation_flux" => Some(Self::radiation_flux()),
            "stefan_boltzmann_const" => Some(Self::stefan_boltzmann_const()),
            "electric_charge" => Some(Self::electric_charge()),
            "voltage" => Some(Self::voltage()),
            "electric_field" => Some(Self::electric_field()),
            "magnetic_field" => Some(Self::magnetic_field()),
            "permeability" | "inductance_gradient" => Some(Self::permeability()),
            "permittivity" => Some(Self::permittivity()),
            "magnetic_flux" => Some(Self::magnetic_flux()),
            "resistance" => Some(Self::resistance()),
            "electrical_resistivity" => Some(Self::electrical_resistivity()),
            "electrical_conductivity" => Some(Self::electrical_conductivity()),
            "current_density" => Some(Self::current_density()),
            "charge_density" => Some(Self::charge_density()),
            "inductance" => Some(Self::inductance()),
            "capacitance" => Some(Self::capacitance()),
            "thermal_conductivity" => Some(Self::thermal_conductivity()),
            "heat_transfer_coeff" => Some(Self::heat_transfer_coeff()),
            "thermal_expansion_coeff" => Some(Self::thermal_expansion_coeff()),
            "dynamic_viscosity" => Some(Self::dynamic_viscosity()),
            "kinematic_viscosity" | "thermal_diffusivity" | "mass_diffusivity" | "specific_angular_momentum" => {
                Some(Self::kinematic_viscosity())
            }
            "surface_energy" | "surface_tension" | "spring_stiffness" => Some(Self::surface_energy()),
            "molar_mass" => Some(Self::molar_mass()),
            "molar_gas_constant" => Some(Self::molar_gas_constant()),
            "angular_momentum" | "action" => Some(Self::angular_momentum()),
            "torque" => Some(Self::torque()),
            "impulse" => Some(Self::impulse()),
            "wavenumber" => Some(Self::wavenumber()),
            "number_density" => Some(Self::number_density()),
            "volumetric_flow_rate" => Some(Self::volumetric_flow_rate()),
            "area_moment_of_inertia" => Some(Self::area_moment_of_inertia()),
            "gravitational_constant" => Some(Self::gravitational_constant()),
            "areal_density" => Some(Self::areal_density()),
            "linear_density" => Some(Self::linear_density()),
            "compressibility" => Some(Self::compressibility()),
            "molar_energy" => Some(Self::molar_energy()),
            "molar_concentration" => Some(Self::molar_concentration()),
            "molar_charge" => Some(Self::molar_charge()),
            "carrier_mobility" => Some(Self::carrier_mobility()),
            "acoustic_impedance" => Some(Self::acoustic_impedance()),
            "damping_coeff" => Some(Self::damping_coeff()),
            "lorenz_number" => Some(Self::lorenz_number()),
            "wien_constant" => Some(Self::wien_constant()),
            "molar_flux" => Some(Self::molar_flux()),
            "thermal_resistance" => Some(Self::thermal_resistance()),
            "molar_attenuation" => Some(Self::molar_attenuation()),
            "illuminance" => Some(Self::illuminance()),
            "luminous_efficacy" => Some(Self::luminous_efficacy()),
            "magnetic_moment" => Some(Self::magnetic_moment()),
            "seebeck_coeff" => Some(Self::seebeck_coeff()),
            "triple_product" => Some(Self::triple_product()),
            "stress_intensity_sq" => Some(Self::stress_intensity_sq()),
            "volumetric_heat_gen" => Some(Self::volumetric_heat_gen()),
            _ => None,
        }
    }
}

/// Faz 1: Ölçekli SI Birimi (ScaledUnit).
/// Hem Z^7 boyut vektörünü hem de temel SI birimine (m, kg, s, Pa, W, K) dönüşüm çarpanını taşır.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledUnit {
    pub symbol: &'static str,
    pub si_scale: f64,
    pub dimension: Dimension,
}

impl ScaledUnit {
    pub const fn new(symbol: &'static str, si_scale: f64, dimension: Dimension) -> Self {
        Self { symbol, si_scale, dimension }
    }

    pub fn to_si(&self, value: f64) -> (f64, Dimension) {
        (value * self.si_scale, self.dimension)
    }

    pub fn from_unit_str(unit: &str) -> Option<Self> {
        match unit {
            "m" => Some(Self::new("m", 1.0, Dimension::length())),
            "mm" => Some(Self::new("mm", 1e-3, Dimension::length())),
            "km" => Some(Self::new("km", 1e3, Dimension::length())),
            "kg" => Some(Self::new("kg", 1.0, Dimension::mass())),
            "g" => Some(Self::new("g", 1e-3, Dimension::mass())),
            "ton" => Some(Self::new("ton", 1e3, Dimension::mass())),
            "s" => Some(Self::new("s", 1.0, Dimension::time())),
            "ms" => Some(Self::new("ms", 1e-3, Dimension::time())),
            "m/s" => Some(Self::new("m/s", 1.0, Dimension::velocity())),
            "km/s" => Some(Self::new("km/s", 1e3, Dimension::velocity())),
            "km/h" => Some(Self::new("km/h", 1.0 / 3.6, Dimension::velocity())),
            "N" => Some(Self::new("N", 1.0, Dimension::force())),
            "kN" => Some(Self::new("kN", 1e3, Dimension::force())),
            "MN" => Some(Self::new("MN", 1e6, Dimension::force())),
            "Pa" => Some(Self::new("Pa", 1.0, Dimension::pressure())),
            "kPa" => Some(Self::new("kPa", 1e3, Dimension::pressure())),
            "MPa" => Some(Self::new("MPa", 1e6, Dimension::pressure())),
            "GPa" => Some(Self::new("GPa", 1e9, Dimension::pressure())),
            "W" => Some(Self::new("W", 1.0, Dimension::power())),
            "kW" => Some(Self::new("kW", 1e3, Dimension::power())),
            "MW" => Some(Self::new("MW", 1e6, Dimension::power())),
            "GW" => Some(Self::new("GW", 1e9, Dimension::power())),
            "K" => Some(Self::new("K", 1.0, Dimension::temperature())),
            _ => None,
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dimensionless() {
            return write!(f, "1 (Boyutsuz)");
        }
        let labels = [
            ("M", self.m), ("L", self.l), ("T", self.t),
            ("I", self.i), ("Theta", self.theta), ("N", self.n), ("J", self.j),
        ];
        let mut first = true;
        for (sym, exp) in labels {
            if exp == 1 {
                if !first { write!(f, " ")?; }
                write!(f, "{}", sym)?;
                first = false;
            } else if exp != 0 {
                if !first { write!(f, " ")?; }
                write!(f, "{}^{}", sym, exp)?;
                first = false;
            }
        }
        Ok(())
    }
}

pub fn check_dimensional_consistency(lhs: Dimension, rhs: Dimension) -> Result<(), String> {
    if lhs == rhs {
        Ok(())
    } else {
        Err(format!("Boyut Uyuşmazlığı: Sol [{}] != Sağ [{}]", lhs, rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z7_abelian_group_axioms_across_all_seven_si_axes() {
        let axes = [
            Dimension::mass(),
            Dimension::length(),
            Dimension::time(),
            Dimension::current(),
            Dimension::temperature(),
            Dimension::amount(),
            Dimension::luminous(),
        ];
        let id = Dimension::dimensionless();
        for a in axes {
            assert_eq!(a.mul_dim(&id), a);
            assert_eq!(a.div_dim(&a), id);
            assert!(!a.is_dimensionless());
        }
        // Birleşmelilik ve değişmelilik: (F * L) / T == P (Güç)
        let power_via_fl = Dimension::force().mul_dim(&Dimension::length()).div_dim(&Dimension::time());
        let power_via_vi = Dimension::voltage().mul_dim(&Dimension::current());
        assert_eq!(power_via_fl, Dimension::power());
        assert_eq!(power_via_vi, Dimension::power());
        // 7. SI ekseni (kandela J): illuminance * area == luminous
        assert_eq!(Dimension::illuminance().mul_dim(&Dimension::area()), Dimension::luminous());
        assert_eq!(Dimension::luminous_efficacy().mul_dim(&Dimension::power()), Dimension::luminous());
    }

    #[test]
    fn test_rational_power_divisibility_and_rejection() {
        let v_sq = Dimension::velocity().pow_dim(2);
        assert_eq!(v_sq.rational_pow(1, 2).unwrap(), Dimension::velocity());
        assert_eq!(Dimension::volume().rational_pow(1, 3).unwrap(), Dimension::length());
        assert!(Dimension::length().rational_pow(1, 2).is_err(), "sqrt(L) Z^7 kafesinde yasak!");
        assert!(Dimension::area().rational_pow(1, 3).is_err(), "cbrt(L^2) Z^7 kafesinde yasak!");
        assert!(Dimension::length().rational_pow(1, 0).is_err(), "q=0 reddedilmeli!");
    }

    #[test]
    fn test_checked_overflow_protection() {
        let huge = Dimension::new(100, 0, 0, 0, 0, 0, 0);
        assert!(huge.checked_mul_dim(&huge).is_err(), "i8 taşması reddedilmeli!");
        assert!(huge.rational_pow(2, 1).is_err(), "i8 üs taşması reddedilmeli!");
        assert!(Dimension::new(-100, 0, 0, 0, 0, 0, 0).checked_div_dim(&huge).is_err());
    }

    #[test]
    fn test_scaled_unit_conversions_and_from_name_lookup() {
        let (n, dim_f) = ScaledUnit::from_unit_str("kN").unwrap().to_si(2.5);
        assert!((n - 2500.0).abs() < 1e-12 && dim_f == Dimension::force());
        let (ms, dim_v) = ScaledUnit::from_unit_str("km/h").unwrap().to_si(36.0);
        assert!((ms - 10.0).abs() < 1e-12 && dim_v == Dimension::velocity());
        let (pa, dim_p) = ScaledUnit::from_unit_str("GPa").unwrap().to_si(210.0);
        assert!((pa - 210.0e9).abs() < 1.0 && dim_p == Dimension::pressure());
        assert!(ScaledUnit::from_unit_str("furlong").is_none());

        assert_eq!(Dimension::from_name("illuminance"), Some(Dimension::illuminance()));
        assert_eq!(Dimension::from_name("triple_product"), Some(Dimension::triple_product()));
        assert_eq!(Dimension::from_name("stress_intensity_sq"), Some(Dimension::stress_intensity_sq()));
        assert_eq!(Dimension::from_name("nonexistent_dim"), None);
    }

    #[test]
    fn test_dimensional_consistency_verdict() {
        assert!(check_dimensional_consistency(Dimension::energy(), Dimension::torque()).is_ok());
        assert!(check_dimensional_consistency(Dimension::force(), Dimension::energy()).is_err());
    }
}
