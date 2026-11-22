// Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
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

    /// Merkez ve mutlak belirsizlik yarıçapından [c - r, c + r] dışa yuvarlamalı aralık üretir.
    pub fn from_pm(center: f64, radius: f64) -> Result<Self, String> {
        if !center.is_finite() || !radius.is_finite() || radius < 0.0 {
            return Err(format!("Interval64::from_pm geçersiz parametre: {} ± {}", center, radius));
        }
        if radius == 0.0 {
            return Self::point(center);
        }
        Self::new((center - radius).next_down(), (center + radius).next_up())
    }

    /// Merkez ve bağıl yüzde belirsizlikten (örn. pct = 0.01 -> ±%1) aralık üretir.
    pub fn from_rel_uncertainty(center: f64, rel_fraction: f64) -> Result<Self, String> {
        if rel_fraction < 0.0 || rel_fraction.is_nan() {
            return Err("Bağıl belirsizlik negatif olamaz!".to_string());
        }
        let radius = center.abs() * rel_fraction;
        Self::from_pm(center, radius)
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

    /// Genel reel üs alma: [a, b]^[c, d] = exp([c, d] * ln([a, b])) veya sabit tamsayı/yarı-tamsayı
    pub fn pow(&self, exp: &Self) -> Result<Self, String> {
        // Üs noktasal bir tamsayı ise powi kullan (negatif tabanlara ve 0 içeren aralıklara izin verir)
        if (exp.hi - exp.lo).abs() == 0.0 {
            let p = exp.lo;
            if (p - p.round()).abs() < 1e-12 && p.abs() <= 64.0 {
                return self.powi(p.round() as i32);
            }
            if (p - 0.5).abs() < 1e-12 {
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

    /// Dışa yuvarlamalı doğal logaritma (Standart libm 1..3.5 ULP sapmasını kapsamak için 4-ULP dışa adım).
    pub fn ln(&self) -> Result<Self, String> {
        if self.lo <= 0.0 {
            return Err(format!(
                "Interval64 ln() tanım kümesi ihlali: {} pozitif olmayan değer içeriyor!",
                self
            ));
        }
        Self::new(step_down_ulps(self.lo.ln(), 4), step_up_ulps(self.hi.ln(), 4))
    }

    /// Dışa yuvarlamalı üstel fonksiyon exp([lo, hi]) (libm toleransı için 4-ULP dışa adım).
    pub fn exp(&self) -> Result<Self, String> {
        let lo_exp = step_down_ulps(self.lo.exp(), 4).max(0.0);
        let hi_exp = step_up_ulps(self.hi.exp(), 4);
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
            if (p - p.round()).abs() < 1e-12 {
                let n = p.round() as i32;
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
