// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — IEEE-754 Dışa Yuvarlamalı Aralık Aritmetiği (Interval64)
// ve İleri Mod Otomatik Türev (Dual64 / Taylor-Mean-Value Aralık Kapanımı)

use std::cmp::Ordering;
use std::fmt;

/// IEEE-754 Dışa Yuvarlamalı (Outward-Rounded) Kapalı Sonlu Reel Aralık: [lo, hi] ⊂ R
/// Değişmez (Invariant): lo.is_finite() && hi.is_finite() && lo <= hi
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval64 {
    pub lo: f64,
    pub hi: f64,
}

/// Sonlu bir IEEE-754 `f64` değerini tam kesir `num / den` (`den > 0`) ile
/// ikili mantissa + `u128` çapraz çarpımı üzerinden sıfır yuvarlama hatasıyla karşılaştırır.
fn cmp_f64_rational(x: f64, num: i128, den: u128) -> Ordering {
    debug_assert!(x.is_finite() && den > 0);
    if x == 0.0 {
        return 0_i128.cmp(&num);
    }
    let bits = x.to_bits();
    let neg = (bits >> 63) != 0;
    let raw_exp = ((bits >> 52) & 0x7ff) as i32;
    let raw_frac = bits & 0x000f_ffff_ffff_ffff;

    if neg && num >= 0 {
        return Ordering::Less;
    }
    if !neg && num <= 0 {
        return Ordering::Greater;
    }

    let (sig, exp): (u128, i32) = if raw_exp == 0 {
        (u128::from(raw_frac), -1074)
    } else {
        (u128::from((1_u64 << 52) | raw_frac), raw_exp - 1075)
    };

    let abs_n = num.unsigned_abs();
    // sig < 2^53 ve den <= 2^63 => sig * den < 2^116 < u128::MAX (taşma imkânsız)
    let prod = sig * den;

    let mag_cmp = if exp >= 0 {
        let sh = exp as u32;
        if sh >= 128 || prod > (u128::MAX >> sh) {
            Ordering::Greater
        } else {
            (prod << sh).cmp(&abs_n)
        }
    } else {
        let sh = (-exp) as u32;
        if sh >= 128 || abs_n > (u128::MAX >> sh) {
            Ordering::Less
        } else {
            prod.cmp(&(abs_n << sh))
        }
    };

    if neg {
        mag_cmp.reverse()
    } else {
        mag_cmp
    }
}

/// Verilen kayan noktalı sayıyı `ulps` adım kadar IEEE-754 eksi sonsuza (-∞) doğru kaydırır.
#[inline]
fn step_down_ulps(mut x: f64, ulps: u32) -> f64 {
    for _ in 0..ulps {
        x = x.next_down();
    }
    x
}

/// Verilen kayan noktalı sayıyı `ulps` adım kadar IEEE-754 artı sonsuza (+∞) doğru kaydırır.
#[inline]
fn step_up_ulps(mut x: f64, ulps: u32) -> f64 {
    for _ in 0..ulps {
        x = x.next_up();
    }
    x
}

/// Knuth TwoSum (TAOCP Cilt 2, Teorem B) hatasız dönüşümü (EFT) ile
/// CPU yuvarlama modunu değiştirmeden (`#![forbid(unsafe_code)]`) IEEE Std 1788-2015 `tightest`
/// yönlendirilmiş toplama `[fl_down(a + b), fl_up(a + b)]` hesaplar:
/// - Toplam kayan noktada tam ise (`err == 0.0`): 0-ULP genişlikli `[s, s]`
/// - Yukarı yuvarlanmışsa (`err < 0.0`): 1-ULP `[s.next_down(), s]`
/// - Aşağı yuvarlanmışsa (`err > 0.0`): 1-ULP `[s, s.next_up()]`
#[inline]
fn add_directed(a: f64, b: f64) -> (f64, f64) {
    if a == 0.0 {
        let z = if b == 0.0 { 0.0 } else { b };
        return (z, z);
    }
    if b == 0.0 {
        return (a, a);
    }
    let raw_s = a + b;
    let s = if raw_s == 0.0 { 0.0 } else { raw_s };
    if !s.is_finite() {
        return (s, s);
    }
    if a.abs() < 1e307 && b.abs() < 1e307 {
        let a_prime = s - b;
        let b_prime = s - a_prime;
        let err = (a - a_prime) + (b - b_prime);
        if err.is_finite() {
            return if err == 0.0 {
                (s, s)
            } else if err > 0.0 {
                (s, s.next_up())
            } else {
                (s.next_down(), s)
            };
        }
    } else if a.abs() >= 1e290 && b.abs() >= 1e290 {
        // Her iki terim de büyük normal sayı (|a|, |b| >= 1e290): 0.5 ile tam üs kaydırması
        // ne alt-taşma (underflow) ne de üst-taşma yapar; TwoSum tamamen hatasızdır.
        let a_h = a * 0.5;
        let b_h = b * 0.5;
        let s_h = a_h + b_h;
        let a_p = s_h - b_h;
        let b_p = s_h - a_p;
        let err_h = (a_h - a_p) + (b_h - b_p);
        if err_h.is_finite() {
            return if err_h == 0.0 {
                (s, s)
            } else if err_h > 0.0 {
                (s, s.next_up())
            } else {
                (s.next_down(), s)
            };
        }
    } else {
        // Biri >= 1e307 (big), diğeri < 1e290 (small != 0): |small| < 0.5 * ulp(big) olduğundan
        // s == big tamdır ve gerçek toplam kesinlikle small'ın işareti yönündedir (subnormal dahil!).
        let (big, small) = if a.abs() >= b.abs() { (a, b) } else { (b, a) };
        return if small > 0.0 {
            (big, big.next_up())
        } else {
            (big.next_down(), big)
        };
    }
    (s.next_down(), s.next_up())
}

/// IEEE-754 FMA (`TwoProdFMA`: `err = fma(a, b, -p)`) ile IEEE Std 1788-2015 `tightest`
/// yönlendirilmiş çarpma `[fl_down(a * b), fl_up(a * b)]` hesaplar.
#[inline]
fn mul_directed(a: f64, b: f64) -> (f64, f64) {
    if a == 0.0 || b == 0.0 {
        return (0.0, 0.0);
    }
    let p = a * b;
    if !p.is_finite() {
        return (p, p);
    }
    if p.abs() >= 1e-285 {
        let err = a.mul_add(b, -p);
        if err.is_finite() {
            return if err == 0.0 {
                (p, p)
            } else if err > 0.0 {
                (p, p.next_up())
            } else {
                (p.next_down(), p)
            };
        }
    }
    if p == 0.0 {
        if (a > 0.0) == (b > 0.0) {
            (0.0, 0.0_f64.next_up())
        } else {
            (0.0_f64.next_down(), 0.0)
        }
    } else {
        (p.next_down(), p.next_up())
    }
}

/// Sterbenz-Boldo-Melquiond FMA kalan teoremi (`r = fma(-q, b, a)`) ile
/// IEEE Std 1788-2015 `tightest` yönlendirilmiş bölme `[fl_down(a / b), fl_up(a / b)]` hesaplar.
#[inline]
fn div_directed(a: f64, b: f64) -> (f64, f64) {
    if a == 0.0 {
        return (0.0, 0.0);
    }
    let q = a / b;
    if !q.is_finite() {
        return (q, q);
    }
    if q.abs() >= 1e-285 && a.abs() >= 1e-285 && (q * b).is_finite() {
        let r = (-q).mul_add(b, a);
        if r.is_finite() {
            return if r == 0.0 {
                (q, q)
            } else if (r > 0.0) == (b > 0.0) {
                (q, q.next_up())
            } else {
                (q.next_down(), q)
            };
        }
    }
    if q == 0.0 {
        if (a > 0.0) == (b > 0.0) {
            (0.0, 0.0_f64.next_up())
        } else {
            (0.0_f64.next_down(), 0.0)
        }
    } else {
        (q.next_down(), q.next_up())
    }
}

/// IEEE-754 FMA kalanı (`r = fma(-s, s, a)`) ile IEEE Std 1788-2015 `tightest`
/// yönlendirilmiş karekök `[fl_down(sqrt(a)), fl_up(sqrt(a))]` hesaplar.
#[inline]
fn sqrt_directed(a: f64) -> (f64, f64) {
    if a <= 0.0 {
        return (0.0, 0.0);
    }
    let s = a.sqrt();
    if a >= 1e-285 && s.is_finite() {
        let r = (-s).mul_add(s, a);
        if r.is_finite() {
            return if r == 0.0 {
                (s, s)
            } else if r > 0.0 {
                (s, s.next_up())
            } else {
                (s.next_down().max(0.0), s)
            };
        }
    }
    (s.next_down().max(0.0), s.next_up())
}

impl Interval64 {
    /// Doğrulanmış sonlu kapalı aralık [lo, hi] oluşturur (lo > hi, NaN veya ±Inf ise reddeder).
    pub fn new(lo: f64, hi: f64) -> Result<Self, String> {
        if !lo.is_finite() || !hi.is_finite() {
            return Err(format!("Interval64: Sınırlar sonlu reel sayı olmalıdır: [{}, {}]", lo, hi));
        }
        if lo > hi {
            return Err(format!("Interval64: Alt sınır ({}) > üst sınır ({})!", lo, hi));
        }
        Ok(Self { lo, hi })
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.lo.is_finite() && self.hi.is_finite()
    }

    /// Tekil (noktasal / dejenere) aralık [x, x] oluşturur.
    pub fn point(x: f64) -> Result<Self, String> {
        Self::new(x, x)
    }

    /// Tam rasyonel kesir (num / den) değerini tüm `i64` uzayı üzerinde (`|num| > 2^53` dahil)
    /// en dar IEEE-754 dışa yuvarlamalı [lo, hi] aralığı içine matematiksel kesinlikle hapseder.
    pub fn from_rational(num: i64, den: i64) -> Result<Self, String> {
        if den == 0 {
            return Err("Interval64::from_rational: Payda 0 olamaz!".to_string());
        }
        let (n128, d128): (i128, u128) = if den < 0 {
            (-i128::from(num), den.unsigned_abs().into())
        } else {
            (i128::from(num), den.unsigned_abs().into())
        };
        if n128 == 0 {
            return Self::point(0.0);
        }

        let q = (n128 as f64) / (d128 as f64);
        if !q.is_finite() {
            return Err("Interval64::from_rational: Sonlu olmayan bölüm!".to_string());
        }

        match cmp_f64_rational(q, n128, d128) {
            Ordering::Equal => Self::new(q, q),
            Ordering::Greater => {
                let hi = q;
                let mut lo = q.next_down();
                while lo.is_finite() && cmp_f64_rational(lo, n128, d128) == Ordering::Greater {
                    lo = lo.next_down();
                }
                Self::new(lo, hi)
            }
            Ordering::Less => {
                let lo = q;
                let mut hi = q.next_up();
                while hi.is_finite() && cmp_f64_rational(hi, n128, d128) == Ordering::Less {
                    hi = hi.next_up();
                }
                Self::new(lo, hi)
            }
        }
    }

    /// Merkez ve mutlak belirsizlik yarıçapından [c - r, c + r] Knuth TwoSum dışa yuvarlamalı aralık üretir.
    pub fn from_pm(center: f64, radius: f64) -> Result<Self, String> {
        if !center.is_finite() || !radius.is_finite() || radius < 0.0 {
            return Err(format!("Interval64::from_pm geçersiz parametre: {} ± {}", center, radius));
        }
        if radius == 0.0 {
            return Self::point(center);
        }
        let (lo, _) = add_directed(center, -radius);
        let (_, hi) = add_directed(center, radius);
        Self::new(lo, hi)
    }

    /// Merkez ve bağıl yüzde belirsizlikten (örn. pct = 0.01 -> ±%1) üstten yuvarlamalı yarıçapla aralık üretir.
    pub fn from_rel_uncertainty(center: f64, rel_fraction: f64) -> Result<Self, String> {
        if !center.is_finite() || !rel_fraction.is_finite() || rel_fraction < 0.0 {
            return Err("Bağıl belirsizlik sonlu ve negatif olmayan reel sayı olmalıdır!".to_string());
        }
        if rel_fraction == 0.0 || center == 0.0 {
            return Self::point(center);
        }
        let (_, radius_hi) = mul_directed(center.abs(), rel_fraction);
        if !radius_hi.is_finite() {
            return Err("Bağıl belirsizlik yarıçapı taştı!".to_string());
        }
        Self::from_pm(center, radius_hi)
    }

    #[inline]
    pub fn midpoint(&self) -> f64 {
        0.5 * self.lo + 0.5 * self.hi
    }

    #[inline]
    pub fn width(&self) -> f64 {
        if self.hi == self.lo {
            0.0
        } else {
            add_directed(self.hi, -self.lo).1.max(0.0)
        }
    }

    #[inline]
    pub fn radius(&self) -> f64 {
        0.5 * self.width()
    }

    #[inline]
    pub fn contains(&self, x: f64) -> bool {
        x >= self.lo && x <= self.hi
    }

    #[inline]
    pub fn contains_zero(&self) -> bool {
        self.lo <= 0.0 && self.hi >= 0.0
    }

    #[inline]
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.lo >= other.lo && self.hi <= other.hi
    }

    /// İki geçerli aralığın kesişimini alır (Mean-Value form daraltması için).
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let lo = self.lo.max(other.lo);
        let hi = self.hi.min(other.hi);
        if lo <= hi {
            Some(Self { lo, hi })
        } else {
            None
        }
    }

    /// IEEE Std 1788-2015 `tightest` dışa yuvarlamalı toplama (Knuth TwoSum EFT ile 0/1-ULP)
    pub fn add(&self, rhs: &Self) -> Self {
        let (lo, _) = add_directed(self.lo, rhs.lo);
        let (_, hi) = add_directed(self.hi, rhs.hi);
        Self { lo, hi }
    }

    /// IEEE Std 1788-2015 `tightest` dışa yuvarlamalı çıkarma: [a, b] - [c, d] = [a, b] + [-d, -c]
    pub fn sub(&self, rhs: &Self) -> Self {
        let (lo, _) = add_directed(self.lo, -rhs.hi);
        let (_, hi) = add_directed(self.hi, -rhs.lo);
        Self { lo, hi }
    }

    /// Tam karşıt işaret: -[a, b] = [-b, -a]
    pub fn neg(&self) -> Self {
        Self {
            lo: if self.hi == 0.0 { 0.0 } else { -self.hi },
            hi: if self.lo == 0.0 { 0.0 } else { -self.lo },
        }
    }

    /// IEEE Std 1788-2015 `tightest` dışa yuvarlamalı çarpma (FMA TwoProd EFT ile 0/1-ULP)
    pub fn mul(&self, rhs: &Self) -> Self {
        let (l1, h1) = mul_directed(self.lo, rhs.lo);
        let (l2, h2) = mul_directed(self.lo, rhs.hi);
        let (l3, h3) = mul_directed(self.hi, rhs.lo);
        let (l4, h4) = mul_directed(self.hi, rhs.hi);
        Self {
            lo: l1.min(l2).min(l3).min(l4),
            hi: h1.max(h2).max(h3).max(h4),
        }
    }

    /// IEEE Std 1788-2015 `tightest` dışa yuvarlamalı bölme (Payda aralığı 0 içeriyorsa kesin olarak reddeder!)
    pub fn div(&self, rhs: &Self) -> Result<Self, String> {
        if rhs.contains_zero() {
            return Err(format!(
                "Interval64 sıfıra bölme tehlikesi: Payda aralığı {} sıfır içeriyor!",
                rhs
            ));
        }
        let (l1, h1) = div_directed(self.lo, rhs.lo);
        let (l2, h2) = div_directed(self.lo, rhs.hi);
        let (l3, h3) = div_directed(self.hi, rhs.lo);
        let (l4, h4) = div_directed(self.hi, rhs.hi);
        let lo = l1.min(l2).min(l3).min(l4);
        let hi = h1.max(h2).max(h3).max(h4);
        Self::new(lo, hi)
    }

    /// Bağımlılık-duyarlı IEEE Std 1788-2015 `tightest` tam kare: 0 ∈ [lo, hi] ise alt sınır tam 0.0'dır!
    pub fn sqr(&self) -> Self {
        let (l_a, h_a) = mul_directed(self.lo, self.lo);
        let (l_b, h_b) = mul_directed(self.hi, self.hi);
        if self.contains_zero() {
            Self {
                lo: 0.0,
                hi: h_a.max(h_b),
            }
        } else {
            Self {
                lo: l_a.min(l_b).max(0.0),
                hi: h_a.max(h_b),
            }
        }
    }

    /// Tamsayı üs alma: Standart `f64::powi` 0.5-ULP doğru yuvarlama garantisi vermediğinden,
    /// dışa yuvarlamalı ikili aralık çarpımı (interval binary exponentiation) kullanılır;
    /// çift üslerde (n = 2k) önce bağımlılık-duyarlı `sqr()` ([a,b]^2 >= 0) uygulanır.
    pub fn powi(&self, n: i32) -> Result<Self, String> {
        if n == 0 {
            if self.contains_zero() {
                return Err(format!("Interval64 belirsiz biçim (0^0): taban aralığı {} sıfır içeriyor!", self));
            }
            return Self::point(1.0);
        }
        if n == 1 {
            return Self::new(self.lo, self.hi);
        }
        if n == 2 {
            let sq = self.sqr();
            return Self::new(sq.lo, sq.hi);
        }
        if n < 0 {
            let neg_n = n
                .checked_neg()
                .ok_or_else(|| "Interval64::powi üs taşması!".to_string())?;
            let pos = self.powi(neg_n)?;
            return Interval64::point(1.0)?.div(&pos);
        }
        if n % 2 == 0 {
            // Çift üs: x^(2k) = (x^2)^k, böylece 0 ∈ self ise alt sınır kesinlikle >= 0 kalır
            let half = self.sqr().powi(n / 2)?;
            Self::new(half.lo.max(0.0), half.hi)
        } else {
            // Tek üs: R üzerinde kesin monoton olduğu için uç noktaları ikili aralık çarpımıyla hesapla
            let lo_iv = Self::point(self.lo)?.powi_positive_binary(n as u32)?;
            let hi_iv = Self::point(self.hi)?.powi_positive_binary(n as u32)?;
            Self::new(lo_iv.lo, hi_iv.hi)
        }
    }

    fn powi_positive_binary(&self, mut exp: u32) -> Result<Self, String> {
        let mut base = *self;
        let mut acc = Self::point(1.0)?;
        while exp > 0 {
            if exp & 1 == 1 {
                acc = acc.mul(&base);
                if !acc.is_finite() {
                    return Err("Interval64::powi taşması (overflow)!".to_string());
                }
            }
            exp >>= 1;
            if exp > 0 {
                base = base.mul(&base);
                if !base.is_finite() {
                    return Err("Interval64::powi taban taşması (overflow)!".to_string());
                }
            }
        }
        Self::new(acc.lo, acc.hi)
    }

    /// Genel reel üs alma: [a, b]^[c, d] = exp([c, d] * ln([a, b])) veya TAM tamsayı / 0.5 üs.
    /// NOT: Yaklaşık tolerans (< 1e-12) ile tamsayıya yuvarlama YASAKTIR; yalnızca tam IEEE-754
    /// tamsayı (`p.fract() == 0.0`) veya tam `p == 0.5` durumlarında hızlı yol kullanılır.
    pub fn pow(&self, exp: &Self) -> Result<Self, String> {
        if exp.hi == exp.lo {
            let p = exp.lo;
            if p.fract() == 0.0 && p.abs() <= 1024.0 {
                return self.powi(p as i32);
            }
            if p == 0.5 {
                return self.sqrt();
            }
        }
        // Taban [0, b_hi] ve üs kesin pozitif (exp.lo > 0) ise b -> b^p kesin artandır (0^p = 0)
        if self.lo == 0.0 && exp.lo > 0.0 {
            if self.hi == 0.0 {
                return Self::point(0.0);
            }
            let hi_iv = Self::point(self.hi)?.ln()?.mul(exp).exp()?;
            return Self::new(0.0, hi_iv.hi);
        }
        let ln_base = self.ln()?;
        ln_base.mul(exp).exp()
    }

    /// IEEE Std 1788-2015 `tightest` dışa yuvarlamalı karekök (FMA kalanı `a - s^2` ile tam karelerde 0-ULP, diğerlerinde 1-ULP).
    pub fn sqrt(&self) -> Result<Self, String> {
        if self.lo < 0.0 {
            return Err(format!(
                "Interval64 karekök tanım kümesi ihlali: {} negatif değer içeriyor!",
                self
            ));
        }
        let (lo_sqrt, _) = sqrt_directed(self.lo);
        let (_, hi_sqrt) = sqrt_directed(self.hi);
        Self::new(lo_sqrt, hi_sqrt)
    }

    /// Dışa yuvarlamalı doğal logaritma (Standart libm 1..3.5 ULP sapmasını kapsamak için 4-ULP dışa adım; ln(1)=0 tam).
    pub fn ln(&self) -> Result<Self, String> {
        if self.lo <= 0.0 {
            return Err(format!(
                "Interval64 ln() tanım kümesi ihlali: {} pozitif olmayan değer içeriyor!",
                self
            ));
        }
        let lo = if self.lo == 1.0 { 0.0 } else { step_down_ulps(self.lo.ln(), 4) };
        let hi = if self.hi == 1.0 { 0.0 } else { step_up_ulps(self.hi.ln(), 4) };
        Self::new(lo, hi)
    }

    /// Dışa yuvarlamalı üstel fonksiyon exp([lo, hi]) (libm toleransı için 4-ULP dışa adım; exp(0)=1 tam).
    pub fn exp(&self) -> Result<Self, String> {
        let lo_exp = if self.lo == 0.0 { 1.0 } else { step_down_ulps(self.lo.exp(), 4).max(0.0) };
        let hi_exp = if self.hi == 0.0 { 1.0 } else { step_up_ulps(self.hi.exp(), 4) };
        if !hi_exp.is_finite() {
            return Err(format!("Interval64 exp({}) taşması (overflow)!", self));
        }
        Self::new(lo_exp, hi_exp)
    }
}

impl fmt::Display for Interval64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.6}, {:.6}]", self.lo, self.hi)
    }
}

/// İleri Mod Otomatik Türev (Forward-Mode Automatic Differentiation) Dual Sayısı:
/// D = x + dx·ε  (ε^2 = 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dual64 {
    pub val: f64,
    pub der: f64,
}

impl Dual64 {
    #[inline]
    pub const fn constant(val: f64) -> Self {
        Self { val, der: 0.0 }
    }

    #[inline]
    pub const fn active_variable(val: f64) -> Self {
        Self { val, der: 1.0 }
    }

    #[inline]
    pub const fn new(val: f64, der: f64) -> Self {
        Self { val, der }
    }

    #[inline]
    pub fn checked_div(self, rhs: Self) -> Result<Self, String> {
        if rhs.val == 0.0 {
            return Err("Dual64 sıfıra bölme hatası!".to_string());
        }
        let inv = 1.0 / rhs.val;
        Ok(Self {
            val: self.val * inv,
            der: (self.der * rhs.val - self.val * rhs.der) * (inv * inv),
        })
    }

    pub fn sqrt(self) -> Result<Self, String> {
        if self.val <= 0.0 {
            return Err(format!("Dual64 sqrt({}) türev tanım kümesi dışı!", self.val));
        }
        let s = self.val.sqrt();
        Ok(Self {
            val: s,
            der: self.der / (2.0 * s),
        })
    }

    pub fn ln(self) -> Result<Self, String> {
        if self.val <= 0.0 {
            return Err(format!("Dual64 ln({}) tanım kümesi dışı!", self.val));
        }
        Ok(Self {
            val: self.val.ln(),
            der: self.der / self.val,
        })
    }

    pub fn exp(self) -> Self {
        let e = self.val.exp();
        Self {
            val: e,
            der: self.der * e,
        }
    }

    pub fn pow(self, exp: Self) -> Result<Self, String> {
        if exp.der == 0.0 {
            let p = exp.val;
            if p.fract() == 0.0 && p.abs() <= 1024.0 {
                let n = p as i32;
                if n == 0 {
                    return Ok(Self::constant(1.0));
                }
                if self.val == 0.0 && n < 1 {
                    return Err("Dual64 0^n (n<=0) türev tekilliği!".to_string());
                }
                return Ok(Self {
                    val: self.val.powi(n),
                    der: (n as f64) * self.val.powi(n - 1) * self.der,
                });
            }
        }
        if self.val <= 0.0 {
            return Err(format!("Dual64 genel üs ({}^{}) için taban > 0 olmalıdır!", self.val, exp.val));
        }
        let v = self.val.powf(exp.val);
        let d = v * (exp.der * self.val.ln() + exp.val * self.der / self.val);
        Ok(Self { val: v, der: d })
    }
}

impl std::ops::Add for Dual64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val + rhs.val,
            der: self.der + rhs.der,
        }
    }
}

impl std::ops::Sub for Dual64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val - rhs.val,
            der: self.der - rhs.der,
        }
    }
}

impl std::ops::Neg for Dual64 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            val: -self.val,
            der: -self.der,
        }
    }
}

impl std::ops::Mul for Dual64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val * rhs.val,
            der: self.der * rhs.val + self.val * rhs.der,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Bağımsız Sonsuz-Hassasiyetli İkili Rasyonel (Exact Dyadic BigInt) Kahini
    // Her sonlu IEEE-754 f64 = m * 2^e (|m| < 2^53) olduğundan, +, -, *, /, sqrt
    // kapsama özelliği sıfır yuvarlama hatasıyla matematiksel olarak kanıtlanır.
    // =========================================================================
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct BigUint {
        limbs: Vec<u64>, // little-endian
    }

    impl BigUint {
        fn zero() -> Self { Self { limbs: Vec::new() } }
        fn from_u128(v: u128) -> Self {
            if v == 0 { return Self::zero(); }
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            if hi == 0 { Self { limbs: vec![lo] } } else { Self { limbs: vec![lo, hi] } }
        }
        fn is_zero(&self) -> bool { self.limbs.is_empty() }
        fn trim(&mut self) {
            while self.limbs.last() == Some(&0) { self.limbs.pop(); }
        }
        fn shl_bits(&self, bits: u32) -> Self {
            if self.is_zero() || bits == 0 { return self.clone(); }
            let word_shift = (bits / 64) as usize;
            let bit_shift = bits % 64;
            let mut out = vec![0u64; word_shift + self.limbs.len() + 1];
            let mut carry = 0u64;
            for (i, &w) in self.limbs.iter().enumerate() {
                if bit_shift == 0 {
                    out[i + word_shift] = w;
                } else {
                    out[i + word_shift] = (w << bit_shift) | carry;
                    carry = w >> (64 - bit_shift);
                }
            }
            if carry != 0 { out[word_shift + self.limbs.len()] = carry; }
            let mut res = Self { limbs: out };
            res.trim();
            res
        }
        fn cmp_mag(&self, other: &Self) -> Ordering {
            match self.limbs.len().cmp(&other.limbs.len()) {
                Ordering::Equal => {
                    for (&a, &b) in self.limbs.iter().rev().zip(other.limbs.iter().rev()) {
                        match a.cmp(&b) {
                            Ordering::Equal => continue,
                            non_eq => return non_eq,
                        }
                    }
                    Ordering::Equal
                }
                non_eq => non_eq,
            }
        }
        fn add_mag(&self, other: &Self) -> Self {
            let n = self.limbs.len().max(other.limbs.len());
            let mut out = Vec::with_capacity(n + 1);
            let mut carry = 0u128;
            for i in 0..n {
                let a = *self.limbs.get(i).unwrap_or(&0) as u128;
                let b = *other.limbs.get(i).unwrap_or(&0) as u128;
                let s = a + b + carry;
                out.push(s as u64);
                carry = s >> 64;
            }
            if carry != 0 { out.push(carry as u64); }
            let mut res = Self { limbs: out };
            res.trim();
            res
        }
        fn sub_mag(&self, other: &Self) -> Self {
            // Requires self >= other
            let mut out = Vec::with_capacity(self.limbs.len());
            let mut borrow = 0i128;
            for i in 0..self.limbs.len() {
                let a = self.limbs[i] as i128;
                let b = *other.limbs.get(i).unwrap_or(&0) as i128;
                let mut diff = a - b - borrow;
                if diff < 0 {
                    diff += 1i128 << 64;
                    borrow = 1;
                } else {
                    borrow = 0;
                }
                out.push(diff as u64);
            }
            let mut res = Self { limbs: out };
            res.trim();
            res
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct BigInt {
        neg: bool,
        mag: BigUint,
    }

    impl BigInt {
        fn from_i128(v: i128) -> Self {
            Self { neg: v < 0, mag: BigUint::from_u128(v.unsigned_abs()) }
        }
        fn shl_bits(&self, bits: u32) -> Self {
            if self.mag.is_zero() {
                Self { neg: false, mag: BigUint::zero() }
            } else {
                Self { neg: self.neg, mag: self.mag.shl_bits(bits) }
            }
        }
        fn add(&self, other: &Self) -> Self {
            if self.neg == other.neg {
                let mag = self.mag.add_mag(&other.mag);
                Self { neg: if mag.is_zero() { false } else { self.neg }, mag }
            } else {
                match self.mag.cmp_mag(&other.mag) {
                    Ordering::Equal => Self { neg: false, mag: BigUint::zero() },
                    Ordering::Greater => Self { neg: self.neg, mag: self.mag.sub_mag(&other.mag) },
                    Ordering::Less => Self { neg: other.neg, mag: other.mag.sub_mag(&self.mag) },
                }
            }
        }
        fn cmp_exact(&self, other: &Self) -> Ordering {
            if self.mag.is_zero() && other.mag.is_zero() { return Ordering::Equal; }
            match (self.neg, other.neg) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (false, false) => self.mag.cmp_mag(&other.mag),
                (true, true) => other.mag.cmp_mag(&self.mag),
            }
        }
    }

    /// Tam IEEE-754 f64 ayrıştırması: x == sig * 2^exp
    fn decompose_f64(x: f64) -> (i128, i32) {
        assert!(x.is_finite());
        if x == 0.0 { return (0, 0); }
        let bits = x.to_bits();
        let neg = (bits >> 63) != 0;
        let raw_exp = ((bits >> 52) & 0x7ff) as i32;
        let raw_frac = bits & 0x000f_ffff_ffff_ffff;
        let (mag, exp) = if raw_exp == 0 {
            (i128::from(raw_frac), -1074)
        } else {
            (i128::from((1u64 << 52) | raw_frac), raw_exp - 1075)
        };
        (if neg { -mag } else { mag }, exp)
    }

    fn cmp_dyadic(m1: i128, e1: i32, m2: i128, e2: i32) -> Ordering {
        if m1 == 0 && m2 == 0 { return Ordering::Equal; }
        let base_e = e1.min(e2);
        let b1 = BigInt::from_i128(m1).shl_bits((e1 - base_e) as u32);
        let b2 = BigInt::from_i128(m2).shl_bits((e2 - base_e) as u32);
        b1.cmp_exact(&b2)
    }

    fn exact_sum_in_interval(lo: f64, hi: f64, a: f64, b: f64) -> bool {
        let (ml, el) = decompose_f64(lo);
        let (mh, eh) = decompose_f64(hi);
        let (ma, ea) = decompose_f64(a);
        let (mb, eb) = decompose_f64(b);
        let base_e = el.min(eh).min(ea).min(eb);
        let bl = BigInt::from_i128(ml).shl_bits((el - base_e) as u32);
        let bh = BigInt::from_i128(mh).shl_bits((eh - base_e) as u32);
        let sum = BigInt::from_i128(ma)
            .shl_bits((ea - base_e) as u32)
            .add(&BigInt::from_i128(mb).shl_bits((eb - base_e) as u32));
        bl.cmp_exact(&sum) != Ordering::Greater && sum.cmp_exact(&bh) != Ordering::Greater
    }

    /// Deterministik Xorshift64 sözde-rastgele üreteç (sıfır dış bağımlılık, tam tekrarlanabilirlik)
    struct Rng(u64);
    impl Rng {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn next_f64_diverse(&mut self) -> f64 {
            // Normal, yakın-sıfır, büyük, küçük ve subnormal sayıları kapsayan karma dağılım
            let mode = self.next_u64() % 10;
            let sign = if (self.next_u64() & 1) == 0 { 0u64 } else { 1u64 << 63 };
            let frac = self.next_u64() & 0x000f_ffff_ffff_ffff;
            let exp: u64 = match mode {
                0 => 0,                                         // subnormal
                1 => 1 + (self.next_u64() % 20),                // ultra-küçük normal (~1e-305)
                2 => 1023 - 20 + (self.next_u64() % 41),        // [2^-20, 2^20]
                3 => 1023 - 5 + (self.next_u64() % 11),         // O(1) kritik çarpışma bölgesi
                4 => 2020 + (self.next_u64() % 25),             // büyük normal (~1e300..1e307)
                _ => 500 + (self.next_u64() % 1050),            // geniş dinamik aralık (~1e-150..1e160)
            };
            f64::from_bits(sign | (exp << 52) | frac)
        }
    }

    #[test]
    fn test_exact_dyadic_oracle_add_sub_10000_vectors() {
        let mut rng = Rng(0xC17A_DE10_0001_u64);
        let specials = [
            0.0, -0.0, 1.0, -1.0, f64::MIN_POSITIVE, -f64::MIN_POSITIVE,
            f64::from_bits(1), -f64::from_bits(1), 1e-300, -1e-300,
            1e307, -1e307, f64::MAX, -f64::MAX, 1.0 + f64::EPSILON, 1.0 - f64::EPSILON / 2.0,
        ];
        // Tüm özel çift kombinasyonları + 10.000 rastgele vektör
        for &a in &specials {
            for &b in &specials {
                if (a + b).is_finite() {
                    let iv = Interval64::point(a).unwrap().add(&Interval64::point(b).unwrap());
                    if iv.is_finite() {
                        assert!(exact_sum_in_interval(iv.lo, iv.hi, a, b), "ADD kapsama ihlali: {} + {} ∉ {:?}", a, b, iv);
                    } else {
                        // Gerçek toplam |a+b| > f64::MAX ise dışa yuvarlama +Inf/-Inf zorunludur (örn. f64::MAX + 1.0)
                        assert!(iv.hi == f64::INFINITY || iv.lo == f64::NEG_INFINITY);
                    }
                }
                if (a - b).is_finite() {
                    let iv = Interval64::point(a).unwrap().sub(&Interval64::point(b).unwrap());
                    if iv.is_finite() {
                        assert!(exact_sum_in_interval(iv.lo, iv.hi, a, -b), "SUB kapsama ihlali: {} - {} ∉ {:?}", a, b, iv);
                    } else {
                        assert!(iv.hi == f64::INFINITY || iv.lo == f64::NEG_INFINITY);
                    }
                }
            }
        }
        let mut checked = 0usize;
        for _ in 0..10_000 {
            let a = rng.next_f64_diverse();
            let b = rng.next_f64_diverse();
            if (a + b).is_finite() {
                let iv = Interval64::point(a).unwrap().add(&Interval64::point(b).unwrap());
                assert!(iv.is_finite() && iv.lo <= iv.hi);
                assert!(exact_sum_in_interval(iv.lo, iv.hi, a, b), "Rastgele ADD ihlali: {} + {} ∉ {:?}", a, b, iv);
                // Sıkılık (tightest): aralık genişliği en fazla 1 ULP olmalıdır
                assert!(iv.lo == iv.hi || iv.lo.next_up() == iv.hi, "ADD 1-ULP sıkılık ihlali: {:?}", iv);
                checked += 1;
            }
        }
        assert!(checked >= 9000);
    }

    #[test]
    fn test_exact_dyadic_oracle_mul_5000_vectors() {
        let mut rng = Rng(0xC17A_DE10_0002_u64);
        let mut checked = 0usize;
        for _ in 0..5_000 {
            let a = rng.next_f64_diverse();
            let b = rng.next_f64_diverse();
            let p = a * b;
            if !p.is_finite() { continue; }
            let iv = Interval64::point(a).unwrap().mul(&Interval64::point(b).unwrap());
            if !iv.is_finite() { continue; }
            let (ma, ea) = decompose_f64(a);
            let (mb, eb) = decompose_f64(b);
            let (ml, el) = decompose_f64(iv.lo);
            let (mh, eh) = decompose_f64(iv.hi);
            let mp = ma * mb; // |ma|,|mb| < 2^53 => |mp| < 2^106 < i128::MAX
            let ep = ea + eb;
            assert!(
                cmp_dyadic(ml, el, mp, ep) != Ordering::Greater
                    && cmp_dyadic(mp, ep, mh, eh) != Ordering::Greater,
                "MUL kapsama ihlali: {} * {} ∉ [{:e}, {:e}]",
                a, b, iv.lo, iv.hi
            );
            checked += 1;
        }
        assert!(checked >= 4000);
    }

    #[test]
    fn test_exact_dyadic_oracle_div_5000_vectors() {
        let mut rng = Rng(0xC17A_DE10_0003_u64);
        let mut checked = 0usize;
        for _ in 0..5_000 {
            let a = rng.next_f64_diverse();
            let b = rng.next_f64_diverse();
            if b == 0.0 || !(a / b).is_finite() { continue; }
            let Ok(iv) = Interval64::point(a).unwrap().div(&Interval64::point(b).unwrap()) else {
                continue;
            };
            let (ma, ea) = decompose_f64(a);
            let (mb, eb) = decompose_f64(b);
            let (ml, el) = decompose_f64(iv.lo);
            let (mh, eh) = decompose_f64(iv.hi);
            // b > 0 => lo * b <= a <= hi * b ; b < 0 => hi * b <= a <= lo * b
            let (c1, c2) = if b > 0.0 {
                (cmp_dyadic(ml * mb, el + eb, ma, ea), cmp_dyadic(ma, ea, mh * mb, eh + eb))
            } else {
                (cmp_dyadic(mh * mb, eh + eb, ma, ea), cmp_dyadic(ma, ea, ml * mb, el + eb))
            };
            assert!(
                c1 != Ordering::Greater && c2 != Ordering::Greater,
                "DIV kapsama ihlali: {} / {} ∉ [{:e}, {:e}]",
                a, b, iv.lo, iv.hi
            );
            checked += 1;
        }
        assert!(checked >= 4000);
    }

    #[test]
    fn test_exact_dyadic_oracle_sqrt_5000_vectors() {
        let mut rng = Rng(0xC17A_DE10_0004_u64);
        let specials = [0.0, f64::from_bits(1), f64::MIN_POSITIVE, 0.25, 1.0, 2.0, 3.0, 4.0, 9.0, 1e-300, 1e300, f64::MAX];
        for &a in &specials {
            let iv = Interval64::point(a).unwrap().sqrt().unwrap();
            let (ma, ea) = decompose_f64(a);
            let (ml, el) = decompose_f64(iv.lo);
            let (mh, eh) = decompose_f64(iv.hi);
            assert!(iv.lo >= 0.0);
            assert!(cmp_dyadic(ml * ml, 2 * el, ma, ea) != Ordering::Greater);
            assert!(cmp_dyadic(ma, ea, mh * mh, 2 * eh) != Ordering::Greater);
        }
        for _ in 0..5_000 {
            let a = rng.next_f64_diverse().abs();
            let iv = Interval64::point(a).unwrap().sqrt().unwrap();
            let (ma, ea) = decompose_f64(a);
            let (ml, el) = decompose_f64(iv.lo);
            let (mh, eh) = decompose_f64(iv.hi);
            assert!(
                cmp_dyadic(ml * ml, 2 * el, ma, ea) != Ordering::Greater
                    && cmp_dyadic(ma, ea, mh * mh, 2 * eh) != Ordering::Greater,
                "SQRT kapsama ihlali: sqrt({:e}) ∉ [{:e}, {:e}]",
                a, iv.lo, iv.hi
            );
        }
    }

    #[test]
    fn test_from_rational_exact_i64_extremes() {
        let cases = [
            (1i64, 3i64), (-7, 11), (0, 5), (355, 113),
            (i64::MAX, 3), (i64::MIN + 1, 7), (1, i64::MAX), (-1, i64::MAX),
            (i64::MAX, i64::MAX - 1), (10_000_000_000_000_003, 1),
        ];
        for (n, d) in cases {
            let iv = Interval64::from_rational(n, d).unwrap();
            assert!(cmp_f64_rational(iv.lo, i128::from(n), d.unsigned_abs().into()) != Ordering::Greater);
            assert!(cmp_f64_rational(iv.hi, i128::from(n), d.unsigned_abs().into()) != Ordering::Less);
            assert!(iv.lo == iv.hi || iv.lo.next_up() == iv.hi, "from_rational({}/{}) 1-ULP sıkılık ihlali", n, d);
        }
        assert!(Interval64::from_rational(5, 0).is_err());
    }

    #[test]
    fn test_pow_no_spurious_integer_snapping_regression() {
        // Regresyon testi: p = 2.0 + 5e-13 kesinlikle powi(2)'ye yuvarlanmamalıdır!
        let p_near_2 = 2.0_f64 + 5.0e-13;
        let iv = Interval64::point(1000.0).unwrap().pow(&Interval64::point(p_near_2).unwrap()).unwrap();
        // 1000^(2 + 5e-13) = 1e6 * exp(5e-13 * ln(1000)) ≈ 1_000_000.0000034538776
        let true_val = 1_000_000.0 * (5.0e-13 * 1000.0_f64.ln()).exp();
        assert!(
            iv.contains(true_val),
            "pow(1000, 2+5e-13) gerçek değeri ({:.10}) kapsamadı: {:?}",
            true_val, iv
        );
        assert!(iv.lo > 1_000_000.0, "pow(1000, 2+5e-13) 1e6'ya yapışmamalı!");

        let p_near_half = 0.5_f64 + 5.0e-13;
        let iv_h = Interval64::point(1000.0).unwrap().pow(&Interval64::point(p_near_half).unwrap()).unwrap();
        let true_h = 1000.0_f64.sqrt() * (5.0e-13 * 1000.0_f64.ln()).exp();
        assert!(iv_h.contains(true_h), "pow(1000, 0.5+5e-13) gerçek değeri kapsamadı: {:?}", iv_h);
    }

    #[test]
    fn test_add_directed_f64_max_no_spurious_inf_regression() {
        let a = Interval64::point(f64::MAX).unwrap();
        let res_zero = a.add(&Interval64::point(0.0).unwrap());
        assert_eq!(res_zero, a);
        let res_neg1 = a.add(&Interval64::point(-1.0).unwrap());
        assert!(res_neg1.is_finite(), "f64::MAX + (-1.0) sahte +Inf üretmemeli: {:?}", res_neg1);
        assert!(exact_sum_in_interval(res_neg1.lo, res_neg1.hi, f64::MAX, -1.0));
    }

    #[test]
    fn test_from_rel_uncertainty_subnormal_and_pm_soundness() {
        let iv = Interval64::from_rel_uncertainty(f64::MIN_POSITIVE, 1e-10).unwrap();
        assert!(iv.lo < f64::MIN_POSITIVE && iv.hi > f64::MIN_POSITIVE, "Subnormal bağıl belirsizlik sıfırlanmamalı: {:?}", iv);
        let pm = Interval64::from_pm(10.0, 0.1).unwrap();
        assert!(exact_sum_in_interval(pm.lo, pm.hi, 10.0, -0.1));
        assert!(exact_sum_in_interval(pm.lo, pm.hi, 10.0, 0.1));
        assert!(Interval64::from_pm(1.0, -0.1).is_err());
        assert!(Interval64::from_rel_uncertainty(1.0, -0.01).is_err());
    }

    #[test]
    fn test_interval_domain_rejections_and_sqr_dependency() {
        let across_zero = Interval64::new(-3.0, 2.0).unwrap();
        assert_eq!(across_zero.sqr(), Interval64 { lo: 0.0, hi: 9.0 });
        assert!(Interval64::point(1.0).unwrap().div(&across_zero).is_err());
        assert!(across_zero.sqrt().is_err());
        assert!(Interval64::new(0.0, 2.0).unwrap().ln().is_err());
        assert!(across_zero.powi(0).is_err());
        assert!(Interval64::new(5.0, 2.0).is_err());
        assert!(Interval64::new(f64::NAN, 1.0).is_err());
    }

    #[test]
    fn test_transcendental_exp_ln_identities_and_bounds() {
        assert_eq!(Interval64::point(0.0).unwrap().exp().unwrap(), Interval64 { lo: 1.0, hi: 1.0 });
        assert_eq!(Interval64::point(1.0).unwrap().ln().unwrap(), Interval64 { lo: 0.0, hi: 0.0 });
        assert!(Interval64::point(1.0).unwrap().exp().unwrap().contains(std::f64::consts::E));
        assert!(Interval64::point(2.0).unwrap().ln().unwrap().contains(std::f64::consts::LN_2));
        for k in -200..=200 {
            let x = 10.0_f64.powi(k);
            let round = Interval64::point(x).unwrap().ln().unwrap().exp().unwrap();
            assert!(round.contains(x), "exp(ln({:e})) kapsaması kırıldı: {:?}", x, round);
        }
    }

    #[test]
    fn test_dual64_chain_rule_and_non_integer_pow() {
        // f(x) = exp(sqrt(x)) / ln(x) at x = 4.0
        let x = Dual64::active_variable(4.0);
        let res = x.sqrt().unwrap().exp().checked_div(x.ln().unwrap()).unwrap();
        let e2 = 2.0_f64.exp();
        let ln4 = 4.0_f64.ln();
        let expected_der = (e2 * 0.25 * ln4 - e2 * 0.25) / (ln4 * ln4);
        assert!((res.der - expected_der).abs() < 1e-13);

        // Non-integer near-2 exponent should NOT snap to 2!
        let p = Dual64::constant(2.0 + 5.0e-13);
        let d_pow = Dual64::active_variable(1000.0).pow(p).unwrap();
        let expected_val = 1000.0_f64.powf(2.0 + 5.0e-13);
        assert!((d_pow.val - expected_val).abs() / expected_val < 1e-14);
        assert!(Dual64::active_variable(0.0).sqrt().is_err());
        assert!(Dual64::active_variable(1.0).checked_div(Dual64::constant(0.0)).is_err());
    }
}
