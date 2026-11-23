// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Tam Tamsayı Z^7 Çekirdek Uzayı (Nullspace) Buckingham-Pi Keşif ve Ölçekleme Motoru

use crate::ast::Expr;
use crate::dimensions::Dimension;
use crate::graph::{KnowledgeGraph, SynthesizedFormula, VarBound};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Taşma denetimli tam kesirli sayı (Q = i128 / i128, den > 0, gcd(|num|, den) == 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rat128 {
    num: i128,
    den: i128,
}

fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn gcd_i64(a: i64, b: i64) -> i64 {
    let g = gcd_u128(a.unsigned_abs() as u128, b.unsigned_abs() as u128);
    i64::try_from(g).unwrap_or(1)
}

impl Rat128 {
    const ZERO: Self = Self { num: 0, den: 1 };
    const ONE: Self = Self { num: 1, den: 1 };

    fn from_int(n: i128) -> Self {
        Self { num: n, den: 1 }
    }

    fn new(num: i128, den: i128) -> Result<Self, String> {
        if den == 0 {
            return Err("Kesir paydası sıfır olamaz!".to_string());
        }
        if num == 0 {
            return Ok(Self::ZERO);
        }
        let sign = if (num < 0) ^ (den < 0) { -1i128 } else { 1i128 };
        let un = num.unsigned_abs();
        let ud = den.unsigned_abs();
        let g = gcd_u128(un, ud);
        let rn = i128::try_from(un / g).map_err(|_| "Kesir pay taşması!".to_string())? * sign;
        let rd = i128::try_from(ud / g).map_err(|_| "Kesir payda taşması!".to_string())?;
        Ok(Self { num: rn, den: rd })
    }

    fn is_zero(self) -> bool {
        self.num == 0
    }

    fn abs_cmp(self, other: Self) -> std::cmp::Ordering {
        let lhs = self.num.unsigned_abs().saturating_mul(other.den.unsigned_abs());
        let rhs = other.num.unsigned_abs().saturating_mul(self.den.unsigned_abs());
        lhs.cmp(&rhs)
    }

    fn checked_neg(self) -> Result<Self, String> {
        let n = self.num.checked_neg().ok_or_else(|| "Kesir işaret taşması!".to_string())?;
        Ok(Self { num: n, den: self.den })
    }

    fn checked_add(self, rhs: Self) -> Result<Self, String> {
        let g = i128::try_from(gcd_u128(self.den.unsigned_abs(), rhs.den.unsigned_abs()))
            .map_err(|_| "GCD dönüşüm hatası!".to_string())?;
        let left_scale = rhs.den / g;
        let right_scale = self.den / g;
        let t1 = self
            .num
            .checked_mul(left_scale)
            .ok_or_else(|| "Kesir toplama çarpım taşması!".to_string())?;
        let t2 = rhs
            .num
            .checked_mul(right_scale)
            .ok_or_else(|| "Kesir toplama çarpım taşması!".to_string())?;
        let num = t1
            .checked_add(t2)
            .ok_or_else(|| "Kesir toplama taşması!".to_string())?;
        let den = self
            .den
            .checked_mul(left_scale)
            .ok_or_else(|| "Kesir ortak payda taşması!".to_string())?;
        Self::new(num, den)
    }

    fn checked_sub(self, rhs: Self) -> Result<Self, String> {
        self.checked_add(rhs.checked_neg()?)
    }

    fn checked_mul(self, rhs: Self) -> Result<Self, String> {
        if self.is_zero() || rhs.is_zero() {
            return Ok(Self::ZERO);
        }
        let g1 = gcd_u128(self.num.unsigned_abs(), rhs.den.unsigned_abs());
        let g2 = gcd_u128(rhs.num.unsigned_abs(), self.den.unsigned_abs());
        let n1 = self.num / i128::try_from(g1).map_err(|_| "GCD taşması!".to_string())?;
        let d2 = rhs.den / i128::try_from(g1).map_err(|_| "GCD taşması!".to_string())?;
        let n2 = rhs.num / i128::try_from(g2).map_err(|_| "GCD taşması!".to_string())?;
        let d1 = self.den / i128::try_from(g2).map_err(|_| "GCD taşması!".to_string())?;
        let num = n1
            .checked_mul(n2)
            .ok_or_else(|| "Kesir çarpım pay taşması!".to_string())?;
        let den = d1
            .checked_mul(d2)
            .ok_or_else(|| "Kesir çarpım payda taşması!".to_string())?;
        Self::new(num, den)
    }

    fn checked_div(self, rhs: Self) -> Result<Self, String> {
        if rhs.is_zero() {
            return Err("Sıfıra bölme hatası!".to_string());
        }
        let inv = Self::new(rhs.den, rhs.num)?;
        self.checked_mul(inv)
    }
}

/// Keşfedilen tekil boyutsuz Buckingham-Pi grubu: Pi_k = prod_j q_j^(alpha_j).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiGroup {
    pub index: usize,
    pub symbol: String,
    pub recognized_number: Option<String>,
    /// Orijinal değişken sırasına göre sıfır olmayan (değişken, üs) çiftleri.
    pub exponents: Vec<(String, i64)>,
    pub expression: String,
    pub latex: String,
    pub verified_dimension: String,
    #[serde(skip)]
    pub symbolic_expr: Option<Expr>,
}

/// Deneysel veya simülasyon örneklerinden log-Pi uzayında kalibre edilen ölçekleme yasası.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalScalingFit {
    pub coefficient_c_pi: f64,
    pub regime_exponents: Vec<(String, f64)>,
    pub fitted_equation: String,
    pub r_squared: f64,
    pub max_relative_error: f64,
    pub sample_count: usize,
}

/// Tam Z^7 Buckingham-Pi keşif raporu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuckinghamDiscoveryReport {
    pub variables: Vec<(String, String)>,
    pub num_variables: usize,
    pub dimensional_rank: usize,
    pub num_pi_groups: usize,
    pub repeating_basis_variables: Vec<String>,
    pub pi_groups: Vec<PiGroup>,
    pub target_variable: Option<String>,
    pub target_identifiable: bool,
    pub target_unidentifiable_reason: Option<String>,
    pub target_scaling_equation: Option<String>,
    pub target_scaling_dimension: Option<String>,
    pub safe_rust_fn: Option<String>,
    pub empirical_fit: Option<EmpiricalScalingFit>,
}

pub struct BuckinghamPiSolver;

impl BuckinghamPiSolver {
    /// Virgülle ayrılmış ("F_drag,rho,v,L_char,mu_visc" veya "r:length,E:energy,rho:density,t:time")
    /// ya da JSON nesnesi ('{"r": "length", ...}') biçimindeki değişken tanımlarını Z^7'ye çözer.
    pub fn parse_variable_specs(spec: &str) -> Result<Vec<(String, Dimension)>, String> {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            return Err("Değişken listesi boş olamaz!".to_string());
        }

        let kg = KnowledgeGraph::new();
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();

        if trimmed.starts_with('{') {
            let raw_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(trimmed)
                .map_err(|e| format!("Değişken JSON nesnesi ayrıştırılamadı: {}", e))?;
            for (name, val) in raw_map {
                let dim_str = val
                    .as_str()
                    .ok_or_else(|| format!("Değişken '{}' için boyut adı metin olmalıdır!", name))?;
                let dim = Self::resolve_dimension(&name, Some(dim_str), &kg)?;
                if seen.insert(name.clone()) {
                    out.push((name, dim));
                }
            }
        } else {
            for token in trimmed.split(',') {
                let tok = token.trim();
                if tok.is_empty() {
                    continue;
                }
                let (name, explicit_dim) = match tok.split_once(':') {
                    Some((n, d)) => (n.trim().to_string(), Some(d.trim())),
                    None => (tok.to_string(), None),
                };
                if name.is_empty() {
                    return Err(format!("Geçersiz değişken tanımı: '{}'", tok));
                }
                let dim = Self::resolve_dimension(&name, explicit_dim, &kg)?;
                if seen.insert(name.clone()) {
                    out.push((name, dim));
                }
            }
        }

        if out.is_empty() {
            return Err("En az bir fiziksel değişken belirtilmelidir!".to_string());
        }
        Ok(out)
    }

    fn resolve_dimension(
        var_name: &str,
        explicit_dim: Option<&str>,
        kg: &KnowledgeGraph,
    ) -> Result<Dimension, String> {
        if let Some(d_key) = explicit_dim {
            if let Some(d) = Dimension::from_name(d_key) {
                return Ok(d);
            }
            if let Some(su) = crate::dimensions::ScaledUnit::from_unit_str(d_key) {
                return Ok(su.dimension);
            }
            return Err(format!(
                "Değişken '{}' için geçersiz boyut/birim anahtarı: '{}'",
                var_name, d_key
            ));
        }

        if let Some(&d) = kg.variable_dimensions.get(var_name) {
            return Ok(d);
        }

        // Yaygın temel fizik sembolleri için güvenli yedek sözlük
        match var_name {
            "G" => Ok(Dimension::gravitational_constant()),
            "c" | "a_sound" | "c_s" => Ok(Dimension::velocity()),
            "h" | "hbar" => Ok(Dimension::angular_momentum()),
            "k_B" => Ok(Dimension::heat_capacity()),
            "E" => Ok(Dimension::energy()),
            "L" | "D" | "d" => Ok(Dimension::length()),
            "t" => Ok(Dimension::time()),
            "m" => Ok(Dimension::mass()),
            "v" | "u" => Ok(Dimension::velocity()),
            "g" | "g_0" => Ok(Dimension::acceleration()),
            "rho" => Ok(Dimension::density()),
            "mu_dyn" => Ok(Dimension::dynamic_viscosity()),
            "nu" | "nu_kin" => Ok(Dimension::kinematic_viscosity()),
            "sigma_surf" => Ok(Dimension::surface_energy()),
            _ => Err(format!(
                "Değişken '{}' SSOT ONTOLOGY.json içinde bulunamadı; '{}:boyut_adi' biçiminde açık boyut belirtin.",
                var_name, var_name
            )),
        }
    }

    fn dim_components(d: Dimension) -> [i8; 7] {
        [d.m, d.l, d.t, d.i, d.theta, d.n, d.j]
    }

    fn dim_nnz_and_l1(d: Dimension) -> (usize, i32) {
        let comps = Self::dim_components(d);
        let nnz = comps.iter().filter(|&&c| c != 0).count();
        let l1: i32 = comps.iter().map(|&c| i32::from(c).abs()).sum();
        (nnz, l1)
    }

    /// Tekrarlayan (pivot) değişkenleri öne, hedef değişkeni (target_var) en sona yerleştirerek
    /// hedef değişkenin tek bir boyutsuz Pi grubunda izole edilmesini garanti eder.
    fn order_columns(
        vars: &[(String, Dimension)],
        target_var: Option<&str>,
        repeating_hint: &[&str],
    ) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..vars.len()).collect();
        indices.sort_by_key(|&idx| {
            let (ref name, dim) = vars[idx];
            let is_target = target_var.is_some_and(|t| t == name.as_str());
            if is_target {
                return (3u8, usize::MAX, i32::MAX, idx);
            }
            if let Some(pos) = repeating_hint.iter().position(|&r| r == name.as_str()) {
                return (0u8, pos, 0, idx);
            }
            if dim.is_dimensionless() {
                return (2u8, 0, 0, idx);
            }
            let (nnz, l1) = Self::dim_nnz_and_l1(dim);
            (1u8, nnz, l1, idx)
        });
        indices
    }

    /// Tam kesirli Gauss-Jordan indirgenmiş satır basamak formu (RREF):
    /// Girdi: 7 x n boyut matrisi. Çıktı: (RREF matrisi, pivot sütun indeksleri).
    fn rref_exact(mut mat: Vec<Vec<Rat128>>, ncols: usize) -> Result<(Vec<Vec<Rat128>>, Vec<usize>), String> {
        let nrows = mat.len();
        let mut pivot_cols = Vec::new();
        let mut pivot_row = 0usize;

        for col in 0..ncols {
            if pivot_row >= nrows {
                break;
            }
            // En büyük mutlak değerli pivot satırını seç
            let mut best_r = None;
            for (r, row_vals) in mat.iter().enumerate().skip(pivot_row).take(nrows - pivot_row) {
                let v = row_vals[col];
                if !v.is_zero() {
                    best_r = match best_r {
                        None => Some(r),
                        Some(prev) => {
                            if v.abs_cmp(mat[prev][col]) == std::cmp::Ordering::Greater {
                                Some(r)
                            } else {
                                Some(prev)
                            }
                        }
                    };
                }
            }

            let Some(sel_row) = best_r else {
                continue;
            };
            mat.swap(pivot_row, sel_row);

            let piv_val = mat[pivot_row][col];
            for cell in mat[pivot_row].iter_mut().skip(col) {
                *cell = cell.checked_div(piv_val)?;
            }

            for r in 0..nrows {
                if r == pivot_row {
                    continue;
                }
                let factor = mat[r][col];
                if factor.is_zero() {
                    continue;
                }
                let piv_slice = mat[pivot_row].clone();
                for (c, &piv_c) in piv_slice.iter().enumerate().skip(col) {
                    let prod = factor.checked_mul(piv_c)?;
                    mat[r][c] = mat[r][c].checked_sub(prod)?;
                }
            }

            pivot_cols.push(col);
            pivot_row += 1;
        }

        Ok((mat, pivot_cols))
    }

    /// Kanonik boyutsuz sayıları (Re, Ma, Eu/Cd, Fr, We, Pr, Nu/Bi, St, Fo, Taylor-Sedov, Kepler)
    /// Z^7 boyut imzasından tanır.
    fn recognize_dimensionless_signature(
        vars: &[(String, Dimension)],
        exps: &[i64],
    ) -> Option<&'static str> {
        let mut pos: Vec<(&str, Dimension, i64)> = Vec::new();
        let mut neg: Vec<(&str, Dimension, i64)> = Vec::new();
        for ((name, dim), &e) in vars.iter().zip(exps.iter()) {
            if e > 0 {
                pos.push((name.as_str(), *dim, e));
            } else if e < 0 {
                neg.push((name.as_str(), *dim, -e));
            }
        }

        let has_dim = |list: &[(&str, Dimension, i64)], d: Dimension, exp: i64| -> bool {
            list.iter().any(|&(_, dm, e)| dm == d && e == exp)
        };

        // 1. Reynolds Sayısı: (rho * v * L) / mu
        if pos.len() == 3
            && neg.len() == 1
            && has_dim(&pos, Dimension::density(), 1)
            && has_dim(&pos, Dimension::velocity(), 1)
            && has_dim(&pos, Dimension::length(), 1)
            && has_dim(&neg, Dimension::dynamic_viscosity(), 1)
        {
            return Some("Reynolds Sayısı (Re)");
        }
        // Kinematik Reynolds: (v * L) / nu
        if pos.len() == 2
            && neg.len() == 1
            && has_dim(&pos, Dimension::velocity(), 1)
            && has_dim(&pos, Dimension::length(), 1)
            && has_dim(&neg, Dimension::kinematic_viscosity(), 1)
        {
            return Some("Reynolds Sayısı (Kinematik Re)");
        }

        // 2. Sürükleme / Kaldırma Kuvvet Katsayısı: F / (rho * v^2 * L^2) veya F / (rho * v^2 * A)
        if pos.len() == 1 && has_dim(&pos, Dimension::force(), 1) {
            if neg.len() == 3
                && has_dim(&neg, Dimension::density(), 1)
                && has_dim(&neg, Dimension::velocity(), 2)
                && has_dim(&neg, Dimension::length(), 2)
            {
                return Some("Aerodinamik Kuvvet / Newton Sürükleme Katsayısı (C_F / Ne)");
            }
            if neg.len() == 3
                && has_dim(&neg, Dimension::density(), 1)
                && has_dim(&neg, Dimension::velocity(), 2)
                && has_dim(&neg, Dimension::area(), 1)
            {
                return Some("Aerodinamik Kuvvet Katsayısı (C_d / C_L)");
            }
        }

        // 3. Euler / Basınç Katsayısı: P / (rho * v^2)
        if pos.len() == 1
            && neg.len() == 2
            && has_dim(&pos, Dimension::pressure(), 1)
            && has_dim(&neg, Dimension::density(), 1)
            && has_dim(&neg, Dimension::velocity(), 2)
        {
            return Some("Euler / Basınç Katsayısı (Eu / C_p)");
        }

        // 4. Froude Sayısı Karesi: v^2 / (g * L)
        if pos.len() == 1
            && neg.len() == 2
            && has_dim(&pos, Dimension::velocity(), 2)
            && has_dim(&neg, Dimension::acceleration(), 1)
            && has_dim(&neg, Dimension::length(), 1)
        {
            return Some("Froude Sayısı Karesi (Fr^2)");
        }

        // 5. Weber Sayısı: (rho * v^2 * L) / sigma_surf
        if pos.len() == 3
            && neg.len() == 1
            && has_dim(&pos, Dimension::density(), 1)
            && has_dim(&pos, Dimension::velocity(), 2)
            && has_dim(&pos, Dimension::length(), 1)
            && has_dim(&neg, Dimension::surface_energy(), 1)
        {
            return Some("Weber Sayısı (We)");
        }

        // 6. Prandtl Sayısı: (c_p * mu) / k_th
        if pos.len() == 2
            && neg.len() == 1
            && has_dim(&pos, Dimension::specific_heat(), 1)
            && has_dim(&pos, Dimension::dynamic_viscosity(), 1)
            && has_dim(&neg, Dimension::thermal_conductivity(), 1)
        {
            return Some("Prandtl Sayısı (Pr)");
        }

        // 7. Nusselt / Biot Sayısı: (h * L) / k_th
        if pos.len() == 2
            && neg.len() == 1
            && has_dim(&pos, Dimension::heat_transfer_coeff(), 1)
            && has_dim(&pos, Dimension::length(), 1)
            && has_dim(&neg, Dimension::thermal_conductivity(), 1)
        {
            return Some("Nusselt / Biot Sayısı (Nu / Bi)");
        }

        // 8. Strouhal Sayısı: (f * L) / v
        if pos.len() == 2
            && neg.len() == 1
            && has_dim(&pos, Dimension::frequency(), 1)
            && has_dim(&pos, Dimension::length(), 1)
            && has_dim(&neg, Dimension::velocity(), 1)
        {
            return Some("Strouhal Sayısı (St)");
        }

        // 9. Taylor-Sedov Patlama Dalga Sayısı: (rho * r^5) / (E * t^2)
        if pos.len() == 2
            && neg.len() == 2
            && has_dim(&pos, Dimension::density(), 1)
            && has_dim(&pos, Dimension::length(), 5)
            && has_dim(&neg, Dimension::energy(), 1)
            && has_dim(&neg, Dimension::time(), 2)
        {
            return Some("Taylor-Sedov Şok Dalgası Değişmezi (Pi_blast)");
        }

        // 10. Kepler 3. Yasa Kütleçekim Değişmezi: (mu * T^2) / a^3 veya (G * M * T^2) / a^3
        if neg.len() == 1 && has_dim(&neg, Dimension::length(), 3) {
            if pos.len() == 2
                && has_dim(&pos, Dimension::gravitational_parameter(), 1)
                && has_dim(&pos, Dimension::time(), 2)
            {
                return Some("Kepler 3. Yasa Yörünge Değişmezi (Pi_Kepler)");
            }
            if pos.len() == 3
                && has_dim(&pos, Dimension::gravitational_constant(), 1)
                && has_dim(&pos, Dimension::mass(), 1)
                && has_dim(&pos, Dimension::time(), 2)
            {
                return Some("Newton-Kepler 3. Yasa Değişmezi (Pi_Kepler)");
            }
        }

        // 11. Sarkaç / Yerçekimi Salınım Değişmezi: (g * T^2) / L
        if pos.len() == 2
            && neg.len() == 1
            && has_dim(&pos, Dimension::acceleration(), 1)
            && has_dim(&pos, Dimension::time(), 2)
            && has_dim(&neg, Dimension::length(), 1)
        {
            return Some("Yerçekimi Salınım / Sarkaç Değişmezi (g*T^2/L)");
        }

        // 12. Hız oranı (Mach / Görelilik beta): v_1 / v_2
        if pos.len() == 1
            && neg.len() == 1
            && pos[0].1 == Dimension::velocity()
            && neg[0].1 == Dimension::velocity()
        {
            let denom_name = neg[0].0;
            if matches!(denom_name, "c" | "a_sound" | "c_s") {
                return Some("Mach / Görelilik Hız Oranı (Ma / beta)");
            }
            return Some("Boyutsuz Hız Oranı");
        }

        // 13. Aynı boyutlu iki niceliğin oranı (Narinlik L/D, kütle oranı m_0/m_f, sıcaklık oranı T_h/T_c)
        if pos.len() == 1 && neg.len() == 1 && pos[0].1 == neg[0].1 && pos[0].2 == 1 && neg[0].2 == 1 {
            return Some("Benzerlik / Ölçek Oranı");
        }

        None
    }

    fn build_monomial_expr_and_latex(vars: &[(String, Dimension)], exps: &[i64]) -> (Expr, String) {
        let mut num_exprs: Vec<Expr> = Vec::new();
        let mut den_exprs: Vec<Expr> = Vec::new();
        let mut num_ltx: Vec<String> = Vec::new();
        let mut den_ltx: Vec<String> = Vec::new();

        for ((name, _), &e) in vars.iter().zip(exps.iter()) {
            if e > 0 {
                let v = Expr::Variable(name.clone());
                if e == 1 {
                    num_exprs.push(v);
                    num_ltx.push(name.clone());
                } else {
                    num_exprs.push(Expr::Pow(Box::new(v), Box::new(Expr::Constant(e as f64))));
                    num_ltx.push(format!("{}^{{{}}}", name, e));
                }
            } else if e < 0 {
                let ae = -e;
                let v = Expr::Variable(name.clone());
                if ae == 1 {
                    den_exprs.push(v);
                    den_ltx.push(name.clone());
                } else {
                    den_exprs.push(Expr::Pow(Box::new(v), Box::new(Expr::Constant(ae as f64))));
                    den_ltx.push(format!("{}^{{{}}}", name, ae));
                }
            }
        }

        let fold_prod = |mut items: Vec<Expr>| -> Expr {
            if items.is_empty() {
                return Expr::Constant(1.0);
            }
            let first = items.remove(0);
            items
                .into_iter()
                .fold(first, |acc, item| Expr::Mul(Box::new(acc), Box::new(item)))
        };

        let num_e = fold_prod(num_exprs);
        let full_expr = if den_exprs.is_empty() {
            num_e
        } else {
            let den_e = fold_prod(den_exprs);
            Expr::Div(Box::new(num_e), Box::new(den_e))
        };

        let num_s = if num_ltx.is_empty() {
            "1".to_string()
        } else {
            num_ltx.join(" \\cdot ")
        };
        let latex = if den_ltx.is_empty() {
            num_s
        } else {
            format!("\\frac{{{}}}{{{}}}", num_s, den_ltx.join(" \\cdot "))
        };

        (full_expr, latex)
    }

    /// Tam Z^7 Buckingham-Pi boyutsuz grup keşfi, sembolik ölçekleme yasası sentezi ve opsiyonel örnek kalibrasyonu.
    pub fn discover(
        vars: &[(String, Dimension)],
        target_var: Option<&str>,
        repeating_hint: &[&str],
        samples: &[HashMap<String, f64>],
    ) -> Result<BuckinghamDiscoveryReport, String> {
        let n = vars.len();
        if n == 0 {
            return Err("En az 1 değişken gereklidir!".to_string());
        }
        if let Some(t) = target_var {
            if !vars.iter().any(|(name, _)| name == t) {
                return Err(format!(
                    "Hedef değişken '{}' verilen değişken kümesinde yer almıyor!",
                    t
                ));
            }
        }

        let perm = Self::order_columns(vars, target_var, repeating_hint);
        let mut mat = vec![vec![Rat128::ZERO; n]; 7];
        for (col_pos, &orig_idx) in perm.iter().enumerate() {
            let comps = Self::dim_components(vars[orig_idx].1);
            for (row, &c) in comps.iter().enumerate() {
                mat[row][col_pos] = Rat128::from_int(i128::from(c));
            }
        }

        let (rref, pivot_cols) = Self::rref_exact(mat, n)?;
        let rank = pivot_cols.len();
        let num_pi = n.saturating_sub(rank);

        let repeating_basis_variables: Vec<String> = pivot_cols
            .iter()
            .map(|&c| vars[perm[c]].0.clone())
            .collect();

        let pivot_set: std::collections::HashSet<usize> = pivot_cols.iter().copied().collect();
        let free_cols: Vec<usize> = (0..n).filter(|c| !pivot_set.contains(c)).collect();

        let mut dim_map: HashMap<String, Dimension> = vars.iter().cloned().collect();
        dim_map.insert("C_pi".to_string(), Dimension::dimensionless());

        let target_orig_idx = target_var.and_then(|t| vars.iter().position(|(nm, _)| nm == t));

        let mut pi_groups = Vec::with_capacity(free_cols.len());
        let mut raw_exp_vectors: Vec<Vec<i64>> = Vec::with_capacity(free_cols.len());

        for &free_col in &free_cols {
            let mut vec_perm = vec![Rat128::ZERO; n];
            vec_perm[free_col] = Rat128::ONE;
            for (piv_row, &piv_col) in pivot_cols.iter().enumerate() {
                vec_perm[piv_col] = rref[piv_row][free_col].checked_neg()?;
            }

            // Ortak payda (EKOK) ile tam tamsayı vektöre çevir
            let mut common_den: u128 = 1;
            for r in &vec_perm {
                if !r.is_zero() {
                    let ud = r.den.unsigned_abs();
                    let g = gcd_u128(common_den, ud);
                    common_den = common_den
                        .checked_mul(ud / g)
                        .ok_or_else(|| "Pi grubu EKOK taşması!".to_string())?;
                }
            }
            let lcm_i128 =
                i128::try_from(common_den).map_err(|_| "Pi grubu EKOK i128 taşması!".to_string())?;

            let mut orig_exps = vec![0i64; n];
            for (col_pos, &r) in vec_perm.iter().enumerate() {
                if !r.is_zero() {
                    let scaled = r
                        .num
                        .checked_mul(lcm_i128 / r.den)
                        .ok_or_else(|| "Pi grubu üs taşması!".to_string())?;
                    orig_exps[perm[col_pos]] =
                        i64::try_from(scaled).map_err(|_| "Pi grubu üs i64 taşması!".to_string())?;
                }
            }

            // İlkel (primitive) Z^n vektörüne indirge: gcd(|alpha_1|, ..., |alpha_n|) == 1
            let mut g_all = 0i64;
            for &e in &orig_exps {
                if e != 0 {
                    g_all = if g_all == 0 { e.abs() } else { gcd_i64(g_all, e) };
                }
            }
            if g_all > 1 {
                for e in &mut orig_exps {
                    *e /= g_all;
                }
            }

            // İşaret yönelimi:
            // 1. Bu grup hedef değişkeni içeriyorsa alpha_target > 0 yap.
            // 2. İçermiyorsa ve ters yönelim (-alpha) kanonik bir boyutsuz sayıya (örn. Re, Ma) karşılık geliyorsa çevir.
            let has_target = target_orig_idx.is_some_and(|ti| orig_exps[ti] != 0);
            if let Some(ti) = target_orig_idx.filter(|&ti| orig_exps[ti] < 0) {
                let _ = ti;
                for e in &mut orig_exps {
                    *e = -*e;
                }
            } else if !has_target && Self::recognize_dimensionless_signature(vars, &orig_exps).is_none() {
                let neg_exps: Vec<i64> = orig_exps.iter().map(|&e| -e).collect();
                if Self::recognize_dimensionless_signature(vars, &neg_exps).is_some() {
                    orig_exps = neg_exps;
                }
            }

            raw_exp_vectors.push(orig_exps);
        }

        // Hedef değişken içeren Pi grubunu varsa Pi_1 (ilk sıra) yap
        if let Some(ti) = target_orig_idx {
            if let Some(pos) = raw_exp_vectors.iter().position(|v| v[ti] != 0) {
                if pos != 0 {
                    raw_exp_vectors.swap(0, pos);
                }
            }
        }

        for (k_idx, exps) in raw_exp_vectors.iter().enumerate() {
            let recognized = Self::recognize_dimensionless_signature(vars, exps).map(str::to_string);
            let (sym_expr, latex) = Self::build_monomial_expr_and_latex(vars, exps);
            let inferred_dim = sym_expr.infer_dimension(&dim_map)?;
            if !inferred_dim.is_dimensionless() {
                return Err(format!(
                    "İçsel Z^7 doğrulama hatası: Pi_{} boyutsuz çıkmadı ([{}])!",
                    k_idx + 1,
                    inferred_dim
                ));
            }
            let non_zero_pairs: Vec<(String, i64)> = vars
                .iter()
                .zip(exps.iter())
                .filter(|(_, &e)| e != 0)
                .map(|((nm, _), &e)| (nm.clone(), e))
                .collect();

            pi_groups.push(PiGroup {
                index: k_idx + 1,
                symbol: format!("Pi_{}", k_idx + 1),
                recognized_number: recognized,
                exponents: non_zero_pairs,
                expression: sym_expr.to_string(),
                latex,
                verified_dimension: inferred_dim.to_string(),
                symbolic_expr: Some(sym_expr),
            });
        }

        // Hedef değişken için boyutsal ölçekleme yasası sentezi ve Safe-Rust üretimi
        let mut target_identifiable = false;
        let mut target_unidentifiable_reason = None;
        let mut target_scaling_equation = None;
        let mut target_scaling_dimension = None;
        let mut safe_rust_fn = None;
        let mut empirical_fit = None;

        if let Some(ti) = target_orig_idx {
            let target_name = &vars[ti].0;
            let target_dim = vars[ti].1;

            if let Some((pi_pos, target_exps)) = raw_exp_vectors
                .iter()
                .enumerate()
                .find(|(_, v)| v[ti] != 0)
            {
                target_identifiable = true;
                let p_exp = target_exps[ti]; // Yönelim gereği p_exp > 0

                // Taşıyıcı monom: carrier = prod_{j != ti} q_j^(-alpha_j) => target^(p_exp) = Pi_1 * carrier
                let mut carrier_exps: Vec<i64> = target_exps.iter().map(|&e| -e).collect();
                carrier_exps[ti] = 0;
                let (carrier_ast, _) = Self::build_monomial_expr_and_latex(vars, &carrier_exps);

                let rooted_carrier = if p_exp == 1 {
                    carrier_ast
                } else if p_exp == 2 {
                    Expr::Sqrt(Box::new(carrier_ast))
                } else {
                    Expr::Pow(
                        Box::new(carrier_ast),
                        Box::new(Expr::Div(
                            Box::new(Expr::Constant(1.0)),
                            Box::new(Expr::Constant(p_exp as f64)),
                        )),
                    )
                };

                let base_scaling_ast = Expr::Mul(
                    Box::new(Expr::Variable("C_pi".to_string())),
                    Box::new(rooted_carrier),
                );
                let verified_target_dim = base_scaling_ast.infer_dimension(&dim_map)?;
                if verified_target_dim != target_dim {
                    return Err(format!(
                        "Ölçekleme yasası Z^7 boyut uyuşmazlığı: beklenen [{}], bulunan [{}]",
                        target_dim, verified_target_dim
                    ));
                }

                let other_pis: Vec<&PiGroup> = pi_groups
                    .iter()
                    .enumerate()
                    .filter(|&(idx, _)| idx != pi_pos)
                    .map(|(_, g)| g)
                    .collect();

                let eq_display = if other_pis.is_empty() {
                    format!("{} = {}", target_name, base_scaling_ast)
                } else {
                    let phi_args: Vec<String> = other_pis
                        .iter()
                        .map(|g| format!("{}={}", g.symbol, g.expression))
                        .collect();
                    format!(
                        "{} = {} * Phi({})",
                        target_name,
                        base_scaling_ast,
                        phi_args.join(", ")
                    )
                };
                target_scaling_equation = Some(eq_display);
                target_scaling_dimension = Some(verified_target_dim.to_string());

                // Tek Pi grubu (tam ölçekleme yasası) veya taşıyıcı formül için doğrulanmış Safe-Rust fonksiyonu üret
                let guards = base_scaling_ast.extract_domain_conditions()?;
                let mut bounds: HashMap<String, VarBound> = HashMap::new();
                for (nm, & e_val) in vars.iter().map(|(nm, _)| nm).zip(carrier_exps.iter()) {
                    if e_val != 0 && p_exp > 1 {
                        bounds.insert(
                            nm.clone(),
                            VarBound {
                                min: Some(0.0),
                                max: None,
                                min_inclusive: e_val > 0,
                                max_inclusive: true,
                                note: "Buckingham-Pi pozitif ölçekleme bölgesi".to_string(),
                            },
                        );
                    }
                }
                let synth = SynthesizedFormula {
                    target: target_name.clone(),
                    chain_ids: vec!["buckingham_pi_z7_nullspace".to_string()],
                    chain_names: vec!["Buckingham-Pi Z^7 Çekirdek Uzayı Keşfi".to_string()],
                    regimes: vec!["universal".to_string()],
                    free_variables: base_scaling_ast.free_variables(),
                    inferred_dimension: verified_target_dim,
                    symbolic_expr: base_scaling_ast,
                    domain_guards: guards,
                    effective_bounds: bounds,
                };
                safe_rust_fn = Some(crate::codegen::VerifiedCodegen::emit_safe_rust_fn(&synth));

                if !samples.is_empty() {
                    let fit_ctx = SampleFitContext {
                        target_name,
                        p_exp,
                        target_pi: &pi_groups[pi_pos],
                        other_pis: &other_pis,
                        target_var_key: &vars[ti].0,
                        carrier_exps: &carrier_exps,
                        vars,
                    };
                    empirical_fit = Self::fit_power_law_samples(&fit_ctx, samples);
                }
            } else {
                target_unidentifiable_reason = Some(format!(
                    "Hedef değişken '{}' diğer {} değişken tarafından dengelenemeyen bağımsız bir temel SI boyutu içeriyor (rank = {}, tüm çekirdek vektörlerinde üssü 0).",
                    target_name,
                    n - 1,
                    rank
                ));
            }
        }

        let var_summary: Vec<(String, String)> = vars
            .iter()
            .map(|(nm, dm)| (nm.clone(), dm.to_string()))
            .collect();

        Ok(BuckinghamDiscoveryReport {
            variables: var_summary,
            num_variables: n,
            dimensional_rank: rank,
            num_pi_groups: num_pi,
            repeating_basis_variables,
            pi_groups,
            target_variable: target_var.map(str::to_string),
            target_identifiable,
            target_unidentifiable_reason,
            target_scaling_equation,
            target_scaling_dimension,
            safe_rust_fn,
            empirical_fit,
        })
    }

    fn fit_power_law_samples(
        ctx: &SampleFitContext<'_>,
        samples: &[HashMap<String, f64>],
    ) -> Option<EmpiricalScalingFit> {
        let m_reg = ctx.other_pis.len();
        if samples.len() < m_reg + 1 {
            return None;
        }
        let target_ast = ctx.target_pi.symbolic_expr.as_ref()?;
        let mut z_rows: Vec<Vec<f64>> = Vec::with_capacity(samples.len());
        let mut y_vals: Vec<f64> = Vec::with_capacity(samples.len());

        for row in samples {
            let ref_map: HashMap<&str, f64> = row.iter().map(|(k, &v)| (k.as_str(), v)).collect();
            let pi_t = target_ast.evaluate(&ref_map).ok()?;
            if !pi_t.is_finite() || pi_t <= 0.0 {
                return None;
            }
            let mut z = Vec::with_capacity(m_reg + 1);
            z.push(1.0);
            for g in ctx.other_pis {
                let g_ast = g.symbolic_expr.as_ref()?;
                let gv = g_ast.evaluate(&ref_map).ok()?;
                if !gv.is_finite() || gv <= 0.0 {
                    return None;
                }
                z.push(gv.ln());
            }
            y_vals.push(pi_t.ln());
            z_rows.push(z);
        }

        let p_cols = m_reg + 1;
        let mut ata = vec![vec![0.0_f64; p_cols + 1]; p_cols];
        for (r_idx, z) in z_rows.iter().enumerate() {
            for i in 0..p_cols {
                for j in 0..p_cols {
                    ata[i][j] += z[i] * z[j];
                }
                ata[i][p_cols] += z[i] * y_vals[r_idx];
            }
        }

        let weights = solve_linear_f64(ata)?;
        let p_f = ctx.p_exp as f64;
        let c_pi = (weights[0] / p_f).exp();
        let mut regime_exponents = Vec::with_capacity(m_reg);
        for (j, g) in ctx.other_pis.iter().enumerate() {
            regime_exponents.push((g.symbol.clone(), weights[j + 1] / p_f));
        }

        let (carrier_ast, _) = Self::build_monomial_expr_and_latex(ctx.vars, ctx.carrier_exps);
        let mut actuals = Vec::with_capacity(samples.len());
        let mut preds = Vec::with_capacity(samples.len());
        let mut max_rel_err = 0.0_f64;

        for row in samples {
            let ref_map: HashMap<&str, f64> = row.iter().map(|(k, &v)| (k.as_str(), v)).collect();
            let actual_t = *ref_map.get(ctx.target_var_key)?;
            let carrier_v = carrier_ast.evaluate(&ref_map).ok()?;
            let mut pred_t = c_pi * carrier_v.powf(1.0 / p_f);
            for (j, g) in ctx.other_pis.iter().enumerate() {
                let gv = g.symbolic_expr.as_ref()?.evaluate(&ref_map).ok()?;
                pred_t *= gv.powf(regime_exponents[j].1);
            }
            let rel = ((pred_t - actual_t) / actual_t.abs().max(1e-15)).abs();
            if rel > max_rel_err {
                max_rel_err = rel;
            }
            actuals.push(actual_t);
            preds.push(pred_t);
        }

        let mean_act = actuals.iter().sum::<f64>() / (actuals.len() as f64);
        let ss_tot: f64 = actuals.iter().map(|&a| (a - mean_act).powi(2)).sum();
        let ss_res: f64 = actuals
            .iter()
            .zip(preds.iter())
            .map(|(&a, &p)| (a - p).powi(2))
            .sum();
        let r_sq = if ss_tot <= 1e-24 {
            1.0
        } else {
            (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
        };

        let carrier_str = if ctx.p_exp == 1 {
            format!("{}", carrier_ast)
        } else {
            format!("({})^(1/{})", carrier_ast, ctx.p_exp)
        };
        let mut terms = vec![format!("{:.6} * {}", c_pi, carrier_str)];
        for (sym, beta) in &regime_exponents {
            terms.push(format!("{}^({:.4})", sym, beta));
        }

        Some(EmpiricalScalingFit {
            coefficient_c_pi: c_pi,
            regime_exponents,
            fitted_equation: format!("{} = {}", ctx.target_name, terms.join(" * ")),
            r_squared: r_sq,
            max_relative_error: max_rel_err,
            sample_count: samples.len(),
        })
    }
}

struct SampleFitContext<'a> {
    target_name: &'a str,
    p_exp: i64,
    target_pi: &'a PiGroup,
    other_pis: &'a [&'a PiGroup],
    target_var_key: &'a str,
    carrier_exps: &'a [i64],
    vars: &'a [(String, Dimension)],
}

fn solve_linear_f64(mut aug: Vec<Vec<f64>>) -> Option<Vec<f64>> {
    let n = aug.len();
    for col in 0..n {
        let mut best_r = col;
        let mut best_v = aug[col][col].abs();
        for (r, row) in aug.iter().enumerate().skip(col + 1) {
            if row[col].abs() > best_v {
                best_v = row[col].abs();
                best_r = r;
            }
        }
        if best_v < 1e-13 {
            return None;
        }
        aug.swap(col, best_r);
        let piv = aug[col][col];
        for cell in aug[col].iter_mut().skip(col) {
            *cell /= piv;
        }
        for r in 0..n {
            if r == col {
                continue;
            }
            let f = aug[r][col];
            if f.abs() < 1e-18 {
                continue;
            }
            let piv_row = aug[col].clone();
            for (c, &pv) in piv_row.iter().enumerate().skip(col) {
                aug[r][c] -= f * pv;
            }
        }
    }
    Some(aug.into_iter().map(|row| row[n]).collect())
}
