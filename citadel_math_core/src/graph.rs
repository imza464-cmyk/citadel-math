// Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
// Matematik Çekirdeği — Tekil Kaynaklı (SSOT JSON) Bilgi Hipergrafı, Fiziksel Geçerlilik Bölgeleri (DomainBounds)
// ve Rejim-Uyumlu AND-OR Sentezleyici

use crate::ast::Expr;
use crate::dimensions::Dimension;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const EMBEDDED_CANONICAL_LAWS_JSON: &str = include_str!("../../database/canonical_laws.json");

const fn default_true() -> bool {
    true
}

fn default_regime() -> String {
    "universal".to_string()
}

/// Faz 1.3: Bir değişkenin yasa kapsamındaki fiziksel geçerlilik aralığı (SI birimlerinde veya boyutsuz).
/// Örn: Sıkıştırılamaz Bernoulli için `Mach: [0.0, 0.3]`, klasik mekanik için `v: [0.0, 2.9979e7]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarBound {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default = "default_true")]
    pub min_inclusive: bool,
    #[serde(default = "default_true")]
    pub max_inclusive: bool,
    #[serde(default)]
    pub note: String,
}

impl Default for VarBound {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            min_inclusive: true,
            max_inclusive: true,
            note: String::new(),
        }
    }
}

impl VarBound {
    /// Verilen sayısal değerin bu geçerlilik bölgesi içinde kalıp kalmadığını denetler.
    pub fn check_value(&self, var_name: &str, val: f64) -> Result<(), String> {
        if !val.is_finite() {
            return Err(format!("Değişken '{}' sonlu bir reel sayı olmalıdır: {}", var_name, val));
        }
        if let Some(lo) = self.min {
            let violated = if self.min_inclusive { val < lo } else { val <= lo };
            if violated {
                let op = if self.min_inclusive { ">=" } else { ">" };
                return Err(format!(
                    "Fiziksel geçerlilik sınırı ihlali: {} = {} (beklenen: {} {} {}{}{})",
                    var_name,
                    val,
                    var_name,
                    op,
                    lo,
                    if self.note.is_empty() { "" } else { " — " },
                    self.note
                ));
            }
        }
        if let Some(hi) = self.max {
            let violated = if self.max_inclusive { val > hi } else { val >= hi };
            if violated {
                let op = if self.max_inclusive { "<=" } else { "<" };
                return Err(format!(
                    "Fiziksel geçerlilik sınırı ihlali: {} = {} (beklenen: {} {} {}{}{})",
                    var_name,
                    val,
                    var_name,
                    op,
                    hi,
                    if self.note.is_empty() { "" } else { " — " },
                    self.note
                ));
            }
        }
        Ok(())
    }

    /// İki geçerlilik aralığının kesişimini hesaplar. Kesişim boş küme (∅) ise `Err` döndürür.
    pub fn intersect(&self, other: &Self, var_name: &str) -> Result<Self, String> {
        let (new_min, new_min_inc) = match (self.min, other.min) {
            (Some(a), Some(b)) => {
                if a > b {
                    (Some(a), self.min_inclusive)
                } else if b > a {
                    (Some(b), other.min_inclusive)
                } else {
                    (Some(a), self.min_inclusive && other.min_inclusive)
                }
            }
            (Some(a), None) => (Some(a), self.min_inclusive),
            (None, Some(b)) => (Some(b), other.min_inclusive),
            (None, None) => (None, true),
        };

        let (new_max, new_max_inc) = match (self.max, other.max) {
            (Some(a), Some(b)) => {
                if a < b {
                    (Some(a), self.max_inclusive)
                } else if b < a {
                    (Some(b), other.max_inclusive)
                } else {
                    (Some(a), self.max_inclusive && other.max_inclusive)
                }
            }
            (Some(a), None) => (Some(a), self.max_inclusive),
            (None, Some(b)) => (Some(b), other.max_inclusive),
            (None, None) => (None, true),
        };

        if let (Some(lo), Some(hi)) = (new_min, new_max) {
            let empty = lo > hi || ((lo - hi).abs() <= 1e-15 && (!new_min_inc || !new_max_inc));
            if empty {
                return Err(format!(
                    "Fiziksel geçerlilik bölgesi çakışması ({}): [{:?}, {:?}] ∩ [{:?}, {:?}] = ∅",
                    var_name, self.min, self.max, other.min, other.max
                ));
            }
        }

        let merged_note = match (self.note.is_empty(), other.note.is_empty()) {
            (false, false) if self.note != other.note => format!("{}; {}", self.note, other.note),
            (false, _) => self.note.clone(),
            (true, false) => other.note.clone(),
            (true, true) => String::new(),
        };

        Ok(Self {
            min: new_min,
            max: new_max,
            min_inclusive: new_min_inc,
            max_inclusive: new_max_inc,
            note: merged_note,
        })
    }
}

/// İki fiziksel rejim etiketinin birbiriyle bağdaşıp bağdaşmadığını denetler.
/// Örn: `incompressible_subsonic` (M <= 0.3) ile `hypersonic_continuum` (M >= 5) aynı zincirde birleşemez!
pub fn are_regimes_compatible(r1: &str, r2: &str) -> bool {
    if r1.is_empty() || r2.is_empty() || r1 == "universal" || r2 == "universal" || r1 == r2 {
        return true;
    }

    let flow_subsonic = ["incompressible_subsonic", "compressible_subsonic"];
    let flow_high_mach = ["supersonic", "hypersonic_continuum"];

    // Ses altı rejimler ile süpersonik/hipersonik rejimler ayrık kümelerdir
    if (flow_subsonic.contains(&r1) && flow_high_mach.contains(&r2))
        || (flow_subsonic.contains(&r2) && flow_high_mach.contains(&r1))
    {
        return false;
    }

    // Süpersonik (1 < M < 5) ile hipersonik (M >= 5) ayrık Mach bölgeleridir
    if (r1 == "supersonic" && r2 == "hypersonic_continuum")
        || (r2 == "supersonic" && r1 == "hypersonic_continuum")
    {
        return false;
    }

    // Sürekli ortam (continuum) akışkan yasaları ile serbest moleküler (Kn > 10) seyreltilmiş rejim ayrıktır
    let continuum_regimes = [
        "incompressible_subsonic",
        "compressible_subsonic",
        "supersonic",
        "hypersonic_continuum",
        "continuum_ideal_gas",
        "laminar_continuum",
    ];
    if (continuum_regimes.contains(&r1) && r2 == "free_molecular_rarefied")
        || (continuum_regimes.contains(&r2) && r1 == "free_molecular_rarefied")
    {
        return false;
    }

    // Klasik göreli-olmayan (v <= 0.1c) ile ultra-göreli (v >= 0.9c) ayrık hız bölgeleridir
    if (r1 == "classical_nonrelativistic" && r2 == "ultrarelativistic")
        || (r2 == "classical_nonrelativistic" && r1 == "ultrarelativistic")
    {
        return false;
    }

    true
}

#[derive(Debug, Clone, Deserialize)]
struct RawLawJson {
    id: String,
    name: String,
    #[serde(default)]
    domain: String,
    #[serde(default = "default_regime")]
    regime: String,
    #[serde(default)]
    latex: String,
    equation: String,
    target: String,
    inputs: Vec<String>,
    dimensions: HashMap<String, String>,
    #[serde(default)]
    domain_bounds: HashMap<String, VarBound>,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Clone)]
pub struct Law {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub regime: String,
    pub latex: String,
    pub equation: String,
    pub target: String,
    pub inputs: Vec<String>,
    pub rhs: Expr,
    pub dimensions: HashMap<String, Dimension>,
    pub domain_guards: Vec<String>,
    pub domain_bounds: HashMap<String, VarBound>,
    pub description: String,
}

impl Law {
    /// Verilen değişken bağlamalarının yasanın fiziksel geçerlilik sınırlarına (`domain_bounds`)
    /// uygunluğunu denetler.
    pub fn check_validity_bounds(&self, bindings: &HashMap<&str, f64>) -> Result<(), String> {
        for (var_name, bound) in &self.domain_bounds {
            if let Some(&val) = bindings.get(var_name.as_str()) {
                bound
                    .check_value(var_name, val)
                    .map_err(|e| format!("[{} | rejim={}] {}", self.id, self.regime, e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SynthesizedFormula {
    pub target: String,
    pub chain_ids: Vec<String>,
    pub chain_names: Vec<String>,
    pub regimes: Vec<String>,
    pub symbolic_expr: Expr,
    pub inferred_dimension: Dimension,
    pub free_variables: Vec<String>,
    pub domain_guards: Vec<String>,
    pub effective_bounds: HashMap<String, VarBound>,
}

struct ConjunctiveSearchCtx {
    max_depth: usize,
    results: Vec<Vec<String>>,
}

pub struct KnowledgeGraph {
    pub laws: HashMap<String, Law>,
    pub law_order: Vec<String>,
    pub producers: HashMap<String, Vec<String>>,
    pub variable_dimensions: HashMap<String, Dimension>,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        let mut kg = Self {
            laws: HashMap::new(),
            law_order: Vec::new(),
            producers: HashMap::new(),
            variable_dimensions: HashMap::new(),
        };
        let manifest_db = Path::new(env!("CARGO_MANIFEST_DIR")).join("../database/canonical_laws.json");
        let cwd_db = Path::new("database/canonical_laws.json");
        let json_content = if manifest_db.exists() {
            fs::read_to_string(&manifest_db).unwrap_or_else(|_| EMBEDDED_CANONICAL_LAWS_JSON.to_string())
        } else if cwd_db.exists() {
            fs::read_to_string(cwd_db).unwrap_or_else(|_| EMBEDDED_CANONICAL_LAWS_JSON.to_string())
        } else {
            EMBEDDED_CANONICAL_LAWS_JSON.to_string()
        };
        kg.load_from_json_str(&json_content)
            .expect("canonical_laws.json yüklenirken kritik hata");
        kg
    }

    pub fn load_from_json_str(&mut self, json_str: &str) -> Result<usize, String> {
        let raw_laws: Vec<RawLawJson> = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON ayrıştırma hatası: {}", e))?;

        let mut seen_ids = HashSet::new();
        let mut count = 0usize;
        for raw in raw_laws {
            if !seen_ids.insert(raw.id.clone()) {
                return Err(format!("Kopya kanonik yasa ID'si tespit edildi: '{}'", raw.id));
            }

            let rhs = Expr::parse_str(&raw.equation)
                .map_err(|e| format!("{}: Denklem '{}' ayrıştırılamadı: {}", raw.id, raw.equation, e))?;

            let mut dims = HashMap::new();
            for (var_name, dim_key) in &raw.dimensions {
                let d = Dimension::from_name(dim_key).ok_or_else(|| {
                    format!("{}: Değişken '{}' için tanımsız boyut anahtarı '{}'", raw.id, var_name, dim_key)
                })?;
                // Değişmez 6 (SSOT Ontoloji Kalkanı): Aynı sembol iki farklı yasada farklı boyutta olamaz!
                if let Some(&existing_dim) = self.variable_dimensions.get(var_name) {
                    if existing_dim != d {
                        return Err(format!(
                            "SSOT Ontoloji İhlali ({}): Sembol '{}' daha önce [{}] boyutuyla tanımlandı, burada [{}] olarak bildirildi!",
                            raw.id, var_name, existing_dim, d
                        ));
                    }
                }
                dims.insert(var_name.clone(), d);
                self.variable_dimensions.insert(var_name.clone(), d);
            }

            for (b_var, bound) in &raw.domain_bounds {
                if !dims.contains_key(b_var) {
                    return Err(format!(
                        "{}: domain_bounds içinde tanımsız sembol '{}'!",
                        raw.id, b_var
                    ));
                }
                if let (Some(lo), Some(hi)) = (bound.min, bound.max) {
                    if lo > hi || ((lo - hi).abs() <= 1e-15 && (!bound.min_inclusive || !bound.max_inclusive)) {
                        return Err(format!(
                            "{}: Değişken '{}' için geçersiz/boş domain_bounds aralığı [{}, {}]!",
                            raw.id, b_var, lo, hi
                        ));
                    }
                }
            }

            let domain_guards = rhs
                .extract_domain_conditions()
                .map_err(|e| format!("{}: {}", raw.id, e))?;

            self.add_law(Law {
                id: raw.id,
                name: raw.name,
                domain: raw.domain,
                regime: if raw.regime.is_empty() {
                    "universal".to_string()
                } else {
                    raw.regime
                },
                latex: raw.latex,
                equation: raw.equation,
                target: raw.target,
                inputs: raw.inputs,
                rhs,
                dimensions: dims,
                domain_guards,
                domain_bounds: raw.domain_bounds,
                description: raw.description,
            });
            count += 1;
        }
        Ok(count)
    }

    pub fn add_law(&mut self, law: Law) {
        let target = law.target.clone();
        let id = law.id.clone();
        if !self.law_order.contains(&id) {
            self.law_order.push(id.clone());
        }
        self.producers.entry(target).or_default().push(id.clone());
        self.laws.insert(id, law);
    }

    /// Verilen yasa zincirindeki tüm rejimlerin ve `domain_bounds` aralıklarının birbiriyle
    /// çelişkisiz kesiştiğini doğrular; boş kesişim varsa `Err` ile zinciri budar.
    pub fn validate_and_intersect_chain_bounds(
        &self,
        plan: &[String],
    ) -> Result<(Vec<String>, HashMap<String, VarBound>), String> {
        let mut regimes: Vec<String> = Vec::new();
        let mut effective_bounds: HashMap<String, VarBound> = HashMap::new();

        for eq_id in plan {
            let Some(law) = self.laws.get(eq_id) else {
                return Err(format!("Tanımsız yasa ID'si: {}", eq_id));
            };

            for prev_r in &regimes {
                if !are_regimes_compatible(prev_r, &law.regime) {
                    return Err(format!(
                        "Rejim çatışması ({}): '{}' rejimi ile '{}' rejimi fiziksel olarak bağdaşmaz!",
                        eq_id, prev_r, law.regime
                    ));
                }
            }
            if !law.regime.is_empty() && !regimes.contains(&law.regime) {
                regimes.push(law.regime.clone());
            }

            for (var_name, bound) in &law.domain_bounds {
                if let Some(existing) = effective_bounds.get(var_name) {
                    let intersected = existing.intersect(bound, var_name)?;
                    effective_bounds.insert(var_name.clone(), intersected);
                } else {
                    effective_bounds.insert(var_name.clone(), bound.clone());
                }
            }
        }

        Ok((regimes, effective_bounds))
    }

    /// Faz 3: Graf içindeki kanonik yasaları `Expr::solve_for` ile cebirsel olarak tersine çevirir,
    /// her ters yasanın Z^7 boyut homojenliğini doğrular ve grafa çift-yönlü (bidirectional) kenarlar olarak ekler.
    pub fn register_algebraic_inversions(&mut self) -> usize {
        let base_ids = self.law_order.clone();
        let mut inverted_count = 0usize;

        for id in base_ids {
            let law = self.laws[&id].clone();
            let lhs_var = Expr::var(&law.target);

            for input_var in &law.inputs {
                let inv_id = format!("{}__inv_{}", law.id, input_var);
                if self.laws.contains_key(&inv_id) {
                    continue;
                }

                if let Ok((inv_rhs, guards)) = Expr::solve_for(&lhs_var, &law.rhs, input_var) {
                    // Terslenen yasanın Z^7 boyut homojenliğini cebirsel olarak kanıtla
                    if let (Some(&expected_dim), Ok(inferred_dim)) = (
                        law.dimensions.get(input_var),
                        inv_rhs.infer_dimension(&law.dimensions),
                    ) {
                        if expected_dim == inferred_dim {
                            let new_inputs = inv_rhs.free_variables();
                            let inv_law = Law {
                                id: inv_id,
                                name: format!("{} [Ters Çözüm: {}]", law.name, input_var),
                                domain: law.domain.clone(),
                                regime: law.regime.clone(),
                                latex: String::new(),
                                equation: format!("{} = {}", input_var, inv_rhs),
                                target: input_var.clone(),
                                inputs: new_inputs,
                                rhs: inv_rhs,
                                dimensions: law.dimensions.clone(),
                                domain_guards: guards,
                                domain_bounds: law.domain_bounds.clone(),
                                description: format!(
                                    "{} yasasının '{}' için sembolik ters çözümü",
                                    law.id, input_var
                                ),
                            };
                            self.add_law(inv_law);
                            inverted_count += 1;
                        }
                    }
                }
            }
        }
        inverted_count
    }

    /// Graf içindeki tüm yasaların:
    /// 1) Tekil ID ve SSOT sembol-boyut sözleşmesini,
    /// 2) Sembolik girdi/çıktı tamlığını (asılı veya kullanılmayan sembol olmadığını),
    /// 3) Z^7 üzerinde AST boyut homojenliğini (`dim(LHS) == dim(RHS)`)
    ///    cebirsel olarak doğrular.
    pub fn verify_all_laws(&self) -> Result<usize, String> {
        let mut verified = 0usize;
        let mut global_var_dims: HashMap<&str, (Dimension, &str)> = HashMap::new();

        for id in &self.law_order {
            let law = &self.laws[id];
            for (v_name, &v_dim) in &law.dimensions {
                if let Some(&(prev_dim, prev_id)) = global_var_dims.get(v_name.as_str()) {
                    if prev_dim != v_dim {
                        return Err(format!(
                            "SSOT Sembol-Boyut Çakışması: '{}' sembolü '{}' içinde [{}], '{}' içinde [{}]!",
                            v_name, prev_id, prev_dim, id, v_dim
                        ));
                    }
                } else {
                    global_var_dims.insert(v_name.as_str(), (v_dim, id.as_str()));
                }
            }

            let lhs_dim = law
                .dimensions
                .get(&law.target)
                .copied()
                .ok_or_else(|| format!("{}: Hedef '{}' boyutu tanımsız!", id, law.target))?;

            // Serbest değişken tamlık kontrolü
            let free_vars: HashSet<String> = law.rhs.free_variables().into_iter().collect();
            let declared_vars: HashSet<String> = law.inputs.iter().cloned().collect();
            if free_vars != declared_vars {
                return Err(format!(
                    "{}: Sembol sözleşmesi ihlali! AST serbest değişkenleri {:?} != Bildirilen girdiler {:?}",
                    id, free_vars, declared_vars
                ));
            }

            let rhs_dim = law
                .rhs
                .infer_dimension(&law.dimensions)
                .map_err(|e| format!("{}: {}", id, e))?;
            if lhs_dim != rhs_dim {
                return Err(format!(
                    "{}: Boyut Uyuşmazlığı! LHS({})=[{}] != RHS=[{}]",
                    id, law.target, lhs_dim, rhs_dim
                ));
            }
            verified += 1;
        }
        Ok(verified)
    }

    /// Hedef bir değişkene ulaşan ve fiziksel rejim/geçerlilik bölgesi çelişkisi içermeyen
    /// tüm geçerli AND-OR hipergraf türetim planlarını bulur.
    pub fn search_paths_to_target(
        &self,
        available_inputs: &HashSet<&str>,
        target_goal: &str,
        max_depth: usize,
    ) -> Vec<Vec<String>> {
        let known: HashSet<String> = available_inputs.iter().map(|s| (*s).to_string()).collect();
        let visiting: HashSet<String> = HashSet::new();
        let mut ctx = ConjunctiveSearchCtx {
            max_depth,
            results: Vec::new(),
        };

        self.solve_conjunctive(
            &[target_goal.to_string()],
            &known,
            Vec::new(),
            &visiting,
            0,
            &mut ctx,
        );
        ctx.results
    }

    /// Bir hedef nicelik için AND-OR hipergrafında bulunan tüm rejim-uyumlu denklem zincirlerini
    /// sembolik olarak birleştirir, sadeleştirir, geçerlilik aralıklarını kesiştirir ve Z^7 boyutunu doğrular.
    pub fn synthesize_formulas(
        &self,
        available_inputs: &HashSet<&str>,
        target_goal: &str,
        max_depth: usize,
    ) -> Vec<SynthesizedFormula> {
        let paths = self.search_paths_to_target(available_inputs, target_goal, max_depth);
        let mut out = Vec::new();
        let mut seen_exprs = HashSet::new();

        for path in paths {
            // Faz 1.3: Fiziksel rejim ve DomainBounds aralık kesişimi denetimi (boş kesişim derhal budanır!)
            let Ok((regimes, effective_bounds)) = self.validate_and_intersect_chain_bounds(&path) else {
                continue;
            };

            let mut env: HashMap<String, Expr> = HashMap::new();
            let mut chain_names = Vec::new();
            let mut combined_dims = self.variable_dimensions.clone();
            let mut inherited_guards = Vec::new();

            for eq_id in &path {
                if let Some(law) = self.laws.get(eq_id) {
                    chain_names.push(law.name.clone());
                    for (k, v) in &law.dimensions {
                        combined_dims.insert(k.clone(), *v);
                    }
                    let mut expr = law.rhs.clone();
                    for (sub_var, sub_expr) in &env {
                        expr = expr.substitute(sub_var, sub_expr);
                    }
                    // Yasanın taşıdığı tüm yan koşulları AST düzeyinde ikame ederek (alt-dize çakışmasız) miras al
                    for g in &law.domain_guards {
                        let parsed_guard = g
                            .strip_suffix(" != 0")
                            .map(|lhs| (lhs, " != 0"))
                            .or_else(|| g.strip_suffix(" >= 0").map(|lhs| (lhs, " >= 0")))
                            .or_else(|| g.strip_suffix(" > 0").map(|lhs| (lhs, " > 0")));

                        let g_sub = if let Some((lhs_str, suffix)) = parsed_guard {
                            if let Ok(mut g_ast) = Expr::parse_str(lhs_str) {
                                for (sub_var, sub_expr) in &env {
                                    g_ast = g_ast.substitute(sub_var, sub_expr);
                                }
                                format!("{}{}", g_ast.simplify(), suffix)
                            } else {
                                g.clone()
                            }
                        } else {
                            g.clone()
                        };

                        if !inherited_guards.contains(&g_sub) {
                            inherited_guards.push(g_sub);
                        }
                    }
                    env.insert(law.target.clone(), expr.simplify());
                }
            }

            if let Some(final_expr) = env.get(target_goal) {
                let simplified = final_expr.simplify();
                let repr = format!("{}", simplified);
                if seen_exprs.contains(&repr) {
                    continue;
                }
                if let (Some(&target_dim), Ok(inferred_dim), Ok(mut domain_guards)) = (
                    combined_dims.get(target_goal),
                    simplified.infer_dimension(&combined_dims),
                    simplified.extract_domain_conditions(),
                ) {
                    if target_dim == inferred_dim {
                        for ig in inherited_guards {
                            if !domain_guards.contains(&ig) {
                                domain_guards.push(ig);
                            }
                        }
                        seen_exprs.insert(repr);
                        out.push(SynthesizedFormula {
                            target: target_goal.to_string(),
                            chain_ids: path,
                            chain_names,
                            regimes,
                            free_variables: simplified.free_variables(),
                            symbolic_expr: simplified,
                            inferred_dimension: inferred_dim,
                            domain_guards,
                            effective_bounds,
                        });
                    }
                }
            }
        }
        out
    }

    fn solve_conjunctive(
        &self,
        goals: &[String],
        known: &HashSet<String>,
        plan: Vec<String>,
        visiting: &HashSet<String>,
        depth: usize,
        ctx: &mut ConjunctiveSearchCtx,
    ) {
        let remaining: Vec<String> = goals
            .iter()
            .filter(|g| !known.contains(*g))
            .cloned()
            .collect();

        if remaining.is_empty() {
            if !plan.is_empty()
                && !ctx.results.contains(&plan)
                && self.validate_and_intersect_chain_bounds(&plan).is_ok()
            {
                ctx.results.push(plan);
            }
            return;
        }

        if depth > ctx.max_depth {
            return;
        }

        let current_goal = &remaining[0];
        let rest_goals = &remaining[1..];

        if visiting.contains(current_goal) {
            return;
        }

        if let Some(eq_ids) = self.producers.get(current_goal) {
            for eq_id in eq_ids {
                let base_eq_id = eq_id.split("__inv_").next().unwrap_or(eq_id);
                if plan
                    .iter()
                    .any(|p| p.split("__inv_").next().unwrap_or(p.as_str()) == base_eq_id)
                {
                    continue;
                }

                // Aday denklemi mevcut plana eklemeden önce rejim ve geçerlilik bölgesi çelişkisini erken buda!
                let mut candidate_plan = plan.clone();
                candidate_plan.push(eq_id.clone());
                if self.validate_and_intersect_chain_bounds(&candidate_plan).is_err() {
                    continue;
                }

                let law = &self.laws[eq_id];
                let missing: Vec<String> = law
                    .inputs
                    .iter()
                    .filter(|inp| !known.contains(*inp))
                    .cloned()
                    .collect();

                if missing.iter().any(|m| visiting.contains(m)) {
                    continue;
                }

                if missing.is_empty() {
                    let mut new_known = known.clone();
                    new_known.insert(law.target.clone());
                    self.solve_conjunctive(
                        rest_goals,
                        &new_known,
                        candidate_plan,
                        visiting,
                        depth,
                        ctx,
                    );
                } else {
                    let mut sub_ctx = ConjunctiveSearchCtx {
                        max_depth: ctx.max_depth,
                        results: Vec::new(),
                    };
                    let mut next_visiting = visiting.clone();
                    next_visiting.insert(current_goal.clone());

                    self.solve_conjunctive(
                        &missing,
                        known,
                        plan.clone(),
                        &next_visiting,
                        depth + 1,
                        &mut sub_ctx,
                    );

                    for mut sub_plan in sub_ctx.results {
                        if sub_plan
                            .iter()
                            .any(|p| p.split("__inv_").next().unwrap_or(p.as_str()) == base_eq_id)
                        {
                            continue;
                        }
                        sub_plan.push(eq_id.clone());
                        if self.validate_and_intersect_chain_bounds(&sub_plan).is_err() {
                            continue;
                        }

                        let mut new_known = known.clone();
                        for step_id in &sub_plan {
                            if let Some(step_law) = self.laws.get(step_id) {
                                new_known.insert(step_law.target.clone());
                            }
                        }
                        self.solve_conjunctive(
                            rest_goals,
                            &new_known,
                            sub_plan,
                            visiting,
                            depth + 1,
                            ctx,
                        );
                    }
                }
            }
        }
    }
}
