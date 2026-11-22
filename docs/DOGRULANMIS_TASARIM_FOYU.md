# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# CITADEL-MATH DOĞRULANMIŞ MÜHENDİSLİK TASARIM VE KANIT FÖYÜ

- **Görev Kısıtları:** Faydalı Yük = `500 kg` | Hedef LEO Yörünge >= `185 km` | Azami İvme <= `8.5 G` | Minimum Yapısal Emniyet >= `1.35x`
- **Taranan Tasarım Uzayı:** `6912` aday konfigürasyon (`987` Kepler enberi kararlılığı, `927` G-limiti, `688` Von Mises + Euler/SP-8007 + PICA-X TPS testini geçti)

## 1. Hipergraf Sembolik Denklem Sentezi ve Z^7 Boyut İspatları

| Hedef | Sentezlenen Sembolik Denklem | Z^7 Boyut | Zorunlu Tanım Kümesi Yan Koşulları | Türetim Zinciri |
| :--- | :--- | :---: | :--- | :--- |
| `delta_v` | `delta_v = ((g_0 * I_sp) * ln((m_0 / m_f)))` | `[L T^-1]` | `m_f != 0, (m_0 / m_f) > 0` | `law_exhaust_velocity_isp -> law_tsiolkovsky` |
| `P_beam` | `P_beam = ((m_dot * (g_0 * I_sp)^2) / (2 * eta))` | `[M L^2 T^-3]` | `(2 * eta) != 0` | `law_exhaust_velocity_isp -> law_thrust_mass_flow -> law_beamed_propulsion_power` |
| `m_0` | `m_0 = (exp((delta_v / (g_0 * I_sp))) * m_f)` | `[M]` | `(g_0 * I_sp) != 0, m_f != 0, (m_0 / m_f) > 0` | `law_exhaust_velocity_isp -> law_tsiolkovsky__inv_m_0` |

## 2. Çok Amaçlı NSGA-II / Kesin Pareto Skyline Cephesi (Rank 0)

- **NSGA-II Cephe Ayrışımı:** `688` uygun tasarım -> `8` cephe | **Kesin Pareto Skyline (`Rank 0`):** `225` baskılanmamış tasarım | **Normalize 2B Hiperhacim:** `0.9468`
- **Ütopya (İdeal) Vektörü:** `GLOW = 6829.4 kg`, `Max-G = 3.56 G`, `Isı Yükü = 3.45 MJ/m²`, `Min Emniyet = 46.41x`
- **Nadir (Skyline Sınır) Vektörü:** `GLOW = 22626.5 kg`, `Max-G = 5.46 G`, `Isı Yükü = 16.71 MJ/m²`, `Min Emniyet = 1.59x`

| # | İtki Mimarisi | Gövde Malzemesi | Kalkış (GLOW) | Yapısal Oran | Kepler Yörüngesi ($h_p \times h_a$, $e$) | Min Emniyet | Ütopya Mesafesi / Baskı | Interval64 İdeal $\Delta v$ | Canlı TPS ($\int q\,dt$) |
| :-: | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **#1** | Elektromanyetik Katapult (1.5 km/s) + LOX/CH4 | Hexcel HexPly IM7/8552 Havacılık Karbon/Epoksi Laminatı | **6829.4 kg** | %9.2 | 178.0 x 300.4 km (0.0093) | 1.80x | $d_u=1.322$ (18 aday) | [9311.9, 9416.3] m/s | 14.44 MJ/m² -> 24.7 mm (28.3 kg) |
| **#2** | Elektromanyetik Katapult (1.5 km/s) + LOX/CH4 | Soğuk Hadde Kriyojenik Paslanmaz Çelik (AISI 304L Full-Hard / Starship) | **10171.8 kg** | %14.7 | 175.9 x 292.0 km (0.0088) | 1.59x | $d_u=1.362$ (6 aday) | [9368.7, 9474.8] m/s | 15.36 MJ/m² -> 6.5 mm (7.5 kg) |
| **#3** | 2-Kademeli Kriyojenik LOX/LH2 Roket | Hexcel HexPly IM7/8552 Havacılık Karbon/Epoksi Laminatı | **14001.0 kg** | %11.1 | 176.1 x 463.1 km (0.0214) | 4.15x | $d_u=1.448$ (49 aday) | [10788.9, 10930.5] m/s | 4.07 MJ/m² -> 19.3 mm (106.8 kg) |
| **#4** | 2-Kademeli Kriyojenik LOX/LH2 Roket **[KNEE]** | Karbon Fiber Kompozit (Toray T1000G / Epoxy) | **15163.5 kg** | %18.2 | 174.4 x 372.8 km (0.0149) | 46.41x | $d_u=0.609$ (164 aday) | [9512.3, 9641.2] m/s | 4.82 MJ/m² -> 19.5 mm (22.3 kg) |
| **#5** | 2-Kademeli Kriyojenik LOX/LH2 Roket | Havacılık Titanyum Alaşımı (Ti-6Al-4V Grade 5) | **21538.6 kg** | %19.2 | 173.8 x 400.7 km (0.0170) | 5.27x | $d_u=1.442$ (1 aday) | [9770.2, 9902.3] m/s | 3.45 MJ/m² -> 7.9 mm (43.6 kg) |
| **#6** | Elektromanyetik Katapult (1.5 km/s) + LOX/CH4 | Inconel 718 Nikel Süperalaşımı | **21901.1 kg** | %20.6 | 172.7 x 291.8 km (0.0090) | 4.83x | $d_u=1.665$ (3 aday) | [9406.2, 9513.6] m/s | 16.71 MJ/m² -> 8.9 mm (10.2 kg) |

## 3. Otomatik Üretilen Sıfır-Panik (`#![forbid(unsafe_code)]`) Rust Fonksiyonları

```rust
#![forbid(unsafe_code)]

/// Otomatik Sentezlenen Doğrulanmış Fizik Fonksiyonu: `delta_v`
/// - Kanonik Denklem : `delta_v = ((g_0 * I_sp) * ln((m_0 / m_f)))`
/// - Z^7 Boyut İspatı: `[L T^-1]`
/// - Türetim Zinciri : `law_exhaust_velocity_isp -> law_tsiolkovsky`
/// - Fiziksel Rejim  : `universal ∩ classical_nonrelativistic`
/// - Yan Koşullar    : `m_f != 0, (m_0 / m_f) > 0`
pub fn compute_delta_v(I_sp: f64, g_0: f64, m_0: f64, m_f: f64) -> Result<f64, &'static str> {
    if !I_sp.is_finite() { return Err("Girdi sonlu bir reel sayı değil: I_sp"); }
    if I_sp < 1.0_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: I_sp (>= 1.0)"); }
    if I_sp > 30570000.0_f64 { return Err("Fiziksel geçerlilik üst sınırı ihlali: I_sp (<= 30570000.0)"); }
    if !g_0.is_finite() { return Err("Girdi sonlu bir reel sayı değil: g_0"); }
    if !m_0.is_finite() { return Err("Girdi sonlu bir reel sayı değil: m_0"); }
    if m_0 <= 1e-6_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: m_0 (> 1e-6)"); }
    if !m_f.is_finite() { return Err("Girdi sonlu bir reel sayı değil: m_f"); }
    if m_f <= 1e-6_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: m_f (> 1e-6)"); }
    if (m_f).abs() <= 1e-15_f64 { return Err("Tanım kümesi ihlali: m_f != 0"); }
    if ((m_0 / m_f)) <= 0.0_f64 { return Err("Tanım kümesi ihlali: (m_0 / m_f) > 0"); }
    let result = ((g_0 * I_sp) * (m_0 / m_f).ln());
    if !result.is_finite() {
        return Err("Hesaplama taşması veya tanımsız aritmetik sonuç!");
    }
    Ok(result)
}

/// Otomatik Sentezlenen Doğrulanmış Fizik Fonksiyonu: `P_beam`
/// - Kanonik Denklem : `P_beam = ((m_dot * (g_0 * I_sp)^2) / (2 * eta))`
/// - Z^7 Boyut İspatı: `[M L^2 T^-3]`
/// - Türetim Zinciri : `law_exhaust_velocity_isp -> law_thrust_mass_flow -> law_beamed_propulsion_power`
/// - Fiziksel Rejim  : `universal ∩ classical_nonrelativistic`
/// - Yan Koşullar    : `(2 * eta) != 0`
pub fn compute_p_beam(I_sp: f64, eta: f64, g_0: f64, m_dot: f64) -> Result<f64, &'static str> {
    if !I_sp.is_finite() { return Err("Girdi sonlu bir reel sayı değil: I_sp"); }
    if I_sp < 1.0_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: I_sp (>= 1.0)"); }
    if I_sp > 30570000.0_f64 { return Err("Fiziksel geçerlilik üst sınırı ihlali: I_sp (<= 30570000.0)"); }
    if !eta.is_finite() { return Err("Girdi sonlu bir reel sayı değil: eta"); }
    if eta < 0.001_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: eta (>= 0.001)"); }
    if eta >= 1.0_f64 { return Err("Fiziksel geçerlilik üst sınırı ihlali: eta (< 1.0)"); }
    if !g_0.is_finite() { return Err("Girdi sonlu bir reel sayı değil: g_0"); }
    if !m_dot.is_finite() { return Err("Girdi sonlu bir reel sayı değil: m_dot"); }
    if m_dot < 0.0_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: m_dot (>= 0.0)"); }
    if ((2.0_f64 * eta)).abs() <= 1e-15_f64 { return Err("Tanım kümesi ihlali: (2 * eta) != 0"); }
    let result = ((m_dot * (g_0 * I_sp).powi(2)) / (2.0_f64 * eta));
    if !result.is_finite() {
        return Err("Hesaplama taşması veya tanımsız aritmetik sonuç!");
    }
    Ok(result)
}

/// Otomatik Sentezlenen Doğrulanmış Fizik Fonksiyonu: `m_0`
/// - Kanonik Denklem : `m_0 = (exp((delta_v / (g_0 * I_sp))) * m_f)`
/// - Z^7 Boyut İspatı: `[M]`
/// - Türetim Zinciri : `law_exhaust_velocity_isp -> law_tsiolkovsky__inv_m_0`
/// - Fiziksel Rejim  : `universal ∩ classical_nonrelativistic`
/// - Yan Koşullar    : `(g_0 * I_sp) != 0, m_f != 0, (m_0 / m_f) > 0`
pub fn compute_m_0(I_sp: f64, delta_v: f64, g_0: f64, m_f: f64) -> Result<f64, &'static str> {
    if !I_sp.is_finite() { return Err("Girdi sonlu bir reel sayı değil: I_sp"); }
    if I_sp < 1.0_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: I_sp (>= 1.0)"); }
    if I_sp > 30570000.0_f64 { return Err("Fiziksel geçerlilik üst sınırı ihlali: I_sp (<= 30570000.0)"); }
    if !delta_v.is_finite() { return Err("Girdi sonlu bir reel sayı değil: delta_v"); }
    if !g_0.is_finite() { return Err("Girdi sonlu bir reel sayı değil: g_0"); }
    if !m_f.is_finite() { return Err("Girdi sonlu bir reel sayı değil: m_f"); }
    if m_f <= 1e-6_f64 { return Err("Fiziksel geçerlilik alt sınırı ihlali: m_f (> 1e-6)"); }
    if ((g_0 * I_sp)).abs() <= 1e-15_f64 { return Err("Tanım kümesi ihlali: (g_0 * I_sp) != 0"); }
    if (m_f).abs() <= 1e-15_f64 { return Err("Tanım kümesi ihlali: m_f != 0"); }
    let result = ((delta_v / (g_0 * I_sp)).exp() * m_f);
    if !result.is_finite() {
        return Err("Hesaplama taşması veya tanımsız aritmetik sonuç!");
    }
    if ((result / m_f)) <= 0.0_f64 { return Err("Sonuç tanım kümesi ihlali: (m_0 / m_f) > 0"); }
    Ok(result)
}

```
