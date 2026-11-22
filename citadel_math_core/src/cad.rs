// Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
// Matematik Çekirdeği — 2D Parametrik CAD, Kesit Mekaniği (Steiner / Atalet / Euler Burkulma)
// ve Standart AutoCAD R12 DXF / SVG Üretici Motoru

use crate::dimensions::Dimension;
use crate::materials::Material;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// 2D Kartezyen Nokta
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Result<Self, String> {
        if !x.is_finite() || !y.is_finite() {
            return Err(format!("Sonlu olmayan 2D koordinat: ({}, {})", x, y));
        }
        Ok(Self { x, y })
    }

    pub fn distance_to(self, other: Self) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }

    pub fn angle_deg_to(self, other: Self) -> f64 {
        (other.y - self.y).atan2(other.x - self.x).to_degrees()
    }
}

/// Çizim birim ölçeği (tüm mekanik hesaplar SI `[m]` birimine dönüştürülerek yapılır)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CadUnit {
    Meters,
    Millimeters,
}

impl CadUnit {
    /// Birimden SI metreye dönüşüm çarpanı (`1 mm = 1e-3 m`, `1 m = 1.0 m`)
    #[inline]
    pub const fn to_meters_factor(self) -> f64 {
        match self {
            CadUnit::Meters => 1.0,
            CadUnit::Millimeters => 1.0e-3,
        }
    }

    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            CadUnit::Meters => "m",
            CadUnit::Millimeters => "mm",
        }
    }
}

/// Kesit katkısı: Dolu gövde (`Solid`, `+`) veya boşaltma / delik (`Hole`, `-`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionSign {
    Solid,
    Hole,
}

impl SectionSign {
    #[inline]
    pub const fn multiplier(self) -> f64 {
        match self {
            SectionSign::Solid => 1.0,
            SectionSign::Hole => -1.0,
        }
    }
}

/// 2D Parametrik CAD Geometri Öğesi
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CadEntity {
    Line {
        start: Point2D,
        end: Point2D,
        layer: String,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        sign: SectionSign,
        layer: String,
    },
    Circle {
        center: Point2D,
        radius: f64,
        sign: SectionSign,
        layer: String,
    },
    Arc {
        center: Point2D,
        radius: f64,
        start_deg: f64,
        end_deg: f64,
        layer: String,
    },
    /// İnce/kalın cidarlı boru veya roket gövdesi halka kesiti
    HollowCircle {
        center: Point2D,
        outer_radius: f64,
        inner_radius: f64,
        layer: String,
    },
    /// Yapısal I-Profili (kanat kirişi / şasi ana kirişi)
    IBeam {
        center: Point2D,
        width: f64,
        height: f64,
        flange_t: f64,
        web_t: f64,
        layer: String,
    },
    /// Keyfi basit çokgen (kapalıysa Shoelace + Green teoremiyle kesin alan & atalet)
    Polygon {
        vertices: Vec<Point2D>,
        closed: bool,
        sign: SectionSign,
        layer: String,
    },
}

/// Eksen hizalı sınırlayıcı kutu (çizim biriminde)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox2D {
    fn include_point(&mut self, x: f64, y: f64) {
        if x < self.min_x { self.min_x = x; }
        if x > self.max_x { self.max_x = x; }
        if y < self.min_y { self.min_y = y; }
        if y > self.max_y { self.max_y = y; }
    }

    pub fn width(&self) -> f64 { (self.max_x - self.min_x).max(0.0) }
    pub fn height(&self) -> f64 { (self.max_y - self.min_y).max(0.0) }
}

/// SI Birimli Kesin Bileşik Kesit Mekanik Özellikleri (Steiner Paralel Eksen Teoremi)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SectionProperties {
    /// Net kesit alanı [m²] — `Dimension::area()`
    pub area_m2: f64,
    /// Toplam çevre / çizgi uzunluğu [m] — `Dimension::length()`
    pub perimeter_m: f64,
    /// Ağırlık merkezi X [m]
    pub centroid_x_m: f64,
    /// Ağırlık merkezi Y [m]
    pub centroid_y_m: f64,
    /// Ağırlık merkezinden geçen X eksenine göre eylemsizlik momenti Ix [m⁴] — `Dimension::area_moment_of_inertia()`
    pub ix_m4: f64,
    /// Ağırlık merkezinden geçen Y eksenine göre eylemsizlik momenti Iy [m⁴]
    pub iy_m4: f64,
    /// Kutupsal eylemsizlik momenti J = Ix + Iy [m⁴]
    pub polar_j_m4: f64,
    /// X atalet yarıçapı rx = sqrt(Ix / A) [m]
    pub radius_gyration_x_m: f64,
    /// Y atalet yarıçapı ry = sqrt(Iy / A) [m]
    pub radius_gyration_y_m: f64,
    /// Elastik kesit modülü Wx = Ix / y_max [m³]
    pub section_modulus_x_m3: f64,
    /// Elastik kesit modülü Wy = Iy / x_max [m³]
    pub section_modulus_y_m3: f64,
    /// Boyut doğrulama damgası (`area_m2` ve `ix_m4` için)
    pub area_dim: Dimension,
    pub inertia_dim: Dimension,
}

/// Kesit + Malzeme Birleşik Kiriş / Kolon Mühendislik Raporu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamEngineeringReport {
    pub material_id: String,
    pub material_name: String,
    pub length_m: f64,
    pub temperature_k: f64,
    pub linear_mass_kg_m: f64,
    pub total_mass_kg: f64,
    pub derated_yield_pa: f64,
    pub derated_youngs_pa: f64,
    pub bending_stress_pa: f64,
    pub axial_stress_pa: f64,
    pub combined_peak_stress_pa: f64,
    pub euler_buckling_load_n: f64,
    pub yield_safety_factor: f64,
    pub buckling_safety_factor: f64,
    pub passed: bool,
    pub section: SectionProperties,
}

/// 2D Parametrik CAD Belgesi (Komut Geçmişi, Snap-to-Grid, Kesit Fiziği, DXF & SVG)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CadDocument {
    pub name: String,
    pub unit: CadUnit,
    pub snap_grid: f64,
    pub active_layer: String,
    pub entities: Vec<CadEntity>,
}

/// Tek bir şekil parçasının ham (yerel ağırlık merkezine göre) kesit katkısı
struct RawPart {
    signed_area: f64,
    perimeter: f64,
    cx: f64,
    cy: f64,
    /// Kendi ağırlık merkezi etrafında Ix (işaretli)
    ix_local: f64,
    /// Kendi ağırlık merkezi etrafında Iy (işaretli)
    iy_local: f64,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl CadDocument {
    pub fn new(name: &str, unit: CadUnit) -> Self {
        Self {
            name: name.to_string(),
            unit,
            snap_grid: 0.0,
            active_layer: "0".to_string(),
            entities: Vec::new(),
        }
    }

    pub fn set_snap_grid(&mut self, step: f64) -> Result<(), String> {
        if !step.is_finite() || step < 0.0 {
            return Err(format!("Geçersiz ızgara adımı (snap): {}", step));
        }
        self.snap_grid = step;
        Ok(())
    }

    /// Snap-to-Grid aktifse noktayı en yakın ızgara düğümüne oturtur.
    pub fn snap_point(&self, p: Point2D) -> Point2D {
        if self.snap_grid > 0.0 {
            Point2D {
                x: (p.x / self.snap_grid).round() * self.snap_grid,
                y: (p.y / self.snap_grid).round() * self.snap_grid,
            }
        } else {
            p
        }
    }

    pub fn add_entity(&mut self, entity: CadEntity) -> Result<(), String> {
        validate_entity(&entity)?;
        self.entities.push(entity);
        Ok(())
    }

    pub fn undo(&mut self) -> Option<CadEntity> {
        self.entities.pop()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// AutoCAD tarzı metin komutunu çalıştırır:
    /// - `LINE x1,y1 x2,y2`
    /// - `RECT x,y w,h` / `HOLE_RECT x,y w,h`
    /// - `CIRCLE cx,cy r` / `HOLE_CIRCLE cx,cy r`
    /// - `ARC cx,cy r start_deg end_deg`
    /// - `RING cx,cy r_out r_in`
    /// - `IBEAM cx,cy w h tf tw`
    /// - `POLY x1,y1 x2,y2 x3,y3 ...`
    /// - `LAYER <ad>` | `SNAP <adim>` | `DIST x1,y1 x2,y2` | `UNDO` | `CLEAR`
    pub fn exec_command(&mut self, raw_cmd: &str) -> Result<String, String> {
        let trimmed = raw_cmd.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            return Ok(String::new());
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let op = tokens[0].to_ascii_uppercase();

        match op.as_str() {
            "LINE" | "L" => {
                if tokens.len() < 3 {
                    return Err("Kullanım: LINE x1,y1 x2,y2".to_string());
                }
                let p1 = self.snap_point(parse_point(tokens[1])?);
                let p2 = self.snap_point(parse_point(tokens[2])?);
                let d = p1.distance_to(p2);
                self.add_entity(CadEntity::Line {
                    start: p1,
                    end: p2,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!(
                    "LINE ({:.3},{:.3})->({:.3},{:.3}) L={:.4} {}",
                    p1.x, p1.y, p2.x, p2.y, d, self.unit.label()
                ))
            }
            "RECT" | "REC" | "HOLE_RECT" => {
                if tokens.len() < 3 {
                    return Err("Kullanım: RECT x,y w,h (veya x,y w h)".to_string());
                }
                let origin = self.snap_point(parse_point(tokens[1])?);
                let (w, h) = if tokens.len() == 3 {
                    let dim = parse_point(tokens[2])?;
                    (dim.x, dim.y)
                } else {
                    (parse_f64(tokens[2])?, parse_f64(tokens[3])?)
                };
                let sign = if op == "HOLE_RECT" { SectionSign::Hole } else { SectionSign::Solid };
                self.add_entity(CadEntity::Rect {
                    x: origin.x,
                    y: origin.y,
                    width: w,
                    height: h,
                    sign,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!(
                    "{} ({:.3},{:.3}) {:.3}x{:.3} {}",
                    op, origin.x, origin.y, w, h, self.unit.label()
                ))
            }
            "CIRCLE" | "C" | "HOLE_CIRCLE" => {
                if tokens.len() < 3 {
                    return Err("Kullanım: CIRCLE cx,cy r".to_string());
                }
                let c = self.snap_point(parse_point(tokens[1])?);
                let r = parse_f64(tokens[2])?;
                let sign = if op == "HOLE_CIRCLE" { SectionSign::Hole } else { SectionSign::Solid };
                self.add_entity(CadEntity::Circle {
                    center: c,
                    radius: r,
                    sign,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!(
                    "{} ({:.3},{:.3}) R={:.3} {}",
                    op, c.x, c.y, r, self.unit.label()
                ))
            }
            "ARC" => {
                if tokens.len() < 5 {
                    return Err("Kullanım: ARC cx,cy r start_deg end_deg".to_string());
                }
                let c = self.snap_point(parse_point(tokens[1])?);
                let r = parse_f64(tokens[2])?;
                let a0 = parse_f64(tokens[3])?;
                let a1 = parse_f64(tokens[4])?;
                self.add_entity(CadEntity::Arc {
                    center: c,
                    radius: r,
                    start_deg: a0,
                    end_deg: a1,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!("ARC ({:.3},{:.3}) R={:.3} [{:.1}°..{:.1}°]", c.x, c.y, r, a0, a1))
            }
            "RING" | "HOLLOW" => {
                if tokens.len() < 4 {
                    return Err("Kullanım: RING cx,cy r_out r_in".to_string());
                }
                let c = self.snap_point(parse_point(tokens[1])?);
                let ro = parse_f64(tokens[2])?;
                let ri = parse_f64(tokens[3])?;
                self.add_entity(CadEntity::HollowCircle {
                    center: c,
                    outer_radius: ro,
                    inner_radius: ri,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!("RING ({:.3},{:.3}) Ro={:.3} Ri={:.3}", c.x, c.y, ro, ri))
            }
            "IBEAM" => {
                if tokens.len() < 6 {
                    return Err("Kullanım: IBEAM cx,cy width height flange_t web_t".to_string());
                }
                let c = self.snap_point(parse_point(tokens[1])?);
                let w = parse_f64(tokens[2])?;
                let h = parse_f64(tokens[3])?;
                let tf = parse_f64(tokens[4])?;
                let tw = parse_f64(tokens[5])?;
                self.add_entity(CadEntity::IBeam {
                    center: c,
                    width: w,
                    height: h,
                    flange_t: tf,
                    web_t: tw,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!("IBEAM ({:.3},{:.3}) {}x{} tf={} tw={}", c.x, c.y, w, h, tf, tw))
            }
            "POLY" | "POLYGON" => {
                if tokens.len() < 4 {
                    return Err("Kullanım: POLY x1,y1 x2,y2 x3,y3 ...".to_string());
                }
                let mut verts = Vec::with_capacity(tokens.len() - 1);
                for tok in &tokens[1..] {
                    verts.push(self.snap_point(parse_point(tok)?));
                }
                let n = verts.len();
                self.add_entity(CadEntity::Polygon {
                    vertices: verts,
                    closed: true,
                    sign: SectionSign::Solid,
                    layer: self.active_layer.clone(),
                })?;
                Ok(format!("POLYGON ({} köşe)", n))
            }
            "SNAP" => {
                if tokens.len() < 2 {
                    return Err("Kullanım: SNAP <adim>".to_string());
                }
                let s = parse_f64(tokens[1])?;
                self.set_snap_grid(s)?;
                Ok(format!("SNAP = {} {}", s, self.unit.label()))
            }
            "LAYER" => {
                if tokens.len() < 2 {
                    return Err("Kullanım: LAYER <katman_adi>".to_string());
                }
                self.active_layer = tokens[1].to_string();
                Ok(format!("LAYER = {}", self.active_layer))
            }
            "DIST" => {
                if tokens.len() < 3 {
                    return Err("Kullanım: DIST x1,y1 x2,y2".to_string());
                }
                let p1 = parse_point(tokens[1])?;
                let p2 = parse_point(tokens[2])?;
                Ok(format!(
                    "DIST = {:.6} {} | Açı = {:.3}°",
                    p1.distance_to(p2),
                    self.unit.label(),
                    p1.angle_deg_to(p2)
                ))
            }
            "UNDO" => {
                self.undo();
                Ok(format!("UNDO (kalan öğe: {})", self.entities.len()))
            }
            "CLEAR" => {
                self.clear();
                Ok("CLEAR (belge temizlendi)".to_string())
            }
            _ => Err(format!("Bilinmeyen CAD komutu: '{}'", tokens[0])),
        }
    }

    /// `;` veya yeni satırla ayrılmış çoklu CAD komut betiğini yürütür.
    pub fn exec_script(&mut self, script: &str) -> Result<Vec<String>, String> {
        let mut logs = Vec::new();
        for piece in script.split([';', '\n']) {
            let res = self.exec_command(piece)?;
            if !res.is_empty() {
                logs.push(res);
            }
        }
        Ok(logs)
    }

    /// Tüm öğeleri saran çizim-birimli sınırlayıcı kutuyu döndürür.
    pub fn bounding_box(&self) -> Option<BoundingBox2D> {
        if self.entities.is_empty() {
            return None;
        }
        let mut bb = BoundingBox2D {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        };
        for e in &self.entities {
            match e {
                CadEntity::Line { start, end, .. } => {
                    bb.include_point(start.x, start.y);
                    bb.include_point(end.x, end.y);
                }
                CadEntity::Rect { x, y, width, height, .. } => {
                    bb.include_point(*x, *y);
                    bb.include_point(*x + *width, *y + *height);
                }
                CadEntity::Circle { center, radius, .. }
                | CadEntity::Arc { center, radius, .. } => {
                    bb.include_point(center.x - *radius, center.y - *radius);
                    bb.include_point(center.x + *radius, center.y + *radius);
                }
                CadEntity::HollowCircle { center, outer_radius, .. } => {
                    bb.include_point(center.x - *outer_radius, center.y - *outer_radius);
                    bb.include_point(center.x + *outer_radius, center.y + *outer_radius);
                }
                CadEntity::IBeam { center, width, height, .. } => {
                    bb.include_point(center.x - *width / 2.0, center.y - *height / 2.0);
                    bb.include_point(center.x + *width / 2.0, center.y + *height / 2.0);
                }
                CadEntity::Polygon { vertices, .. } => {
                    for v in vertices {
                        bb.include_point(v.x, v.y);
                    }
                }
            }
        }
        bb.min_x.is_finite().then_some(bb)
    }

    /// Belgedeki tüm kesit öğelerinin SI birimli (`m`, `m²`, `m³`, `m⁴`) mekanik kesit
    /// özelliklerini Steiner Paralel Eksen Teoremi ile kesin hesaplar.
    pub fn section_properties(&self) -> Result<SectionProperties, String> {
        let scale = self.unit.to_meters_factor();
        let mut parts: Vec<RawPart> = Vec::new();
        let mut total_perimeter = 0.0_f64;

        for e in &self.entities {
            match e {
                CadEntity::Line { start, end, .. } => {
                    total_perimeter += start.distance_to(*end) * scale;
                }
                CadEntity::Arc { radius, start_deg, end_deg, .. } => {
                    let sweep = (*end_deg - *start_deg).abs().to_radians();
                    total_perimeter += *radius * sweep * scale;
                }
                CadEntity::Rect { x, y, width, height, sign, .. } => {
                    let s = sign.multiplier();
                    let w = *width * scale;
                    let h = *height * scale;
                    let x0 = *x * scale;
                    let y0 = *y * scale;
                    let a = w * h;
                    parts.push(RawPart {
                        signed_area: s * a,
                        perimeter: 2.0 * (w + h),
                        cx: x0 + w / 2.0,
                        cy: y0 + h / 2.0,
                        ix_local: s * (w * h.powi(3) / 12.0),
                        iy_local: s * (h * w.powi(3) / 12.0),
                        min_x: x0,
                        max_x: x0 + w,
                        min_y: y0,
                        max_y: y0 + h,
                    });
                }
                CadEntity::Circle { center, radius, sign, .. } => {
                    let s = sign.multiplier();
                    let r = *radius * scale;
                    let cx = center.x * scale;
                    let cy = center.y * scale;
                    let i_circ = PI * r.powi(4) / 4.0;
                    parts.push(RawPart {
                        signed_area: s * PI * r * r,
                        perimeter: 2.0 * PI * r,
                        cx,
                        cy,
                        ix_local: s * i_circ,
                        iy_local: s * i_circ,
                        min_x: cx - r,
                        max_x: cx + r,
                        min_y: cy - r,
                        max_y: cy + r,
                    });
                }
                CadEntity::HollowCircle { center, outer_radius, inner_radius, .. } => {
                    let ro = *outer_radius * scale;
                    let ri = *inner_radius * scale;
                    let cx = center.x * scale;
                    let cy = center.y * scale;
                    let a = PI * (ro * ro - ri * ri);
                    let i_ring = PI * (ro.powi(4) - ri.powi(4)) / 4.0;
                    parts.push(RawPart {
                        signed_area: a,
                        perimeter: 2.0 * PI * (ro + ri),
                        cx,
                        cy,
                        ix_local: i_ring,
                        iy_local: i_ring,
                        min_x: cx - ro,
                        max_x: cx + ro,
                        min_y: cy - ro,
                        max_y: cy + ro,
                    });
                }
                CadEntity::IBeam { center, width, height, flange_t, web_t, .. } => {
                    let w = *width * scale;
                    let h = *height * scale;
                    let tf = *flange_t * scale;
                    let tw = *web_t * scale;
                    let hw = h - 2.0 * tf;
                    let cx = center.x * scale;
                    let cy = center.y * scale;
                    let a = 2.0 * w * tf + tw * hw;
                    let perim = 2.0 * w + 4.0 * tf + 2.0 * (w - tw) + 2.0 * hw;
                    // Ix = (w*h^3 - (w - tw)*hw^3) / 12
                    let ix = (w * h.powi(3) - (w - tw) * hw.powi(3)) / 12.0;
                    // Iy = (2*tf*w^3 + hw*tw^3) / 12
                    let iy = (2.0 * tf * w.powi(3) + hw * tw.powi(3)) / 12.0;
                    parts.push(RawPart {
                        signed_area: a,
                        perimeter: perim,
                        cx,
                        cy,
                        ix_local: ix,
                        iy_local: iy,
                        min_x: cx - w / 2.0,
                        max_x: cx + w / 2.0,
                        min_y: cy - h / 2.0,
                        max_y: cy + h / 2.0,
                    });
                }
                CadEntity::Polygon { vertices, closed, sign, .. } => {
                    let mut p_len = 0.0;
                    let n = vertices.len();
                    for i in 0..n {
                        if !*closed && i + 1 >= n {
                            break;
                        }
                        let a = vertices[i];
                        let b = vertices[(i + 1) % n];
                        p_len += a.distance_to(b) * scale;
                    }
                    if !*closed {
                        total_perimeter += p_len;
                    } else {
                        let part = polygon_section_part(vertices, scale, *sign, p_len)?;
                        parts.push(part);
                    }
                }
            }
        }

        let mut total_area = 0.0_f64;
        let mut sum_ax = 0.0_f64;
        let mut sum_ay = 0.0_f64;
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for p in &parts {
            total_area += p.signed_area;
            total_perimeter += p.perimeter;
            sum_ax += p.signed_area * p.cx;
            sum_ay += p.signed_area * p.cy;
            if p.signed_area > 0.0 {
                min_x = min_x.min(p.min_x);
                max_x = max_x.max(p.max_x);
                min_y = min_y.min(p.min_y);
                max_y = max_y.max(p.max_y);
            }
        }

        if total_area <= 0.0 {
            return Err(format!(
                "Geçerli pozitif net kesit alanı yok (A = {:.6e} m²). En az bir kapalı dolu kesit ekleyin.",
                total_area
            ));
        }

        let cx = sum_ax / total_area;
        let cy = sum_ay / total_area;

        // Steiner Paralel Eksen Teoremi: I_total = Σ (I_local + A_i * d_i^2)
        let mut ix = 0.0_f64;
        let mut iy = 0.0_f64;
        for p in &parts {
            let dy = p.cy - cy;
            let dx = p.cx - cx;
            ix += p.ix_local + p.signed_area * dy * dy;
            iy += p.iy_local + p.signed_area * dx * dx;
        }
        if ix <= 0.0 || iy <= 0.0 {
            return Err("Kesit eylemsizlik momenti (Ix veya Iy) fiziksel olmayan (<= 0) değer üretti!".to_string());
        }

        let y_ext = (max_y - cy).abs().max((cy - min_y).abs()).max(1e-12);
        let x_ext = (max_x - cx).abs().max((cx - min_x).abs()).max(1e-12);

        Ok(SectionProperties {
            area_m2: total_area,
            perimeter_m: total_perimeter,
            centroid_x_m: cx,
            centroid_y_m: cy,
            ix_m4: ix,
            iy_m4: iy,
            polar_j_m4: ix + iy,
            radius_gyration_x_m: (ix / total_area).sqrt(),
            radius_gyration_y_m: (iy / total_area).sqrt(),
            section_modulus_x_m3: ix / y_ext,
            section_modulus_y_m3: iy / x_ext,
            area_dim: Dimension::area(),
            inertia_dim: Dimension::area_moment_of_inertia(),
        })
    }

    /// Kesiti `materials::Material` ile birleştirip eğilme + eksenel basma + Euler burkulma
    /// güvenlik katsayılarını hesaplar (`SF >= 1.35`).
    pub fn evaluate_beam(
        &self,
        mat: &Material,
        length_m: f64,
        bending_moment_nm: f64,
        axial_compression_n: f64,
        temp_k: f64,
    ) -> Result<BeamEngineeringReport, String> {
        if !length_m.is_finite() || length_m <= 0.0 {
            return Err("Kiriş uzunluğu pozitif ve sonlu olmalıdır!".to_string());
        }
        let sec = self.section_properties()?;
        let (yield_t, youngs_t) = mat.derated_properties_at_temp_k(temp_k);
        let linear_mass = mat.density * sec.area_m2;
        let total_mass = linear_mass * length_m;

        let sigma_b = bending_moment_nm.abs() / sec.section_modulus_x_m3;
        let sigma_a = axial_compression_n.abs() / sec.area_m2;
        let sigma_peak = sigma_b + sigma_a;

        let i_min = sec.ix_m4.min(sec.iy_m4);
        let p_cr = (PI * PI * youngs_t * i_min) / (length_m * length_m);

        let sf_yield = if sigma_peak <= 0.0 {
            999.0
        } else {
            yield_t / sigma_peak
        };
        let sf_buckling = if axial_compression_n <= 0.0 {
            999.0
        } else {
            p_cr / axial_compression_n
        };

        let passed = yield_t > 0.0 && sf_yield >= 1.35 && sf_buckling >= 1.35;

        Ok(BeamEngineeringReport {
            material_id: mat.id.clone(),
            material_name: mat.name.clone(),
            length_m,
            temperature_k: temp_k,
            linear_mass_kg_m: linear_mass,
            total_mass_kg: total_mass,
            derated_yield_pa: yield_t,
            derated_youngs_pa: youngs_t,
            bending_stress_pa: sigma_b,
            axial_stress_pa: sigma_a,
            combined_peak_stress_pa: sigma_peak,
            euler_buckling_load_n: p_cr,
            yield_safety_factor: sf_yield,
            buckling_safety_factor: sf_buckling,
            passed,
            section: sec,
        })
    }

    /// Standart **AutoCAD R12 ASCII DXF** dışa aktarımı (AutoCAD, FreeCAD, LibreCAD, QCAD uyumlu).
    pub fn export_dxf(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1009\n0\nENDSEC\n");
        out.push_str("0\nSECTION\n2\nENTITIES\n");

        for e in &self.entities {
            match e {
                CadEntity::Line { start, end, layer } => {
                    dxf_line(&mut out, layer, start.x, start.y, end.x, end.y);
                }
                CadEntity::Rect { x, y, width, height, layer, .. } => {
                    let pts = [
                        (*x, *y),
                        (*x + *width, *y),
                        (*x + *width, *y + *height),
                        (*x, *y + *height),
                    ];
                    dxf_closed_polyline(&mut out, layer, &pts);
                }
                CadEntity::Circle { center, radius, layer, .. } => {
                    dxf_circle(&mut out, layer, center.x, center.y, *radius);
                }
                CadEntity::Arc { center, radius, start_deg, end_deg, layer } => {
                    out.push_str(&format!(
                        "0\nARC\n8\n{}\n10\n{:.6}\n20\n{:.6}\n30\n0.0\n40\n{:.6}\n50\n{:.6}\n51\n{:.6}\n",
                        layer, center.x, center.y, radius, start_deg, end_deg
                    ));
                }
                CadEntity::HollowCircle { center, outer_radius, inner_radius, layer } => {
                    dxf_circle(&mut out, layer, center.x, center.y, *outer_radius);
                    dxf_circle(&mut out, layer, center.x, center.y, *inner_radius);
                }
                CadEntity::IBeam { center, width, height, flange_t, web_t, layer } => {
                    let pts = ibeam_outline(*center, *width, *height, *flange_t, *web_t);
                    dxf_closed_polyline(&mut out, layer, &pts);
                }
                CadEntity::Polygon { vertices, closed, layer, .. } => {
                    let pts: Vec<(f64, f64)> = vertices.iter().map(|p| (p.x, p.y)).collect();
                    if *closed {
                        dxf_closed_polyline(&mut out, layer, &pts);
                    } else {
                        for pair in pts.windows(2) {
                            dxf_line(&mut out, layer, pair[0].0, pair[0].1, pair[1].0, pair[1].1);
                        }
                    }
                }
            }
        }

        out.push_str("0\nENDSEC\n0\nEOF\n");
        out
    }

    /// Teknik Blueprint **SVG** vektörel dışa aktarımı (tarayıcı & dokümantasyon uyumlu).
    pub fn export_svg(&self) -> String {
        let bb = self.bounding_box().unwrap_or(BoundingBox2D {
            min_x: -10.0,
            min_y: -10.0,
            max_x: 10.0,
            max_y: 10.0,
        });
        let pad = bb.width().max(bb.height()).max(1.0) * 0.15;
        let vx = bb.min_x - pad;
        let vy = bb.min_y - pad;
        let vw = bb.width() + 2.0 * pad;
        let vh = bb.height() + 2.0 * pad;
        let sw = (vw.max(vh) * 0.006).max(0.05);

        let mut out = String::with_capacity(2048);
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.3} {:.3} {:.3} {:.3}\" width=\"800\" height=\"600\">\n",
            vx, -(vy + vh), vw, vh
        ));
        out.push_str(&format!(
            "  <rect x=\"{:.3}\" y=\"{:.3}\" width=\"{:.3}\" height=\"{:.3}\" fill=\"#070b14\"/>\n",
            vx, -(vy + vh), vw, vh
        ));
        // Y eksenini matematiksel (yukarı +Y) yöne çevir
        out.push_str("  <g transform=\"scale(1,-1)\" fill=\"none\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n");
        // Eksenler
        out.push_str(&format!(
            "    <line x1=\"{:.3}\" y1=\"0\" x2=\"{:.3}\" y2=\"0\" stroke=\"#334155\" stroke-width=\"{:.4}\"/>\n",
            vx, vx + vw, sw * 0.6
        ));
        out.push_str(&format!(
            "    <line x1=\"0\" y1=\"{:.3}\" x2=\"0\" y2=\"{:.3}\" stroke=\"#334155\" stroke-width=\"{:.4}\"/>\n",
            vy, vy + vh, sw * 0.6
        ));

        for e in &self.entities {
            match e {
                CadEntity::Line { start, end, .. } => {
                    out.push_str(&format!(
                        "    <line x1=\"{:.4}\" y1=\"{:.4}\" x2=\"{:.4}\" y2=\"{:.4}\" stroke=\"#38bdf8\" stroke-width=\"{:.4}\"/>\n",
                        start.x, start.y, end.x, end.y, sw
                    ));
                }
                CadEntity::Rect { x, y, width, height, sign, .. } => {
                    let col = if *sign == SectionSign::Hole { "#f87171" } else { "#38bdf8" };
                    out.push_str(&format!(
                        "    <rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" stroke=\"{}\" stroke-width=\"{:.4}\"/>\n",
                        x, y, width, height, col, sw
                    ));
                }
                CadEntity::Circle { center, radius, sign, .. } => {
                    let col = if *sign == SectionSign::Hole { "#f87171" } else { "#38bdf8" };
                    out.push_str(&format!(
                        "    <circle cx=\"{:.4}\" cy=\"{:.4}\" r=\"{:.4}\" stroke=\"{}\" stroke-width=\"{:.4}\"/>\n",
                        center.x, center.y, radius, col, sw
                    ));
                }
                CadEntity::Arc { center, radius, start_deg, end_deg, .. } => {
                    let a0 = start_deg.to_radians();
                    let a1 = end_deg.to_radians();
                    let x0 = center.x + radius * a0.cos();
                    let y0 = center.y + radius * a0.sin();
                    let x1 = center.x + radius * a1.cos();
                    let y1 = center.y + radius * a1.sin();
                    let large = if (*end_deg - *start_deg).abs() > 180.0 { 1 } else { 0 };
                    out.push_str(&format!(
                        "    <path d=\"M {:.4} {:.4} A {:.4} {:.4} 0 {} 1 {:.4} {:.4}\" stroke=\"#fbbf24\" stroke-width=\"{:.4}\"/>\n",
                        x0, y0, radius, radius, large, x1, y1, sw
                    ));
                }
                CadEntity::HollowCircle { center, outer_radius, inner_radius, .. } => {
                    out.push_str(&format!(
                        "    <circle cx=\"{:.4}\" cy=\"{:.4}\" r=\"{:.4}\" stroke=\"#38bdf8\" stroke-width=\"{:.4}\"/>\n",
                        center.x, center.y, outer_radius, sw
                    ));
                    out.push_str(&format!(
                        "    <circle cx=\"{:.4}\" cy=\"{:.4}\" r=\"{:.4}\" stroke=\"#f87171\" stroke-width=\"{:.4}\"/>\n",
                        center.x, center.y, inner_radius, sw
                    ));
                }
                CadEntity::IBeam { center, width, height, flange_t, web_t, .. } => {
                    let pts = ibeam_outline(*center, *width, *height, *flange_t, *web_t);
                    let pts_str: Vec<String> = pts.iter().map(|(x, y)| format!("{:.4},{:.4}", x, y)).collect();
                    out.push_str(&format!(
                        "    <polygon points=\"{}\" stroke=\"#38bdf8\" stroke-width=\"{:.4}\"/>\n",
                        pts_str.join(" "), sw
                    ));
                }
                CadEntity::Polygon { vertices, closed, sign, .. } => {
                    let col = if *sign == SectionSign::Hole { "#f87171" } else { "#38bdf8" };
                    let pts_str: Vec<String> = vertices.iter().map(|p| format!("{:.4},{:.4}", p.x, p.y)).collect();
                    let tag = if *closed { "polygon" } else { "polyline" };
                    out.push_str(&format!(
                        "    <{} points=\"{}\" stroke=\"{}\" stroke-width=\"{:.4}\"/>\n",
                        tag, pts_str.join(" "), col, sw
                    ));
                }
            }
        }

        // Ağırlık merkezi artı imleci (varsa)
        if let Ok(sec) = self.section_properties() {
            let inv = 1.0 / self.unit.to_meters_factor();
            let cx = sec.centroid_x_m * inv;
            let cy = sec.centroid_y_m * inv;
            let r = sw * 3.0;
            out.push_str(&format!(
                "    <line x1=\"{:.4}\" y1=\"{:.4}\" x2=\"{:.4}\" y2=\"{:.4}\" stroke=\"#34d399\" stroke-width=\"{:.4}\"/>\n",
                cx - r, cy, cx + r, cy, sw
            ));
            out.push_str(&format!(
                "    <line x1=\"{:.4}\" y1=\"{:.4}\" x2=\"{:.4}\" y2=\"{:.4}\" stroke=\"#34d399\" stroke-width=\"{:.4}\"/>\n",
                cx, cy - r, cx, cy + r, sw
            ));
        }

        out.push_str("  </g>\n</svg>\n");
        out
    }
}

fn validate_entity(e: &CadEntity) -> Result<(), String> {
    match e {
        CadEntity::Line { start, end, .. } => {
            if !start.x.is_finite() || !start.y.is_finite() || !end.x.is_finite() || !end.y.is_finite() {
                return Err("Line koordinatları sonlu olmalıdır!".to_string());
            }
            if start.distance_to(*end) <= 0.0 {
                return Err("Sıfır uzunluklu Line eklenemez!".to_string());
            }
        }
        CadEntity::Rect { x, y, width, height, .. } => {
            if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() || *width <= 0.0 || *height <= 0.0 {
                return Err("Rect genişlik ve yüksekliği pozitif sonlu olmalıdır!".to_string());
            }
        }
        CadEntity::Circle { center, radius, .. } | CadEntity::Arc { center, radius, .. } => {
            if !center.x.is_finite() || !center.y.is_finite() || !radius.is_finite() || *radius <= 0.0 {
                return Err("Daire/Yay yarıçapı pozitif sonlu olmalıdır!".to_string());
            }
        }
        CadEntity::HollowCircle { center, outer_radius, inner_radius, .. } => {
            if !center.x.is_finite()
                || !center.y.is_finite()
                || !outer_radius.is_finite()
                || !inner_radius.is_finite()
                || *inner_radius <= 0.0
                || *outer_radius <= *inner_radius
            {
                return Err("Halka kesitinde 0 < inner_radius < outer_radius olmalıdır!".to_string());
            }
        }
        CadEntity::IBeam { center, width, height, flange_t, web_t, .. } => {
            if !center.x.is_finite()
                || !center.y.is_finite()
                || *width <= 0.0
                || *height <= 0.0
                || *flange_t <= 0.0
                || *web_t <= 0.0
                || (*flange_t * 2.0) >= *height
                || *web_t >= *width
            {
                return Err("I-Profili geçersiz geometri: 2*flange_t < height ve web_t < width olmalıdır!".to_string());
            }
        }
        CadEntity::Polygon { vertices, closed, .. } => {
            let min_v = if *closed { 3 } else { 2 };
            if vertices.len() < min_v {
                return Err(format!("Çokgen en az {} köşe içermelidir!", min_v));
            }
            for v in vertices {
                if !v.x.is_finite() || !v.y.is_finite() {
                    return Err("Çokgen köşesi sonlu olmalıdır!".to_string());
                }
            }
        }
    }
    Ok(())
}

/// Kapalı basit çokgen için Green Teoremi / Shoelace kesin alan, ağırlık merkezi ve Ix/Iy.
fn polygon_section_part(
    vertices: &[Point2D],
    scale: f64,
    sign: SectionSign,
    perimeter: f64,
) -> Result<RawPart, String> {
    let n = vertices.len();
    let mut a2 = 0.0_f64;
    let mut cx6 = 0.0_f64;
    let mut cy6 = 0.0_f64;
    let mut ix12 = 0.0_f64;
    let mut iy12 = 0.0_f64;
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for i in 0..n {
        let x0 = vertices[i].x * scale;
        let y0 = vertices[i].y * scale;
        let x1 = vertices[(i + 1) % n].x * scale;
        let y1 = vertices[(i + 1) % n].y * scale;
        min_x = min_x.min(x0);
        max_x = max_x.max(x0);
        min_y = min_y.min(y0);
        max_y = max_y.max(y0);
        let cross = x0 * y1 - x1 * y0;
        a2 += cross;
        cx6 += (x0 + x1) * cross;
        cy6 += (y0 + y1) * cross;
        ix12 += (y0 * y0 + y0 * y1 + y1 * y1) * cross;
        iy12 += (x0 * x0 + x0 * x1 + x1 * x1) * cross;
    }

    if a2.abs() < 1e-18 {
        return Err("Dejenere (sıfır alanlı) çokgen kesiti!".to_string());
    }
    let orient = if a2 >= 0.0 { 1.0 } else { -1.0 };
    let area = 0.5 * a2.abs();
    let cx = cx6 / (3.0 * a2);
    let cy = cy6 / (3.0 * a2);
    // Orijine göre ataletten yerel ağırlık merkezine geçiş (Steiner tersi)
    let ix_origin = (ix12 * orient) / 12.0;
    let iy_origin = (iy12 * orient) / 12.0;
    let ix_local = (ix_origin - area * cy * cy).max(0.0);
    let iy_local = (iy_origin - area * cx * cx).max(0.0);
    let s = sign.multiplier();

    Ok(RawPart {
        signed_area: s * area,
        perimeter,
        cx,
        cy,
        ix_local: s * ix_local,
        iy_local: s * iy_local,
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

fn ibeam_outline(c: Point2D, w: f64, h: f64, tf: f64, tw: f64) -> Vec<(f64, f64)> {
    let hw = w / 2.0;
    let hh = h / 2.0;
    let htw = tw / 2.0;
    let iy = hh - tf;
    vec![
        (c.x - hw, c.y - hh),
        (c.x + hw, c.y - hh),
        (c.x + hw, c.y - iy),
        (c.x + htw, c.y - iy),
        (c.x + htw, c.y + iy),
        (c.x + hw, c.y + iy),
        (c.x + hw, c.y + hh),
        (c.x - hw, c.y + hh),
        (c.x - hw, c.y + iy),
        (c.x - htw, c.y + iy),
        (c.x - htw, c.y - iy),
        (c.x - hw, c.y - iy),
    ]
}

fn dxf_line(out: &mut String, layer: &str, x1: f64, y1: f64, x2: f64, y2: f64) {
    out.push_str(&format!(
        "0\nLINE\n8\n{}\n10\n{:.6}\n20\n{:.6}\n30\n0.0\n11\n{:.6}\n21\n{:.6}\n31\n0.0\n",
        layer, x1, y1, x2, y2
    ));
}

fn dxf_circle(out: &mut String, layer: &str, cx: f64, cy: f64, r: f64) {
    out.push_str(&format!(
        "0\nCIRCLE\n8\n{}\n10\n{:.6}\n20\n{:.6}\n30\n0.0\n40\n{:.6}\n",
        layer, cx, cy, r
    ));
}

fn dxf_closed_polyline(out: &mut String, layer: &str, pts: &[(f64, f64)]) {
    out.push_str(&format!("0\nPOLYLINE\n8\n{}\n66\n1\n70\n1\n", layer));
    for &(x, y) in pts {
        out.push_str(&format!(
            "0\nVERTEX\n8\n{}\n10\n{:.6}\n20\n{:.6}\n30\n0.0\n",
            layer, x, y
        ));
    }
    out.push_str("0\nSEQEND\n");
}

fn parse_f64(s: &str) -> Result<f64, String> {
    let v = s
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Geçersiz sayı: '{}'", s))?;
    if !v.is_finite() {
        return Err(format!("Sonlu olmayan sayı: '{}'", s));
    }
    Ok(v)
}

fn parse_point(s: &str) -> Result<Point2D, String> {
    let (xs, ys) = s
        .split_once(',')
        .ok_or_else(|| format!("Koordinat 'x,y' biçiminde olmalıdır: '{}'", s))?;
    Point2D::new(parse_f64(xs)?, parse_f64(ys)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::MaterialsDatabase;

    #[test]
    fn test_rect_with_circular_hole_steiner_exact() {
        // 100mm x 40mm dikdörtgen, merkezde R=10mm dairesel boşaltma
        let mut doc = CadDocument::new("spar_plate", CadUnit::Millimeters);
        doc.exec_script("RECT 0,0 100,40; HOLE_CIRCLE 50,20 10").unwrap();
        let sec = doc.section_properties().unwrap();

        let expected_a = (0.1 * 0.04) - PI * 0.01_f64.powi(2);
        let expected_ix = (0.1 * 0.04_f64.powi(3) / 12.0) - (PI * 0.01_f64.powi(4) / 4.0);
        assert!((sec.area_m2 - expected_a).abs() < 1e-12);
        assert!((sec.centroid_x_m - 0.05).abs() < 1e-12);
        assert!((sec.centroid_y_m - 0.02).abs() < 1e-12);
        assert!((sec.ix_m4 - expected_ix).abs() < 1e-15);
        assert_eq!(sec.area_dim, Dimension::area());
        assert_eq!(sec.inertia_dim, Dimension::area_moment_of_inertia());
    }

    #[test]
    fn test_ibeam_matches_polygon_green_theorem_and_euler_buckling() {
        let mut doc_ibeam = CadDocument::new("ibeam", CadUnit::Millimeters);
        doc_ibeam.exec_command("IBEAM 0,0 100 200 15 10").unwrap();
        let sec_i = doc_ibeam.section_properties().unwrap();

        // Aynı I-profilini 12 köşeli genel Polygon (Green Teoremi) ile doğrula!
        let outline = ibeam_outline(Point2D { x: 0.0, y: 0.0 }, 100.0, 200.0, 15.0, 10.0);
        let verts: Vec<Point2D> = outline.into_iter().map(|(x, y)| Point2D { x, y }).collect();
        let mut doc_poly = CadDocument::new("ibeam_poly", CadUnit::Millimeters);
        doc_poly
            .add_entity(CadEntity::Polygon {
                vertices: verts,
                closed: true,
                sign: SectionSign::Solid,
                layer: "0".to_string(),
            })
            .unwrap();
        let sec_p = doc_poly.section_properties().unwrap();

        assert!((sec_i.area_m2 - sec_p.area_m2).abs() < 1e-12);
        assert!((sec_i.ix_m4 - sec_p.ix_m4).abs() < 1e-12);
        assert!((sec_i.iy_m4 - sec_p.iy_m4).abs() < 1e-12);

        // Malzeme & Euler Burkulma testi (Ti-6Al-4V, L=2.5 m, M=25 kN*m, F=100 kN, T=350 K)
        let db = MaterialsDatabase::load_canonical();
        let ti = db.get("mat_titanium_ti6al4v").unwrap();
        let rep = doc_ibeam.evaluate_beam(ti, 2.5, 25_000.0, 100_000.0, 350.0).unwrap();
        assert!(rep.passed, "expected PASS: SF_y={:.2} SF_b={:.2}", rep.yield_safety_factor, rep.buckling_safety_factor);
        // Zayıf eksen Iy (~2.51e-6 m^4) üzerinde L=2.5m Euler burkulma yükü ~445 kN
        assert!(rep.euler_buckling_load_n > 400_000.0 && rep.euler_buckling_load_n < 500_000.0);
    }

    #[test]
    fn test_dxf_and_svg_export_structure_and_snap() {
        let mut doc = CadDocument::new("rocket_ring", CadUnit::Millimeters);
        doc.exec_script("SNAP 5; RING 1.2,2.4 100 94; LINE 0,0 100,0").unwrap();
        let dxf = doc.export_dxf();
        assert!(dxf.contains("$ACADVER\n1\nAC1009"));
        assert!(dxf.contains("0\nCIRCLE\n"));
        assert!(dxf.contains("0\nLINE\n"));
        assert!(dxf.ends_with("0\nEOF\n"));

        let svg = doc.export_svg();
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("<circle "));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_invalid_cad_geometries_rejected() {
        let mut doc = CadDocument::new("bad", CadUnit::Millimeters);
        assert!(doc.exec_command("CIRCLE 0,0 -5").is_err());
        assert!(doc.exec_command("RING 0,0 10 15").is_err());
        assert!(doc.exec_command("IBEAM 0,0 50 40 25 10").is_err());
    }
}
