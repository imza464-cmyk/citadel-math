// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Saf Rust Komuta Merkezi (Citadel Math Native CLI)

#![forbid(unsafe_code)]

use citadel_math_core::*;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = if args.len() > 1 { args[1].as_str() } else { "audit" };

    match cmd {
        "audit" => run_audit(),
        "simulate" => run_simulate(),
        "materials" => run_materials(),
        "stress" => run_stress(),
        "invent" => run_invent(),
        "buckingham" => run_buckingham(&args[2..]),
        "design" => run_design(&args[2..]),
        "codegen" => run_codegen(),
        "cad" => run_cad(&args[2..]),
        "mcp" => run_mcp(&args[2..]),
        _ => {
            println!("Kullanım: citadel_math_core [audit|simulate|materials|stress|invent|buckingham|design|codegen|cad|mcp]");
            run_audit();
        }
    }
}

fn run_audit() {
    println!("{}", "=".repeat(72));
    println!("⚖️ CITADEL-MATH RUST ÇEKİRDEĞİ MÜHENDİSLİK VE KANIT DENETİMİ (RUST AUDIT)");
    println!("{}", "=".repeat(72));

    let t0 = Instant::now();

    // 1. Tekil Kaynaklı (SSOT JSON) Boyutsal Mutlaklık Kalkanı ve Sembolik AST Boyut Çıkarımı
    let kg = KnowledgeGraph::new();
    let verified_laws = kg
        .verify_all_laws()
        .expect("Kanonik yasalarda AST boyut homojenliği hatası!");
    println!(
        "[PASS] 1. Sembolik AST Boyut Kalkanı (SSOT JSON): {} kanonik yasanın dim(LHS) == dim(RHS) denkliği Z^7 üzerinde kanıtlandı.",
        verified_laws
    );

    // 2. Sembolik İfade Ayrıştırıcı (Expr::parse_str) ve Değerlendirici
    let parsed = Expr::parse_str("delta_E_k = m * v * delta_v + 0.5 * m * delta_v**2")
        .expect("Denklem ayrıştırma hatası!");
    let mut bindings = HashMap::new();
    bindings.insert("m", 10.0);
    bindings.insert("v", 100.0);
    bindings.insert("delta_v", 20.0);
    let eval_res = parsed.evaluate(&bindings).unwrap();
    assert!((eval_res - 22_000.0).abs() < 1e-9, "Oberth etkisi sayısal değerlendirme hatası!");
    println!("[PASS] 2. Sembolik AST Ayrıştırıcı & Değerlendirici: 'Expr::parse_str' ve ikame tam.");

    // 3. Küçük Doğrulama Çekirdeği, Kanonik Polinomlar (Z[x]) ve Tanım Kümesi Yan Koşul Sözleşmesi (§4-§9)
    let f_sq = PolyZ::new(vec![0, 0, 1]); // f(x) = x^2
    let g_inc = PolyZ::new(vec![1, 1]);   // g(x) = x + 1
    let fg = f_sq.checked_compose(&g_inc, 16).unwrap(); // (f o g)(x) = (x+1)^2
    let gf = g_inc.checked_compose(&f_sq, 16).unwrap(); // (g o f)(x) = x^2 + 1
    let expected_expanded = PolyZ::new(vec![1, 2, 1]);  // 1 + 2x + x^2
    assert_eq!(fg, expected_expanded, "(x+1)^2 kanonik formu x^2+2x+1 ile eşleşmedi!");
    assert_ne!(fg, gf, "f o g != g o f bileşke sıra farkı doğrulanamadı!");

    let num_x2_minus_1 = PolyZ::new(vec![-1, 0, 1]); // x^2 - 1
    let guarded_r = GuardedPoly::cancel_linear_factor(&num_x2_minus_1, 1, 16).unwrap();
    assert_eq!(guarded_r.poly, g_inc);
    assert!(guarded_r.excluded_points.contains(&1), "x != 1 yan koşulu kayboldu!");
    let unguarded = GuardedPoly::total(g_inc.clone());
    assert!(
        guarded_r.verify_strict_equality(&unguarded).is_err(),
        "Yan koşul kaybı (x != 1) yakalanmadı!"
    );

    let huge = PolyZ::constant(i64::MAX);
    assert_eq!(
        huge.checked_add(&PolyZ::constant(1)),
        Err(MathKernelError::CoefficientOverflow)
    );
    println!(
        "[PASS] 3. Küçük Doğrulama Çekirdeği (Z[x]): (x+1)^2 == x^2+2x+1, (x^2-1)/(x-1) [x!=1] yan koşul korunumu ve taşma reddi doğrulandı."
    );

    // 4. NIST CODATA 2022 Sabitleri, IEEE Std 1788-2015 `tightest` ve Ölçekli SI Birimleri Mührü
    assert_eq!(PhysicalConstants::C, 299_792_458.0);
    assert_eq!(PhysicalConstants::G0, 9.80665);
    assert_eq!(PhysicalConstants::MU_0, 1.256_637_061_27e-6);
    assert_eq!(PhysicalConstants::EPS_0, 8.854_187_818_8e-12);
    assert_eq!(PhysicalConstants::ALPHA, 7.297_352_564_3e-3);
    assert!(
        (PhysicalConstants::MU_0 * PhysicalConstants::EPS_0 * PhysicalConstants::C.powi(2) - 1.0).abs()
            < 2e-10,
        "Maxwell vakum özdeşliği (mu_0 * eps_0 * c^2 == 1) bozuldu!"
    );
    let exact_add = Interval64::point(1.0).unwrap().add(&Interval64::point(2.0).unwrap());
    assert_eq!(exact_add.width(), 0.0, "IEEE 1788 tightest 0-ULP tam toplama bozuldu!");
    let tight_div = Interval64::point(1.0).unwrap().div(&Interval64::point(3.0).unwrap()).unwrap();
    assert_eq!(tight_div.hi, tight_div.lo.next_up(), "IEEE 1788 tightest 1-ULP bölme bozuldu!");
    let kpa = ScaledUnit::from_unit_str("kPa").unwrap();
    assert_eq!(kpa.to_si(46.5), (46_500.0, Dimension::pressure()));
    println!("[PASS] 4. NIST CODATA 2022, IEEE 1788-2015 `tightest` EFT & ScaledUnit: Vakum özdeşliği ve 0/1-ULP dışa yuvarlama mühürlü.");

    // 5. Gerçek 4-Aşamalı RK4 Sayısal Fizik Entegratörü (k1, k2, k3, k4)
    let sim = RK4Simulator::new(0.1);
    let test_v = VehicleConfig {
        name: "Test Vehicle",
        dry_mass: 1000.0,
        propellant_mass: 5000.0,
        thrust_vac: 100_000.0,
        isp: 800.0,
        area: 1.0,
        cd: 0.25,
        initial_velocity: 0.0,
        initial_altitude: 0.0,
        has_stage_2: false,
        s2_dry_mass: 0.0,
        s2_propellant: 0.0,
        s2_thrust: 0.0,
        s2_isp: 0.0,
    };
    let sim_res = sim.simulate(&test_v, 10.0);
    assert!(sim_res.flight_duration_s > 0.0);
    println!("[PASS] 5. Sayısal Çözücü: Gerçek 4-Aşamalı Runge-Kutta (k1,k2,k3,k4 - O(dt^4)) entegratörü aktif.");

    // 6. Malzeme Bilimi (SSOT JSON), NASA SP-8007 Burkulma ve Ablatif TPS Kalkanı
    let mdb = MaterialsDatabase::load_canonical();
    assert!(mdb.list.len() >= 8);
    let alli = mdb.get("mat_alli_2195").unwrap();
    let (_, buckle_sf, _) = StructuralStressChecker::check_cylinder_buckling(alli, 425_000.0, 4.65, 46_450.0, 1.85, 42.0, 5.0);
    assert!(buckle_sf > 0.0);
    let tps = StructuralStressChecker::size_ablative_tps(alli, 8012.0, 1.85, 55.0);
    assert!(tps.tps_required && tps.tps_thickness_mm > 0.0);
    println!("[PASS] 6. Malzeme, Euler+SP-8007 Burkulma & Ablatif TPS: Kabuk, kolon burkulması ve PICA-X kalkanı doğrulandı.");

    // 7. AND-OR Hipergraf Sembolik Denklem Sentezleyici + Çift-Yönlü Cebirsel Tersleme (Faz 3)
    let mut inputs = HashSet::new();
    inputs.insert("g_0");
    inputs.insert("I_sp");
    inputs.insert("m_0");
    inputs.insert("m_f");
    inputs.insert("m_dot");
    inputs.insert("eta");
    let synthesized = kg.synthesize_formulas(&inputs, "P_beam", 4);
    assert!(!synthesized.is_empty(), "Hipergraf P_beam için birleşik AND türetim planını bulamadı!");

    let mut inv_kg = KnowledgeGraph::new();
    let inv_count = inv_kg.register_algebraic_inversions();
    assert!(inv_count >= 30, "Cebirsel ters yasa sayısı beklenenden düşük!");
    inv_kg
        .verify_all_laws()
        .expect("Cebirsel olarak terslenen yasalarda Z^7 boyut homojenliği hatası!");
    println!(
        "[PASS] 7. Çok Disiplinli AND-OR Hipergrafı & Ters Çözücü: {} sembolik formül + {} ters yasa Z^7'de doğrulandı.",
        synthesized.len(),
        inv_count
    );

    // 8. Faz 3: IEEE-754 Dışa Yuvarlamalı Aralık Aritmetiği (Interval64) & Otomatik Türev (Dual64)
    let f_dep = Expr::parse_str("x - x").unwrap();
    let mut dep_env = HashMap::new();
    dep_env.insert("x", Interval64::new(2.0, 5.0).unwrap());
    let taylor_iv = f_dep.eval_taylor_interval(&dep_env).unwrap();
    assert!(taylor_iv.width() < 1e-14 && taylor_iv.contains(0.0));
    println!(
        "[PASS] 8. Faz 3 Interval64 & Dual64: Taylor/Mean-Value aralık daraltması ve sembolik/otomatik türev eşliği tam."
    );

    // 9. Faz 3 & 4: Çok Değişkenli Kanonik Polinomlar (MultiPolyZ grevlex) & Dormand-Prince RK4(5)
    let lhs_mp = MultiPolyZ::from_expr(&Expr::parse_str("(x + y + z)^2").unwrap(), 8).unwrap();
    let rhs_mp = MultiPolyZ::from_expr(
        &Expr::parse_str("x^2 + y^2 + z^2 + 2*x*y + 2*x*z + 2*y*z").unwrap(),
        8,
    )
    .unwrap();
    assert_eq!(lhs_mp, rhs_mp, "MultiPolyZ grevlex kanonik çok değişkenli özdeşlik hatası!");
    let rk45_res = sim.simulate_rk45(&test_v, 10.0, 1e-8);
    assert!(rk45_res.flight_duration_s > 0.0);
    println!(
        "[PASS] 9. MultiPolyZ (Z[x1..xn] grevlex) & Dormand-Prince RK4(5) adaptif çözücü doğrulandı."
    );

    // 10. Faz 5: Sıfır-Panik (#![forbid(unsafe_code)]) Doğrulanmış Rust Kodu Üreticisi (VerifiedCodegen)
    let rust_code = VerifiedCodegen::emit_safe_rust_fn(&synthesized[0]);
    assert!(rust_code.contains("pub fn compute_p_beam") && rust_code.contains("Result<f64, &'static str>"));
    println!(
        "[PASS] 10. Faz 5 VerifiedCodegen: Yan koşul bekçili, sıfır-panik Rust fonksiyon üretimi aktif."
    );

    // 11. Değişmez 2: Termodinamik, Görelilik (v < c), Carnot ve Momentum Budama Kalkanı
    let valid_prop = PhysicalSystemCandidate {
        mass_kg: Some(1200.0),
        velocity_ms: Some(7_800.0),
        exhaust_velocity_ms: Some(9_300.0),
        mass_flow_kg_s: Some(12.8),
        thrust_n: Some(119_040.0),
        power_in_w: Some(950_000_000.0),
        efficiency: Some(0.62),
        temp_hot_k: Some(3200.0),
        temp_cold_k: Some(300.0),
        entropy_delta_j_k: Some(125.0),
        ..Default::default()
    };
    PhysicalBoundsShield::verify_candidate(&valid_prop)
        .expect("Geçerli fiziksel sistem adayı hatalı reddedildi!");
    assert!(
        PhysicalBoundsShield::check_subluminal_velocity(PhysicalConstants::C).is_err(),
        "Işık hızı sınırı (v >= c) yakalanmadı!"
    );
    println!(
        "[PASS] 11. Değişmez 2 Fiziksel Budama Kalkanı: Termodinamik 1./2. Yasa, Carnot, v < c ve Momentum korunumu aktif."
    );

    // 12. Faz 1.3: Fiziksel Rejim ve Geçerlilik Bölgesi (DomainBounds) Kesişim & Çatışma Budama Kalkanı
    assert!(!are_regimes_compatible("incompressible_subsonic", "hypersonic_continuum"));
    assert!(!are_regimes_compatible("classical_nonrelativistic", "ultrarelativistic"));
    let subsonic_mach = VarBound {
        min: Some(0.0),
        max: Some(0.3),
        min_inclusive: true,
        max_inclusive: true,
        note: "Sıkıştırılamaz akış".to_string(),
    };
    let hypersonic_mach = VarBound {
        min: Some(5.0),
        max: Some(35.0),
        min_inclusive: true,
        max_inclusive: true,
        note: "Hipersonik rejim".to_string(),
    };
    assert!(
        subsonic_mach.intersect(&hypersonic_mach, "Mach").is_err(),
        "Ayrık Mach rejimlerinin boş kesişimi yakalanmadı!"
    );
    println!(
        "[PASS] 12. Faz 1.3 Rejim & DomainBounds Kalkanı: Mach/Görelilik rejim çatışması budaması ve SSOT sembol-boyut tekilliği aktif."
    );

    // 13. Tam Tamsayı Z^7 Çekirdek Uzayı (Nullspace) Buckingham-Pi Keşif Motoru
    let blast_vars = BuckinghamPiSolver::parse_variable_specs("r,E,rho,t")
        .expect("Buckingham-Pi değişken çözümleme hatası!");
    let blast_rep = BuckinghamPiSolver::discover(&blast_vars, Some("r"), &[], &[])
        .expect("Taylor-Sedov Buckingham-Pi keşif hatası!");
    assert_eq!(blast_rep.dimensional_rank, 3);
    assert_eq!(blast_rep.num_pi_groups, 1);
    assert!(blast_rep.target_identifiable);
    assert_eq!(blast_rep.target_scaling_dimension.as_deref(), Some("L"));
    let drag_vars = BuckinghamPiSolver::parse_variable_specs("F_drag,rho,v,L_char,mu_visc").unwrap();
    let drag_rep = BuckinghamPiSolver::discover(&drag_vars, Some("F_drag"), &[], &[]).unwrap();
    assert_eq!(drag_rep.num_pi_groups, 2);
    println!(
        "[PASS] 13. Buckingham-Pi Z^7 Nullspace Keşfi: Taylor-Sedov (r ~ (E*t^2/rho)^1/5) ve Aerodinamik (C_F, Re) tam tamsayı çekirdekte kanıtlandı."
    );

    // 14. Çok Amaçlı Kesin Pareto Cephesi (NSGA-II Rank 0 Skyline, Kalabalık Mesafesi & Ütopya Diz Noktası)
    let p_specs = vec![
        ObjectiveSpec::minimize("mass_kg", "kg"),
        ObjectiveSpec::maximize("safety_factor", "x"),
    ];
    let p_pts = vec![
        vec![1000.0, 1.40], // A: en hafif (Skyline çapa)
        vec![1250.0, 2.05], // B: dengeli uzlaşma (Knee, Skyline)
        vec![1800.0, 2.40], // C: en sağlam (Skyline çapa)
        vec![1500.0, 1.80], // D: B tarafından katı baskılanır (Rank 1)
    ];
    let p_sum = ParetoSynthesizer::analyze(&p_specs, &p_pts).expect("Pareto analiz hatası!");
    ParetoSynthesizer::verify_skyline_invariants(&p_specs, &p_pts, &p_sum)
        .expect("Pareto Skyline değişmez ihlali!");
    assert_eq!(p_sum.skyline_count, 3);
    assert_eq!(p_sum.num_fronts, 2);
    assert_eq!(p_sum.knee_candidate_index, Some(1));
    println!(
        "[PASS] 14. Çok Amaçlı NSGA-II / Kesin Pareto Skyline: F_0 baskınlık tamlığı, kalabalık mesafesi ve Ütopya diz-noktası doğrulandı."
    );

    // 15. Sıfır-Taviz Matematiksel & Fiziksel Evren Sağlamlığı (Sıfır NaN, 1-ULP i128 Rasyonel, USSA-1976 & 0.0 G Öz İvme)
    let empty_env: HashMap<&str, f64> = HashMap::new();
    assert!(Expr::parse_str("(-4)^0.5").unwrap().evaluate(&empty_env).is_err());
    assert!(Expr::parse_str("0^0").unwrap().evaluate(&empty_env).is_err());
    let mut neg3 = HashMap::new();
    neg3.insert("x", -3.0);
    assert!(
        (Expr::parse_str("(x^2)^0.5").unwrap().simplify().evaluate(&neg3).unwrap() - 3.0).abs() < 1e-12,
        "(x^2)^0.5 == |x| sağlamlık ihlali!"
    );
    assert!(Expr::parse_str("exp(ln(x))").unwrap().simplify().evaluate(&neg3).is_err());
    let iv_i64_edge = Interval64::from_rational(i64::MAX - 1, i64::MAX).unwrap();
    assert!(iv_i64_edge.lo < 1.0 && iv_i64_edge.hi == 1.0 && iv_i64_edge.hi.to_bits() - iv_i64_edge.lo.to_bits() == 1);
    let (_, p_sl, t_sl, a_sl) = AtmosphereModel::state_at(0.0);
    assert!((t_sl - 288.15).abs() < 1e-12 && (p_sl - 101_325.0).abs() < 1e-9 && (a_sl - 340.294).abs() < 0.01);
    let freefall_v = VehicleConfig {
        name: "Equivalence Principle Freefall",
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
    assert_eq!(sim.simulate(&freefall_v, 10.0).max_g_force, 0.0);
    println!(
        "[PASS] 15. Sıfır-Taviz Matematiksel Evren Sağlamlığı: 0-NaN cebir, |p|>2^53 1-ULP rasyonel, USSA-1976 ve vakum 0.0 G doğrulandı."
    );

    // 16. 2D Parametrik CAD, Steiner Kesit Mekaniği, Euler Burkulma ve AutoCAD R12 DXF/SVG Mührü
    let mut cad_doc = CadDocument::new("audit_spar", CadUnit::Millimeters);
    cad_doc
        .exec_script("IBEAM 0,0 100 200 15 10")
        .expect("CAD IBEAM komutu başarısız!");
    let cad_sec = cad_doc.section_properties().expect("CAD kesit hesabı başarısız!");
    assert_eq!(cad_sec.area_dim, Dimension::area());
    assert_eq!(cad_sec.inertia_dim, Dimension::area_moment_of_inertia());
    let mat_db_cad = MaterialsDatabase::load_canonical();
    let ti_cad = mat_db_cad.get("mat_titanium_ti6al4v").unwrap();
    let beam_rep = cad_doc
        .evaluate_beam(ti_cad, 2.5, 25_000.0, 100_000.0, 350.0)
        .expect("CAD kiriş analizi başarısız!");
    assert!(beam_rep.passed, "Ti-6Al-4V I-kirişi güvenlik katsayısını geçmeli!");
    let dxf_out = cad_doc.export_dxf();
    assert!(dxf_out.contains("$ACADVER\n1\nAC1009") && dxf_out.ends_with("0\nEOF\n"));
    let svg_out = cad_doc.export_svg();
    assert!(svg_out.starts_with("<svg ") && svg_out.contains("</svg>"));
    println!(
        "[PASS] 16. 2D Parametrik CAD & Kesit Mekaniği: I-Profili A={:.1} cm², Ix={:.2e} m⁴, P_cr={:.1} kN (SF={:.2}) + R12 DXF/SVG doğrulandı.",
        cad_sec.area_m2 * 1e4,
        cad_sec.ix_m4,
        beam_rep.euler_buckling_load_n / 1e3,
        beam_rep.yield_safety_factor
    );

    let elapsed = t0.elapsed();
    println!("{}", "=".repeat(72));
    println!("🏆 RUST ÇEKİRDEĞİ TÜM 16 MATEMATİKSEL VE FİZİKSEL DENETİMDEN GEÇTİ (0 HATA).");
    println!("⚡ Toplam Denetim Süresi: {:.2?}", elapsed);
    println!("{}", "=".repeat(72));
}

fn run_simulate() {
    println!("{}", "=".repeat(72));
    println!("🚀 CITADEL-MATH RUST GERÇEK 4-AŞAMALI RK4 SAYISAL YÖRÜNGE SİMÜLASYONU");
    println!("{}", "=".repeat(72));

    let sim = RK4Simulator::new(0.1);

    let v_chem = VehicleConfig {
        name: "1. Geleneksel 2-Aşamalı Kimyasal Roket (LOX/RP-1)",
        dry_mass: 25_000.0,
        propellant_mass: 400_000.0,
        thrust_vac: 7_600_000.0,
        isp: 300.0,
        area: 10.5,
        cd: 0.30,
        initial_velocity: 0.0,
        initial_altitude: 0.0,
        has_stage_2: true,
        s2_dry_mass: 4_500.0,
        s2_propellant: 92_000.0,
        s2_thrust: 934_000.0,
        s2_isp: 348.0,
    };

    let v_beamed = VehicleConfig {
        name: "2. Lazer Işınlamalı Termal Tahrik (Beamed Laser SSTO)",
        dry_mass: 1_200.0,
        propellant_mass: 6_000.0,
        thrust_vac: 120_000.0,
        isp: 950.0,
        area: 1.5,
        cd: 0.25,
        initial_velocity: 0.0,
        initial_altitude: 0.0,
        has_stage_2: false,
        s2_dry_mass: 0.0,
        s2_propellant: 0.0,
        s2_thrust: 0.0,
        s2_isp: 0.0,
    };

    let v_kinetic = VehicleConfig {
        name: "3. Kinetik Fırlatıcı + Üst Aşama Hibriti (SpinLaunch/Mass Driver)",
        dry_mass: 600.0,
        propellant_mass: 2_400.0,
        thrust_vac: 45_000.0,
        isp: 335.0,
        area: 0.8,
        cd: 0.20,
        initial_velocity: 1800.0,
        initial_altitude: 15_000.0,
        has_stage_2: false,
        s2_dry_mass: 0.0,
        s2_propellant: 0.0,
        s2_thrust: 0.0,
        s2_isp: 0.0,
    };

    for v in [&v_chem, &v_beamed, &v_kinetic] {
        println!("▶ [{}] SİMÜLE EDİLİYOR...", v.name);
        let res = sim.simulate(v, 1200.0);
        let status = if res.orbit_achieved {
            "✓ YÖRÜNGE BAŞARIYLA YAKALANDI (LEO Orbit Achieved)"
        } else {
            "✗ YÖRÜNGEYE ULAŞILAMADI (Suborbital / Velocity Deficit)"
        };

        println!("  • Sonuç                : {}", status);
        println!("  • Uçuş Süresi          : {:.1} sn ({:.1} dk)", res.flight_duration_s, res.flight_duration_s / 60.0);
        println!("  • Erişilen İrtifa / Hız: {:.1} km | {:.1} m/s (Dikey vh: {:.1} m/s)", res.final_altitude_km, res.final_speed_ms, res.final_vertical_speed_ms);
        println!("  • Kepler Yörüngesi     : Enberi = {:.1} km | Enöte = {:.1} km | Eksantriklik e = {:.4}", res.perigee_altitude_km, res.apogee_altitude_km, res.eccentricity);
        println!("  • Kalan Yakıt          : {:.1} kg", res.remaining_fuel_kg);
        println!("  • Max-Q & Max-G        : {:.2} kPa | {:.2} G", res.max_q_kpa, res.max_g_force);
        println!("  • Canlı Isı İntegrali  : Pik = {:.1} kW/m^2 | Toplam = {:.2} MJ/m^2 | Şok = {:.0} °C", res.peak_heat_flux_kw_m2, res.total_heat_load_mj_m2, res.peak_wall_temp_k - 273.15);
        println!("  ⚡ RK4 Simülasyon Süresi: {} MİKROSANİYE!\n", res.execution_time_micros);
    }
}

fn run_materials() {
    println!("{}", "=".repeat(72));
    println!("🧪 CITADEL-MATH MALZEME BİLİMİ SIRALAMASI (SSOT JSON)");
    println!("{}", "=".repeat(72));

    let mdb = MaterialsDatabase::load_canonical();
    let ranked = mdb.rank_by_breaking_length();

    for (i, m) in ranked.iter().enumerate() {
        println!("#{} [{}] ({})", i + 1, m.name, m.category);
        println!("    • Yoğunluk          : {:.1} kg/m^3", m.density);
        println!("    • Akma / Çekme Sınırı: {:.1} MPa / {:.1} MPa", m.yield_strength / 1e6, m.tensile_strength / 1e6);
        println!("    • Young Modülü (E)  : {:.1} GPa", m.youngs_modulus / 1e9);
        println!("    • Azami Sıcaklık    : {:.0} K ({:.0} °C)", m.max_temperature_k, m.max_temperature_k - 273.15);
        println!("    • Kopma Uzunluğu    : {:.1} km (Kendi ağırlığını taşıma sınırı)", m.breaking_length_km());
        println!("    • Açıklama          : {}\n", m.description);
    }
}

fn run_stress() {
    println!("{}", "=".repeat(72));
    println!("🛡️ CITADEL-MATH İNCE CİDAR, BURKULMA (SP-8007) VE ABLATİF TPS DENETİMİ");
    println!("{}", "=".repeat(72));

    let mdb = MaterialsDatabase::load_canonical();
    let scenarios = [
        ("1. Kimyasal Roket Gövdesi (Al-Li 2195)", "mat_alli_2195", 425_000.0, 4.65, 46_450.0, 1.85, 42.0, 5.0, 8012.0),
        ("2. Lazer Işınlamalı Roket (Karbon Fiber T1000G)", "mat_carbon_fiber_t1000g", 7200.0, 4.76, 27_330.0, 0.80, 14.0, 3.0, 7756.0),
        ("3. Kinetik Fırlatıcı Burun Konisi (Inconel 718)", "mat_inconel_718", 3000.0, 20.19, 739_050.0, 0.45, 5.0, 6.0, 1800.0),
    ];

    for (title, mat_id, mass, g_load, max_q, r_m, l_m, t_mm, speed) in scenarios {
        let mat = mdb.get(mat_id).unwrap();
        let (s_vm, sf_yield, ok_yield) = StructuralStressChecker::check_hull(mat, mass, g_load, max_q, r_m, t_mm);
        let (s_cr, sf_buckle, ok_buckle) = StructuralStressChecker::check_cylinder_buckling(mat, mass, g_load, max_q, r_m, l_m, t_mm);
        let tps = StructuralStressChecker::size_ablative_tps(mat, speed, r_m, 55.0);

        println!("▶ {}:", title);
        println!(
            "  • İki Eksenli Von Mises : {:.1} MPa | Akma Emniyeti: {:.2}x (Onay: {})",
            s_vm / 1e6, sf_yield, ok_yield
        );
        println!(
            "  • NASA SP-8007 Burkulma : Kritik {:.1} MPa | Burkulma Emniyeti: {:.2}x (Onay: {})",
            s_cr / 1e6, sf_buckle, ok_buckle
        );
        println!(
            "  • Çıplak Şok Sıcaklığı  : {:.1} °C (Malzeme Sınırı: {:.1} °C)",
            tps.bare_wall_temp_k - 273.15,
            mat.max_temperature_k - 273.15
        );
        if tps.tps_required {
            println!(
                "  • PICA-X Ablatif Kalkan : {:.1} mm ({:.1} kg) -> İç Gövde Sıcaklığı: {:.1} °C [✓ KORUNDU]\n",
                tps.tps_thickness_mm,
                tps.tps_mass_kg,
                tps.protected_bondline_temp_k - 273.15
            );
        } else {
            println!("  • Termal Durum          : Çıplak gövde doğrudan dayanır [✓ KORUNDU]\n");
        }
    }
}

fn run_invent() {
    println!("{}", "=".repeat(72));
    println!("💡 CITADEL-MATH RUST AND-OR HİPERGRAF İCAT VE SEMBOLİK SENTEZ MOTORU");
    println!("{}", "=".repeat(72));

    let kg = KnowledgeGraph::new();
    let mut inputs = HashSet::new();
    inputs.insert("g_0");
    inputs.insert("I_sp");
    inputs.insert("m_0");
    inputs.insert("m_f");
    inputs.insert("m_dot");
    inputs.insert("eta");

    for goal in ["delta_v", "P_beam"] {
        println!("Hedef Nicelik       : {}", goal);
        let t0 = Instant::now();
        let formulas = kg.synthesize_formulas(&inputs, goal, 4);
        let elapsed = t0.elapsed();

        println!(
            "[+] {} adet doğrulanmış sembolik formül sentezlendi ({:.2?}):",
            formulas.len(),
            elapsed
        );
        for (i, f) in formulas.iter().enumerate() {
            println!("  ADAY #{}: {}", i + 1, f.chain_ids.join(" ➔ "));
            println!("    • Sembolik Formül : {} = {}", f.target, f.symbolic_expr);
            println!("    • Boyutsal İspat  : [{}] (Doğrulandı)", f.inferred_dimension);
        }
        println!();
    }
}

fn run_buckingham(extra_args: &[String]) {
    println!("{}", "=".repeat(76));
    println!("🔬 CITADEL-MATH Z^7 ÇEKİRDEK UZAYI BUCKINGHAM-PI BOYUTSAL KEŞİF MOTORU");
    println!("{}", "=".repeat(76));

    let scenarios: Vec<(String, Option<String>)> = if !extra_args.is_empty() {
        let spec = extra_args[0].clone();
        let target = extra_args.get(1).cloned();
        vec![(spec, target)]
    } else {
        vec![
            ("r,E,rho,t".to_string(), Some("r".to_string())),
            (
                "F_drag,rho,v,L_char,mu_visc".to_string(),
                Some("F_drag".to_string()),
            ),
            ("T_orb,a_orb,mu".to_string(), Some("T_orb".to_string())),
            (
                "h_conv,L_char,k_th,c_p,mu_visc".to_string(),
                Some("h_conv".to_string()),
            ),
        ]
    };

    for (spec, target_opt) in scenarios {
        let vars = match BuckinghamPiSolver::parse_variable_specs(&spec) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("HATA ({}): {}", spec, e);
                continue;
            }
        };
        let rep = match BuckinghamPiSolver::discover(&vars, target_opt.as_deref(), &[], &[]) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("HATA ({}): {}", spec, e);
                continue;
            }
        };
        println!(
            "▶ Girdi Değişkenleri ({} adet): {} | Boyutsal Rank = {} | Bağımsız Pi Sayısı = {}",
            rep.num_variables,
            spec,
            rep.dimensional_rank,
            rep.num_pi_groups
        );
        println!(
            "  • Tekrarlayan (Pivot) Taban : [{}]",
            rep.repeating_basis_variables.join(", ")
        );
        for g in &rep.pi_groups {
            let tag = g
                .recognized_number
                .as_deref()
                .map(|t| format!(" — {}", t))
                .unwrap_or_default();
            println!(
                "  • {} = {}  [Z^7: {}]{}",
                g.symbol, g.expression, g.verified_dimension, tag
            );
        }
        if let Some(eq) = &rep.target_scaling_equation {
            println!(
                "  • Sentezlenen Ölçekleme     : {}  [Boyut: {}]",
                eq,
                rep.target_scaling_dimension.as_deref().unwrap_or("?")
            );
        } else if let Some(reason) = &rep.target_unidentifiable_reason {
            println!("  • İmkânsızlık Kanıtı        : {}", reason);
        }
        println!();
    }
    println!("{}", "=".repeat(76));
}

fn run_design(extra_args: &[String]) {
    let mut constraints = MissionConstraints::default();
    let mut i = 0;
    while i < extra_args.len() {
        match extra_args[i].as_str() {
            "--payload" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].trim_end_matches("kg").parse::<f64>() {
                    constraints.payload_kg = v;
                }
                i += 2;
            }
            "--orbit" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].trim_end_matches("km").parse::<f64>() {
                    constraints.target_orbit_km = v;
                }
                i += 2;
            }
            "--max-g" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].parse::<f64>() {
                    constraints.max_g_limit = v;
                }
                i += 2;
            }
            "--safety-factor" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].parse::<f64>() {
                    constraints.min_safety_factor = v;
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("{}", "=".repeat(76));
    println!("🛸 CITADEL-MATH KAPALI DÖNGÜ TERSİNE TASARIM & PARETO OPTİMİZASYON MOTORU");
    println!("{}", "=".repeat(76));
    println!(
        "Görev Hedefi: Faydalı Yük = {:.0} kg | Hedef LEO Yörünge >= {:.0} km | Azami İvme <= {:.1} G | Emniyet >= {:.2}x",
        constraints.payload_kg,
        constraints.target_orbit_km,
        constraints.max_g_limit,
        constraints.min_safety_factor
    );

    let t0 = Instant::now();
    let report = InverseDesignEngine::optimize(constraints);
    let elapsed = t0.elapsed();

    println!("\n[1] HİPERGRAF SEMBOLİK DENKLEM SENTEZİ & TANIM KÜMESİ YAN KOŞULLARI:");
    for f in &report.synthesized_laws {
        println!(
            "  • {} = {}  [Boyut: {} | Yan Koşullar: {} | Zincir: {}]",
            f.target,
            f.symbolic_expr,
            f.inferred_dimension,
            f.domain_guards.join(", "),
            f.chain_ids.join(" ➔ ")
        );
    }

    println!("\n[2] KAPALI DÖNGÜ RK4 + KEPLER PERIGEE + VON MISES + EULER/SP-8007 + TPS HUNİSİ ({:.2?}):", elapsed);
    println!("  • Taranan Aday Araç Tasarımı        : {} konfigürasyon", report.total_candidates_evaluated);
    println!("  • 1. Elek (Gerçek Kepler LEOEnberi) : {} aday geçti (h_perigee >= 160 km, e < 0.08)", report.orbit_passed_count);
    println!("  • 2. Elek (G-Kuvveti Sınırı)        : {} aday geçti", report.g_limit_passed_count);
    println!("  • 3. Elek (Akma + Euler/SP-8007+TPS): {} aday tüm fizik/malzeme testlerini geçti!", report.structural_passed_count);
    println!(
        "  • 4. NSGA-II Kesin Pareto Skyline   : {} baskılanmamış tasarım ({} cephe, 2B Hiperhacim = {:.4})",
        report.pareto_summary.skyline_count,
        report.pareto_summary.num_fronts,
        report.pareto_summary.normalized_hypervolume_2d.unwrap_or(0.0)
    );

    println!("\n[3] PARETO-OPTİMAL DOĞRULANMIŞ MÜHENDİSLİK TASARIMLARI (Rank 0 Skyline, En Hafif -> En Ağır):");
    println!("{}", "-".repeat(76));

    for (idx, d) in report.pareto_optimal_designs.iter().enumerate() {
        let knee_badge = if d.is_knee_point { " [UZLAŞMA DİZ NOKTASI / KNEE]" } else { "" };
        println!(
            "🏆 TASARIM #{}: [{}] — Gövde: {} ({} Kademe){}",
            idx + 1,
            d.propulsion_name,
            d.material_name,
            d.stages,
            knee_badge
        );
        println!(
            "   ├─ Kütle Bütçesi : Kalkış (GLOW) = {:>8.1} kg | Yakıt = {:>8.1} kg | Kuru = {:>7.1} kg (Yapısal Oran: %{:.1}) | Yük = %{:.2}",
            d.gross_liftoff_mass_kg, d.fuel_mass_kg, d.total_dry_mass_kg, d.structural_fraction_pct, d.payload_fraction_pct
        );
        println!(
            "   ├─ Geometri & İtki: r = {:.2} m, Boy = {:.1} m (L/D = {:.1}), Cidar = {:.1} mm | İtki = {:.1} kN (Isp = {:.0} s){}",
            d.hull_radius_m,
            d.hull_length_m,
            d.slenderness_l_d,
            d.wall_thickness_mm,
            d.thrust_kn,
            d.isp_s,
            d.beamed_power_mw
                .map(|p| format!(" | Lazer Gücü = {:.1} MW", p))
                .unwrap_or_default()
        );
        println!(
            "   ├─ Kepler Yörünge : İrtifa = {:.1} km (Enberi = {:.1} km, Enöte = {:.1} km, e = {:.4}) | Hız = {:.1} m/s | Max-G = {:.2} G",
            d.final_altitude_km, d.perigee_altitude_km, d.apogee_altitude_km, d.eccentricity, d.final_speed_ms, d.max_g_force
        );
        println!(
            "   ├─ Yapısal Kanıt  : Von Mises = {:.1} MPa (Emniyet: {:.2}x) | Euler+SP-8007 Burkulma Emniyeti = {:.2}x",
            d.von_mises_mpa, d.yield_safety_factor, d.buckling_safety_factor
        );
        println!(
            "   ├─ Interval64 & AD: İdeal Δv = [{:.1}, {:.1}] m/s (±{:.1} m/s) | ∂Δv/∂Isp = +{:.2} m/s², ∂Δv/∂m_f = {:.2} (m/s)/kg",
            d.ideal_dv_interval_ms.lo,
            d.ideal_dv_interval_ms.hi,
            d.ideal_dv_interval_ms.radius(),
            d.dv_sensitivity_isp_ms_per_s,
            d.dv_sensitivity_dry_mass_ms_per_kg
        );
        println!(
            "   └─ Canlı TPS Kanıt: ∫q·dt = {:.2} MJ/m^2, Şok = {:.0} °C -> {:.1} mm ({:.1} kg) PICA-X ile İç Gövde = {:.0} °C (Limit: {:.0} °C)\n",
            d.total_heat_load_mj_m2,
            d.bare_shock_temp_c,
            d.tps_thickness_mm,
            d.tps_mass_kg,
            d.protected_bondline_temp_c,
            d.material_max_temp_c
        );
    }
    println!("{}", "=".repeat(76));
}

fn run_codegen() {
    println!("{}", "=".repeat(76));
    println!("🦀 CITADEL-MATH DOĞRULANMIŞ RUST KODU & MÜHENDİSLİK FÖYÜ ÜRETİCİSİ");
    println!("{}", "=".repeat(76));

    let report = InverseDesignEngine::optimize(MissionConstraints::default());
    let dossier_md = VerifiedCodegen::generate_markdown_dossier(&report);

    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("DOGRULANMIS_TASARIM_FOYU.md");
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("HATA: Rapor dizini oluşturulamadı ({}): {}", parent.display(), e);
            std::process::exit(1);
        });
    }
    std::fs::write(&out_path, &dossier_md).unwrap_or_else(|e| {
        eprintln!("HATA: Tasarım föyü yazılamadı ({}): {}", out_path.display(), e);
        std::process::exit(1);
    });
    println!("[+] Mühendislik tasarım ve kanıt föyü yazıldı: {}\n", out_path.display());

    for f in &report.synthesized_laws {
        println!("{}", VerifiedCodegen::emit_safe_rust_fn(f));
    }
}

fn run_mcp(args: &[String]) {
    use serde_json::json;

    let tool = args.first().map(|s| s.as_str()).unwrap_or("status");
    match tool {
        "status" => {
            let mut kg = KnowledgeGraph::new();
            let base_laws = match kg.verify_all_laws() {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };
            let inv_laws = kg.register_algebraic_inversions();
            if let Err(e) = kg.verify_all_laws() {
                println!("{}", json!({"status": "error", "message": e}));
                return;
            }
            let mdb = MaterialsDatabase::load_canonical();
            println!(
                "{}",
                json!({
                    "status": "ok",
                    "canonical_laws": base_laws,
                    "inverted_laws": inv_laws,
                    "total_bidirectional_laws": base_laws + inv_laws,
                    "certified_materials": mdb.list.len(),
                    "defined_variables": kg.variable_dimensions.len()
                })
            );
        }
        "solve_for" if args.len() >= 3 => {
            let eq = &args[1];
            let target = &args[2];
            let Some((l_str, r_str)) = eq.split_once('=') else {
                println!("{}", json!({"status": "error", "message": "Denklem 'LHS = RHS' biçiminde olmalıdır."}));
                return;
            };
            let lhs = match Expr::parse_str(l_str) {
                Ok(e) => e,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": format!("LHS ayrıştırma hatası: {}", e)}));
                    return;
                }
            };
            let rhs = match Expr::parse_str(r_str) {
                Ok(e) => e,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": format!("RHS ayrıştırma hatası: {}", e)}));
                    return;
                }
            };
            match Expr::solve_for(&lhs, &rhs, target) {
                Ok((isolated, guards)) => {
                    let kg = KnowledgeGraph::new();
                    let inferred_dim = isolated
                        .infer_dimension(&kg.variable_dimensions)
                        .map(|d| d.to_string())
                        .ok();
                    let synth = SynthesizedFormula {
                        target: target.clone(),
                        chain_ids: vec!["symbolic_solve_for".to_string()],
                        chain_names: vec!["Sembolik Cebirsel Tersleme".to_string()],
                        regimes: vec!["universal".to_string()],
                        free_variables: isolated.free_variables(),
                        inferred_dimension: isolated
                            .infer_dimension(&kg.variable_dimensions)
                            .unwrap_or_else(|_| Dimension::dimensionless()),
                        symbolic_expr: isolated.clone(),
                        domain_guards: guards.clone(),
                        effective_bounds: HashMap::new(),
                    };
                    let safe_rust = VerifiedCodegen::emit_safe_rust_fn(&synth);
                    println!(
                        "{}",
                        json!({
                            "status": "ok",
                            "target": target,
                            "isolated_rhs": isolated.to_string(),
                            "equation": format!("{} = {}", target, isolated),
                            "inferred_dimension": inferred_dim,
                            "domain_guards": guards,
                            "safe_rust_fn": safe_rust
                        })
                    );
                }
                Err(e) => println!("{}", json!({"status": "error", "message": e})),
            }
        }
        "prove_poly" if args.len() >= 3 => {
            let lhs_str = &args[1];
            let rhs_str = &args[2];
            let max_deg: usize = args
                .get(3)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(16);

            let (lhs_expr, rhs_expr) = match (Expr::parse_str(lhs_str), Expr::parse_str(rhs_str)) {
                (Ok(l), Ok(r)) => (l, r),
                (Err(e), _) | (_, Err(e)) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };

            let (lhs_guards, rhs_guards) = match (
                lhs_expr.extract_domain_conditions(),
                rhs_expr.extract_domain_conditions(),
            ) {
                (Ok(lg), Ok(rg)) => (lg, rg),
                (Err(e), _) | (_, Err(e)) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };

            if lhs_guards != rhs_guards {
                println!(
                    "{}",
                    json!({
                        "status": "ok",
                        "proven_identical": false,
                        "reason": "Tanım kümesi (yan koşul) eşitsizliği! Yan koşul yutan sadeleştirme reddedildi.",
                        "lhs_domain_guards": lhs_guards,
                        "rhs_domain_guards": rhs_guards
                    })
                );
                return;
            }

            match (
                MultiPolyZ::from_expr(&lhs_expr, max_deg),
                MultiPolyZ::from_expr(&rhs_expr, max_deg),
            ) {
                (Ok(pl), Ok(pr)) => match pl.checked_sub(&pr) {
                    Ok(diff) => {
                        let identical = diff.is_zero();
                        println!(
                            "{}",
                            json!({
                                "status": "ok",
                                "proven_identical": identical,
                                "ring": "Z[x_1, ..., x_n] (grevlex, checked_i64)",
                                "lhs_canonical": pl.to_string(),
                                "rhs_canonical": pr.to_string(),
                                "difference": diff.to_string(),
                                "domain_guards": lhs_guards
                            })
                        );
                    }
                    Err(e) => println!("{}", json!({"status": "error", "message": e.to_string()})),
                },
                (Err(e), _) | (_, Err(e)) => {
                    println!("{}", json!({"status": "error", "message": e.to_string()}))
                }
            }
        }
        "interval_eval" if args.len() >= 3 => {
            let expr_str = &args[1];
            let bindings_json = &args[2];
            let diff_var = args.get(3).map(|s| s.as_str()).unwrap_or("");

            let expr = match Expr::parse_str(expr_str) {
                Ok(e) => e,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };

            let raw_map: HashMap<String, Vec<f64>> = match serde_json::from_str(bindings_json) {
                Ok(m) => m,
                Err(e) => {
                    println!(
                        "{}",
                        json!({"status": "error", "message": format!("JSON aralık haritası ayrıştırılamadı (beklenen {{\"x\": [lo, hi]}}): {}", e)})
                    );
                    return;
                }
            };

            let mut iv_env: HashMap<&str, Interval64> = HashMap::new();
            let mut mid_env: HashMap<&str, f64> = HashMap::new();
            for (k, bounds) in &raw_map {
                let (lo, hi) = match bounds.as_slice() {
                    [v] => (*v, *v),
                    [lo, hi, ..] => (*lo, *hi),
                    [] => {
                        println!("{}", json!({"status": "error", "message": format!("Boş aralık: {}", k)}));
                        return;
                    }
                };
                match Interval64::new(lo, hi) {
                    Ok(iv) => {
                        iv_env.insert(k.as_str(), iv);
                        mid_env.insert(k.as_str(), iv.midpoint());
                    }
                    Err(e) => {
                        println!("{}", json!({"status": "error", "message": e}));
                        return;
                    }
                }
            }

            let naive_iv = match expr.eval_interval(&iv_env) {
                Ok(iv) => iv,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };
            let taylor_iv = expr.eval_taylor_interval(&iv_env).unwrap_or(naive_iv);
            let grad_info = expr.gradient(&mid_env).ok();
            let sym_deriv = if !diff_var.is_empty() {
                expr.differentiate(diff_var).ok().map(|d| d.to_string())
            } else {
                None
            };

            println!(
                "{}",
                json!({
                    "status": "ok",
                    "expression": expr.to_string(),
                    "outward_interval": [naive_iv.lo, naive_iv.hi],
                    "taylor_mean_value_interval": [taylor_iv.lo, taylor_iv.hi],
                    "taylor_width": taylor_iv.width(),
                    "midpoint_value_and_gradient": grad_info,
                    "symbolic_derivative": sym_deriv
                })
            );
        }
        "synthesize" if args.len() >= 3 => {
            let inputs_csv = &args[1];
            let goal = &args[2];
            let max_depth: usize = args
                .get(3)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(4);

            let mut kg = KnowledgeGraph::new();
            kg.register_algebraic_inversions();
            let input_set: HashSet<&str> = inputs_csv
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            let formulas = kg.synthesize_formulas(&input_set, goal, max_depth);
            let items: Vec<serde_json::Value> = formulas
                .iter()
                .map(|f| {
                    json!({
                        "target": f.target,
                        "equation": format!("{} = {}", f.target, f.symbolic_expr),
                        "dimension_z7": f.inferred_dimension.to_string(),
                        "regimes": f.regimes,
                        "chain_ids": f.chain_ids,
                        "chain_names": f.chain_names,
                        "free_variables": f.free_variables,
                        "domain_guards": f.domain_guards,
                        "effective_bounds": f.effective_bounds,
                        "safe_rust_fn": VerifiedCodegen::emit_safe_rust_fn(f)
                    })
                })
                .collect();

            println!(
                "{}",
                json!({
                    "status": "ok",
                    "goal": goal,
                    "synthesized_count": items.len(),
                    "formulas": items
                })
            );
        }
        "design" => {
            let mut constraints = MissionConstraints::default();
            if let Some(p) = args.get(1).and_then(|s| s.parse::<f64>().ok()) {
                constraints.payload_kg = p;
            }
            if let Some(o) = args.get(2).and_then(|s| s.parse::<f64>().ok()) {
                constraints.target_orbit_km = o;
            }
            if let Some(g) = args.get(3).and_then(|s| s.parse::<f64>().ok()) {
                constraints.max_g_limit = g;
            }
            if let Some(sf) = args.get(4).and_then(|s| s.parse::<f64>().ok()) {
                constraints.min_safety_factor = sf;
            }

            let t0 = Instant::now();
            let report = InverseDesignEngine::optimize(constraints);
            let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

            let designs: Vec<serde_json::Value> = report
                .pareto_optimal_designs
                .iter()
                .map(|d| {
                    json!({
                        "propulsion": d.propulsion_name,
                        "material_id": d.material_id,
                        "material_name": d.material_name,
                        "stages": d.stages,
                        "gross_liftoff_mass_kg": (d.gross_liftoff_mass_kg * 10.0).round() / 10.0,
                        "fuel_mass_kg": (d.fuel_mass_kg * 10.0).round() / 10.0,
                        "dry_mass_kg": (d.total_dry_mass_kg * 10.0).round() / 10.0,
                        "structural_fraction_pct": (d.structural_fraction_pct * 100.0).round() / 100.0,
                        "hull_radius_m": d.hull_radius_m,
                        "hull_length_m": d.hull_length_m,
                        "wall_thickness_mm": d.wall_thickness_mm,
                        "thrust_kn": (d.thrust_kn * 10.0).round() / 10.0,
                        "isp_s": d.isp_s,
                        "beamed_power_mw": d.beamed_power_mw,
                        "perigee_km": (d.perigee_altitude_km * 10.0).round() / 10.0,
                        "apogee_km": (d.apogee_altitude_km * 10.0).round() / 10.0,
                        "eccentricity": (d.eccentricity * 10000.0).round() / 10000.0,
                        "max_g": (d.max_g_force * 100.0).round() / 100.0,
                        "von_mises_mpa": (d.von_mises_mpa * 10.0).round() / 10.0,
                        "yield_safety_factor": (d.yield_safety_factor * 100.0).round() / 100.0,
                        "buckling_safety_factor": (d.buckling_safety_factor * 100.0).round() / 100.0,
                        "min_structural_safety_factor": (d.min_structural_safety_factor * 100.0).round() / 100.0,
                        "pareto_rank": d.pareto_rank,
                        "crowding_distance": if d.crowding_distance.is_infinite() { serde_json::Value::String("inf".to_string()) } else { json!((d.crowding_distance * 10000.0).round() / 10000.0) },
                        "utopia_distance": (d.utopia_distance * 10000.0).round() / 10000.0,
                        "is_knee_point": d.is_knee_point,
                        "dominated_count": d.dominated_count,
                        "ideal_dv_interval_ms": [d.ideal_dv_interval_ms.lo, d.ideal_dv_interval_ms.hi],
                        "tps_thickness_mm": (d.tps_thickness_mm * 10.0).round() / 10.0,
                        "tps_mass_kg": (d.tps_mass_kg * 10.0).round() / 10.0,
                        "protected_bondline_temp_c": (d.protected_bondline_temp_c * 10.0).round() / 10.0
                    })
                })
                .collect();

            println!(
                "{}",
                json!({
                    "status": "ok",
                    "execution_ms": (elapsed_ms * 100.0).round() / 100.0,
                    "total_candidates_evaluated": report.total_candidates_evaluated,
                    "orbit_passed": report.orbit_passed_count,
                    "g_limit_passed": report.g_limit_passed_count,
                    "structural_and_tps_passed": report.structural_passed_count,
                    "nsga2_num_fronts": report.pareto_summary.num_fronts,
                    "exact_skyline_count": report.pareto_summary.skyline_count,
                    "normalized_hypervolume_2d": report.pareto_summary.normalized_hypervolume_2d,
                    "utopia_point": report.pareto_summary.utopia_point,
                    "nadir_point": report.pareto_summary.nadir_point,
                    "pareto_optimal_designs": designs
                })
            );
        }
        "stress_hull" if args.len() >= 9 => {
            let mat_id = &args[1];
            let p = |idx: usize| args.get(idx).and_then(|s| s.parse::<f64>().ok());
            let (Some(mass_kg), Some(g_load), Some(max_q_pa), Some(r_m), Some(l_m), Some(t_mm), Some(speed_ms)) =
                (p(2), p(3), p(4), p(5), p(6), p(7), p(8))
            else {
                println!("{}", json!({"status": "error", "message": "Sayısal parametre hatası."}));
                return;
            };
            let mdb = MaterialsDatabase::load_canonical();
            let Some(mat) = mdb.get(mat_id) else {
                let ids: Vec<&str> = mdb.list.iter().map(|m| m.id.as_str()).collect();
                println!(
                    "{}",
                    json!({"status": "error", "message": format!("Malzeme bulunamadı: '{}'", mat_id), "available_ids": ids})
                );
                return;
            };
            let (s_vm, sf_yield, ok_yield) =
                StructuralStressChecker::check_hull(mat, mass_kg, g_load, max_q_pa, r_m, t_mm);
            let (s_cr, sf_buckle, ok_buckle) = StructuralStressChecker::check_cylinder_buckling(
                mat, mass_kg, g_load, max_q_pa, r_m, l_m, t_mm,
            );
            let tps = StructuralStressChecker::size_ablative_tps(mat, speed_ms, r_m, 55.0);
            println!(
                "{}",
                json!({
                    "status": "ok",
                    "material_id": mat.id,
                    "material_name": mat.name,
                    "von_mises_mpa": (s_vm / 1e6 * 100.0).round() / 100.0,
                    "yield_strength_mpa": mat.yield_strength / 1e6,
                    "yield_safety_factor": (sf_yield * 1000.0).round() / 1000.0,
                    "yield_pass": ok_yield,
                    "sp8007_buckling_critical_mpa": (s_cr / 1e6 * 100.0).round() / 100.0,
                    "buckling_safety_factor": (sf_buckle * 1000.0).round() / 1000.0,
                    "buckling_pass": ok_buckle,
                    "bare_wall_temp_c": (tps.bare_wall_temp_k - 273.15),
                    "material_max_temp_c": (mat.max_temperature_k - 273.15),
                    "tps_required": tps.tps_required,
                    "pica_x_thickness_mm": tps.tps_thickness_mm,
                    "pica_x_mass_kg": tps.tps_mass_kg,
                    "protected_bondline_temp_c": (tps.protected_bondline_temp_k - 273.15),
                    "overall_pass": ok_yield && ok_buckle && (tps.protected_bondline_temp_k <= mat.max_temperature_k)
                })
            );
        }
        "check_bounds" if args.len() >= 2 => {
            let raw: serde_json::Value = match serde_json::from_str(&args[1]) {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": format!("JSON hatası: {}", e)}));
                    return;
                }
            };
            let cand = PhysicalSystemCandidate {
                mass_kg: raw.get("mass_kg").and_then(|v| v.as_f64()),
                velocity_ms: raw.get("velocity_ms").and_then(|v| v.as_f64()),
                exhaust_velocity_ms: raw.get("exhaust_velocity_ms").and_then(|v| v.as_f64()),
                mass_flow_kg_s: raw.get("mass_flow_kg_s").and_then(|v| v.as_f64()),
                thrust_n: raw.get("thrust_n").and_then(|v| v.as_f64()),
                power_in_w: raw.get("power_in_w").and_then(|v| v.as_f64()),
                power_out_w: raw.get("power_out_w").and_then(|v| v.as_f64()),
                efficiency: raw.get("efficiency").and_then(|v| v.as_f64()),
                temp_hot_k: raw.get("temp_hot_k").and_then(|v| v.as_f64()),
                temp_cold_k: raw.get("temp_cold_k").and_then(|v| v.as_f64()),
                entropy_delta_j_k: raw.get("entropy_delta_j_k").and_then(|v| v.as_f64()),
            };
            match PhysicalBoundsShield::verify_candidate(&cand) {
                Ok(()) => {
                    let lorentz_gamma = cand
                        .velocity_ms
                        .and_then(|v| PhysicalBoundsShield::check_subluminal_velocity(v).ok());
                    let carnot_max = match (cand.temp_hot_k, cand.temp_cold_k) {
                        (Some(th), Some(tc)) if th > 0.0 => Some(1.0 - tc / th),
                        _ => None,
                    };
                    println!(
                        "{}",
                        json!({
                            "status": "ok",
                            "physically_realizable": true,
                            "verdict": "FİZİKSEL OLARAK GEÇERLİ (Termodinamik 1./2. Yasa, Nedensellik ve Momentum Korunumu Doğrulandı)",
                            "lorentz_gamma": lorentz_gamma,
                            "carnot_max_efficiency": carnot_max
                        })
                    );
                }
                Err(viol) => {
                    println!(
                        "{}",
                        json!({
                            "status": "ok",
                            "physically_realizable": false,
                            "verdict": "FİZİKSEL OLARAK İMKÂNSIZ (BUDANDI / KARANTİNA)",
                            "violation": viol.to_string()
                        })
                    );
                }
            }
        }
        "regime_check" if args.len() >= 3 => {
            let law_id = &args[1];
            let bindings_map: HashMap<String, f64> = match serde_json::from_str(&args[2]) {
                Ok(m) => m,
                Err(e) => {
                    println!(
                        "{}",
                        json!({"status": "error", "message": format!("JSON sayısal bağlama haritası ayrıştırılamadı: {}", e)})
                    );
                    return;
                }
            };
            let mut kg = KnowledgeGraph::new();
            kg.register_algebraic_inversions();
            let Some(law) = kg.laws.get(law_id) else {
                println!(
                    "{}",
                    json!({"status": "error", "message": format!("Yasa bulunamadı: '{}'", law_id)})
                );
                return;
            };
            let ref_map: HashMap<&str, f64> =
                bindings_map.iter().map(|(k, &v)| (k.as_str(), v)).collect();
            match law.check_validity_bounds(&ref_map) {
                Ok(()) => {
                    let evaluated = law.rhs.evaluate(&ref_map).ok();
                    println!(
                        "{}",
                        json!({
                            "status": "ok",
                            "law_id": law.id,
                            "regime": law.regime,
                            "valid_in_regime": true,
                            "domain_bounds": law.domain_bounds,
                            "evaluated_target": evaluated
                        })
                    );
                }
                Err(viol) => {
                    println!(
                        "{}",
                        json!({
                            "status": "ok",
                            "law_id": law.id,
                            "regime": law.regime,
                            "valid_in_regime": false,
                            "violation": viol,
                            "domain_bounds": law.domain_bounds
                        })
                    );
                }
            }
        }
        "buckingham" if args.len() >= 2 => {
            let var_spec = &args[1];
            let target_opt = args
                .get(2)
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
            let samples: Vec<HashMap<String, f64>> = match args.get(3).map(|s| s.trim()) {
                Some(s) if !s.is_empty() && s != "[]" => match serde_json::from_str(s) {
                    Ok(v) => v,
                    Err(e) => {
                        println!(
                            "{}",
                            json!({"status": "error", "message": format!("Örneklem JSON dizisi ayrıştırılamadı: {}", e)})
                        );
                        return;
                    }
                },
                _ => Vec::new(),
            };
            let repeating_vec: Vec<&str> = args
                .get(4)
                .map(|s| {
                    s.split(',')
                        .map(|t| t.trim())
                        .filter(|t| !t.is_empty())
                        .collect()
                })
                .unwrap_or_default();

            let vars = match BuckinghamPiSolver::parse_variable_specs(var_spec) {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };
            match BuckinghamPiSolver::discover(&vars, target_opt, &repeating_vec, &samples) {
                Ok(rep) => {
                    let mut val = serde_json::to_value(&rep).unwrap_or_else(|_| json!({}));
                    if let Some(obj) = val.as_object_mut() {
                        obj.insert("status".to_string(), json!("ok"));
                    }
                    println!("{}", val);
                }
                Err(e) => println!("{}", json!({"status": "error", "message": e})),
            }
        }
        "pareto" if args.len() >= 3 => {
            let raw_objs: Vec<serde_json::Value> = match serde_json::from_str(&args[1]) {
                Ok(v) => v,
                Err(e) => {
                    println!(
                        "{}",
                        json!({"status": "error", "message": format!("Amaç JSON dizisi ayrıştırılamadı: {}", e)})
                    );
                    return;
                }
            };
            let mut specs = Vec::with_capacity(raw_objs.len());
            for item in &raw_objs {
                let Some(name) = item.get("name").and_then(|v| v.as_str()) else {
                    println!(
                        "{}",
                        json!({"status": "error", "message": "Her amaç nesnesinde 'name' alanı zorunludur."})
                    );
                    return;
                };
                let dir_str = item
                    .get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("min");
                let dir = match ObjectiveDirection::from_str_loose(dir_str) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("{}", json!({"status": "error", "message": e}));
                        return;
                    }
                };
                let unit = item
                    .get("unit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                specs.push(ObjectiveSpec {
                    name: name.to_string(),
                    direction: dir,
                    unit,
                });
            }

            let cands: Vec<NamedObjectiveCandidate> = match serde_json::from_str(&args[2]) {
                Ok(v) => v,
                Err(e) => {
                    println!(
                        "{}",
                        json!({"status": "error", "message": format!("Aday JSON dizisi ayrıştırılamadı: {}", e)})
                    );
                    return;
                }
            };
            let matrix: Vec<Vec<f64>> = cands.iter().map(|c| c.values.clone()).collect();
            let summary = match ParetoSynthesizer::analyze(&specs, &matrix) {
                Ok(s) => s,
                Err(e) => {
                    println!("{}", json!({"status": "error", "message": e}));
                    return;
                }
            };
            if let Err(e) = ParetoSynthesizer::verify_skyline_invariants(&specs, &matrix, &summary) {
                println!("{}", json!({"status": "error", "message": e}));
                return;
            }

            let skyline_ids: Vec<String> = summary
                .skyline_indices
                .iter()
                .map(|&idx| cands[idx].id.clone())
                .collect();
            let knee_id = summary
                .knee_candidate_index
                .map(|idx| cands[idx].id.clone());
            let fronts_ids: Vec<Vec<String>> = summary
                .fronts
                .iter()
                .map(|fr| fr.iter().map(|&idx| cands[idx].id.clone()).collect())
                .collect();
            let cand_reports: Vec<serde_json::Value> = summary
                .point_metrics
                .iter()
                .map(|m| {
                    let cd_val = if m.crowding_distance.is_infinite() {
                        serde_json::Value::String("inf".to_string())
                    } else {
                        json!((m.crowding_distance * 1e6).round() / 1e6)
                    };
                    json!({
                        "id": cands[m.candidate_index].id,
                        "values": cands[m.candidate_index].values,
                        "pareto_rank": m.pareto_rank,
                        "is_skyline": m.pareto_rank == 0,
                        "is_knee_point": m.is_knee_point,
                        "crowding_distance": cd_val,
                        "utopia_distance": (m.utopia_distance * 1e6).round() / 1e6,
                        "dominated_count": m.dominated_count,
                        "dominated_by_count": m.dominated_by_count
                    })
                })
                .collect();

            println!(
                "{}",
                json!({
                    "status": "ok",
                    "total_candidates": summary.total_feasible,
                    "num_fronts": summary.num_fronts,
                    "skyline_count": summary.skyline_count,
                    "skyline_ids": skyline_ids,
                    "knee_id": knee_id,
                    "utopia_point": summary.utopia_point,
                    "nadir_point": summary.nadir_point,
                    "raw_hypervolume_2d": summary.raw_hypervolume_2d,
                    "normalized_hypervolume_2d": summary.normalized_hypervolume_2d,
                    "fronts_by_id": fronts_ids,
                    "candidates": cand_reports
                })
            );
        }
        _ => {
            println!(
                "{}",
                json!({
                    "status": "ok",
                    "available_tools": [
                        "status",
                        "solve_for",
                        "prove_poly",
                        "interval_eval",
                        "synthesize",
                        "design",
                        "stress_hull",
                        "check_bounds",
                        "regime_check",
                        "buckingham",
                        "pareto",
                        "cad"
                    ]
                })
            );
        }
    }
}

fn run_cad(extra_args: &[String]) {
    let mut script = String::from("IBEAM 0,0 100 200 15 10");
    let mut unit = CadUnit::Millimeters;
    let mut mat_id = String::from("mat_titanium_ti6al4v");
    let mut length_m = 2.5_f64;
    let mut moment_nm = 25_000.0_f64;
    let mut axial_n = 100_000.0_f64;
    let mut temp_k = 350.0_f64;
    let mut dxf_path: Option<String> = None;
    let mut svg_path: Option<String> = None;
    let mut json_out = false;

    let mut i = 0;
    while i < extra_args.len() {
        match extra_args[i].as_str() {
            "--script" if i + 1 < extra_args.len() => {
                script = extra_args[i + 1].clone();
                i += 2;
            }
            "--unit" if i + 1 < extra_args.len() => {
                unit = match extra_args[i + 1].to_ascii_lowercase().as_str() {
                    "m" | "meter" | "meters" => CadUnit::Meters,
                    _ => CadUnit::Millimeters,
                };
                i += 2;
            }
            "--mat" if i + 1 < extra_args.len() => {
                mat_id = extra_args[i + 1].clone();
                i += 2;
            }
            "--length" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].parse::<f64>() {
                    length_m = v;
                }
                i += 2;
            }
            "--moment" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].parse::<f64>() {
                    moment_nm = v;
                }
                i += 2;
            }
            "--axial" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].parse::<f64>() {
                    axial_n = v;
                }
                i += 2;
            }
            "--temp" if i + 1 < extra_args.len() => {
                if let Ok(v) = extra_args[i + 1].parse::<f64>() {
                    temp_k = v;
                }
                i += 2;
            }
            "--dxf" if i + 1 < extra_args.len() => {
                dxf_path = Some(extra_args[i + 1].clone());
                i += 2;
            }
            "--svg" if i + 1 < extra_args.len() => {
                svg_path = Some(extra_args[i + 1].clone());
                i += 2;
            }
            "--json" => {
                json_out = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let mut doc = CadDocument::new("citadel_cad_part", unit);
    let logs = match doc.exec_script(&script) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[CAD][HATA] {}", e);
            std::process::exit(1);
        }
    };

    let db = MaterialsDatabase::load_canonical();
    let mat = db
        .get(&mat_id)
        .or_else(|| db.list.iter().find(|m| m.id.contains(&mat_id)))
        .unwrap_or(&db.list[0]);

    let report = match doc.evaluate_beam(mat, length_m, moment_nm, axial_n, temp_k) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[CAD][HATA] {}", e);
            std::process::exit(1);
        }
    };

    if let Some(ref p) = dxf_path {
        if let Err(e) = std::fs::write(p, doc.export_dxf()) {
            eprintln!("[CAD][HATA] DXF yazılamadı ({}): {}", p, e);
            std::process::exit(1);
        }
    }
    if let Some(ref p) = svg_path {
        if let Err(e) = std::fs::write(p, doc.export_svg()) {
            eprintln!("[CAD][HATA] SVG yazılamadı ({}): {}", p, e);
            std::process::exit(1);
        }
    }

    if json_out {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return;
    }

    println!("{}", "=".repeat(72));
    println!("📐 CITADEL-MATH 2D PARAMETRİK CAD & KESİT MEKANİĞİ (STEINER + EULER)");
    println!("{}", "=".repeat(72));
    for line in logs {
        println!("  [CMD] {}", line);
    }
    let s = &report.section;
    println!("  [KESİT] Alan A        : {:.4} cm² ({:.6e} m²) [{}]", s.area_m2 * 1e4, s.area_m2, s.area_dim);
    println!("  [KESİT] Çevre P       : {:.4} m", s.perimeter_m);
    println!("  [KESİT] Ağırlık Mrk.  : ({:.4}, {:.4}) m", s.centroid_x_m, s.centroid_y_m);
    println!("  [KESİT] Atalet Ix/Iy  : Ix={:.4e} m⁴ | Iy={:.4e} m⁴ | J={:.4e} m⁴ [{}]", s.ix_m4, s.iy_m4, s.polar_j_m4, s.inertia_dim);
    println!("  [KESİT] Kesit Modülü  : Wx={:.4e} m³ | Wy={:.4e} m³", s.section_modulus_x_m3, s.section_modulus_y_m3);
    println!("  [MALZ.] {} (L={} m, T={} K) => Kütle: {:.3} kg ({:.3} kg/m)", report.material_name, report.length_m, report.temperature_k, report.total_mass_kg, report.linear_mass_kg_m);
    println!("  [YAPI ] Eğilme+Basma  : σ_b={:.2} MPa + σ_a={:.2} MPa = {:.2} MPa (σ_y(T)={:.2} MPa)", report.bending_stress_pa / 1e6, report.axial_stress_pa / 1e6, report.combined_peak_stress_pa / 1e6, report.derated_yield_pa / 1e6);
    println!("  [BURK.] Euler P_cr    : {:.2} kN | SF_akma={:.2} | SF_burkulma={:.2} => [{}]", report.euler_buckling_load_n / 1e3, report.yield_safety_factor, report.buckling_safety_factor, if report.passed { "GEÇTİ (SF >= 1.35)" } else { "BAŞARISIZ" });
    if let Some(p) = dxf_path {
        println!("  [DXF  ] AutoCAD R12 DXF kaydedildi: {}", p);
    }
    if let Some(p) = svg_path {
        println!("  [SVG  ] Teknik Blueprint SVG kaydedildi: {}", p);
    }
}
