// Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
// Matematik Çekirdeği — Çok Amaçlı Kesin Pareto Cephesi (Exact Skyline), NSGA-II Baskılanmamış Sıralama,
// Kalabalık Mesafesi (Crowding Distance), Hiperhacim ve Ütopya Diz-Noktası (Knee) Sentezleyicisi

use serde::{Deserialize, Serialize};

/// Optimizasyon hedef yönü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveDirection {
    Minimize,
    Maximize,
}

impl ObjectiveDirection {
    pub fn from_str_loose(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "min" | "minimize" | "minimise" => Ok(Self::Minimize),
            "max" | "maximize" | "maximise" => Ok(Self::Maximize),
            other => Err(format!(
                "Geçersiz amaç yönü '{}': 'min'/'minimize' veya 'max'/'maximize' bekleniyordu.",
                other
            )),
        }
    }

    #[inline]
    pub fn to_canonical_min(self, raw_value: f64) -> f64 {
        match self {
            Self::Minimize => raw_value,
            Self::Maximize => -raw_value,
        }
    }

    #[inline]
    pub fn from_canonical_min(self, canon_value: f64) -> f64 {
        match self {
            Self::Minimize => canon_value,
            Self::Maximize => -canon_value,
        }
    }
}

/// Tek bir amaç fonksiyonunun tanımı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveSpec {
    pub name: String,
    pub direction: ObjectiveDirection,
    #[serde(default)]
    pub unit: String,
}

impl ObjectiveSpec {
    pub fn minimize(name: &str, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            direction: ObjectiveDirection::Minimize,
            unit: unit.to_string(),
        }
    }

    pub fn maximize(name: &str, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            direction: ObjectiveDirection::Maximize,
            unit: unit.to_string(),
        }
    }
}

/// Her bir aday noktanın NSGA-II / Pareto metrikleri.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPointMetrics {
    pub candidate_index: usize,
    pub pareto_rank: usize,
    pub crowding_distance: f64,
    pub utopia_distance: f64,
    pub is_knee_point: bool,
    pub dominated_count: usize,
    pub dominated_by_count: usize,
}

/// Tüm uygun tasarım uzayının çok amaçlı Pareto cephesi özeti.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoFrontSummary {
    pub objectives: Vec<ObjectiveSpec>,
    pub total_feasible: usize,
    pub num_fronts: usize,
    pub skyline_count: usize,
    pub skyline_indices: Vec<usize>,
    pub fronts: Vec<Vec<usize>>,
    pub utopia_point: Vec<f64>,
    pub nadir_point: Vec<f64>,
    pub knee_candidate_index: Option<usize>,
    pub raw_hypervolume_2d: Option<f64>,
    pub normalized_hypervolume_2d: Option<f64>,
    pub point_metrics: Vec<ParetoPointMetrics>,
}

/// Genel JSON / MCP çok amaçlı analiz girdisi için aday kaydı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedObjectiveCandidate {
    pub id: String,
    pub values: Vec<f64>,
}

pub struct ParetoSynthesizer;

impl ParetoSynthesizer {
    /// Kanonik minimizasyon uzayında katı Pareto baskınlığı (kesin kısmi sıra: geçişli, yansımasız, asimetrik):
    /// `a` adayı `b` adayını baskılar (a ≺ b) <=> ∀m: a_m <= b_m VE ∃k: a_k < b_k.
    #[inline]
    pub fn dominates_canonical(canon_a: &[f64], canon_b: &[f64]) -> bool {
        if canon_a.len() != canon_b.len() || canon_a.is_empty() {
            return false;
        }
        let mut strictly_better = false;
        for (&va, &vb) in canon_a.iter().zip(canon_b.iter()) {
            if !va.is_finite() || !vb.is_finite() {
                return false;
            }
            if va > vb {
                return false;
            }
            if va < vb {
                strictly_better = true;
            }
        }
        strictly_better
    }

    /// Ham amaç vektörleri üzerinde katı Pareto baskınlığı testi.
    pub fn dominates(values_a: &[f64], values_b: &[f64], specs: &[ObjectiveSpec]) -> bool {
        if values_a.len() != specs.len() || values_b.len() != specs.len() || specs.is_empty() {
            return false;
        }
        let ca: Vec<f64> = values_a
            .iter()
            .zip(specs.iter())
            .map(|(&v, s)| s.direction.to_canonical_min(v))
            .collect();
        let cb: Vec<f64> = values_b
            .iter()
            .zip(specs.iter())
            .map(|(&v, s)| s.direction.to_canonical_min(v))
            .collect();
        Self::dominates_canonical(&ca, &cb)
    }

    /// NSGA-II Hızlı Baskılanmamış Sıralama (Deb vd., 2002) + Kalabalık Mesafesi +
    /// Ütopya/Nadir Vektörleri + Uzlaşma Diz Noktası (Knee) + Kesin 2B Hiperhacim Sentezi.
    pub fn analyze(
        specs: &[ObjectiveSpec],
        raw_matrix: &[Vec<f64>],
    ) -> Result<ParetoFrontSummary, String> {
        let m_obj = specs.len();
        if m_obj == 0 {
            return Err("En az 1 amaç fonksiyonu tanımlanmalıdır!".to_string());
        }
        let n_pts = raw_matrix.len();
        if n_pts == 0 {
            return Ok(ParetoFrontSummary {
                objectives: specs.to_vec(),
                total_feasible: 0,
                num_fronts: 0,
                skyline_count: 0,
                skyline_indices: Vec::new(),
                fronts: Vec::new(),
                utopia_point: vec![0.0; m_obj],
                nadir_point: vec![0.0; m_obj],
                knee_candidate_index: None,
                raw_hypervolume_2d: None,
                normalized_hypervolume_2d: None,
                point_metrics: Vec::new(),
            });
        }

        let mut canon: Vec<Vec<f64>> = Vec::with_capacity(n_pts);
        for (idx, row) in raw_matrix.iter().enumerate() {
            if row.len() != m_obj {
                return Err(format!(
                    "Aday #{} amaç vektörü uzunluğu ({}) != tanımlı amaç sayısı ({})!",
                    idx,
                    row.len(),
                    m_obj
                ));
            }
            if row.iter().any(|v| !v.is_finite()) {
                return Err(format!("Aday #{} sonlu olmayan (NaN/Inf) amaç değeri içeriyor!", idx));
            }
            let c_row: Vec<f64> = row
                .iter()
                .zip(specs.iter())
                .map(|(&v, s)| s.direction.to_canonical_min(v))
                .collect();
            canon.push(c_row);
        }

        // 1. NSGA-II Hızlı Baskılanmamış Sıralama: F_0, F_1, F_2, ...
        let mut dominated_sets: Vec<Vec<usize>> = vec![Vec::new(); n_pts];
        let mut dom_counts: Vec<usize> = vec![0usize; n_pts];
        let mut orig_dom_by: Vec<usize> = vec![0usize; n_pts];

        for p in 0..n_pts {
            for q in (p + 1)..n_pts {
                if Self::dominates_canonical(&canon[p], &canon[q]) {
                    dominated_sets[p].push(q);
                    dom_counts[q] += 1;
                } else if Self::dominates_canonical(&canon[q], &canon[p]) {
                    dominated_sets[q].push(p);
                    dom_counts[p] += 1;
                }
            }
        }
        orig_dom_by.copy_from_slice(&dom_counts);

        let mut fronts: Vec<Vec<usize>> = Vec::new();
        let mut current_front: Vec<usize> = (0..n_pts).filter(|&p| dom_counts[p] == 0).collect();
        let mut ranks = vec![usize::MAX; n_pts];

        let mut rank_idx = 0usize;
        while !current_front.is_empty() {
            for &p in &current_front {
                ranks[p] = rank_idx;
            }
            let mut next_front = Vec::new();
            for &p in &current_front {
                for &q in &dominated_sets[p] {
                    dom_counts[q] -= 1;
                    if dom_counts[q] == 0 {
                        next_front.push(q);
                    }
                }
            }
            fronts.push(current_front);
            current_front = next_front;
            rank_idx += 1;
        }

        // 2. Her cephe (front) içinde NSGA-II Kalabalık Mesafesi (Crowding Distance)
        let mut crowding = vec![0.0_f64; n_pts];
        for front in &fronts {
            Self::assign_crowding_for_front(front, &canon, m_obj, &mut crowding);
        }

        // 3. F_0 (Kesin Pareto Skyline) üzerinde Ütopya (İdeal) ve Nadir Noktaları
        let f0 = &fronts[0];
        let mut canon_utopia = vec![f64::INFINITY; m_obj];
        let mut canon_nadir = vec![f64::NEG_INFINITY; m_obj];
        for &p in f0 {
            for m in 0..m_obj {
                if canon[p][m] < canon_utopia[m] {
                    canon_utopia[m] = canon[p][m];
                }
                if canon[p][m] > canon_nadir[m] {
                    canon_nadir[m] = canon[p][m];
                }
            }
        }

        let utopia_point: Vec<f64> = canon_utopia
            .iter()
            .zip(specs.iter())
            .map(|(&cv, s)| s.direction.from_canonical_min(cv))
            .collect();
        let nadir_point: Vec<f64> = canon_nadir
            .iter()
            .zip(specs.iter())
            .map(|(&cv, s)| s.direction.from_canonical_min(cv))
            .collect();

        // 4. Normalize Ütopya Öklid Mesafesi ve Uzlaşma Diz-Noktası (Knee Point)
        let mut utopia_dists = vec![0.0_f64; n_pts];
        for p in 0..n_pts {
            let mut sq_sum = 0.0_f64;
            for m in 0..m_obj {
                let span = (canon_nadir[m] - canon_utopia[m]).max(1e-12);
                let norm = (canon[p][m] - canon_utopia[m]) / span;
                sq_sum += norm * norm;
            }
            utopia_dists[p] = sq_sum.sqrt();
        }

        let knee_candidate_index = f0.iter().copied().min_by(|&a, &b| {
            utopia_dists[a]
                .total_cmp(&utopia_dists[b])
                .then_with(|| a.cmp(&b))
        });

        // 5. İlk 2 amaç üzerinde Kesin 2B Baskılanan Hiperhacim (Exact 2D Sweepline Hypervolume)
        let (raw_hv_2d, norm_hv_2d) = if m_obj >= 2 {
            let span0 = (canon_nadir[0] - canon_utopia[0]).abs();
            let span1 = (canon_nadir[1] - canon_utopia[1]).abs();
            let ref0 = canon_nadir[0] + (span0 * 0.10).max(1e-6);
            let ref1 = canon_nadir[1] + (span1 * 0.10).max(1e-6);

            let raw_pts_2d: Vec<(f64, f64)> = f0.iter().map(|&p| (canon[p][0], canon[p][1])).collect();
            let raw_hv = if m_obj == 2 {
                Some(Self::exact_hypervolume_2d(&raw_pts_2d, (ref0, ref1)))
            } else {
                None
            };

            let norm_pts_2d: Vec<(f64, f64)> = f0
                .iter()
                .map(|&p| {
                    let x = if span0 > 1e-12 {
                        (canon[p][0] - canon_utopia[0]) / span0
                    } else {
                        0.0
                    };
                    let y = if span1 > 1e-12 {
                        (canon[p][1] - canon_utopia[1]) / span1
                    } else {
                        0.0
                    };
                    (x, y)
                })
                .collect();
            // [0, 1.1]^2 referans kutusuna göre normalize oran (0..1 aralığında)
            let area = Self::exact_hypervolume_2d(&norm_pts_2d, (1.1, 1.1));
            Some(raw_hv)
                .map(|r| (r, Some((area / (1.1 * 1.1)).clamp(0.0, 1.0))))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        // F_0 indislerini NSGA-II kalabalık operatörü + ütopya yakınlığına göre sırala
        let mut skyline_indices = f0.clone();
        skyline_indices.sort_by(|&a, &b| {
            let ka = knee_candidate_index == Some(a);
            let kb = knee_candidate_index == Some(b);
            kb.cmp(&ka)
                .then_with(|| crowding[b].total_cmp(&crowding[a]))
                .then_with(|| utopia_dists[a].total_cmp(&utopia_dists[b]))
                .then_with(|| a.cmp(&b))
        });

        let point_metrics: Vec<ParetoPointMetrics> = (0..n_pts)
            .map(|i| ParetoPointMetrics {
                candidate_index: i,
                pareto_rank: ranks[i],
                crowding_distance: crowding[i],
                utopia_distance: utopia_dists[i],
                is_knee_point: knee_candidate_index == Some(i),
                dominated_count: dominated_sets[i].len(),
                dominated_by_count: orig_dom_by[i],
            })
            .collect();

        Ok(ParetoFrontSummary {
            objectives: specs.to_vec(),
            total_feasible: n_pts,
            num_fronts: fronts.len(),
            skyline_count: f0.len(),
            skyline_indices,
            fronts,
            utopia_point,
            nadir_point,
            knee_candidate_index,
            raw_hypervolume_2d: raw_hv_2d,
            normalized_hypervolume_2d: norm_hv_2d,
            point_metrics,
        })
    }

    fn assign_crowding_for_front(
        front: &[usize],
        canon: &[Vec<f64>],
        m_obj: usize,
        crowding: &mut [f64],
    ) {
        let k_len = front.len();
        if k_len == 0 {
            return;
        }
        if k_len <= 2 {
            for &idx in front {
                crowding[idx] = f64::INFINITY;
            }
            return;
        }

        let mut order: Vec<usize> = front.to_vec();
        (0..m_obj).for_each(|m| {
            order.sort_by(|&a, &b| canon[a][m].total_cmp(&canon[b][m]).then_with(|| a.cmp(&b)));
            let min_v = canon[order[0]][m];
            let max_v = canon[order[k_len - 1]][m];
            let span = max_v - min_v;
            if span <= 1e-15 {
                return;
            }
            crowding[order[0]] = f64::INFINITY;
            crowding[order[k_len - 1]] = f64::INFINITY;
            for pos in 1..(k_len - 1) {
                let idx = order[pos];
                if crowding[idx].is_finite() {
                    let gap = (canon[order[pos + 1]][m] - canon[order[pos - 1]][m]) / span;
                    crowding[idx] += gap;
                }
            }
        });
    }

    /// Minimizasyon koordinatlarında referans noktası `(ref_x, ref_y)` altında kesin 2B süpürme hiperhacmi.
    pub fn exact_hypervolume_2d(points: &[(f64, f64)], reference: (f64, f64)) -> f64 {
        let (ref_x, ref_y) = reference;
        let mut valid: Vec<(f64, f64)> = points
            .iter()
            .copied()
            .filter(|&(x, y)| x.is_finite() && y.is_finite() && x <= ref_x && y <= ref_y)
            .collect();
        if valid.is_empty() {
            return 0.0;
        }
        valid.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));

        let mut area = 0.0_f64;
        let mut y_ceiling = ref_y;
        for (x, y) in valid {
            if y < y_ceiling {
                area += (ref_x - x) * (y_ceiling - y);
                y_ceiling = y;
            }
        }
        area
    }

    /// Skyline değişmezini matematiksel olarak doğrular:
    /// 1) F_0 içindeki hiçbir iki nokta birbirini baskılayamaz.
    /// 2) F_0 dışındaki her uygun nokta, F_0 içindeki EN AZ BİR nokta tarafından katı biçimde baskılanır.
    pub fn verify_skyline_invariants(
        specs: &[ObjectiveSpec],
        raw_matrix: &[Vec<f64>],
        summary: &ParetoFrontSummary,
    ) -> Result<(), String> {
        let f0_set: std::collections::HashSet<usize> =
            summary.skyline_indices.iter().copied().collect();

        for &a in &summary.skyline_indices {
            for &b in &summary.skyline_indices {
                if a != b && Self::dominates(&raw_matrix[a], &raw_matrix[b], specs) {
                    return Err(format!(
                        "Pareto Skyline ihlali: F_0 içindeki #{} adayı #{} adayını baskılıyor!",
                        a, b
                    ));
                }
            }
        }

        for (c_idx, row_c) in raw_matrix.iter().enumerate() {
            if !f0_set.contains(&c_idx) {
                let dominated_by_skyline = summary
                    .skyline_indices
                    .iter()
                    .any(|&s_idx| Self::dominates(&raw_matrix[s_idx], row_c, specs));
                if !dominated_by_skyline {
                    return Err(format!(
                        "Pareto Tamlık ihlali: F_0 dışındaki #{} adayını baskılayan hiçbir F_0 elemanı yok!",
                        c_idx
                    ));
                }
            }
        }

        Ok(())
    }
}
