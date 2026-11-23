// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Faz 4 & Faz 5: Doğrulanmış Sıfır-Panik Rust Kodu Üreticisi (Verified Codegen)
// ve Denetlenebilir Mühendislik Tasarım Föyü (Engineering Dossier) Jeneratörü

use crate::ast::Expr;
use crate::designer::InverseDesignReport;
use crate::graph::SynthesizedFormula;

pub struct VerifiedCodegen;

impl VerifiedCodegen {
    /// Sembolik bir `Expr` ağacını geçerli, parantezli Rust `f64` aritmetik ifadesine derler.
    pub fn expr_to_rust_code(expr: &Expr) -> String {
        match expr {
            Expr::Constant(c) => {
                if (*c - c.round()).abs() < 1e-15 {
                    format!("{:.1}_f64", c)
                } else {
                    format!("{}_f64", c)
                }
            }
            Expr::Variable(v) => v.clone(),
            Expr::Add(l, r) => format!("({} + {})", Self::expr_to_rust_code(l), Self::expr_to_rust_code(r)),
            Expr::Sub(l, r) => format!("({} - {})", Self::expr_to_rust_code(l), Self::expr_to_rust_code(r)),
            Expr::Mul(l, r) => format!("({} * {})", Self::expr_to_rust_code(l), Self::expr_to_rust_code(r)),
            Expr::Div(l, r) => format!("({} / {})", Self::expr_to_rust_code(l), Self::expr_to_rust_code(r)),
            Expr::Pow(b, e) => {
                let se = e.simplify();
                if let Expr::Constant(p) = se {
                    if (p - p.round()).abs() < 1e-12 && p.abs() <= 16.0 {
                        return format!("{}.powi({})", Self::expr_to_rust_code(b), p.round() as i32);
                    }
                }
                format!("{}.powf({})", Self::expr_to_rust_code(b), Self::expr_to_rust_code(&se))
            }
            Expr::Sqrt(inner) => format!("{}.sqrt()", Self::expr_to_rust_code(inner)),
            Expr::Ln(inner) => format!("{}.ln()", Self::expr_to_rust_code(inner)),
            Expr::Exp(inner) => format!("{}.exp()", Self::expr_to_rust_code(inner)),
            Expr::Neg(inner) => format!("(-{})", Self::expr_to_rust_code(inner)),
        }
    }

    /// Sembolik olarak sentezlenmiş ve Z^7 boyutu kanıtlanmış bir formülü,
    /// Citadel-Math standartlarına uygun (`#![forbid(unsafe_code)]`, sıfır-panik, `Result<f64, &'static str>`)
    /// ve çalışma zamanı tanım kümesi bekçilerini (`domain_guards`) zorunlu koşan Rust fonksiyonuna derler.
    pub fn emit_safe_rust_fn(formula: &SynthesizedFormula) -> String {
        Self::emit_safe_rust_fn_with_meta(formula, true, None)
    }

    /// Boyut doğrulama durumu (`dim_verified`) ve opsiyonel kök dalı uyarısıyla (`branch_warning`)
    /// sıfır-panik Rust fonksiyonu üretir. Doğrulanmamış boyut asla `[1 (Boyutsuz)]` olarak damgalanmaz.
    pub fn emit_safe_rust_fn_with_meta(
        formula: &SynthesizedFormula,
        dim_verified: bool,
        branch_warning: Option<&str>,
    ) -> String {
        let mut sorted_vars = formula.free_variables.clone();
        sorted_vars.sort();
        let params: Vec<String> = sorted_vars.iter().map(|v| format!("{}: f64", v)).collect();

        let mut code = String::new();
        code.push_str(&format!(
            "/// Otomatik Sentezlenen Doğrulanmış Fizik Fonksiyonu: `{}`\n",
            formula.target
        ));
        code.push_str(&format!(
            "/// - Kanonik Denklem : `{} = {}`\n",
            formula.target, formula.symbolic_expr
        ));
        if dim_verified {
            code.push_str(&format!(
                "/// - Z^7 Boyut İspatı: `[{}]`\n",
                formula.inferred_dimension
            ));
        } else {
            code.push_str(
                "/// - Z^7 Boyut İspatı: `[Bilinmiyor / Doğrulanmadı — SSOT Ontoloji Dışı Sembol]`\n",
            );
        }
        if let Some(warn_msg) = branch_warning {
            code.push_str(&format!("/// - Dal Varsayımı   : {}\n", warn_msg));
        }
        code.push_str(&format!(
            "/// - Türetim Zinciri : `{}`\n",
            formula.chain_ids.join(" -> ")
        ));
        if !formula.regimes.is_empty() {
            code.push_str(&format!(
                "/// - Fiziksel Rejim  : `{}`\n",
                formula.regimes.join(" ∩ ")
            ));
        }
        if !formula.domain_guards.is_empty() {
            code.push_str(&format!(
                "/// - Yan Koşullar    : `{}`\n",
                formula.domain_guards.join(", ")
            ));
        }
        code.push_str(&format!(
            "pub fn compute_{}({}) -> Result<f64, &'static str> {{\n",
            formula.target.to_lowercase(),
            params.join(", ")
        ));

        // Girdi NaN/Inf ve Faz 1.3 Fiziksel Geçerlilik Bölgesi (DomainBounds) kontrolleri
        for v in &sorted_vars {
            code.push_str(&format!(
                "    if !{0}.is_finite() {{ return Err(\"Girdi sonlu bir reel sayı değil: {0}\"); }}\n",
                v
            ));
            if let Some(bound) = formula.effective_bounds.get(v) {
                if let Some(lo) = bound.min {
                    let cmp = if bound.min_inclusive { "<" } else { "<=" };
                    let expected = if bound.min_inclusive { ">=" } else { ">" };
                    code.push_str(&format!(
                        "    if {0} {1} {2:?}_f64 {{ return Err(\"Fiziksel geçerlilik alt sınırı ihlali: {0} ({3} {2:?})\"); }}\n",
                        v, cmp, lo, expected
                    ));
                }
                if let Some(hi) = bound.max {
                    let cmp = if bound.max_inclusive { ">" } else { ">=" };
                    let expected = if bound.max_inclusive { "<=" } else { "<" };
                    code.push_str(&format!(
                        "    if {0} {1} {2:?}_f64 {{ return Err(\"Fiziksel geçerlilik üst sınırı ihlali: {0} ({3} {2:?})\"); }}\n",
                        v, cmp, hi, expected
                    ));
                }
            }
        }

        // Tanım kümesi yan koşul bekçileri (Domain Guards):
        // Sadece girdi parametrelerine bağlı olanlar önkoşul (pre-guard),
        // hedef çıktı değişkenine (`formula.target`) bağlı olanlar ise `result` ikamesiyle sonkoşul (post-guard) olarak derlenir.
        let mut post_guards: Vec<(String, String)> = Vec::new();
        for g in &formula.domain_guards {
            let parsed_guard = g
                .strip_suffix(" != 0")
                .map(|lhs| (lhs, "!= 0"))
                .or_else(|| g.strip_suffix(" >= 0").map(|lhs| (lhs, ">= 0")))
                .or_else(|| g.strip_suffix(" > 0").map(|lhs| (lhs, "> 0")));

            if let Some((lhs, op)) = parsed_guard {
                if let Ok(g_expr) = Expr::parse_str(lhs) {
                    let g_vars = g_expr.free_variables();
                    let all_inputs = g_vars.iter().all(|v| sorted_vars.contains(v));
                    if all_inputs {
                        let rust_g = Self::expr_to_rust_code(&g_expr);
                        match op {
                            "!= 0" => code.push_str(&format!(
                                "    if ({}).abs() <= 1e-15_f64 {{ return Err(\"Tanım kümesi ihlali: {}\"); }}\n",
                                rust_g, g
                            )),
                            "> 0" => code.push_str(&format!(
                                "    if ({}) <= 0.0_f64 {{ return Err(\"Tanım kümesi ihlali: {}\"); }}\n",
                                rust_g, g
                            )),
                            ">= 0" => code.push_str(&format!(
                                "    if ({}) < 0.0_f64 {{ return Err(\"Tanım kümesi ihlali: {}\"); }}\n",
                                rust_g, g
                            )),
                            _ => {}
                        }
                    } else if g_expr.contains_var(&formula.target)
                        && g_vars
                            .iter()
                            .all(|v| v == &formula.target || sorted_vars.contains(v))
                    {
                        let post_expr = g_expr.substitute(&formula.target, &Expr::var("result"));
                        let rust_post = Self::expr_to_rust_code(&post_expr);
                        post_guards.push((rust_post, format!("{}: {}", op, g)));
                    }
                }
            }
        }

        let body_expr = Self::expr_to_rust_code(&formula.symbolic_expr);
        code.push_str(&format!("    let result = {};\n", body_expr));
        code.push_str("    if !result.is_finite() {\n");
        code.push_str("        return Err(\"Hesaplama taşması veya tanımsız aritmetik sonuç!\");\n");
        code.push_str("    }\n");
        for (rust_post, op_and_label) in &post_guards {
            if let Some(label) = op_and_label.strip_prefix("!= 0: ") {
                code.push_str(&format!(
                    "    if ({}).abs() <= 1e-15_f64 {{ return Err(\"Sonuç tanım kümesi ihlali: {}\"); }}\n",
                    rust_post, label
                ));
            } else if let Some(label) = op_and_label.strip_prefix("> 0: ") {
                code.push_str(&format!(
                    "    if ({}) <= 0.0_f64 {{ return Err(\"Sonuç tanım kümesi ihlali: {}\"); }}\n",
                    rust_post, label
                ));
            } else if let Some(label) = op_and_label.strip_prefix(">= 0: ") {
                code.push_str(&format!(
                    "    if ({}) < 0.0_f64 {{ return Err(\"Sonuç tanım kümesi ihlali: {}\"); }}\n",
                    rust_post, label
                ));
            }
        }
        code.push_str("    Ok(result)\n");
        code.push_str("}\n");
        code
    }

    /// Faz 5: Tersine tasarım ve Pareto optimizasyon sonuçlarını denetlenebilir Markdown teknik föyüne dönüştürür.
    pub fn generate_markdown_dossier(report: &InverseDesignReport) -> String {
        let mut md = String::new();
        md.push_str("# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).\n");
        md.push_str("# CITADEL-MATH DOĞRULANMIŞ MÜHENDİSLİK TASARIM VE KANIT FÖYÜ\n\n");
        md.push_str(&format!(
            "- **Görev Kısıtları:** Faydalı Yük = `{:.0} kg` | Hedef LEO Yörünge >= `{:.0} km` | Azami İvme <= `{:.1} G` | Minimum Yapısal Emniyet >= `{:.2}x`\n",
            report.constraints.payload_kg,
            report.constraints.target_orbit_km,
            report.constraints.max_g_limit,
            report.constraints.min_safety_factor
        ));
        md.push_str(&format!(
            "- **Taranan Tasarım Uzayı:** `{}` aday konfigürasyon (`{}` Kepler enberi kararlılığı, `{}` G-limiti, `{}` Von Mises + Euler/SP-8007 + PICA-X TPS testini geçti)\n\n",
            report.total_candidates_evaluated,
            report.orbit_passed_count,
            report.g_limit_passed_count,
            report.structural_passed_count
        ));

        md.push_str("## 1. Hipergraf Sembolik Denklem Sentezi ve Z^7 Boyut İspatları\n\n");
        md.push_str("| Hedef | Sentezlenen Sembolik Denklem | Z^7 Boyut | Zorunlu Tanım Kümesi Yan Koşulları | Türetim Zinciri |\n");
        md.push_str("| :--- | :--- | :---: | :--- | :--- |\n");
        for f in &report.synthesized_laws {
            md.push_str(&format!(
                "| `{}` | `{} = {}` | `[{}]` | `{}` | `{}` |\n",
                f.target,
                f.target,
                f.symbolic_expr,
                f.inferred_dimension,
                f.domain_guards.join(", "),
                f.chain_ids.join(" -> ")
            ));
        }

        md.push_str("\n## 2. Çok Amaçlı NSGA-II / Kesin Pareto Skyline Cephesi (Rank 0)\n\n");
        if report.pareto_summary.total_feasible > 0 {
            let u = &report.pareto_summary.utopia_point;
            let n = &report.pareto_summary.nadir_point;
            md.push_str(&format!(
                "- **NSGA-II Cephe Ayrışımı:** `{}` uygun tasarım -> `{}` cephe | **Kesin Pareto Skyline (`Rank 0`):** `{}` baskılanmamış tasarım | **Normalize 2B Hiperhacim:** `{:.4}`\n",
                report.pareto_summary.total_feasible,
                report.pareto_summary.num_fronts,
                report.pareto_summary.skyline_count,
                report.pareto_summary.normalized_hypervolume_2d.unwrap_or(0.0)
            ));
            if u.len() == 4 && n.len() == 4 {
                md.push_str(&format!(
                    "- **Ütopya (İdeal) Vektörü:** `GLOW = {:.1} kg`, `Max-G = {:.2} G`, `Isı Yükü = {:.2} MJ/m²`, `Min Emniyet = {:.2}x`\n",
                    u[0], u[1], u[2], u[3]
                ));
                md.push_str(&format!(
                    "- **Nadir (Skyline Sınır) Vektörü:** `GLOW = {:.1} kg`, `Max-G = {:.2} G`, `Isı Yükü = {:.2} MJ/m²`, `Min Emniyet = {:.2}x`\n\n",
                    n[0], n[1], n[2], n[3]
                ));
            }
        }
        md.push_str("| # | İtki Mimarisi | Gövde Malzemesi | Kalkış (GLOW) | Yapısal Oran | Kepler Yörüngesi ($h_p \\times h_a$, $e$) | Min Emniyet | Ütopya Mesafesi / Baskı | Interval64 İdeal $\\Delta v$ | Canlı TPS ($\\int q\\,dt$) |\n");
        md.push_str("| :-: | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n");
        for (i, d) in report.pareto_optimal_designs.iter().enumerate() {
            let knee_tag = if d.is_knee_point { " **[KNEE]**" } else { "" };
            md.push_str(&format!(
                "| **#{}** | {}{} | {} | **{:.1} kg** | %{:.1} | {:.1} x {:.1} km ({:.4}) | {:.2}x | $d_u={:.3}$ ({} aday) | [{:.1}, {:.1}] m/s | {:.2} MJ/m² -> {:.1} mm ({:.1} kg) |\n",
                i + 1,
                d.propulsion_name,
                knee_tag,
                d.material_name,
                d.gross_liftoff_mass_kg,
                d.structural_fraction_pct,
                d.perigee_altitude_km,
                d.apogee_altitude_km,
                d.eccentricity,
                d.min_structural_safety_factor,
                d.utopia_distance,
                d.dominated_count,
                d.ideal_dv_interval_ms.lo,
                d.ideal_dv_interval_ms.hi,
                d.total_heat_load_mj_m2,
                d.tps_thickness_mm,
                d.tps_mass_kg
            ));
        }

        md.push_str("\n## 3. Otomatik Üretilen Sıfır-Panik (`#![forbid(unsafe_code)]`) Rust Fonksiyonları\n\n");
        md.push_str("```rust\n#![forbid(unsafe_code)]\n\n");
        for f in &report.synthesized_laws {
            md.push_str(&Self::emit_safe_rust_fn(f));
            md.push('\n');
        }
        md.push_str("```\n");
        md
    }
}
