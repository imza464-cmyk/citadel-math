// Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
// Matematik Çekirdeği — Küçük Doğrulama Çekirdeği, Kanonik Polinomlar ve Tanım Kümesi Sözleşmesi
// Referans: docs/MATEMATIK_CEKIRDEGI_MIMARI_REFERANS.md (§4 - §9)

use crate::ast::Expr;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Doğrulama Çekirdeği Hata Türleri (§7 & §9: Taşma, eksik önkoşul ve hatalı kanıt başarıya çevrilemez)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathKernelError {
    CoefficientOverflow,
    DegreeBudgetExceeded { degree: usize, max_allowed: usize },
    DivisionByZeroPolynomial,
    NotExactDivisible,
    DomainMismatch { lhs_excluded: BTreeSet<i64>, rhs_excluded: BTreeSet<i64> },
    InvalidProofStep(String),
}

impl fmt::Display for MathKernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathKernelError::CoefficientOverflow => {
                write!(f, "Katsayı taşması (i64::checked_* sınırı aşıldı; sarmalama reddedildi)")
            }
            MathKernelError::DegreeBudgetExceeded { degree, max_allowed } => {
                write!(f, "Derece bütçesi aşıldı: {} > {}", degree, max_allowed)
            }
            MathKernelError::DivisionByZeroPolynomial => write!(f, "Sıfır polinomuna bölme tanımsızdır"),
            MathKernelError::NotExactDivisible => write!(f, "Polinomlar Z[x] üzerinde tam bölünmüyor"),
            MathKernelError::DomainMismatch { lhs_excluded, rhs_excluded } => {
                write!(
                    f,
                    "Tanım kümesi (yan koşul) kaybı! Sol hariç tutulanlar {:?} != Sağ {:?}",
                    lhs_excluded, rhs_excluded
                )
            }
            MathKernelError::InvalidProofStep(msg) => write!(f, "Geçersiz kanıt adımı: {}", msg),
        }
    }
}

/// §7: Tek değişkenli, tamsayı katsayılı kanonik polinom: p(x) = sum_{k=0}^n a_k x^k
/// Sondaki sıfır katsayıları her zaman temizlenir; sıfır polinomu tekil olarak `vec![]` ile gösterilir.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PolyZ {
    coeffs: Vec<i64>,
}

impl PolyZ {
    /// Katsayı listesinden (a_0, a_1, ..., a_n) kanonik polinom oluşturur.
    pub fn new(mut coeffs: Vec<i64>) -> Self {
        while coeffs.last() == Some(&0) {
            coeffs.pop();
        }
        Self { coeffs }
    }

    pub fn zero() -> Self {
        Self { coeffs: Vec::new() }
    }

    pub fn constant(c: i64) -> Self {
        Self::new(vec![c])
    }

    pub fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    pub fn degree(&self) -> usize {
        if self.coeffs.is_empty() {
            0
        } else {
            self.coeffs.len() - 1
        }
    }

    pub fn coeffs(&self) -> &[i64] {
        &self.coeffs
    }

    /// Denetimli (checked) polinom toplama: p(x) + q(x)
    pub fn checked_add(&self, other: &Self) -> Result<Self, MathKernelError> {
        let max_len = self.coeffs.len().max(other.coeffs.len());
        let mut out = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let a = self.coeffs.get(i).copied().unwrap_or(0);
            let b = other.coeffs.get(i).copied().unwrap_or(0);
            let sum = a.checked_add(b).ok_or(MathKernelError::CoefficientOverflow)?;
            out.push(sum);
        }
        Ok(Self::new(out))
    }

    /// Denetimli (checked) polinom çıkarma: p(x) - q(x)
    pub fn checked_sub(&self, other: &Self) -> Result<Self, MathKernelError> {
        let max_len = self.coeffs.len().max(other.coeffs.len());
        let mut out = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let a = self.coeffs.get(i).copied().unwrap_or(0);
            let b = other.coeffs.get(i).copied().unwrap_or(0);
            let diff = a.checked_sub(b).ok_or(MathKernelError::CoefficientOverflow)?;
            out.push(diff);
        }
        Ok(Self::new(out))
    }

    /// Denetimli (checked) polinom çarpma: p(x) * q(x)
    pub fn checked_mul(&self, other: &Self, max_degree: usize) -> Result<Self, MathKernelError> {
        if self.is_zero() || other.is_zero() {
            return Ok(Self::zero());
        }
        let target_deg = self.degree() + other.degree();
        if target_deg > max_degree {
            return Err(MathKernelError::DegreeBudgetExceeded {
                degree: target_deg,
                max_allowed: max_degree,
            });
        }
        let mut out = vec![0i64; self.coeffs.len() + other.coeffs.len() - 1];
        for (i, &a) in self.coeffs.iter().enumerate() {
            for (j, &b) in other.coeffs.iter().enumerate() {
                let prod = a.checked_mul(b).ok_or(MathKernelError::CoefficientOverflow)?;
                out[i + j] = out[i + j]
                    .checked_add(prod)
                    .ok_or(MathKernelError::CoefficientOverflow)?;
            }
        }
        Ok(Self::new(out))
    }

    /// Denetimli (checked) polinom bileşkesi: (f o g)(x) = f(g(x)) (Horner şeması ile)
    pub fn checked_compose(&self, g: &Self, max_degree: usize) -> Result<Self, MathKernelError> {
        if self.is_zero() {
            return Ok(Self::zero());
        }
        let mut acc = Self::zero();
        for &coeff in self.coeffs.iter().rev() {
            acc = acc.checked_mul(g, max_degree)?;
            acc = acc.checked_add(&Self::constant(coeff))?;
        }
        Ok(acc)
    }

    /// Denetimli Horner değerlendirmesi: p(x_0)
    pub fn checked_eval(&self, x0: i64) -> Result<i64, MathKernelError> {
        let mut acc = 0i64;
        for &coeff in self.coeffs.iter().rev() {
            acc = acc
                .checked_mul(x0)
                .ok_or(MathKernelError::CoefficientOverflow)?
                .checked_add(coeff)
                .ok_or(MathKernelError::CoefficientOverflow)?;
        }
        Ok(acc)
    }
}

impl fmt::Display for PolyZ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        let mut terms = Vec::new();
        for (k, &a) in self.coeffs.iter().enumerate().rev() {
            if a == 0 {
                continue;
            }
            let term = match k {
                0 => format!("{}", a),
                1 => {
                    if a == 1 {
                        "x".to_string()
                    } else if a == -1 {
                        "-x".to_string()
                    } else {
                        format!("{}x", a)
                    }
                }
                _ => {
                    if a == 1 {
                        format!("x^{}", k)
                    } else if a == -1 {
                        format!("-x^{}", k)
                    } else {
                        format!("{}x^{}", a, k)
                    }
                }
            };
            terms.push(term);
        }
        write!(f, "{}", terms.join(" + ").replace("+ -", "- "))
    }
}

/// §5.2: Tanım Kümesi ve Yan Koşul Sözleşmesi.
/// Örn: r(x) = (x^2 - 1)/(x - 1) = x + 1, x != 1 koşulunu taşır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedPoly {
    pub poly: PolyZ,
    pub excluded_points: BTreeSet<i64>,
    pub unresolved_preimage_guards: Vec<(PolyZ, i64)>, // f(x) != c0 koşulları
}

impl GuardedPoly {
    pub fn total(poly: PolyZ) -> Self {
        Self {
            poly,
            excluded_points: BTreeSet::new(),
            unresolved_preimage_guards: Vec::new(),
        }
    }

    pub fn with_excluded(poly: PolyZ, excluded: impl IntoIterator<Item = i64>) -> Self {
        Self {
            poly,
            excluded_points: excluded.into_iter().collect(),
            unresolved_preimage_guards: Vec::new(),
        }
    }

    /// §5.2: (P(x) * (x - r0)) / (x - r0) sadeleştirmesini YALNIZCA x != r0 yan koşulunu
    /// koruyarak yapar. Koşulsuz sadeleştirme yasaktır.
    pub fn cancel_linear_factor(
        numerator: &PolyZ,
        root_r0: i64,
        max_degree: usize,
    ) -> Result<Self, MathKernelError> {
        // Payın x = root_r0 noktasında gerçekten sıfır olduğunu (tam bölündüğünü) denetle
        if numerator.checked_eval(root_r0)? != 0 {
            return Err(MathKernelError::NotExactDivisible);
        }
        // Sentetik bölme (Horner / Ruffini): P(x) / (x - r0)
        let n = numerator.degree();
        if numerator.is_zero() {
            let mut ex = BTreeSet::new();
            ex.insert(root_r0);
            return Ok(Self {
                poly: PolyZ::zero(),
                excluded_points: ex,
                unresolved_preimage_guards: Vec::new(),
            });
        }
        let mut q_rev = Vec::with_capacity(n);
        let mut rem = 0i64;
        for &c in numerator.coeffs().iter().rev() {
            let cur = rem
                .checked_mul(root_r0)
                .ok_or(MathKernelError::CoefficientOverflow)?
                .checked_add(c)
                .ok_or(MathKernelError::CoefficientOverflow)?;
            q_rev.push(cur);
            rem = cur;
        }
        if rem != 0 {
            return Err(MathKernelError::NotExactDivisible);
        }
        q_rev.pop(); // son eleman kalan (0)
        q_rev.reverse();
        let quotient = PolyZ::new(q_rev);

        // Sağlama: quotient * (x - r0) == numerator
        let linear = PolyZ::new(vec![
            root_r0.checked_neg().ok_or(MathKernelError::CoefficientOverflow)?,
            1,
        ]);
        if quotient.checked_mul(&linear, max_degree)? != *numerator {
            return Err(MathKernelError::NotExactDivisible);
        }

        let mut excluded = BTreeSet::new();
        excluded.insert(root_r0);
        Ok(Self {
            poly: quotient,
            excluded_points: excluded,
            unresolved_preimage_guards: Vec::new(),
        })
    }

    /// §5.2: Fonksiyon bileşkesi (g o f)(x) = g(f(x)) ve f(A) <= C tanım kümesi kontrolü.
    pub fn checked_compose(&self, inner_f: &GuardedPoly, max_degree: usize) -> Result<Self, MathKernelError> {
        let composed_poly = self.poly.checked_compose(&inner_f.poly, max_degree)?;
        let mut excluded = inner_f.excluded_points.clone();
        let mut guards = inner_f.unresolved_preimage_guards.clone();

        // g(y) fonksiyonu y != c0 koşuluna sahipse, g(f(x)) için f(x) != c0 yan koşulu zorunludur!
        for &c0 in &self.excluded_points {
            if inner_f.poly.degree() == 0 {
                let const_val = inner_f.poly.coeffs().first().copied().unwrap_or(0);
                if const_val == c0 {
                    return Err(MathKernelError::DivisionByZeroPolynomial);
                }
            } else if inner_f.poly.degree() == 1 {
                // a_1 * x + a_0 = c0 => tam bölünüyorsa x noktasını doğrudan çıkar
                let a0 = inner_f.poly.coeffs()[0];
                let a1 = inner_f.poly.coeffs()[1];
                let rhs = c0.checked_sub(a0).ok_or(MathKernelError::CoefficientOverflow)?;
                if rhs % a1 == 0 {
                    excluded.insert(rhs / a1);
                } else {
                    guards.push((inner_f.poly.clone(), c0));
                }
            } else {
                guards.push((inner_f.poly.clone(), c0));
            }
        }

        Ok(Self {
            poly: composed_poly,
            excluded_points: excluded,
            unresolved_preimage_guards: guards,
        })
    }

    /// İki korunumlu fonksiyonun hem cebirsel hem de tanım kümesi (yan koşul) bakımından tam eşitliğini denetler.
    pub fn verify_strict_equality(&self, other: &Self) -> Result<(), MathKernelError> {
        if self.excluded_points != other.excluded_points
            || self.unresolved_preimage_guards != other.unresolved_preimage_guards
        {
            return Err(MathKernelError::DomainMismatch {
                lhs_excluded: self.excluded_points.clone(),
                rhs_excluded: other.excluded_points.clone(),
            });
        }
        if self.poly != other.poly {
            return Err(MathKernelError::InvalidProofStep(format!(
                "Polinomlar eşit değil: {} != {}",
                self.poly, other.poly
            )));
        }
        Ok(())
    }
}

/// §8: Çok Eksenli Durum Modeli
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewStage {
    Candidate,
    Audited,
    NeedsReaudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditVerdict {
    Accepted,
    Rejected,
    Unprovable,
    ResourceLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicKind {
    Definition,
    Assumption,
    VerifiedTheorem,
    NumericalObservation,
}

/// §6: Küçük Doğrulama Çekirdeği Kanıt Adımı
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofRule {
    PolyAddition { a: PolyZ, b: PolyZ },
    PolyMultiplication { a: PolyZ, b: PolyZ },
    PolyComposition { f: PolyZ, g: PolyZ },
    GuardedLinearCancellation { numerator: PolyZ, root: i64 },
}

#[derive(Debug, Clone)]
pub struct ProofRecord {
    pub rule: ProofRule,
    pub claimed_result: GuardedPoly,
    pub max_degree_budget: usize,
}

pub struct ProofVerifier;

impl ProofVerifier {
    /// Üreticinin sunduğu kanıt kaydını (ProofRecord) bağımsız olarak yeniden hesaplayıp denetler.
    pub fn verify(record: &ProofRecord) -> Result<(ReviewStage, AuditVerdict, EpistemicKind), MathKernelError> {
        let computed = match &record.rule {
            ProofRule::PolyAddition { a, b } => {
                GuardedPoly::total(a.checked_add(b)?)
            }
            ProofRule::PolyMultiplication { a, b } => {
                GuardedPoly::total(a.checked_mul(b, record.max_degree_budget)?)
            }
            ProofRule::PolyComposition { f, g } => {
                GuardedPoly::total(f.checked_compose(g, record.max_degree_budget)?)
            }
            ProofRule::GuardedLinearCancellation { numerator, root } => {
                GuardedPoly::cancel_linear_factor(numerator, *root, record.max_degree_budget)?
            }
        };

        computed.verify_strict_equality(&record.claimed_result)?;
        Ok((
            ReviewStage::Audited,
            AuditVerdict::Accepted,
            EpistemicKind::VerifiedTheorem,
        ))
    }
}

/// Faz 3: Çok değişkenli monom (Graded Reverse Lexicographic — `grevlex` sıralamalı):
/// Yalnızca üssü > 0 olan değişkenler `BTreeMap<String, u16>` içinde tutulur.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Monomial {
    exps: BTreeMap<String, u16>,
}

impl Monomial {
    pub fn one() -> Self {
        Self {
            exps: BTreeMap::new(),
        }
    }

    pub fn var(name: &str) -> Self {
        let mut exps = BTreeMap::new();
        exps.insert(name.to_string(), 1);
        Self { exps }
    }

    pub fn total_degree(&self) -> usize {
        self.exps.values().map(|&e| e as usize).sum()
    }

    pub fn mul(&self, other: &Self, max_degree: usize) -> Result<Self, MathKernelError> {
        let mut exps = self.exps.clone();
        for (k, &v) in &other.exps {
            let e = exps.entry(k.clone()).or_insert(0);
            *e = e.checked_add(v).ok_or(MathKernelError::CoefficientOverflow)?;
        }
        let res = Self { exps };
        if res.total_degree() > max_degree {
            return Err(MathKernelError::DegreeBudgetExceeded {
                degree: res.total_degree(),
                max_allowed: max_degree,
            });
        }
        Ok(res)
    }
}

impl Ord for Monomial {
    fn cmp(&self, other: &Self) -> Ordering {
        let d_self = self.total_degree();
        let d_other = other.total_degree();
        if d_self != d_other {
            return d_self.cmp(&d_other);
        }
        // grevlex: tüm değişkenlerin birleşimini ters alfabetik sırada karşılaştır;
        // son değişkende daha küçük üsse sahip olan grevlex'te daha büyüktür.
        let mut all_vars: BTreeSet<&String> = self.exps.keys().collect();
        all_vars.extend(other.exps.keys());
        for v in all_vars.into_iter().rev() {
            let e1 = self.exps.get(v).copied().unwrap_or(0);
            let e2 = other.exps.get(v).copied().unwrap_or(0);
            if e1 != e2 {
                return e2.cmp(&e1);
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Monomial {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Faz 3: Çok Değişkenli Kanonik Polinom Halkası Z[x_1, ..., x_n] (`MultiPolyZ`).
/// Sıfır katsayılı terimler her zaman silinir; tüm katsayı işlemleri `i64::checked_*` ile taşma korumalıdır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiPolyZ {
    terms: BTreeMap<Monomial, i64>,
}

impl MultiPolyZ {
    pub fn zero() -> Self {
        Self {
            terms: BTreeMap::new(),
        }
    }

    pub fn constant(c: i64) -> Self {
        let mut terms = BTreeMap::new();
        if c != 0 {
            terms.insert(Monomial::one(), c);
        }
        Self { terms }
    }

    pub fn var(name: &str) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(Monomial::var(name), 1);
        Self { terms }
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn total_degree(&self) -> usize {
        self.terms.keys().map(|m| m.total_degree()).max().unwrap_or(0)
    }

    pub fn checked_add(&self, rhs: &Self) -> Result<Self, MathKernelError> {
        let mut terms = self.terms.clone();
        for (m, &c2) in &rhs.terms {
            let new_c = match terms.get(m) {
                Some(&c1) => c1.checked_add(c2).ok_or(MathKernelError::CoefficientOverflow)?,
                None => c2,
            };
            if new_c == 0 {
                terms.remove(m);
            } else {
                terms.insert(m.clone(), new_c);
            }
        }
        Ok(Self { terms })
    }

    pub fn checked_sub(&self, rhs: &Self) -> Result<Self, MathKernelError> {
        let mut terms = self.terms.clone();
        for (m, &c2) in &rhs.terms {
            let neg_c2 = c2.checked_neg().ok_or(MathKernelError::CoefficientOverflow)?;
            let new_c = match terms.get(m) {
                Some(&c1) => c1
                    .checked_add(neg_c2)
                    .ok_or(MathKernelError::CoefficientOverflow)?,
                None => neg_c2,
            };
            if new_c == 0 {
                terms.remove(m);
            } else {
                terms.insert(m.clone(), new_c);
            }
        }
        Ok(Self { terms })
    }

    pub fn checked_mul(&self, rhs: &Self, max_degree: usize) -> Result<Self, MathKernelError> {
        if self.is_zero() || rhs.is_zero() {
            return Ok(Self::zero());
        }
        let mut terms: BTreeMap<Monomial, i64> = BTreeMap::new();
        for (m1, &c1) in &self.terms {
            for (m2, &c2) in &rhs.terms {
                let m_prod = m1.mul(m2, max_degree)?;
                let c_prod = c1.checked_mul(c2).ok_or(MathKernelError::CoefficientOverflow)?;
                let new_c = match terms.get(&m_prod) {
                    Some(&cur) => cur
                        .checked_add(c_prod)
                        .ok_or(MathKernelError::CoefficientOverflow)?,
                    None => c_prod,
                };
                if new_c == 0 {
                    terms.remove(&m_prod);
                } else {
                    terms.insert(m_prod, new_c);
                }
            }
        }
        Ok(Self { terms })
    }

    pub fn checked_pow(&self, exp: u32, max_degree: usize) -> Result<Self, MathKernelError> {
        let mut acc = Self::constant(1);
        for _ in 0..exp {
            acc = acc.checked_mul(self, max_degree)?;
        }
        Ok(acc)
    }

    /// Sembolik bir `Expr` ağacını (tamsayı katsayılı polinom ise) kanonik `MultiPolyZ` formuna dönüştürür.
    pub fn from_expr(expr: &Expr, max_degree: usize) -> Result<Self, MathKernelError> {
        match expr {
            Expr::Constant(c) => {
                if (*c - c.round()).abs() > 1e-9 || c.abs() > (i64::MAX as f64) {
                    return Err(MathKernelError::InvalidProofStep(format!(
                        "Tamsayı olmayan sabit '{}' Z[x1..xn] polinomuna çevrilemez",
                        c
                    )));
                }
                Ok(Self::constant(c.round() as i64))
            }
            Expr::Variable(v) => Ok(Self::var(v)),
            Expr::Add(l, r) => Self::from_expr(l, max_degree)?.checked_add(&Self::from_expr(r, max_degree)?),
            Expr::Sub(l, r) => Self::from_expr(l, max_degree)?.checked_sub(&Self::from_expr(r, max_degree)?),
            Expr::Mul(l, r) => {
                Self::from_expr(l, max_degree)?.checked_mul(&Self::from_expr(r, max_degree)?, max_degree)
            }
            Expr::Neg(inner) => Self::zero().checked_sub(&Self::from_expr(inner, max_degree)?),
            Expr::Pow(base, exp) => {
                if let Expr::Constant(p) = exp.as_ref() {
                    if *p >= 0.0 && (*p - p.round()).abs() < 1e-9 && *p <= 32.0 {
                        let b_poly = Self::from_expr(base, max_degree)?;
                        return b_poly.checked_pow(p.round() as u32, max_degree);
                    }
                }
                Err(MathKernelError::InvalidProofStep(format!(
                    "Geçersiz çok değişkenli polinom üssü: {}",
                    expr
                )))
            }
            other => Err(MathKernelError::InvalidProofStep(format!(
                "Polinom olmayan ifade: {}",
                other
            ))),
        }
    }
}

impl fmt::Display for MultiPolyZ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.terms.is_empty() {
            return write!(f, "0");
        }
        let mut first = true;
        for (monom, &coeff) in self.terms.iter().rev() {
            let abs_c = coeff.unsigned_abs();
            if first {
                if coeff < 0 {
                    write!(f, "-")?;
                }
                first = false;
            } else if coeff < 0 {
                write!(f, " - ")?;
            } else {
                write!(f, " + ")?;
            }

            if monom.exps.is_empty() {
                write!(f, "{}", abs_c)?;
            } else {
                let mut m_parts = Vec::new();
                if abs_c != 1 {
                    m_parts.push(abs_c.to_string());
                }
                for (var, &exp) in &monom.exps {
                    if exp == 1 {
                        m_parts.push(var.clone());
                    } else if exp > 1 {
                        m_parts.push(format!("{}^{}", var, exp));
                    }
                }
                write!(f, "{}", m_parts.join("*"))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraph;

    #[test]
    fn test_canonical_trailing_zero_normalization() {
        let p1 = PolyZ::new(vec![1, 2, 0, 0, 0]);
        let p2 = PolyZ::new(vec![1, 2]);
        assert_eq!(p1, p2);
        assert_eq!(p1.coeffs(), &[1, 2]);
        assert_eq!(PolyZ::new(vec![0, 0, 0]).coeffs(), &[] as &[i64]);
    }

    #[test]
    fn test_binomial_expansion_and_ring_laws() {
        let x_plus_1 = PolyZ::new(vec![1, 1]);
        let sq = x_plus_1.checked_mul(&x_plus_1, 8).unwrap();
        assert_eq!(sq, PolyZ::new(vec![1, 2, 1]));

        let diff = sq.checked_sub(&PolyZ::new(vec![1, 2, 1])).unwrap();
        assert!(diff.is_zero());
    }

    #[test]
    fn test_guarded_cancellation_rejects_unconditional_equality() {
        // (x^2 - 1) / (x - 1) = x + 1, x != 1
        let num = PolyZ::new(vec![-1, 0, 1]);
        let guarded = GuardedPoly::cancel_linear_factor(&num, 1, 8).unwrap();
        let unconditional = GuardedPoly::total(PolyZ::new(vec![1, 1]));

        // Cebirsel olarak x + 1 olsa bile x != 1 yan koşulu olmadan eşitlik reddedilmelidir (§5.2)
        assert!(guarded.verify_strict_equality(&unconditional).is_err());

        // Yan koşul korunduğunda ProofVerifier kabul etmelidir
        let rec = ProofRecord {
            rule: ProofRule::GuardedLinearCancellation {
                numerator: num,
                root: 1,
            },
            claimed_result: GuardedPoly::with_excluded(PolyZ::new(vec![1, 1]), [1]),
            max_degree_budget: 8,
        };
        let (stage, verdict, kind) = ProofVerifier::verify(&rec).unwrap();
        assert_eq!(stage, ReviewStage::Audited);
        assert_eq!(verdict, AuditVerdict::Accepted);
        assert_eq!(kind, EpistemicKind::VerifiedTheorem);
    }

    #[test]
    fn test_checked_i64_overflow_rejected() {
        let huge = PolyZ::constant(i64::MAX);
        assert_eq!(
            huge.checked_add(&PolyZ::constant(1)),
            Err(MathKernelError::CoefficientOverflow)
        );
    }

    #[test]
    fn test_canonical_laws_dimensional_homogeneity() {
        let kg = KnowledgeGraph::new();
        assert!(kg.law_order.len() >= 130);
        assert_eq!(kg.verify_all_laws(), Ok(kg.law_order.len()));
    }

    #[test]
    fn test_ast_domain_guard_extraction() {
        let expr = crate::ast::Expr::parse_str("delta_v = (g_0 * I_sp) * ln(m_0 / m_f)").unwrap();
        let guards = expr.extract_domain_conditions().unwrap();
        assert!(guards.contains(&"m_f != 0".to_string()));
        assert!(guards.contains(&"(m_0 / m_f) > 0".to_string()));

        let div_zero = crate::ast::Expr::parse_str("y = x / (2 - 2)").unwrap();
        assert!(div_zero.extract_domain_conditions().is_err());
    }

    #[test]
    fn test_orbital_energy_and_angular_momentum_conservation() {
        use crate::constants::PhysicalConstants;
        use crate::simulator::{RK4Simulator, VehicleConfig};

        // 300 km irtifada (vakum, sürükleme = 0, itki = 0) eliptik yörüngede serbest uçuş (coasting)
        let h0 = 300_000.0;
        let vx0 = 7500.0;
        let vh0 = 400.0;
        let r0 = PhysicalConstants::R_EARTH + h0;
        let eps0 = 0.5 * (vx0 * vx0 + vh0 * vh0) - PhysicalConstants::MU_EARTH / r0;
        let (p0, a0, e0) = RK4Simulator::compute_orbital_elements(h0, vx0, vh0);

        let sim = RK4Simulator::new(0.1);
        let v_coast = VehicleConfig {
            name: "Vacuum Coast Conservation Test",
            dry_mass: 1000.0,
            propellant_mass: 0.0,
            thrust_vac: 0.0,
            isp: 300.0,
            area: 1.0,
            cd: 0.0,
            initial_velocity: (vx0 * vx0 + vh0 * vh0).sqrt(),
            initial_altitude: h0,
            has_stage_2: false,
            s2_dry_mass: 0.0,
            s2_propellant: 0.0,
            s2_thrust: 0.0,
            s2_isp: 0.0,
        };
        let res = sim.simulate(&v_coast, 200.0);
        let rf = PhysicalConstants::R_EARTH + res.final_altitude_km * 1000.0;
        let eps_f = 0.5 * res.final_speed_ms.powi(2) - PhysicalConstants::MU_EARTH / rf;
        // İlk enerji (0.78, 0.62 dağılımıyla aynı büyüklükte hız olduğu için):
        let v_init = v_coast.initial_velocity;
        let eps_init = 0.5 * v_init * v_init - PhysicalConstants::MU_EARTH / r0;

        // Mekanik enerjinin 200 saniyelik RK4 entegrasyonu boyunca 1e-12 hassasiyetle korunduğunu kanıtla
        let rel_err = ((eps_f - eps_init) / eps_init).abs();
        assert!(
            rel_err < 1e-12,
            "Özgül yörünge enerjisi korunmadı! eps_init={}, eps_f={}, rel_err={}",
            eps_init,
            eps_f,
            rel_err
        );
        assert!(p0 < a0 && e0 > 0.0 && eps0 < 0.0);
    }

    #[test]
    fn test_interval64_outward_rounding_and_sqr_dependency() {
        use crate::interval::Interval64;

        // 1/10 + 2/10 rasyonel aralıklarının dışa yuvarlamalı toplamı 0.3'ü kesin olarak içermelidir
        let a = Interval64::from_rational(1, 10).unwrap();
        let b = Interval64::from_rational(2, 10).unwrap();
        let sum = a.add(&b);
        assert!(sum.lo < 0.3 && sum.hi > 0.3 && sum.contains(0.3));

        // 0 içeren aralığın karesi [-2, 3]^2 == [0, 9] olmalıdır (naif çarpım [-6, 9] hatasına düşmemelidir)
        let span_zero = Interval64::new(-2.0, 3.0).unwrap();
        let sq = span_zero.sqr();
        assert_eq!(sq.lo, 0.0);
        assert!(sq.hi >= 9.0 && (sq.hi - 9.0) < 1e-13);

        // Payda aralığı 0 içerdiğinde bölme kesin olarak reddedilmelidir
        assert!(a.div(&span_zero).is_err());
    }

    #[test]
    fn test_taylor_mean_value_interval_eliminates_dependency() {
        use crate::ast::Expr;
        use crate::interval::Interval64;
        use std::collections::HashMap;

        // f(x) = (2*x + 1) - 2*x = 1 ifadesinde naif aralık x ∈ [10, 20] için [21-40, 41-20] = [-19, 21] verirken,
        // Taylor / Mean-Value biçimi ∂f/∂x = 0 olduğu için tam 1.0 etrafında ~0 genişlikli aralık vermelidir!
        let f = Expr::parse_str("(2 * x + 1) - 2 * x").unwrap();
        let mut env = HashMap::new();
        env.insert("x", Interval64::new(10.0, 20.0).unwrap());

        let naive = f.eval_interval(&env).unwrap();
        let taylor = f.eval_taylor_interval(&env).unwrap();
        assert!(naive.width() > 39.0);
        assert!(taylor.contains(1.0) && taylor.width() < 1e-12);
    }

    fn canonical_test_point_for_law(
        law: &crate::graph::Law,
        base_seed: f64,
    ) -> std::collections::HashMap<&str, f64> {
        let mut pt = std::collections::HashMap::new();
        for (idx, inp) in law.inputs.iter().enumerate() {
            let val = match inp.as_str() {
                // Evrensel Fizik Sabitleri
                "c" => 299_792_458.0,
                "G" | "G_grav" => 6.67430e-11,
                "h" | "h_planck" => 6.62607015e-34,
                "k_B" => 1.380649e-23,
                "q_e" | "q_ion" | "q_2" => 1.602176634e-19,
                "eps_0" => 8.8541878128e-12,
                "mu_0" => 1.25663706212e-6,
                "sigma_SB" => 5.670374419e-8,
                "R_univ" => 8.314462618,
                "g_0" => 9.80665,

                // Hızlar
                "v" => {
                    if law.id == "law_hyperbolic_excess_velocity" {
                        15_000.0
                    } else if law.regime == "relativistic" || law.regime == "special_relativistic" {
                        1.8e8
                    } else if law.regime == "hypersonic_continuum" {
                        2_500.0
                    } else {
                        250.0
                    }
                }
                "v_inf" => 1_500.0,
                "v_e" | "v_noz" => 3_500.0,
                "v_ion" => 30_000.0,
                "v_ntr" => 8_500.0,
                "v_perp" => 5_000.0,
                "v_max" => 2_500.0,
                "a_sound" => 340.0,

                // Gezegen ve Yörünge
                "M_primary" => 1.989e30,
                "M_body" | "m_body" => 5.972e24,
                "mu" => 3.986004418e14,
                "R_E" | "R_body" => 6_378_137.0,
                "a_orb" => {
                    if law.id == "law_laplace_sphere_of_influence" {
                        150_000_000_000.0
                    } else {
                        20_000_000.0
                    }
                }
                "p_semi" => 7_000_000.0,
                "e_orb" => 0.25,
                "cos_inc" => 0.6,
                "J_2" => 1.08262668e-3,
                "T_sid" => 86164.0,
                "n_mean" => 0.001,
                "T_orb_1" => 5_000.0,
                "T_orb_2" => 8_000.0,
                "delta_v" | "delta_v_1" | "delta_v_2" => 2_500.0,
                "r_1" => 7_000_000.0,
                "r_2" => 12_000_000.0,
                "r" => {
                    if law.id.contains("cylinder")
                        || law.id.contains("stress")
                        || law.id.contains("shaft")
                        || law.id.contains("vessel")
                    {
                        1.5
                    } else {
                        8_000_000.0
                    }
                }

                // Kütleler
                "m_0" => {
                    if law.id == "law_payload_mass_fraction" {
                        500_000.0
                    } else {
                        15.0
                    }
                }
                "m_f" => 3.0,
                "m_dry" => 2_500.0,
                "m_prop" => 40_000.0,
                "m_payload" => 1_500.0,
                "m_ull" => 5.0,
                "m_ion" => 2.18e-25,
                "m_e" => 9.1093837e-31,
                "m" => {
                    if law.id == "law_compton_wavelength" {
                        9.1093837e-31
                    } else {
                        500.0
                    }
                }
                "M_mol" => 0.028,
                "m_dot" => 25.0,

                // Gaz Dinamiği ve Basınçlar
                "gamma_gas" => 1.4,
                "R_spec" => 287.058,
                "P_0" => {
                    if law.id.contains("choked")
                        || law.id.contains("thrust_coefficient")
                        || law.id.contains("cstar")
                    {
                        2_000_000.0
                    } else {
                        500_000.0
                    }
                }
                "P_e" => 5_000.0,
                "P_stat" => 10_000.0,
                "P_pitot" => 12_500.0,
                "P_a" => 2_000.0,
                "P_int" => 200_000.0,
                "P_B" => 50_000.0,
                "P_ull" => 150_000.0,
                "delta_P_pump" => 5.0e6,
                "P" => {
                    if law.id == "law_laser_sail_thrust" {
                        10_000_000.0
                    } else {
                        50_000.0
                    }
                }
                "P_rad" => 1000.0,

                // Aerodinamik ve Mach
                "Mach" => {
                    if law.regime == "supersonic"
                        || law.regime == "hypersonic_continuum"
                        || law.id.contains("shock")
                        || law.id == "law_mach_angle"
                        || law.id == "law_pitot_rayleigh_pressure_ratio"
                    {
                        2.5
                    } else {
                        0.35
                    }
                }
                "theta_wedge" => 0.12,
                "s_cone" => 0.35,
                "r_nose" => 0.5,
                "A_nose" => 0.05,
                "C_sg" => 1.74e-4,
                "C_p0" => 0.4,
                "C_p_max" => 1.8,
                "C_F" => 1.65,
                "C_d_fm" => 2.2,
                "C_d" | "C_L" => 0.35,
                "q_dyn" => 25_000.0,
                "H_scale" => 8_500.0,

                // Sıcaklık ve Termodinamik
                "T_h" => 1_200.0,
                "T_c" => 300.0,
                "T_0" => 1_800.0,
                "T" => 300.0,
                "delta_T" => 120.0,
                "T_e" => {
                    if law.id == "law_plasma_beta_parameter" {
                        2_000_000.0
                    } else {
                        50_000.0
                    }
                }
                "c_p" => 1005.0,
                "k_th" => 150.0,
                "h_conv" => 250.0,
                "alpha_thermal" | "alpha_cte" => 1.2e-5,
                "Q_vol" => 5000.0,
                "q_net" => 50_000.0,
                "H_eff" => 2.5e7,

                // Yoğunluklar
                "rho" => {
                    if law.id == "law_alfven_velocity" {
                        1e-4
                    } else if law.id == "law_turbopump_hydraulic_power" {
                        1000.0
                    } else if law.id == "law_flywheel_rim_stress" {
                        2800.0
                    } else {
                        1.225
                    }
                }
                "rho_0" => 1.225,
                "rho_body" => 5_510.0,

                // Elektromanyetizma ve Plazma
                "B" | "B_max" => 1.5,
                "B_min" => 0.5,
                "n_e" => 1e20,
                "E_field" => 10_000.0,
                "V_acc" => 1_000.0,
                "V_accel" => 1_500.0,
                "d_grid" => 0.005,
                "I" => 500.0,
                "R_elec" => 50.0,
                "rho_res" => 1.7e-8,
                "L_ind" => 10.0e-3,
                "L_prime" => 0.4e-6,
                "C_cap" => 100.0e-6,
                "n_turns" => 500.0,
                "f_em" => 10.0e9,
                "V_bus" => 28.0,

                // Yapı ve Malzeme
                "E_mod" => 7.0e10,
                "nu_pois" => 0.30,
                "eps_str" => 0.004,
                "eps_struct" => 0.08,
                "t_wall" => 0.004,
                "sigma" | "sigma_ax" => 50.0e6,
                "sigma_hoop" => 100.0e6,
                "sigma_yield" => 250.0e6,
                "G_Ic" => 25.0,
                "F" => 5_000.0,
                "I_moment" | "J_polar" => 1e-4,
                "I_area" => 1e-5,
                "M_torq" => 150.0,
                "tau_torque" => 500.0,
                "k_spr" => 5000.0,
                "mu_lin" => 0.05,
                "mu_visc" => 1.8e-5,
                "gamma_surf" => 0.072,
                "c_fiber" => 0.05,
                "y_dist" => 0.05,
                "L" => 2.5,
                "L_char" => 1.2,
                "L_eff" => 1.5,
                "h_col" => 10.0,
                "D_pipe" => 0.1,
                "f_darcy" => 0.02,

                // Optik, Radyasyon ve Verim
                "R_refl" => 0.95,
                "I_solar" | "I_flux" => 1361.0,
                "sin_theta" => 0.5,
                "epsilon" => 0.85,
                "eta" => 0.65,
                "eta_elec" => 0.75,
                "gamma_kd" => 0.65,
                "f_par" => 0.75,
                "alpha_act" => 0.40,
                "D_ap" => 0.3,
                "G_tx" => 25.0,
                "G_rx" => 20.0,
                "B_bw" => 20.0e6,
                "SNR" => 15.0,

                // Roket ve Genel Geometri
                "A" => 1.5,
                "A_t" => 0.05,
                "A_e" => 0.8,
                "V_chamb" => 0.05,
                "V_ull" => 2.0,
                "I_sp" => 320.0,
                "t_burn" => 60.0,
                "r_comp" => 8.0,
                "r_a" => 0.10,
                "r_c" => 0.02,
                "lambda_pl" => 0.05,
                "lambda_arr" => 5.0,
                "mu_srv" => 25.0,
                "N_cores" => 8.0,
                "N_cycles" => 1000.0,
                "f_clk" => 1.5e9,
                "W_resp" => 0.05,
                "gamma_rel" => 1.25,
                "h_alt" => 400_000.0,
                "I_rot" => 50.0,
                "omega" => 50.0,
                "lambda_db" | "lambda_C" => 1e-10,
                "lambda_mfp" => 1e-4,
                "p_mom" => 500.0,
                "p_lin" => 1e-22,
                "p_rel" => 1e-18,
                "delta_m" => 1e-5,
                "a" => 15.0,
                "a_c" => 25.0,

                _ => base_seed + (idx as f64) * 0.7,
            };
            let adjusted_val = if let Some(bound) = law.domain_bounds.get(inp) {
                if bound.check_value(inp, val).is_err() {
                    match (bound.min, bound.max) {
                        (Some(lo), Some(hi)) => lo + (hi - lo) * (0.35 + 0.05 * ((idx % 5) as f64)),
                        (Some(lo), None) => if lo > 0.0 { lo * 1.5 } else { lo + 1.0 },
                        (None, Some(hi)) => if hi > 0.0 { hi * 0.5 } else { hi - 1.0 },
                        (None, None) => val,
                    }
                } else {
                    val
                }
            } else {
                val
            };
            pt.insert(inp.as_str(), adjusted_val);
        }
        pt
    }

    #[test]
    fn test_symbolic_differentiation_and_dual64_equivalence_all_laws() {
        let kg = KnowledgeGraph::new();
        assert!(
            kg.law_order.len() >= 105,
            "En az 105 kanonik yasa bekleniyordu, bulunan: {}",
            kg.law_order.len()
        );

        for id in &kg.law_order {
            let law = &kg.laws[id];
            let f_dim = law.dimensions[&law.target];

            let pt = canonical_test_point_for_law(law, 2.0);
            law.check_validity_bounds(&pt).unwrap_or_else(|e| {
                panic!("{} yasası kanonik test noktasında geçerlilik sınırını ihlal etti: {}", id, e)
            });

            for var in &law.inputs {
                let sym_deriv = law.rhs.differentiate(var).unwrap();
                let sym_val = sym_deriv.evaluate(&pt).unwrap();
                let dual_res = law.rhs.eval_dual(&pt, var).unwrap();

                let abs_err = (sym_val - dual_res.der).abs();
                let scale = dual_res.der.abs().max(1.0);
                assert!(
                    abs_err / scale < 1e-11,
                    "{} yasasında '{}' değişkenine göre sembolik türev ({}) != Dual64 ({})!",
                    id,
                    var,
                    sym_val,
                    dual_res.der
                );

                // Türevin Z^7 fiziksel boyutu dim(∂f/∂x) == dim(f) / dim(x) olmalıdır!
                let var_dim = law.dimensions[var];
                let expected_deriv_dim = f_dim.div_dim(&var_dim);
                let inferred_deriv_dim = sym_deriv.infer_dimension(&law.dimensions).unwrap();
                assert_eq!(
                    inferred_deriv_dim, expected_deriv_dim,
                    "{} yasasında ∂({})/∂({}) boyutu uyuşmadı!",
                    id, law.target, var
                );
            }
        }
    }

    #[test]
    fn test_symbolic_equation_inversion_and_bidirectional_hypergraph() {
        let mut kg = KnowledgeGraph::new();
        let base_order = kg.law_order.clone();
        assert!(base_order.len() >= 105);
        let inv_count = kg.register_algebraic_inversions();
        assert!(inv_count >= 150, "En az 150 ters yasa üretilmeliydi, üretilen: {}", inv_count);

        // Terslenen tüm yasalar dahil grafın tamamı Z^7 boyut denetiminden geçmelidir!
        let total_verified = kg.verify_all_laws().unwrap();
        assert_eq!(total_verified, base_order.len() + inv_count);

        // Her ters yasanın sayısal gidiş-dönüş (round-trip: x == f^-1(f(x))) doğruluğunu kanıtla
        for base_id in &base_order {
            let base_law = &kg.laws[base_id];
            let mut env = canonical_test_point_for_law(base_law, 1.8);
            let target_val = base_law.rhs.evaluate(&env).unwrap();
            env.insert(base_law.target.as_str(), target_val);

            for inp in &base_law.inputs {
                let inv_id = format!("{}__inv_{}", base_id, inp);
                if let Some(inv_law) = kg.laws.get(&inv_id) {
                    let recovered = inv_law.rhs.evaluate(&env).unwrap();
                    let orig = env[inp.as_str()];
                    let rel_err = ((recovered - orig) / orig.abs().max(1e-12)).abs();
                    assert!(
                        rel_err < 1e-10,
                        "Ters yasa {} gidiş-dönüş hatası: beklenen {}, bulunan {} (ifade: {})",
                        inv_id,
                        orig,
                        recovered,
                        inv_law.rhs
                    );
                }
            }
        }
    }

    #[test]
    fn test_multipoly_z_grevlex_and_multivariate_identities() {
        use crate::ast::Expr;

        // (a + b + c)^2 == a^2 + b^2 + c^2 + 2ab + 2ac + 2bc
        let p_lhs = MultiPolyZ::from_expr(&Expr::parse_str("(a + b + c)^2").unwrap(), 8).unwrap();
        let p_rhs = MultiPolyZ::from_expr(
            &Expr::parse_str("a^2 + b^2 + c^2 + 2*a*b + 2*a*c + 2*b*c").unwrap(),
            8,
        )
        .unwrap();
        assert_eq!(p_lhs, p_rhs);
        assert!(p_lhs.checked_sub(&p_rhs).unwrap().is_zero());

        // Derece sınırı aşımı reddedilmeli
        assert!(MultiPolyZ::from_expr(&Expr::parse_str("(a + b)^5").unwrap(), 4).is_err());
    }

    #[test]
    fn test_dormand_prince_rk45_adaptive_conservation() {
        use crate::constants::PhysicalConstants;
        use crate::simulator::{RK4Simulator, VehicleConfig};

        let h0 = 300_000.0;
        let v_init = (7500.0_f64.powi(2) + 400.0_f64.powi(2)).sqrt();
        let r0 = PhysicalConstants::R_EARTH + h0;
        let eps_init = 0.5 * v_init * v_init - PhysicalConstants::MU_EARTH / r0;

        let sim = RK4Simulator::new(0.5);
        let v_coast = VehicleConfig {
            name: "RK45 Vacuum Coast Test",
            dry_mass: 1000.0,
            propellant_mass: 0.0,
            thrust_vac: 0.0,
            isp: 300.0,
            area: 1.0,
            cd: 0.0,
            initial_velocity: v_init,
            initial_altitude: h0,
            has_stage_2: false,
            s2_dry_mass: 0.0,
            s2_propellant: 0.0,
            s2_thrust: 0.0,
            s2_isp: 0.0,
        };
        let res = sim.simulate_rk45(&v_coast, 200.0, 1e-10);
        let rf = PhysicalConstants::R_EARTH + res.final_altitude_km * 1000.0;
        let eps_f = 0.5 * res.final_speed_ms.powi(2) - PhysicalConstants::MU_EARTH / rf;
        let rel_err = ((eps_f - eps_init) / eps_init).abs();
        assert!(
            rel_err < 1e-12,
            "RK45 yörünge enerjisi korunmadı: rel_err={}",
            rel_err
        );
    }

    #[test]
    fn test_verified_codegen_emits_safe_rust_with_domain_guards() {
        use crate::codegen::VerifiedCodegen;
        use std::collections::HashSet;

        let kg = KnowledgeGraph::new();
        let mut inputs = HashSet::new();
        inputs.insert("g_0");
        inputs.insert("I_sp");
        inputs.insert("m_0");
        inputs.insert("m_f");
        let formulas = kg.synthesize_formulas(&inputs, "delta_v", 3);
        let code = VerifiedCodegen::emit_safe_rust_fn(&formulas[0]);
        assert!(code.contains("pub fn compute_delta_v"));
        assert!(code.contains("Result<f64, &'static str>"));
        assert!(code.contains("Tanım kümesi ihlali: m_f != 0"));
        assert!(code.contains("Tanım kümesi ihlali: (m_0 / m_f) > 0"));
    }

    #[test]
    fn test_simplifier_and_interval_never_swallow_domain_hazards() {
        use crate::ast::Expr;
        use crate::interval::Interval64;
        use crate::materials::MaterialsDatabase;
        use std::collections::HashMap;

        // 1. (x - 1) / (x - 1) sadeleştirildiğinde x - 1 != 0 yan koşulu silinmemelidir (§5.2)
        let rational = Expr::parse_str("(x - 1) / (x - 1)").unwrap().simplify();
        let guards = rational.extract_domain_conditions().unwrap();
        assert!(
            guards.contains(&"(x - 1) != 0".to_string()),
            "Sadeleştirici (x - 1) != 0 yan koşulunu yuttu: {:?}",
            guards
        );

        // 2. (1 / x) - (1 / x) ifadesi x ∈ [-1, 1] aralığında değerlendirildiğinde 0'a bölme hatası vermelidir!
        let f_sub = Expr::parse_str("(1 / x) - (1 / x)").unwrap();
        let mut env = HashMap::new();
        env.insert("x", Interval64::new(-1.0, 1.0).unwrap());
        assert!(
            f_sub.eval_interval(&env).is_err(),
            "eval_interval l == r kısayolu x ∈ [-1, 1] sıfıra bölme hatasını yuttu!"
        );

        // 3. Termal yumuşama (Thermal Derating): Ti-6Al-4V 630 K'de oda sıcaklığından daha düşük dayanıma sahip olmalıdır
        let mdb = MaterialsDatabase::load_canonical();
        let ti = mdb.get("mat_titanium_ti6al4v").unwrap();
        let (sy_room, e_room) = ti.derated_properties_at_temp_k(293.15);
        let (sy_hot, e_hot) = ti.derated_properties_at_temp_k(630.0);
        let (sy_melt, _) = ti.derated_properties_at_temp_k(700.0);
        assert!(sy_hot < sy_room * 0.75 && e_hot < e_room * 0.85);
        assert_eq!(sy_melt, 0.0);
    }

    #[test]
    fn test_inverted_codegen_has_no_unbound_target_variables() {
        use crate::codegen::VerifiedCodegen;
        use std::collections::HashSet;

        let mut kg = KnowledgeGraph::new();
        kg.register_algebraic_inversions();
        let mut inputs = HashSet::new();
        inputs.insert("delta_v");
        inputs.insert("g_0");
        inputs.insert("I_sp");
        inputs.insert("m_f");

        let formulas = kg.synthesize_formulas(&inputs, "m_0", 2);
        assert!(!formulas.is_empty(), "Ters Tsiolkovsky m_0 sentezlenemedi!");

        let code = VerifiedCodegen::emit_safe_rust_fn(&formulas[0]);
        assert!(code.contains("pub fn compute_m_0"));
        // Önkoşul satırlarında bağlanmamış `m_0` değişkeni geçmemeli; sonkoşulda `result / m_f` kullanılmalıdır!
        let (before_result, after_result) = code
            .split_once("let result =")
            .expect("Üretilen kodda 'let result =' satırı bulunamadı");
        let body_before_result = before_result
            .split_once('{')
            .map(|(_, b)| b)
            .unwrap_or("");
        assert!(
            !body_before_result.contains("(m_0"),
            "Üretilen kodda tanımlanmamış hedef değişken 'm_0' önkoşulda kullanıldı:\n{}",
            code
        );
        assert!(
            after_result.contains("(result / m_f)"),
            "Ters yasa sonkoşulu '(result / m_f)' olarak üretilmedi:\n{}",
            code
        );
    }

    #[test]
    fn test_physical_bounds_shield_conservation_carnot_relativity_and_momentum() {
        use crate::constants::PhysicalConstants;
        use crate::physical_bounds::{
            PhysicalBoundsShield, PhysicalLawViolation, PhysicalSystemCandidate,
        };

        // 1. Geçerli fiziksel sistem
        let ok_sys = PhysicalSystemCandidate {
            mass_kg: Some(500.0),
            velocity_ms: Some(11_200.0),
            exhaust_velocity_ms: Some(4_500.0),
            mass_flow_kg_s: Some(10.0),
            thrust_n: Some(45_000.0),
            power_in_w: Some(200_000_000.0),
            power_out_w: Some(101_250_000.0),
            efficiency: Some(0.55),
            temp_hot_k: Some(2500.0),
            temp_cold_k: Some(500.0),
            entropy_delta_j_k: Some(12.5),
        };
        assert!(PhysicalBoundsShield::verify_candidate(&ok_sys).is_ok());

        // 2. Özel Görelilik İhlali (v >= c)
        let superluminal = PhysicalSystemCandidate {
            mass_kg: Some(1.0),
            velocity_ms: Some(PhysicalConstants::C * 1.01),
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&superluminal),
            Err(PhysicalLawViolation::SuperluminalCausalityViolation { .. })
        ));

        // 3. Termodinamik 1. Yasa İhlali (P_out > P_in — 1. Tür Devridaim)
        let over_unity = PhysicalSystemCandidate {
            power_in_w: Some(1_000.0),
            power_out_w: Some(1_050.0),
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&over_unity),
            Err(PhysicalLawViolation::FirstLawEnergyViolation { .. })
        ));

        // 4. Carnot Sınırı İhlali (T_h=600K, T_c=300K => Carnot max 0.50 iken eta=0.75 — 2. Tür Devridaim)
        let carnot_viol = PhysicalSystemCandidate {
            efficiency: Some(0.75),
            temp_hot_k: Some(600.0),
            temp_cold_k: Some(300.0),
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&carnot_viol),
            Err(PhysicalLawViolation::CarnotBoundViolation { .. })
        ));

        // 5. Momentum Korunumu İhlali (m_dot = 0 iken 1 kW güçle 10 N tepkisiz itki iddiası; foton sınırı 6.67e-6 N)
        let reactionless = PhysicalSystemCandidate {
            mass_flow_kg_s: Some(0.0),
            power_in_w: Some(1_000.0),
            thrust_n: Some(10.0),
            ..Default::default()
        };
        assert!(matches!(
            PhysicalBoundsShield::verify_candidate(&reactionless),
            Err(PhysicalLawViolation::ReactionlessMomentumViolation { .. })
        ));

        // 6. Geçerli Foton Yelkeni (m_dot = 0, P_in = 300 MW => F_max = 2.0 N, F = 1.8 N geçerli!)
        let photon_sail = PhysicalSystemCandidate {
            mass_flow_kg_s: Some(0.0),
            power_in_w: Some(300_000_000.0),
            thrust_n: Some(1.8),
            ..Default::default()
        };
        assert!(PhysicalBoundsShield::verify_candidate(&photon_sail).is_ok());
    }

    #[test]
    fn test_domain_bounds_regime_conflict_pruning_and_ssot_ontology() {
        use std::collections::{HashMap, HashSet};

        let kg = KnowledgeGraph::new();

        // 1. Sıkıştırılamaz Pitot dinamik basıncı (M <= 0.3) -> Genel sürükleme (universal) GEÇERLİDİR,
        //    fakat Sıkıştırılamaz Pitot (M <= 0.3) -> Hipersonik Newtoncu Şok Dalga Sürüklemesi (M >= 5)
        //    rejim çatışması nedeniyle OTOMATİK BUDANIR!
        let mut pitot_inputs = HashSet::new();
        pitot_inputs.insert("P_pitot");
        pitot_inputs.insert("P_stat");
        pitot_inputs.insert("C_d");
        pitot_inputs.insert("A");
        pitot_inputs.insert("s_cone");
        pitot_inputs.insert("A_nose");

        let subsonic_drag = kg.synthesize_formulas(&pitot_inputs, "F_drag", 3);
        assert!(
            !subsonic_drag.is_empty(),
            "Sıkıştırılamaz Pitot -> F_drag (universal) zinciri sentezlenebilmeliydi!"
        );

        let illegal_hypersonic = kg.synthesize_formulas(&pitot_inputs, "F_drag_hyp", 3);
        assert!(
            illegal_hypersonic.is_empty(),
            "Sıkıştırılamaz Pitot (M<=0.3) ile Hipersonik Newtoncu Sürükleme (M>=5) zinciri budanmalıydı!"
        );

        // Buna karşılık evrensel dinamik basınç (0.5 * rho * v^2) -> Hipersonik Newtoncu Sürükleme GEÇERLİDİR!
        let mut hyp_inputs = HashSet::new();
        hyp_inputs.insert("rho");
        hyp_inputs.insert("v");
        hyp_inputs.insert("s_cone");
        hyp_inputs.insert("A_nose");
        let valid_hypersonic = kg.synthesize_formulas(&hyp_inputs, "F_drag_hyp", 3);
        assert_eq!(valid_hypersonic.len(), 1);
        assert!(valid_hypersonic[0].regimes.contains(&"hypersonic_continuum".to_string()));

        // 2. Çalışma Zamanı DomainBounds Denetimi: Prandtl-Glauert (Mach < 0.8) ve Lorentz (v < c)
        let pg = &kg.laws["law_prandtl_glauert_subsonic_cp"];
        let mut ok_pg = HashMap::new();
        ok_pg.insert("Mach", 0.55);
        ok_pg.insert("C_p0", -0.4);
        assert!(pg.check_validity_bounds(&ok_pg).is_ok());

        let mut bad_pg = HashMap::new();
        bad_pg.insert("Mach", 0.85);
        assert!(pg.check_validity_bounds(&bad_pg).is_err());

        let lor = &kg.laws["law_lorentz_factor"];
        let mut bad_lor = HashMap::new();
        bad_lor.insert("v", 299_792_458.0);
        bad_lor.insert("c", 299_792_458.0);
        assert!(
            lor.check_validity_bounds(&bad_lor).is_err(),
            "v == c kesin açık üst sınır (max_inclusive = false) ihlali yakalanmalıydı!"
        );

        // 3. SSOT Sembol-Boyut Tekilliği Kalkanı (Değişmez 6): Aynı sembol farklı boyutta yüklenemez!
        let conflicting_json = r#"[
            {
                "id": "test_law_1",
                "name": "Test 1",
                "equation": "y_test = x_conflict * 2",
                "target": "y_test",
                "inputs": ["x_conflict"],
                "dimensions": {"y_test": "length", "x_conflict": "length"}
            },
            {
                "id": "test_law_2",
                "name": "Test 2",
                "equation": "z_test = x_conflict * 3",
                "target": "z_test",
                "inputs": ["x_conflict"],
                "dimensions": {"z_test": "velocity", "x_conflict": "velocity"}
            }
        ]"#;
        let mut test_kg = KnowledgeGraph {
            laws: HashMap::new(),
            law_order: Vec::new(),
            producers: HashMap::new(),
            variable_dimensions: HashMap::new(),
        };
        let err = test_kg.load_from_json_str(conflicting_json).unwrap_err();
        assert!(err.contains("SSOT Ontoloji İhlali"), "Beklenen SSOT hatası alınamadı: {}", err);
    }

    #[test]
    fn test_buckingham_pi_taylor_sedov_reynolds_kepler_and_impossibility() {
        use crate::buckingham::BuckinghamPiSolver;
        use std::collections::HashMap;

        // 1. G.I. Taylor (1950) Patlama Şok Dalgası: r = C_pi * ((E * t^2) / rho)^(1/5)
        let blast_vars = BuckinghamPiSolver::parse_variable_specs("r,E,rho,t").unwrap();
        let c_true = 1.033_f64;
        let mut samples = Vec::new();
        for &(e_j, rho_kgm3, t_s) in &[
            (8.4e13_f64, 1.225_f64, 0.006_f64),
            (8.4e13_f64, 1.225_f64, 0.016_f64),
            (8.4e13_f64, 1.225_f64, 0.053_f64),
        ] {
            let r_m = c_true * ((e_j * t_s * t_s) / rho_kgm3).powf(0.2);
            let mut row = HashMap::new();
            row.insert("r".to_string(), r_m);
            row.insert("E".to_string(), e_j);
            row.insert("rho".to_string(), rho_kgm3);
            row.insert("t".to_string(), t_s);
            samples.push(row);
        }
        let blast_rep =
            BuckinghamPiSolver::discover(&blast_vars, Some("r"), &[], &samples).unwrap();
        assert_eq!(blast_rep.dimensional_rank, 3);
        assert_eq!(blast_rep.num_pi_groups, 1);
        assert!(blast_rep.target_identifiable);
        assert_eq!(blast_rep.target_scaling_dimension.as_deref(), Some("L"));
        assert!(blast_rep.safe_rust_fn.as_deref().unwrap().contains("pub fn compute_r"));
        let fit = blast_rep.empirical_fit.unwrap();
        assert!((fit.coefficient_c_pi - c_true).abs() < 1e-9);
        assert!(fit.r_squared > 0.999999);

        // 2. Aerodinamik Sürükleme: Pi_1 = C_F, Pi_2 = Reynolds Sayısı (F_drag Pi_2'de yer almaz!)
        let drag_vars =
            BuckinghamPiSolver::parse_variable_specs("F_drag,rho,v,L_char,mu_visc").unwrap();
        let drag_rep =
            BuckinghamPiSolver::discover(&drag_vars, Some("F_drag"), &[], &[]).unwrap();
        assert_eq!(drag_rep.dimensional_rank, 3);
        assert_eq!(drag_rep.num_pi_groups, 2);
        assert!(
            drag_rep.pi_groups[0]
                .exponents
                .iter()
                .any(|(nm, e)| nm == "F_drag" && *e == 1)
        );
        assert!(
            !drag_rep.pi_groups[1]
                .exponents
                .iter()
                .any(|(nm, _)| nm == "F_drag"),
            "Hedef değişken F_drag benzerlik parametresi Pi_2 içinde izole edilmiş olmalıydı!"
        );
        assert_eq!(
            drag_rep.pi_groups[1].recognized_number.as_deref(),
            Some("Reynolds Sayısı (Re)")
        );

        // 3. Basit Sarkaç: Kütle (m) bağımsızlığının Z^7 çekirdek uzayında kanıtı!
        let pend_vars =
            BuckinghamPiSolver::parse_variable_specs("T_orb,L_char,g_0,m").unwrap();
        let pend_rep =
            BuckinghamPiSolver::discover(&pend_vars, Some("T_orb"), &[], &[]).unwrap();
        assert_eq!(pend_rep.num_pi_groups, 1);
        assert!(
            !pend_rep.pi_groups[0].exponents.iter().any(|(nm, _)| nm == "m"),
            "Sarkaç periyodu kütleden (m) bağımsız çıkmalıydı!"
        );

        // 4. Boyutsal İmkânsızlık Kanıtı: Kütle boyutu içermeyen (L_char, t) ile F_drag türetilemez!
        let imp_vars = BuckinghamPiSolver::parse_variable_specs("F_drag,L_char,t").unwrap();
        let imp_rep =
            BuckinghamPiSolver::discover(&imp_vars, Some("F_drag"), &[], &[]).unwrap();
        assert!(!imp_rep.target_identifiable);
        assert!(imp_rep.target_unidentifiable_reason.is_some());
    }

    #[test]
    fn test_pareto_nsga2_skyline_crowding_hypervolume_and_knee() {
        use crate::pareto::{ObjectiveSpec, ParetoSynthesizer};

        // Kesin 2B süpürme hiperhacmi: R=(10,10), A=(2,6), B=(5,3) -> (10-2)*(10-6) + (10-5)*(6-3) = 47
        let hv = ParetoSynthesizer::exact_hypervolume_2d(&[(2.0, 6.0), (5.0, 3.0)], (10.0, 10.0));
        assert!((hv - 47.0).abs() < 1e-12);

        let specs = vec![
            ObjectiveSpec::minimize("mass_kg", "kg"),
            ObjectiveSpec::maximize("safety_factor", "x"),
        ];
        let matrix = vec![
            vec![1000.0, 1.40], // 0: F_0 çapa (en hafif)
            vec![1220.0, 2.10], // 1: F_0 diz noktası (Knee)
            vec![1850.0, 2.45], // 2: F_0 çapa (en yüksek emniyet)
            vec![1450.0, 1.85], // 3: F_1 (#1 tarafından baskılanır)
            vec![1700.0, 1.50], // 4: F_2 (#3 ve #1 tarafından baskılanır)
        ];

        let summary = ParetoSynthesizer::analyze(&specs, &matrix).unwrap();
        ParetoSynthesizer::verify_skyline_invariants(&specs, &matrix, &summary).unwrap();
        assert_eq!(summary.skyline_count, 3);
        assert_eq!(summary.num_fronts, 3);
        assert_eq!(summary.knee_candidate_index, Some(1));
        assert_eq!(summary.utopia_point, vec![1000.0, 2.45]);
        assert_eq!(summary.nadir_point, vec![1850.0, 1.40]);
        assert!(summary.point_metrics[0].crowding_distance.is_infinite());
        assert!(summary.point_metrics[2].crowding_distance.is_infinite());
        assert!(summary.point_metrics[1].crowding_distance.is_finite());
        assert_eq!(summary.point_metrics[3].pareto_rank, 1);
        assert_eq!(summary.point_metrics[4].pareto_rank, 2);
    }

    #[test]
    fn test_inverse_design_exact_pareto_skyline_invariants() {
        use crate::designer::{InverseDesignEngine, MissionConstraints};
        use crate::pareto::ParetoSynthesizer;

        let report = InverseDesignEngine::optimize(MissionConstraints::default());
        assert!(report.pareto_summary.skyline_count >= 3);
        assert!(report.pareto_summary.normalized_hypervolume_2d.unwrap_or(0.0) > 0.0);
        for d in &report.pareto_optimal_designs {
            assert_eq!(
                d.pareto_rank, 0,
                "Vitrin Pareto tasarımında Rank > 0 aday bulunamaz!"
            );
        }
        // Skyline içindeki hiçbir iki tasarım birbirini katı biçimde baskılayamaz
        let specs = &report.pareto_summary.objectives;
        let sky_vecs: Vec<Vec<f64>> = report
            .exact_skyline_designs
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
        for i in 0..sky_vecs.len() {
            for j in 0..sky_vecs.len() {
                if i != j {
                    assert!(!ParetoSynthesizer::dominates(&sky_vecs[i], &sky_vecs[j], specs));
                }
            }
        }
    }

    #[test]
    fn test_mathematical_universe_zero_compromise_soundness() {
        use crate::ast::Expr;
        use crate::codegen::VerifiedCodegen;
        use crate::dimensions::Dimension;
        use crate::graph::SynthesizedFormula;
        use crate::interval::Interval64;
        use crate::simulator::{AtmosphereModel, RK4Simulator, VehicleConfig};
        use std::collections::HashMap;

        // 1. Sıfır NaN / Sıfır ±Infinity Sızıntısı (Expr::evaluate, Interval64, Dual64)
        let empty: HashMap<&str, f64> = HashMap::new();
        for bad_expr in [
            "(-4)^0.5",
            "0^(-2)",
            "0^0",
            "1 / 0",
            "ln(0)",
            "ln(-3)",
            "sqrt(-1)",
            "exp(1000)",
        ] {
            let parsed = Expr::parse_str(bad_expr).unwrap();
            assert!(
                parsed.evaluate(&empty).is_err(),
                "'{}' tanımsız/taşan ifadesi Ok(...) döndürmemeliydi!",
                bad_expr
            );
            let simp = parsed.simplify();
            assert!(
                simp.evaluate(&empty).is_err(),
                "'{}' sadeleştirme sonrası NaN/Inf sızdırdı: {}",
                bad_expr,
                simp
            );
        }
        // Negatif tabanın tamsayı kuvvetleri reel uzayda kesindir: (-2)^3 = -8, (-2)^(-3) = -0.125
        assert_eq!(Expr::parse_str("(-2)^3").unwrap().evaluate(&empty).unwrap(), -8.0);
        assert_eq!(
            Expr::parse_str("(-2)^(-3)").unwrap().evaluate(&empty).unwrap(),
            -0.125
        );

        // 2. Cebirsel Sadeleştirici Sağlamlığı: Tanım Kümesi ve İşaret Yutan Kuralların Reddi
        // (x^2)^0.5 == |x| != x: x = -3 için +3.0 vermelidir (asla -3.0 değil!)
        let root_sq = Expr::parse_str("(x^2)^0.5").unwrap().simplify();
        let mut neg_env = HashMap::new();
        neg_env.insert("x", -3.0);
        let root_val = root_sq.evaluate(&neg_env).unwrap();
        assert!(
            (root_val - 3.0).abs() < 1e-12,
            "(x^2)^0.5 ifadesi x=-3 noktasında |x|=3 yerine {} üretti!",
            root_val
        );

        // exp(ln(x)) tanım kümesi x > 0'dır: sadeleştirme x > 0 yan koşulunu silemez
        let exp_ln = Expr::parse_str("exp(ln(x))").unwrap().simplify();
        assert!(exp_ln.evaluate(&neg_env).is_err());
        let exp_ln_guards = exp_ln.extract_domain_conditions().unwrap();
        assert!(exp_ln_guards.contains(&"x > 0".to_string()));

        // (x^0.5)^2 tanım kümesi x >= 0'dır: x = -3 için reel uzayda tanımsızdır
        let sq_root = Expr::parse_str("(x^0.5)^2").unwrap().simplify();
        assert!(sq_root.evaluate(&neg_env).is_err());
        let sq_root_guards = sq_root.extract_domain_conditions().unwrap();
        assert!(sq_root_guards.contains(&"x >= 0".to_string()));

        // x^(-1) ve (x^(-1))^(-1) ifadeleri x != 0 yan koşulunu korumak zorundadır
        let inv_x = Expr::parse_str("x^(-1)").unwrap();
        assert!(inv_x.extract_domain_conditions().unwrap().contains(&"x != 0".to_string()));
        let double_inv = Expr::parse_str("(x^(-1))^(-1)").unwrap().simplify();
        assert!(
            double_inv
                .extract_domain_conditions()
                .unwrap()
                .contains(&"x != 0".to_string())
        );

        // 3. Denklem Terslemede Sahte Kök (Extraneous Root) Engeli: y = x^0.5 => x = y^2, [y >= 0, x >= 0]
        let lhs_y = Expr::var("y");
        let rhs_sqrt_x = Expr::parse_str("x^0.5").unwrap();
        let (isolated_x, solve_guards) = Expr::solve_for(&lhs_y, &rhs_sqrt_x, "x").unwrap();
        assert!(solve_guards.contains(&"y >= 0".to_string()));
        assert!(solve_guards.contains(&"x >= 0".to_string()));
        let synth_fn = VerifiedCodegen::emit_safe_rust_fn(&SynthesizedFormula {
            target: "x".to_string(),
            chain_ids: vec!["sound_inv".to_string()],
            chain_names: vec!["Sound Inversion".to_string()],
            regimes: vec!["universal".to_string()],
            free_variables: vec!["y".to_string()],
            inferred_dimension: Dimension::dimensionless(),
            symbolic_expr: isolated_x,
            domain_guards: solve_guards,
            effective_bounds: HashMap::new(),
        });
        assert!(
            synth_fn.contains("(y) < 0.0_f64"),
            "Sahte kök önkoşulu (y >= 0) üretilen Rust koduna derlenmedi:\n{}",
            synth_fn
        );

        // 4. 53-Bit Üstü (|p| > 2^53) Tam 1-ULP Interval64::from_rational ve 0^0 Reddi
        assert!(Interval64::new(0.0, f64::INFINITY).is_err());
        assert!(Interval64::from_rational(0, 1).unwrap().powi(0).is_err());

        // (2^54 + 3) / 2 = 2^53 + 1.5 -> iki ardışık f64 tam sayısı [2^53 + 0, 2^53 + 2] arasında tam 1 ULP!
        let big_odd = (1_i64 << 54) + 3;
        let iv_big = Interval64::from_rational(big_odd, 2).unwrap();
        assert_eq!(iv_big.lo, 9_007_199_254_740_992.0);
        assert_eq!(iv_big.hi, 9_007_199_254_740_994.0);
        assert_eq!(iv_big.hi.to_bits() - iv_big.lo.to_bits(), 1);

        // (i64::MAX - 1) / i64::MAX < 1.0 olduğu için lo kesinlikle < 1.0 ve hi == 1.0 (1 ULP) olmalıdır!
        let iv_near_one = Interval64::from_rational(i64::MAX - 1, i64::MAX).unwrap();
        assert!(iv_near_one.lo < 1.0 && iv_near_one.hi == 1.0);
        assert_eq!(iv_near_one.hi.to_bits() - iv_near_one.lo.to_bits(), 1);

        let iv_neg_near_one = Interval64::from_rational(-(i64::MAX - 1), i64::MAX).unwrap();
        assert!(iv_neg_near_one.lo == -1.0 && iv_neg_near_one.hi > -1.0);
        assert_eq!(iv_neg_near_one.lo.to_bits() - iv_neg_near_one.hi.to_bits(), 1);

        // 5. Z^7 Boyut Üssü i8 Taşma Kalkanı
        let l80 = Dimension::length().pow_dim(80);
        assert!(l80.checked_mul_dim(&l80).is_err());

        // 6. USSA-1976 7-Katmanlı Hidrostatik Atmosfer & Vakum Yörüngede Tam 0.0 G Eşdeğerlik İlkesi
        let (rho0, p0, t0, a0) = AtmosphereModel::state_at(0.0);
        assert!((t0 - 288.15).abs() < 1e-12);
        assert!((p0 - 101_325.0).abs() < 1e-9);
        assert!((rho0 - 1.225).abs() < 1e-4);
        assert!((a0 - 340.294).abs() < 0.01);

        // Katman sınırlarında (Z = 11, 20, 32, 47, 51, 71 km ve h = 86 km) hidrostatik süreklilik
        for z_bound in [11_000.0_f64, 20_000.0, 32_000.0, 47_000.0, 51_000.0, 71_000.0] {
            let r0 = AtmosphereModel::R0_GEOPOTENTIAL;
            let h_bound = (r0 * z_bound) / (r0 - z_bound);
            let (rho_m, p_m, t_m, _) = AtmosphereModel::state_at(h_bound - 0.05);
            let (rho_p, p_p, t_p, _) = AtmosphereModel::state_at(h_bound + 0.05);
            assert!((t_p - t_m).abs() < 1e-3, "Z={} m sınırında sıcaklık süreksizliği!", z_bound);
            assert!(
                ((p_p - p_m) / p_m).abs() < 1e-4,
                "Z={} m sınırında basınç süreksizliği: {} vs {}",
                z_bound,
                p_m,
                p_p
            );
            assert!(((rho_p - rho_m) / rho_m).abs() < 1e-4);
        }

        // Vakum serbest düşüş yörüngesi (h = 300 km, F_thrust = 0, C_d = 0) -> Öz ivme (G-yükü) tam 0.0 G!
        let sim = RK4Simulator::new(0.1);
        let freefall_sat = VehicleConfig {
            name: "Zero-G Equivalence Principle Satellite",
            dry_mass: 1000.0,
            propellant_mass: 0.0,
            thrust_vac: 0.0,
            isp: 300.0,
            area: 1.0,
            cd: 0.0,
            initial_velocity: 7725.0,
            initial_altitude: 300_000.0,
            has_stage_2: false,
            s2_dry_mass: 0.0,
            s2_propellant: 0.0,
            s2_thrust: 0.0,
            s2_isp: 0.0,
        };
        let res_rk4 = sim.simulate(&freefall_sat, 30.0);
        let res_rk45 = sim.simulate_rk45(&freefall_sat, 30.0, 1e-9);
        assert_eq!(res_rk4.max_g_force, 0.0);
        assert_eq!(res_rk45.max_g_force, 0.0);
    }

    #[test]
    fn test_external_standards_ieee1788_codata2022_ussa76_sp8007_and_dopri5() {
        use crate::constants::PhysicalConstants;
        use crate::interval::Interval64;
        use crate::materials::{CylinderGeometry, MaterialsDatabase, StructuralStressChecker};
        use crate::simulator::{AtmosphereModel, RK4Simulator, VehicleConfig};
        use std::f64::consts::PI;

        // 1. IEEE Std 1788-2015 `tightest` Dışa Yuvarlama (Rump / Dekker / Boldo-Melquiond EFT & `inari` eşliği)
        let i1 = Interval64::point(1.0).unwrap();
        let i2 = Interval64::point(2.0).unwrap();
        let i3 = Interval64::point(3.0).unwrap();
        let i7 = Interval64::point(7.0).unwrap();
        let i9 = Interval64::point(9.0).unwrap();
        let i21 = Interval64::point(21.0).unwrap();

        // Tam temsil edilebilen kayan nokta işlemlerinde 0-ULP sıfır şişme (zero bloat)
        assert_eq!(i1.add(&i2), i3);
        assert_eq!(i3.mul(&i7), i21);
        assert_eq!(i21.div(&i7).unwrap(), i3);
        assert_eq!(i9.sqrt().unwrap(), i3);
        assert_eq!(i3.width(), 0.0);
        assert_eq!(
            Interval64::new(-2.0, 3.0).unwrap().sqr(),
            Interval64::new(0.0, 9.0).unwrap()
        );

        // Tam olmayan işlemlerde tam 1-ULP en dar dışa yuvarlama (`hi == lo.next_up()`)
        let tiny = Interval64::point(1e-20).unwrap();
        let sum_inexact = i1.add(&tiny);
        assert_eq!(sum_inexact.lo, 1.0);
        assert_eq!(sum_inexact.hi, 1.0_f64.next_up());

        let one_third = i1.div(&i3).unwrap();
        assert_eq!(one_third.hi, one_third.lo.next_up());
        assert!(one_third.lo < 1.0 / 3.0 || one_third.hi > 1.0 / 3.0);

        let sqrt2 = i2.sqrt().unwrap();
        assert_eq!(sqrt2.hi, sqrt2.lo.next_up());

        // Pozitif subnormal underflow korunumu: [1e-200, 1e-200]^2 -> [0.0, MIN_POSITIVE_SUBNORMAL]
        let sub_u = Interval64::point(1e-200)
            .unwrap()
            .mul(&Interval64::point(1e-200).unwrap());
        assert_eq!(sub_u.lo, 0.0);
        assert_eq!(sub_u.hi, f64::from_bits(1));

        // 2. NIST CODATA 2022 & 2019 SI Tam Sabit İlişkileri
        assert_eq!(PhysicalConstants::MU_0, 1.256_637_061_27e-6);
        assert_eq!(PhysicalConstants::EPS_0, 8.854_187_818_8e-12);
        assert_eq!(PhysicalConstants::ALPHA, 7.297_352_564_3e-3);
        assert!(
            (PhysicalConstants::HBAR - PhysicalConstants::H / (2.0 * PI)).abs() < 1e-49,
            "HBAR != h / (2*pi)"
        );
        assert!(
            (PhysicalConstants::R_GAS - PhysicalConstants::N_A * PhysicalConstants::K_B).abs()
                < 1e-14,
            "R_GAS != N_A * k_B"
        );
        let sigma_exact = (PI.powi(2) * PhysicalConstants::K_B.powi(4))
            / (60.0 * PhysicalConstants::HBAR.powi(3) * PhysicalConstants::C.powi(2));
        assert!(((PhysicalConstants::SIGMA_SB - sigma_exact) / sigma_exact).abs() < 1e-15);
        assert!(
            (PhysicalConstants::MU_0 * PhysicalConstants::EPS_0 * PhysicalConstants::C.powi(2)
                - 1.0)
                .abs()
                < 2e-10
        );
        let qed_rel = (2.0 * PhysicalConstants::ALPHA * PhysicalConstants::H)
            / (PhysicalConstants::MU_0 * PhysicalConstants::C * PhysicalConstants::E.powi(2));
        assert!((qed_rel - 1.0).abs() < 2e-10);

        // 3. NASA-TM-X-74335 (US Standard Atmosphere 1976) 8 Katman Resmi Tablo Karşılaştırması
        let ussa76_table: [(f64, f64, f64); 8] = [
            (0.0, 288.15, 101_325.0),
            (11_000.0, 216.65, 22_632.06),
            (20_000.0, 216.65, 5_474.89),
            (32_000.0, 228.65, 868.019),
            (47_000.0, 270.65, 110.906),
            (51_000.0, 270.65, 66.9389),
            (71_000.0, 214.65, 3.95642),
            (84_852.0, 186.946, 0.37338),
        ];
        let r0 = AtmosphereModel::R0_GEOPOTENTIAL;
        for (z_m, t_ref, p_ref) in ussa76_table {
            let h_geom = (r0 * z_m) / (r0 - z_m);
            let (_rho, p_pa, t_k, _a) = AtmosphereModel::state_at(h_geom);
            assert!(
                (t_k - t_ref).abs() < 5e-3,
                "USSA-1976 Z={} m sıcaklık sapması: {} vs {}",
                z_m,
                t_k,
                t_ref
            );
            assert!(
                ((p_pa - p_ref) / p_ref).abs() < 2e-4,
                "USSA-1976 Z={} m basınç sapması: {} vs {}",
                z_m,
                p_pa,
                p_ref
            );
        }

        // 4. NASA SP-8007 (1968 / 2020 REV2) Eq. (11), (13), (17) Eksenel + Eğilme Etkileşimi
        let mdb = MaterialsDatabase::load_canonical();
        let alli = mdb.get("mat_alli_2195").unwrap();
        let geom_short = CylinderGeometry {
            radius_m: 2.0,
            length_m: 10.0,
            wall_thickness_mm: 4.0,
        };
        let (_, sf_pure_axial, _) = StructuralStressChecker::check_cylinder_buckling_at_temp(
            alli, 250_000.0, 4.0, 0.0, geom_short, 293.15,
        );
        let (_, sf_with_maxq_bend, _) = StructuralStressChecker::check_cylinder_buckling_at_temp(
            alli, 250_000.0, 4.0, 45_000.0, geom_short, 293.15,
        );
        assert!(
            sf_pure_axial > sf_with_maxq_bend && sf_with_maxq_bend > 0.0,
            "Max-Q eğilme ve sürükleme yükü SP-8007 etkileşim emniyet katsayısını düşürmeli!"
        );

        // 5. Dormand-Prince RK4(5) (`DOPRI5`, 1980) Analitik Kepler Elipsi Korunumu (e = 0.60, a = r_0)
        // Başlangıç hızı (0.8*v0, 0.6*v0) için r_0 = R_earth + 10_000 km ve v_0 = sqrt(mu / r_0) seçildiğinde:
        //   E_0 = -mu / (2*r_0) => yarı-büyük eksen a = r_0, dışmerkezlik e = sqrt(1 - 0.8^2) = 0.60 (analitik tam)
        let h0_m = 10_000_000.0;
        let r0_orb = PhysicalConstants::R_EARTH + h0_m;
        let v0_kepler = (PhysicalConstants::MU_EARTH / r0_orb).sqrt();
        let sat = VehicleConfig {
            name: "Kepler Conservation Benchmark",
            dry_mass: 1000.0,
            propellant_mass: 0.0,
            thrust_vac: 0.0,
            isp: 300.0,
            area: 1.0,
            cd: 0.0,
            initial_velocity: v0_kepler,
            initial_altitude: h0_m,
            has_stage_2: false,
            s2_dry_mass: 0.0,
            s2_propellant: 0.0,
            s2_thrust: 0.0,
            s2_isp: 0.0,
        };
        let dopri_res = RK4Simulator::new(0.5).simulate_rk45(&sat, 1200.0, 1e-11);
        assert!((dopri_res.eccentricity - 0.60).abs() < 1e-10);
        let sma_alt_km = 0.5 * (dopri_res.apogee_altitude_km + dopri_res.perigee_altitude_km);
        assert!((sma_alt_km - 10_000.0).abs() < 1e-6);
        let r_final = PhysicalConstants::R_EARTH + dopri_res.final_altitude_km * 1000.0;
        let e_init = -PhysicalConstants::MU_EARTH / (2.0 * r0_orb);
        let e_final = 0.5 * dopri_res.final_speed_ms.powi(2) - PhysicalConstants::MU_EARTH / r_final;
        assert!(((e_final - e_init) / e_init.abs()).abs() < 1e-10);
    }

    #[test]
    fn test_astrodynamics_j2_secular_drift_and_lambert_transfer() {
        use crate::astrodynamics::{J2OrbitalModel, LambertSolver, Vector3};

        // 1. J2 Pertürbasyon İvmesi ve Vektör Doğrulaması
        let r_leo = Vector3::new(7_000_000.0, 0.0, 0.0);
        let a_j2 = J2OrbitalModel::acceleration_j2(r_leo).unwrap();
        assert!(a_j2.x < 0.0, "Ekvatorda J2 ivmesi merkeze doğru ek çekim sağlamalıdır");
        assert_eq!(a_j2.y, 0.0);
        assert_eq!(a_j2.z, 0.0);

        // 2. ISS / LEO Düğüm Çizgisi (RAAN) Presesyonu: a = 6778 km, e = 0.001, i = 51.64°
        let a_iss = 6_778_137.0;
        let e_iss = 0.001;
        let i_iss = 51.64_f64.to_radians();
        let raan_rate = J2OrbitalModel::raan_precession_rate_rad_s(a_iss, e_iss, i_iss).unwrap();
        let deg_per_day = raan_rate.to_degrees() * 86400.0;
        // 400 km LEO irtifasında analitik değer: tam -5.0° / gün (-4.998 deg/day)
        assert!((deg_per_day - (-5.0)).abs() < 0.05, "ISS RAAN kayması -5.0 deg/day olmalı: {}", deg_per_day);

        // 3. Molniya Kritik Eğim Açısı (domega/dt = 0)
        let i_crit = J2OrbitalModel::CRITICAL_INCLINATION_RAD;
        let perigee_rate = J2OrbitalModel::perigee_precession_rate_rad_s(a_iss, 0.1, i_crit).unwrap();
        assert!(perigee_rate.abs() < 1e-18, "Kritik eğim açısında enberi argümanı presesyonu 0 olmalıdır: {}", perigee_rate);
        assert!(J2OrbitalModel::is_critical_inclination(i_crit, 0.01));

        // 4. Güneş-Eşzamanlı Yörünge (SSO): 800 km dairesel yörüngede eğim ~ 98.6°
        let a_sso = J2OrbitalModel::WGS84_R_EARTH + 800_000.0;
        let i_sso = J2OrbitalModel::sun_synchronous_inclination_rad(a_sso, 0.0).unwrap();
        let i_sso_deg = i_sso.to_degrees();
        assert!((i_sso_deg - 98.6).abs() < 0.2, "800 km SSO eğimi 98.6° olmalı: {}", i_sso_deg);
        let sso_rate = J2OrbitalModel::raan_precession_rate_rad_s(a_sso, 0.0, i_sso).unwrap();
        assert!(
            ((sso_rate - J2OrbitalModel::SSO_RAAN_RATE_RAD_S) / J2OrbitalModel::SSO_RAAN_RATE_RAD_S).abs() < 1e-10,
            "SSO RAAN kayma hızı tanım değeriyle eşleşmeli"
        );

        // 5. Evrensel Değişkenli Lambert Çözücüsü (90° Transfer)
        let r1 = Vector3::new(7_000_000.0, 0.0, 0.0);
        let r2 = Vector3::new(0.0, 9_000_000.0, 0.0);
        let tof = 2000.0;
        let lambert = LambertSolver::solve(r1, r2, tof, true, J2OrbitalModel::WGS84_MU).unwrap();

        // Enerji Korunumu: eps1 == eps2
        let mu = J2OrbitalModel::WGS84_MU;
        let eps1 = 0.5 * lambert.v1.norm_sq() - mu / r1.norm();
        let eps2 = 0.5 * lambert.v2.norm_sq() - mu / r2.norm();
        assert!(((eps2 - eps1) / eps1.abs()).abs() < 1e-7, "Lambert transferinde enerji korunumu bozuldu: {} vs {}", eps1, eps2);

        // Açısal Momentum Korunumu: h1 == h2
        let h1 = r1.cross(&lambert.v1).norm();
        let h2 = r2.cross(&lambert.v2).norm();
        assert!(((h2 - h1) / h1).abs() < 1e-7, "Lambert transferinde açısal momentum korunumu bozuldu: {} vs {}", h1, h2);
        assert!(lambert.semi_major_axis_m > 7_000_000.0);
        assert!(lambert.eccentricity > 0.0 && lambert.eccentricity < 1.0);
    }
}
