// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — İleri Astrodinamik, J2 Geopotansiyel Yörünge Modeli ve Lambert Hedefleme Çözücüsü
// (Astrodynamics, WGS84/EGM96 J2 Secular Perturbations & Universal Variable Lambert Solver)

#![forbid(unsafe_code)]

use crate::constants::PhysicalConstants;
use std::f64::consts::PI;

/// 3-Boyutlu Öklid Vektörü (m veya m/s)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[inline(always)]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline(always)]
    pub fn norm_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline(always)]
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    #[inline(always)]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline(always)]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline(always)]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    #[inline(always)]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn unit(&self) -> Result<Self, String> {
        let n = self.norm();
        if n <= 1e-15 || !n.is_finite() {
            return Err("Sıfır veya tanımsız normlu vektör birimleştirilemez!".to_string());
        }
        Ok(self.scale(1.0 / n))
    }
}

/// WGS84 / EGM96 Jeodezik ve İkinci Derece Zonel Harmonik (J2) Yerçekimi Modeli
pub struct J2OrbitalModel;

impl J2OrbitalModel {
    /// WGS84 Ekvator Yarıçapı (m)
    pub const WGS84_R_EARTH: f64 = 6_378_137.0;
    /// WGS84 / EGM96 J2 Zonel Harmonik Katsayısı (boyutsuz)
    pub const WGS84_J2: f64 = 1.082_626_68e-3;
    /// Standart Dünya Gravitasyonel Parametresi GM (m^3/s^2)
    pub const WGS84_MU: f64 = PhysicalConstants::MU_EARTH;
    /// Güneş-Eşzamanlı Yörünge (SSO) İstenen RAAN Kayma Hızı (rad/s) — 360° / 365.2422 gün
    pub const SSO_RAAN_RATE_RAD_S: f64 = 1.991_063_853e-7;
    /// Donmuş Enberi Kritik Eğim Açısı (Molniya dω/dt = 0): arccos(1/sqrt(5)) ≈ 63.4349°
    pub const CRITICAL_INCLINATION_RAD: f64 = 1.107_148_717_794_090_5; // 63.4349488°

    /// J2 harmonik bozulmasından kaynaklanan net pertürbasyon ivmesi:
    /// a_J2 = - (3/2) * J2 * (mu * R_E^2 / r^5) * [ x*(1 - 5*z^2/r^2), y*(1 - 5*z^2/r^2), z*(3 - 5*z^2/r^2) ]
    pub fn acceleration_j2(r: Vector3) -> Result<Vector3, String> {
        let r2 = r.norm_sq();
        let r_mag = r2.sqrt();
        if r_mag < Self::WGS84_R_EARTH * 0.90 || !r_mag.is_finite() {
            return Err(format!("Geçersiz Dünya merkezli konum yarıçapı: {} m", r_mag));
        }

        let z2 = r.z * r.z;
        let z2_over_r2 = z2 / r2;
        let factor = -1.5 * Self::WGS84_J2 * Self::WGS84_MU * Self::WGS84_R_EARTH.powi(2) / (r2 * r2 * r_mag);

        let ax = factor * r.x * (1.0 - 5.0 * z2_over_r2);
        let ay = factor * r.y * (1.0 - 5.0 * z2_over_r2);
        let az = factor * r.z * (3.0 - 5.0 * z2_over_r2);

        Ok(Vector3::new(ax, ay, az))
    }

    /// Küresel Newton yerçekimi + J2 pertürbasyonu toplam ivmesi:
    pub fn total_gravitational_acceleration(r: Vector3) -> Result<Vector3, String> {
        let r_mag = r.norm();
        if r_mag < 1.0 {
            return Err("Tekillik: r=0 konumunda yerçekimi tanımsız!".to_string());
        }
        let a_newton = r.scale(-Self::WGS84_MU / (r_mag * r_mag * r_mag));
        let a_j2 = Self::acceleration_j2(r)?;
        Ok(a_newton.add(&a_j2))
    }

    /// Yörünge ortalama açısal hızı (Mean Motion): n = sqrt(mu / a^3) [rad/s]
    pub fn mean_motion_rad_s(semi_major_axis_m: f64) -> Result<f64, String> {
        if semi_major_axis_m <= 0.0 || !semi_major_axis_m.is_finite() {
            return Err("Yarı-büyük eksen a > 0 olmalıdır!".to_string());
        }
        Ok((Self::WGS84_MU / semi_major_axis_m.powi(3)).sqrt())
    }

    /// Kepleryen Yörünge Periyodu: T = 2 * pi * sqrt(a^3 / mu) [s]
    pub fn orbital_period_s(semi_major_axis_m: f64) -> Result<f64, String> {
        let n = Self::mean_motion_rad_s(semi_major_axis_m)?;
        Ok(2.0 * PI / n)
    }

    /// J2 Etkisiyle Düğüm Çizgisi (RAAN) Presesyon Hızı:
    /// dOmega/dt = - (3/2) * J2 * (R_E / p)^2 * n * cos(i)  [rad/s]
    /// (Burada p = a * (1 - e^2) semi-latus rectum parametresidir)
    pub fn raan_precession_rate_rad_s(
        semi_major_axis_m: f64,
        eccentricity: f64,
        inclination_rad: f64,
    ) -> Result<f64, String> {
        if !(0.0..1.0).contains(&eccentricity) {
            return Err(format!("Kapalı yörünge dışmerkezliği 0 <= e < 1 olmalıdır: e={}", eccentricity));
        }
        let p = semi_major_axis_m * (1.0 - eccentricity * eccentricity);
        if p <= Self::WGS84_R_EARTH {
            return Err(format!("Yörünge parametresi p yer kabuğu içinde: {} m", p));
        }
        let n = Self::mean_motion_rad_s(semi_major_axis_m)?;
        let r_over_p = Self::WGS84_R_EARTH / p;
        let rate = -1.5 * Self::WGS84_J2 * r_over_p * r_over_p * n * inclination_rad.cos();
        Ok(rate)
    }

    /// J2 Etkisiyle Enberi Argümanı (Argument of Perigee) Presesyon Hızı:
    /// domega/dt = (3/4) * J2 * (R_E / p)^2 * n * (5 * cos^2(i) - 1)  [rad/s]
    pub fn perigee_precession_rate_rad_s(
        semi_major_axis_m: f64,
        eccentricity: f64,
        inclination_rad: f64,
    ) -> Result<f64, String> {
        if !(0.0..1.0).contains(&eccentricity) {
            return Err(format!("Kapalı yörünge dışmerkezliği 0 <= e < 1 olmalıdır: e={}", eccentricity));
        }
        let p = semi_major_axis_m * (1.0 - eccentricity * eccentricity);
        if p <= Self::WGS84_R_EARTH {
            return Err(format!("Yörünge parametresi p yer kabuğu içinde: {} m", p));
        }
        let n = Self::mean_motion_rad_s(semi_major_axis_m)?;
        let r_over_p = Self::WGS84_R_EARTH / p;
        let cos_i = inclination_rad.cos();
        let rate = 0.75 * Self::WGS84_J2 * r_over_p * r_over_p * n * (5.0 * cos_i * cos_i - 1.0);
        Ok(rate)
    }

    /// Verilen yarı-büyük eksen ve dışmerkezlik için Güneş-Eşzamanlı (Sun-Synchronous) Yörünge eğim açısı:
    /// cos(i) = - (2/3) * (dOmega_SSO / J2) * (p / R_E)^2 * (1 / n)
    pub fn sun_synchronous_inclination_rad(
        semi_major_axis_m: f64,
        eccentricity: f64,
    ) -> Result<f64, String> {
        if !(0.0..1.0).contains(&eccentricity) {
            return Err("Dışmerkezlik 0 <= e < 1 olmalıdır!".to_string());
        }
        let p = semi_major_axis_m * (1.0 - eccentricity * eccentricity);
        let n = Self::mean_motion_rad_s(semi_major_axis_m)?;
        let p_over_r = p / Self::WGS84_R_EARTH;

        let cos_i = - (2.0 / 3.0) * (Self::SSO_RAAN_RATE_RAD_S / (Self::WGS84_J2 * n)) * (p_over_r * p_over_r);
        if !(-1.0..=1.0).contains(&cos_i) {
            return Err(format!(
                "Bu irtifada ({:.1} km) Güneş-Eşzamanlı yörünge fiziksel olarak imkânsız: cos(i) = {:.4}",
                (semi_major_axis_m - Self::WGS84_R_EARTH) / 1000.0,
                cos_i
            ));
        }
        Ok(cos_i.acos())
    }

    /// Verilen eğim açısının kritik Molniya enberi dondurma açısı (63.4349° veya 116.5651°) olup olmadığını denetler.
    pub fn is_critical_inclination(inclination_rad: f64, tol_deg: f64) -> bool {
        let tol_rad = tol_deg.to_radians();
        let i1 = Self::CRITICAL_INCLINATION_RAD;
        let i2 = PI - Self::CRITICAL_INCLINATION_RAD;
        (inclination_rad - i1).abs() < tol_rad || (inclination_rad - i2).abs() < tol_rad
    }
}

/// Evrensel Değişkenli (Universal Variable) Lambert Yörünge Transfer Sonucu
#[derive(Debug, Clone, PartialEq)]
pub struct LambertResult {
    /// Kalkış anındaki başlangıç hız vektörü v1 (m/s)
    pub v1: Vector3,
    /// Varış anındaki nihai hız vektörü v2 (m/s)
    pub v2: Vector3,
    /// Transfer yörüngesinin yarı-büyük ekseni a (m)
    pub semi_major_axis_m: f64,
    /// Transfer yörüngesinin dışmerkezliği e
    pub eccentricity: f64,
    /// İki konum vektörü arasındaki transfer açısı dnu (rad)
    pub transfer_angle_rad: f64,
    /// Karakteristik kalkış enerjisi C3 = v_inf^2 (m^2/s^2)
    pub c3_energy_m2_s2: f64,
}

impl LambertResult {
    /// Dairesel başlangıç yörüngesinden (hız v_circ1 = sqrt(mu / r1)) ayrılmak için gereken Delta-V (m/s)
    pub fn delta_v1_from_circular(&self, r1_vec: Vector3, mu: f64) -> f64 {
        let r1 = r1_vec.norm();
        let v_circ = (mu / r1).sqrt();
        // Teğetsel dairesel hız yönü: r x (r x v) düzleminde
        let h_dir = r1_vec.cross(&self.v1);
        let h_unit = match h_dir.unit() {
            Ok(u) => u,
            Err(_) => return (self.v1.norm() - v_circ).abs(),
        };
        let v_circ_dir = h_unit.cross(&r1_vec).scale(1.0 / r1);
        let v_circ_vec = v_circ_dir.scale(v_circ);
        self.v1.sub(&v_circ_vec).norm()
    }

    /// Dairesel hedef yörüngeye (hız v_circ2 = sqrt(mu / r2)) girmek için gereken Delta-V (m/s)
    pub fn delta_v2_to_circular(&self, r2_vec: Vector3, mu: f64) -> f64 {
        let r2 = r2_vec.norm();
        let v_circ = (mu / r2).sqrt();
        let h_dir = r2_vec.cross(&self.v2);
        let h_unit = match h_dir.unit() {
            Ok(u) => u,
            Err(_) => return (self.v2.norm() - v_circ).abs(),
        };
        let v_circ_dir = h_unit.cross(&r2_vec).scale(1.0 / r2);
        let v_circ_vec = v_circ_dir.scale(v_circ);
        self.v2.sub(&v_circ_vec).norm()
    }
}

/// Evrensel Değişkenli (Universal Variable / Bate-Mueller-White & Prussing) Lambert Çözücüsü
pub struct LambertSolver;

impl LambertSolver {
    /// Stumpff C(z) fonksiyonu: C(z) = (1 - cos(sqrt(z))) / z (z > 0), (cosh(sqrt(-z)) - 1) / (-z) (z < 0).
    /// Yakın-parabolik bölgede (|z| <= 0.1) yıkıcı sadeleşmeyi (catastrophic cancellation) önlemek için
    /// 7-terimli tam Taylor serisi kullanılır (kesme hatası < 5e-21 << f64::EPSILON).
    pub fn stumpff_c(z: f64) -> f64 {
        if z > 0.1 {
            let sz = z.sqrt();
            (1.0 - sz.cos()) / z
        } else if z < -0.1 {
            let sz = (-z).sqrt();
            (sz.cosh() - 1.0) / (-z)
        } else {
            // C(z) = 1/2! - z/4! + z^2/6! - z^3/8! + z^4/10! - z^5/12! + z^6/14!
            let z2 = z * z;
            let z3 = z2 * z;
            let z4 = z2 * z2;
            let z5 = z4 * z;
            let z6 = z3 * z3;
            0.5 - z / 24.0 + z2 / 720.0 - z3 / 40_320.0 + z4 / 3_628_800.0
                - z5 / 479_001_600.0
                + z6 / 87_178_291_200.0
        }
    }

    /// Stumpff S(z) fonksiyonu: S(z) = (sqrt(z) - sin(sqrt(z))) / z^1.5 (z > 0).
    /// |z| <= 0.1 bölgesinde 7-terimli Taylor serisi ile tam 16 basamaklı hassasiyet korunur.
    pub fn stumpff_s(z: f64) -> f64 {
        if z > 0.1 {
            let sz = z.sqrt();
            (sz - sz.sin()) / (sz * sz * sz)
        } else if z < -0.1 {
            let sz = (-z).sqrt();
            (sz.sinh() - sz) / (sz * sz * sz)
        } else {
            // S(z) = 1/3! - z/5! + z^2/7! - z^3/9! + z^4/11! - z^5/13! + z^6/15!
            let z2 = z * z;
            let z3 = z2 * z;
            let z4 = z2 * z2;
            let z5 = z4 * z;
            let z6 = z3 * z3;
            1.0 / 6.0 - z / 120.0 + z2 / 5_040.0 - z3 / 362_880.0 + z4 / 39_916_800.0
                - z5 / 6_227_020_800.0
                + z6 / 1_307_674_368_000.0
        }
    }

    /// Stumpff C'(z) türevi: dC/dz = (1 - z*S(z) - 2*C(z)) / (2*z).
    /// |z| <= 0.1 bölgesinde tam türetilmiş Taylor serisi sıfır sadeleşme kaybıyla çalışır.
    pub fn stumpff_c_prime(z: f64) -> f64 {
        if z.abs() > 0.1 {
            (1.0 - z * Self::stumpff_s(z) - 2.0 * Self::stumpff_c(z)) / (2.0 * z)
        } else {
            let z2 = z * z;
            let z3 = z2 * z;
            let z4 = z2 * z2;
            let z5 = z4 * z;
            -1.0 / 24.0 + z / 360.0 - z2 / 13_440.0 + z3 / 907_200.0 - z4 / 95_800_320.0
                + z5 / 14_529_715_200.0
        }
    }

    /// Stumpff S'(z) türevi: dS/dz = (C(z) - 3*S(z)) / (2*z).
    /// |z| <= 0.1 bölgesinde tam türetilmiş Taylor serisi sıfır sadeleşme kaybıyla çalışır.
    pub fn stumpff_s_prime(z: f64) -> f64 {
        if z.abs() > 0.1 {
            (Self::stumpff_c(z) - 3.0 * Self::stumpff_s(z)) / (2.0 * z)
        } else {
            let z2 = z * z;
            let z3 = z2 * z;
            let z4 = z2 * z2;
            let z5 = z4 * z;
            -1.0 / 120.0 + z / 2_520.0 - z2 / 120_960.0 + z3 / 9_979_200.0 - z4 / 1_245_404_160.0
                + z5 / 217_945_728_000.0
        }
    }

    /// İki konum vektörü r1, r2 ve uçuş süresi tof_s verildiğinde Kepleryen transfer hız vektörlerini çözer.
    /// - `prograde`: true ise dnu < 180° doğrudan transfer, false ise ters yönlü transfer.
    /// - `mu`: merkezi cismin gravitasyonel parametresi (Dünya için `PhysicalConstants::MU_EARTH`).
    pub fn solve(
        r1_vec: Vector3,
        r2_vec: Vector3,
        tof_s: f64,
        prograde: bool,
        mu: f64,
    ) -> Result<LambertResult, String> {
        let r1 = r1_vec.norm();
        let r2 = r2_vec.norm();
        if r1 < 1e3 || r2 < 1e3 || !r1.is_finite() || !r2.is_finite() {
            return Err("Konum vektörleri yarıçapı geçersiz veya aşırı küçük!".to_string());
        }
        if tof_s <= 0.0 || !tof_s.is_finite() {
            return Err("Uçuş süresi pozitif olmalıdır!".to_string());
        }
        if mu <= 0.0 {
            return Err("Gravitasyonel parametre mu > 0 olmalıdır!".to_string());
        }

        let cos_dnu = (r1_vec.dot(&r2_vec) / (r1 * r2)).clamp(-1.0, 1.0);
        let cross = r1_vec.cross(&r2_vec);
        let mut dnu = cos_dnu.acos();

        // Prograde / Retrograde yön denetimi:
        if prograde {
            if cross.z < 0.0 {
                dnu = 2.0 * PI - dnu;
            }
        } else if cross.z >= 0.0 {
            dnu = 2.0 * PI - dnu;
        }

        if (dnu - PI).abs() < 1e-7 {
            return Err("180° tekil transfer düzlemi (collinear r1 ve r2): Transfer düzlemi belirsizdir!".to_string());
        }

        let a_geom = dnu.sin() * (r1 * r2 / (1.0 - cos_dnu)).sqrt();
        if a_geom == 0.0 {
            return Err("Lambert geometrik katsayısı A sıfır olamaz!".to_string());
        }

        // Başlangıç aralığı ve Newton-Raphson kök bulma:
        let mut z = 0.0_f64;
        let mut z_low = -4.0 * PI * PI;
        let mut z_up = 4.0 * PI * PI;
        let max_iter = 120;
        let tol = 1e-8;
        let mut converged = false;

        for _ in 0..max_iter {
            let c = Self::stumpff_c(z);
            let s = Self::stumpff_s(z);
            let sqrt_c = c.sqrt();

            let y = r1 + r2 + a_geom * (z * s - 1.0) / sqrt_c;
            if a_geom > 0.0 && y < 0.0 {
                // Fiziksel olmayan bölge: z'yi sınırla ve ikiye böl
                z_low = z;
                z = 0.5 * (z + z_up);
                continue;
            }

            let y_safe = y.max(1e-12);
            let sqrt_y = y_safe.sqrt();
            let x = (y_safe / c).sqrt();
            let t = (x.powi(3) * s + a_geom * sqrt_y) / mu.sqrt();

            let f_diff = t - tof_s;
            if f_diff.abs() < tol {
                converged = true;
                break;
            }

            if f_diff <= 0.0 {
                z_low = z;
            } else {
                z_up = z;
            }

            // Bate-Mueller-White türevi: dt/dz
            let dtdz = if z.abs() > 1e-5 {
                let term1 = x.powi(3) * (Self::stumpff_s_prime(z) - 1.5 * s * Self::stumpff_c_prime(z) / c);
                let term2 = 0.125 * a_geom * (3.0 * s * sqrt_y / c + a_geom / x);
                (term1 + term2) / mu.sqrt()
            } else {
                let term1 = (2.0_f64.sqrt() / 40.0) * y_safe.powf(1.5);
                let term2 = 0.125 * a_geom * (sqrt_y + a_geom * (0.5 / y_safe).sqrt());
                (term1 + term2) / mu.sqrt()
            };

            let z_new = z - f_diff / dtdz;
            if z_new <= z_low || z_new >= z_up || !z_new.is_finite() {
                // Bisection koruması: Newton adımı aralık dışına çıkarsa bisection'a dön
                z = 0.5 * (z_low + z_up);
            } else {
                z = z_new;
            }
        }

        if !converged {
            return Err("Lambert evrensel değişkenli kök bulucu belirtilen toleransta yakınsamadı!".to_string());
        }

        // Lagrange f ve g katsayıları ile hız vektörleri hesabı:
        let c = Self::stumpff_c(z);
        let s = Self::stumpff_s(z);
        let y = (r1 + r2 + a_geom * (z * s - 1.0) / c.sqrt()).max(1e-12);

        let f_coeff = 1.0 - y / r1;
        let g_coeff = a_geom * (y / mu).sqrt();
        let g_dot = 1.0 - y / r2;

        if g_coeff.abs() <= 1e-12 {
            return Err("Lagrange g katsayısı tekillik noktasında!".to_string());
        }

        let v1 = Vector3::new(
            (r2_vec.x - f_coeff * r1_vec.x) / g_coeff,
            (r2_vec.y - f_coeff * r1_vec.y) / g_coeff,
            (r2_vec.z - f_coeff * r1_vec.z) / g_coeff,
        );

        let v2 = Vector3::new(
            (g_dot * r2_vec.x - r1_vec.x) / g_coeff,
            (g_dot * r2_vec.y - r1_vec.y) / g_coeff,
            (g_dot * r2_vec.z - r1_vec.z) / g_coeff,
        );

        // Yörünge elemanları hesabı:
        let v1_sq = v1.norm_sq();
        let eps = 0.5 * v1_sq - mu / r1;
        let semi_major_axis_m = if eps.abs() > 1e-12 {
            -mu / (2.0 * eps)
        } else {
            f64::INFINITY
        };

        // Açısal momentum ve dışmerkezlik:
        let h_vec = r1_vec.cross(&v1);
        let h_mag = h_vec.norm();
        let p_orb = (h_mag * h_mag) / mu;
        let ecc = if semi_major_axis_m.is_finite() && semi_major_axis_m > 0.0 {
            (1.0 - p_orb / semi_major_axis_m).max(0.0).sqrt()
        } else if semi_major_axis_m < 0.0 {
            (1.0 - p_orb / semi_major_axis_m).sqrt()
        } else {
            1.0 // Parabolik
        };

        let c3_energy_m2_s2 = (v1.norm() - (mu / r1).sqrt()).powi(2);

        Ok(LambertResult {
            v1,
            v2,
            semi_major_axis_m,
            eccentricity: ecc,
            transfer_angle_rad: dnu,
            c3_energy_m2_s2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambert_curtis_example_5_2_independent_textbook_benchmark() {
        // Bağımsız ders kitabı referansı: Howard Curtis, Orbital Mechanics for Engineering Students
        // (4th Ed., Example 5.2) / Vallado Ch. 7:
        //   r1 = [5000, 10000, 2100] km, r2 = [-14600, 2500, 7000] km, dt = 3600 s, prograde
        //   Yayınlanmış çözüm: v1 = [-5.9925, 1.9254, 3.2456] km/s, v2 = [-3.3125, -4.1966, -0.3853] km/s
        let r1 = Vector3::new(5_000_000.0, 10_000_000.0, 2_100_000.0);
        let r2 = Vector3::new(-14_600_000.0, 2_500_000.0, 7_000_000.0);
        let mu = 3.986_004_418e14;
        let res = LambertSolver::solve(r1, r2, 3600.0, true, mu).unwrap();

        assert!((res.v1.x / 1000.0 - (-5.9925)).abs() < 0.002, "v1.x = {:.4} km/s", res.v1.x / 1000.0);
        assert!((res.v1.y / 1000.0 - 1.9254).abs() < 0.002, "v1.y = {:.4} km/s", res.v1.y / 1000.0);
        assert!((res.v1.z / 1000.0 - 3.2456).abs() < 0.002, "v1.z = {:.4} km/s", res.v1.z / 1000.0);
        assert!((res.v2.x / 1000.0 - (-3.3125)).abs() < 0.002, "v2.x = {:.4} km/s", res.v2.x / 1000.0);
        assert!((res.v2.y / 1000.0 - (-4.1966)).abs() < 0.002, "v2.y = {:.4} km/s", res.v2.y / 1000.0);
        assert!((res.v2.z / 1000.0 - (-0.3853)).abs() < 0.002, "v2.z = {:.4} km/s", res.v2.z / 1000.0);

        // İki bağımsız korunum değişmezi: Özgül yörünge enerjisi ve açısal momentum vektörü
        let eps1 = 0.5 * res.v1.norm_sq() - mu / r1.norm();
        let eps2 = 0.5 * res.v2.norm_sq() - mu / r2.norm();
        assert!((eps1 - eps2).abs() / eps1.abs() < 1e-10, "Enerji korunumu bozuldu!");
        let h1 = r1.cross(&res.v1);
        let h2 = r2.cross(&res.v2);
        assert!(h1.sub(&h2).norm() / h1.norm() < 1e-10, "Açısal momentum korunumu bozuldu!");
    }

    #[test]
    fn test_lambert_quarter_circular_orbit_exact_kepler_conservation() {
        // 400 km dairesel yörüngede 90 derecelik çeyrek tur (tof = T / 4) -> v1 = [0, v_c, 0], v2 = [-v_c, 0, 0]
        let r = J2OrbitalModel::WGS84_R_EARTH + 400_000.0;
        let mu = J2OrbitalModel::WGS84_MU;
        let v_circ = (mu / r).sqrt();
        let period = J2OrbitalModel::orbital_period_s(r).unwrap();
        let r1 = Vector3::new(r, 0.0, 0.0);
        let r2 = Vector3::new(0.0, r, 0.0);
        let res = LambertSolver::solve(r1, r2, period / 4.0, true, mu).unwrap();
        assert!((res.v1.y - v_circ).abs() < 1e-3);
        assert!(res.v1.x.abs() < 1e-3);
        assert!((res.v2.x - (-v_circ)).abs() < 1e-3);
        assert!(res.eccentricity < 1e-6);
        assert!(res.delta_v1_from_circular(r1, mu) < 1e-2);
    }

    #[test]
    fn test_j2_sso_inclination_smad_and_iss_nodal_drift() {
        // SMAD / Vallado Güneş-Eşzamanlı (SSO) referansları: 600 km ≈ 97.79°, 800 km ≈ 98.60°
        let inc_600 = J2OrbitalModel::sun_synchronous_inclination_rad(
            J2OrbitalModel::WGS84_R_EARTH + 600_000.0,
            0.0,
        )
        .unwrap()
        .to_degrees();
        assert!((inc_600 - 97.79).abs() < 0.05, "600 km SSO eğimi: {:.3}°", inc_600);

        let inc_800 = J2OrbitalModel::sun_synchronous_inclination_rad(
            J2OrbitalModel::WGS84_R_EARTH + 800_000.0,
            0.0,
        )
        .unwrap()
        .to_degrees();
        assert!((inc_800 - 98.60).abs() < 0.05, "800 km SSO eğimi: {:.3}°", inc_800);

        // 6000 km irtifada SSO fiziksel olarak imkânsızdır (|cos i| > 1)
        assert!(J2OrbitalModel::sun_synchronous_inclination_rad(
            J2OrbitalModel::WGS84_R_EARTH + 6_000_000.0,
            0.0
        )
        .is_err());

        // ISS 400 km, i = 51.64° RAAN düğüm kayması ≈ -5.00°/gün
        let raan_iss = J2OrbitalModel::raan_precession_rate_rad_s(
            J2OrbitalModel::WGS84_R_EARTH + 400_000.0,
            0.0005,
            51.64_f64.to_radians(),
        )
        .unwrap()
        .to_degrees()
            * 86400.0;
        assert!((raan_iss - (-5.00)).abs() < 0.06, "ISS RAAN kayması: {:.3} deg/day", raan_iss);
    }

    #[test]
    fn test_j2_molniya_critical_inclination_and_stumpff_continuity() {
        let rate = J2OrbitalModel::perigee_precession_rate_rad_s(
            26_554_000.0,
            0.72,
            J2OrbitalModel::CRITICAL_INCLINATION_RAD,
        )
        .unwrap();
        assert!(rate.abs() < 1e-12, "Kritik eğimde domega/dt == 0 olmalı: {:.3e}", rate);
        assert!(J2OrbitalModel::is_critical_inclination(J2OrbitalModel::CRITICAL_INCLINATION_RAD, 0.01));

        // Stumpff C(z) ve S(z) sıfır etrafında süreklilik ve türev tutarlılığı
        assert!((LambertSolver::stumpff_c(0.0) - 0.5).abs() < 1e-15);
        assert!((LambertSolver::stumpff_s(0.0) - (1.0 / 6.0)).abs() < 1e-15);
        for &z in &[-2.5, -1e-4, 0.0, 1e-4, 3.2] {
            let h = 1e-6;
            let dc_fd = (LambertSolver::stumpff_c(z + h) - LambertSolver::stumpff_c(z - h)) / (2.0 * h);
            let ds_fd = (LambertSolver::stumpff_s(z + h) - LambertSolver::stumpff_s(z - h)) / (2.0 * h);
            assert!((LambertSolver::stumpff_c_prime(z) - dc_fd).abs() < 1e-7);
            assert!((LambertSolver::stumpff_s_prime(z) - ds_fd).abs() < 1e-7);
        }
    }
}
