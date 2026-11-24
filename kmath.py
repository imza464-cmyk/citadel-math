# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
# Matematik Çekirdeği — Mühendislik Kokpiti ve Anında Kural Denetimi (kmath)

"""
Citadel-Math sıfır toleranslı mühendislik denetim ve komuta kokpiti.
Kullanım:
  python kmath.py audit                 # Tüm denklemleri, boyutları ve kuralları denetler
  python kmath.py rules                 # AGENTS.md anayasal değişmezlerini listeler
  python kmath.py stats                 # Bilgi tabanı metriklerini ve gerçek ölçeği raporlar
  python kmath.py harvest [local|online|all] # LaTeX denklemlerini otomatik tarar ve staging'e alır
  python kmath.py simulate [chemical|beamed|kinetic|compare] # 4. Derece Runge-Kutta fizik simülatörü
  python kmath.py materials             # Havacılık ve uzay malzeme sınırlarını listeler
  python kmath.py stress                # Uçuş yükleri altında yapısal malzeme ve termal stres denetimi
  python kmath.py rust [cmd]            # ⚡ Saf RUST derlenmiş yerel ikiliyi çalıştırır (Mikrosaniye hızında!)
"""

import sys
import os
import json
import shlex
import shutil
import subprocess
from typing import List, Optional, Tuple

os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")

import sympy as sp

try:
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')
except (AttributeError, OSError, ValueError):
    pass

ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

import engine
from engine.dimensions import (
    Dimension,
    DIMENSIONS,
    check_dimensional_consistency,
    safe_parse_math_expr,
    verify_law_dimensional_homogeneity,
)
from engine.constants import PhysicalConstants
from engine.knowledge_graph import KnowledgeGraph
from engine.harvester import LatexEquationHarvester
from engine.simulator import AtmosphereModel, RocketVehicle, RK4TrajectorySimulator
from engine.materials import Material, MaterialsDatabase, StructuralStressChecker
from engine.discovery import BuckinghamPiDiscovery, MultiObjectivePareto


def _resolve_data_file(*parts: str) -> str:
    """Kaynak ağaçtaki (repo) veya paketlenmiş wheel (engine/_ssot_data) içindeki SSOT dosyasını çözer."""
    repo_candidate = os.path.join(ROOT_DIR, *parts)
    if os.path.exists(repo_candidate):
        return repo_candidate
    pkg_candidate = os.path.join(
        os.path.dirname(os.path.abspath(engine.__file__)),
        "_ssot_data",
        *parts,
    )
    if os.path.exists(pkg_candidate):
        return pkg_candidate
    return repo_candidate


def resolve_rust_bin() -> Tuple[Optional[str], List[str]]:
    """Derlenmiş citadel_math_core ikilisini KMATH_RUST_BIN -> release -> debug -> bin -> PATH sırasıyla arar."""
    searched: List[str] = []
    env_override = os.environ.get("KMATH_RUST_BIN", "").strip()
    if env_override:
        expanded = os.path.abspath(os.path.expanduser(env_override))
        if os.path.isfile(expanded):
            return expanded, [expanded]
        return None, [f"KMATH_RUST_BIN={expanded} (bulunamadı / not found)"]

    bin_name = "citadel_math_core.exe" if os.name == "nt" else "citadel_math_core"
    for candidate in (
        os.path.join(ROOT_DIR, "citadel_math_core", "target", "release", bin_name),
        os.path.join(ROOT_DIR, "citadel_math_core", "target", "debug", bin_name),
        os.path.join(ROOT_DIR, "bin", bin_name),
    ):
        searched.append(candidate)
        if os.path.isfile(candidate):
            return candidate, searched

    path_names = [
        "citadel_math_core",
        "citadel_math_core-windows-x86_64.exe"
        if os.name == "nt"
        else "citadel_math_core-linux-x86_64",
    ]
    for pname in path_names:
        found = shutil.which(pname)
        searched.append(f"PATH:{pname}")
        if found and os.path.isfile(found):
            return os.path.abspath(found), searched

    return None, searched


DB_PATH = _resolve_data_file("database", "canonical_laws.json")
ONT_PATH = _resolve_data_file("database", "ONTOLOGY.json")
MAT_PATH = _resolve_data_file("database", "materials.json")
NIST_PATH = _resolve_data_file("data", "nist_constants.json")
ARCHIVE_MD = os.path.join(ROOT_DIR, "formuller_ve_denklemler.md")


def cmd_rust(subcmd: str = "audit"):
    rust_bin, searched = resolve_rust_bin()
    if not rust_bin:
        cargo_toml = os.path.join(ROOT_DIR, "citadel_math_core", "Cargo.toml")
        if os.path.isfile(cargo_toml) and shutil.which("cargo") and not os.environ.get("KMATH_RUST_BIN", "").strip():
            print("[-] Rust ikilisi bulunamadı! Derleniyor: cargo build --release...")
            subprocess.run(["cargo", "build", "--release", "--manifest-path", cargo_toml], check=True)
            rust_bin, searched = resolve_rust_bin()

    if rust_bin and os.path.isfile(rust_bin):
        res = subprocess.run([rust_bin, *shlex.split(subcmd)], check=False)
        if res.returncode != 0:
            sys.exit(res.returncode)
    else:
        print(
            "[-] citadel_math_core Rust ikilisi bulunamadı / Rust binary not found!\n"
            f"    Aranan yollar / Searched: {', '.join(searched)}\n"
            "    Çözüm / Fix:\n"
            "      1) Kaynaktan derleyin: cargo build --release --manifest-path citadel_math_core/Cargo.toml\n"
            "      2) Veya GitHub Releases hazır ikilisini indirip KMATH_RUST_BIN (veya PATH) ayarlayın:\n"
            "         https://github.com/imza464-cmyk/citadel-math/releases"
        )
        sys.exit(127)


def cmd_rules():
    agents_path = _resolve_data_file("AGENTS.md")
    if os.path.exists(agents_path):
        with open(agents_path, "r", encoding="utf-8") as f:
            print(f.read())
    else:
        print("[-] AGENTS.md bulunamadı!")

def cmd_stats():
    print("=" * 70)
    print("📊 MATEMATİK ÇEKİRDEĞİ & İCAT MOTORU — SİSTEM METRİKLERİ")
    print("=" * 70)

    kg = KnowledgeGraph()
    if os.path.exists(DB_PATH):
        kg.load_canonical_database(DB_PATH)
        s = kg.summary()
        print(f"• Kanonik Fizik/Matematik Yasaları : {s['equations']} adet")
        print(f"• Tanımlı Fiziksel Değişkenler      : {s['variables']} adet")
        print(f"• Kapsanan Fizik Disiplinleri       : {len(s['domains'])} alan ({', '.join(s['domains'])})")
    else:
        print("• Kanonik Yasa Veritabanı          : BULUNAMADI")

    staging_path = os.path.join(os.path.dirname(__file__), "staging", "harvested_equations.json")
    if os.path.exists(staging_path):
        with open(staging_path, "r", encoding="utf-8") as f:
            st_data = json.load(f)
            print(f"• Staging Havuzundaki Aday Denklem : {len(st_data)} adet")

    if os.path.exists(MAT_PATH):
        with open(MAT_PATH, "r", encoding="utf-8") as f:
            mat_data = json.load(f)
            print(f"• Kayıtlı Havacılık/Uzay Malzemesi : {len(mat_data)} adet")

    if os.path.exists(NIST_PATH):
        with open(NIST_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
            print(f"• İnternetten İndirilen NIST Sabiti: {len(data)} adet")
    else:
        print("• Çevrimiçi NIST Verisi             : HENÜZ ÇEKİLMEDİ (python icat_motoru.py fetch)")

    print(f"• Temel SI Boyut Tanımları          : {len(DIMENSIONS)} kategori")
    print("=" * 70)

def cmd_materials():
    print("=" * 70)
    print("🧪 HAVACILIK, UZAY VE AŞIRI ORTAM MALZEME VERİTABANI")
    print("=" * 70)
    db = MaterialsDatabase(MAT_PATH)
    ranked = db.rank_by_breaking_length()

    print("▶ MALZEMELERİN ÖZGÜL MUKAVEMET VE KOPMA UZUNLUĞU SIRALAMASI:\n")
    for i, m in enumerate(ranked, 1):
        print(f"#{i} [{m.name.upper()}] ({m.category})")
        print(f"    • Yoğunluk          : {m.density:,.1f} kg/m^3")
        print(f"    • Akma / Çekme Sınırı: {m.yield_strength/1e6:,.1f} MPa / {m.tensile_strength/1e6:,.1f} MPa")
        print(f"    • Young Modülü (E)  : {m.youngs_modulus/1e9:,.1f} GPa")
        print(f"    • Azami Sıcaklık    : {m.max_temperature:.0f} K ({m.max_temperature - 273.15:.0f} °C)")
        print(f"    • Kopma Uzunluğu    : {m.breaking_length_km:,.1f} km (Kendi ağırlığını taşıma sınırı)")
        print(f"    • Açıklama          : {m.description}\n")

    print("=" * 70)

def cmd_stress():
    print("=" * 70)
    print("🛡️ UÇUŞ YÜKLERİ ALTINDA YAPISAL GERİLME VE TERMAL KALKAN DENETİMİ")
    print("=" * 70)
    db = MaterialsDatabase(MAT_PATH)

    # Örnek 3 Araç ve Senaryo
    scenarios = [
        {
            "title": "1. Geleneksel Kimyasal Roket (Falcon-9 Gövdesi: Al-Li 2195)",
            "mat_id": "mat_alli_2195",
            "mass": 425000.0,
            "g": 4.65,
            "max_q": 46450.0,
            "radius": 1.85,
            "wall_mm": 5.0,
            "speed": 8012.0
        },
        {
            "title": "2. Lazer Işınlamalı Roket Gövdesi (Beamed Laser: Karbon Fiber T1000G)",
            "mat_id": "mat_carbon_fiber_t1000g",
            "mass": 7200.0,
            "g": 4.76,
            "max_q": 27330.0,
            "radius": 0.8,
            "wall_mm": 3.0,
            "speed": 7756.0
        },
        {
            "title": "3. Kinetik Fırlatıcı Burun Konisi (SpinLaunch: 739 kPa Max-Q & 20.2 G)",
            "mat_id": "mat_inconel_718",
            "mass": 3000.0,
            "g": 20.19,
            "max_q": 739050.0,
            "radius": 0.45,
            "wall_mm": 6.0,
            "speed": 1800.0
        }
    ]

    for sc in scenarios:
        mat = db.get(sc["mat_id"])
        print(f"▶ {sc['title']}")
        print(f"  Malzeme: {mat.name}")

        stress_res = StructuralStressChecker.check_rocket_hull(
            material=mat,
            total_mass_kg=sc["mass"],
            g_force=sc["g"],
            max_q_pa=sc["max_q"],
            hull_radius_m=sc["radius"],
            wall_thickness_mm=sc["wall_mm"]
        )

        thermal_res = StructuralStressChecker.check_aerodynamic_heating(
            material=mat,
            max_speed_ms=sc["speed"]
        )

        print(f"  • Hesaplanan Von Mises Gerilmesi: {stress_res['calculated_von_mises_MPa']} MPa (Akma Sınırı: {stress_res['material_yield_limit_MPa']} MPa)")
        print(f"  • Emniyet Katsayısı             : {stress_res['safety_factor_achieved']}x (Hedef: >= 1.35x)")
        print(f"  • Yapısal Karar                 : {stress_res['verdict']}")
        print(f"  • Hipersonik Şok Sıcaklığı       : {thermal_res['wall_temp_estimate_C']} °C (Malzeme Limiti: {thermal_res['material_max_temp_C']} °C)")
        print(f"  • Termal Karar                  : {thermal_res['verdict']}\n")

    print("=" * 70)

def cmd_harvest(target_type: str = "all"):
    print("=" * 70)
    print("🌾 OTOMATİZE LATEX DENKLEMİ HASAT MOTORU (EQUATION HARVESTER)")
    print("=" * 70)
    harvester = LatexEquationHarvester()

    total_harvested = []

    if target_type in ["local", "all"] and os.path.exists(ARCHIVE_MD):
        local_res = harvester.harvest_from_file(ARCHIVE_MD)
        total_harvested.extend(local_res)

    if target_type in ["online", "all"]:
        topics = [
            "Tsiolkovsky_rocket_equation",
            "Vis-viva_equation",
            "Bernoulli's_principle",
            "Lorentz_force",
            "Ideal_gas_law",
            "Hooke's_law",
            "Stefan–Boltzmann_law",
            "Escape_velocity",
            "Specific_impulse",
            "Oberth_effect"
        ]
        online_res = harvester.harvest_batch_topics(topics)
        total_harvested.extend(online_res)

    harvester.save_to_staging(total_harvested)
    triage = harvester.triage_staging_pool(DB_PATH)
    print(f"\n[+] İşlem tamamlandı. Toplam {triage['total_staging_candidates']} aday denklem elekten geçirildi.")
    print("=" * 70)


def cmd_triage():
    print("=" * 70)
    print("🔬 STAGING ADAY DENKLEM HAVUZU ELEME VE KARANTİNA RAPORU (DEĞİŞMEZ 3)")
    print("=" * 70)
    harvester = LatexEquationHarvester()
    summary = harvester.triage_staging_pool(DB_PATH)
    print(f"• Toplam Staging Aday Denklem : {summary['total_staging_candidates']} adet")
    print(f"• Mühürlü Kanonik Çekirdek    : {summary['canonical_core_laws']} yasa ({len(summary['canonical_domains'])} disiplin)")
    tb = summary["triage_breakdown"]
    print(f"  ├─ Kanonik Çekirdekte Karşılığı Olan Fizik/Sistem : {tb['promoted_or_covered_physics']} adet")
    print(f"  ├─ PolyZ/MultiPolyZ Saf Cebir & Halka Aksiyomları : {tb['pure_algebraic_ring_axioms']} adet")
    print(f"  ├─ Diferansiyel / İntegral / Limit Analitik Formu : {tb['differential_integral_forms']} adet")
    print(f"  └─ Karantinada Tutulan Eksik/Belirsiz Adaylar     : {tb['quarantined_staging']} adet")
    print(f"• Rapor Yazıldı               : {harvester.triage_report_path}")
    print("=" * 70)

def cmd_simulate(model_type: str = "compare"):
    print("=" * 70)
    print("🚀 4. DERECE RUNGE-KUTTA (RK4) SAYISAL YÖRÜNGE VE UÇUŞ SİMÜLATÖRÜ")
    print("=" * 70)
    print("Fiziksel Model: 2D Uçuş Dinamiği + Atmosferik Sürükleme + Barometrik Yoğunluk")
    print("Hedef Yörünge : LEO 200 km (Gereken Hız: ~7.784 m/s)\n")

    sim = RK4TrajectorySimulator(dt_seconds=0.1)

    v_chem = RocketVehicle(
        name="1. Geleneksel 2-Aşamalı Kimyasal Roket (LOX/RP-1)",
        dry_mass_kg=25000.0,
        propellant_mass_kg=400000.0,
        thrust_vacuum_n=7600000.0,
        isp_vacuum_s=300.0,
        cross_section_m2=10.5,
        drag_coeff=0.30,
        stage_2_dry_mass_kg=4500.0,
        stage_2_propellant_kg=92000.0,
        stage_2_thrust_n=934000.0,
        stage_2_isp_s=348.0
    )

    v_beamed = RocketVehicle(
        name="2. Lazer Işınlamalı Termal Tahrik (Beamed Laser SSTO)",
        dry_mass_kg=1200.0,
        propellant_mass_kg=6000.0,
        thrust_vacuum_n=120000.0,
        isp_vacuum_s=950.0,
        cross_section_m2=1.5,
        drag_coeff=0.25
    )

    v_kinetic = RocketVehicle(
        name="3. Kinetik Fırlatıcı + Üst Aşama Hibriti (SpinLaunch/Mass Driver)",
        dry_mass_kg=600.0,
        propellant_mass_kg=2400.0,
        thrust_vacuum_n=45000.0,
        isp_vacuum_s=335.0,
        cross_section_m2=0.8,
        drag_coeff=0.20,
        initial_velocity_ms=1800.0,
        initial_altitude_m=15000.0
    )

    if model_type == "chemical":
        vehicles = [v_chem]
    elif model_type == "beamed":
        vehicles = [v_beamed]
    elif model_type == "kinetic":
        vehicles = [v_kinetic]
    else:
        vehicles = [v_chem, v_beamed, v_kinetic]

    for veh in vehicles:
        print(f"▶ [{veh.name.upper()}] SİMÜLE EDİLİYOR...")
        print(f"  • Toplam Kalkış Kütlesi : {veh.total_mass:,.1f} kg")
        print(f"  • Sayısal RK4 Entegrasyonu yürütülüyor...")

        res = sim.simulate(veh, target_orbit_km=200.0)

        print(f"  [SONUÇ] {res['summary']}")
        print(f"    - Uçuş Süresi         : {res['flight_duration_s']} saniye ({res['flight_duration_s']/60.0:.1f} dk)")
        print(f"    - Erişilen İrtifa     : {res['final_altitude_km']} km (Azami: {res['max_altitude_km']} km)")
        print(f"    - Erişilen Hız        : {res['final_speed_ms']:,.1f} m/s ({res['final_speed_ms']*3.6:,.0f} km/h)")
        print(f"    - Kalan Yakıt         : {res['remaining_fuel_kg']:,.1f} kg")
        print(f"    - Max-Q Dinamik Basınç: {res['max_q_kPa']:.2f} kPa")
        print(f"    - Azami G Kuvveti     : {res['max_g_force']:.2f} G\n")

    print("=" * 70)

def cmd_audit() -> bool:
    print("=" * 70)
    print("⚖️ MATEMATİK ÇEKİRDEĞİ MÜHENDİSLİK KURAL DENETİMİ (KMATH AUDIT)")
    print("=" * 70)

    failures = 0

    # 1. Veritabanı Bütünlüğü ve JSON Denetimi
    if not os.path.exists(DB_PATH):
        print("[FAIL] 1. Kanonik Veritabanı (canonical_laws.json) bulunamadı!")
        return False

    with open(DB_PATH, "r", encoding="utf-8") as f:
        laws = json.load(f)

    print(f"[PASS] 1. Veritabanı Yapısı: {len(laws)} kanonik yasa yüklendi.")

    # 2. Kopya ID ve Tekillik Denetimi (Değişmez 6)
    seen_ids = set()
    dup_ids = []
    for l in laws:
        lid = l.get("id")
        if lid in seen_ids:
            dup_ids.append(lid)
        seen_ids.add(lid)

    if dup_ids:
        print(f"[FAIL] 2. Kopya Yasa ID'leri tespit edildi: {dup_ids}")
        failures += 1
    else:
        print("[PASS] 2. Saf Tekillik (SSOT): 0 adet kopya yasa ID'si.")

    # 3. Boyutsal Mutlaklık ve Tutarlılık Denetimi (Değişmez 1)
    dim_errors = []
    for l in laws:
        dims = l.get("dimensions", {})
        target = l.get("target")
        inputs = l.get("inputs", [])

        if target not in dims or dims[target] not in DIMENSIONS:
            dim_errors.append(f"{l['id']}: Hedef '{target}' için boyut tanımsız!")
            continue

        for inp in inputs:
            if inp not in dims or dims[inp] not in DIMENSIONS:
                dim_errors.append(f"{l['id']}: Girdi '{inp}' için boyut tanımsız!")

        ok_homog, msg_homog = verify_law_dimensional_homogeneity(l)
        if not ok_homog:
            dim_errors.append(msg_homog)

    if dim_errors:
        print(f"[FAIL] 3. Boyutsal Homojenlik / AST Hataları ({len(dim_errors)} hata):\n  - " + "\n  - ".join(dim_errors[:5]))
        failures += 1
    else:
        print(f"[PASS] 3. Boyutsal Mutlaklık: Tüm {len(laws)} yasanın AST cebiri dim(LHS) == dim(RHS) doğrulandı.")

    # 4. Asılı Sembol ve Değişken Sözleşme Denetimi (Değişmez 4)
    dangling = []
    for l in laws:
        eq_str = l.get("equation", "")
        if "=" not in eq_str:
            dangling.append(f"{l['id']}: Denklem '=' içermiyor!")
            continue
        lhs, rhs = eq_str.split("=", 1)
        lhs = lhs.strip()
        if lhs != l.get("target"):
            dangling.append(f"{l['id']}: LHS '{lhs}' ile target '{l.get('target')}' uyuşmuyor!")

        try:
            local_symbols = {name: sp.Symbol(name) for name in l.get("dimensions", {})}
            expr = safe_parse_math_expr(rhs.strip(), allowed_symbols=local_symbols)
            symbols = set(str(s) for s in expr.free_symbols)
            declared = set(l.get("inputs", []))
            unaccounted = symbols - declared
            if unaccounted:
                dangling.append(f"{l['id']}: Bildirilmemiş serbest semboller: {unaccounted}")
            missing_inputs = declared - symbols
            if missing_inputs:
                dangling.append(f"{l['id']}: Bildirildiği halde denklemde kullanılmayan semboller: {missing_inputs}")
        except Exception as e:
            dangling.append(f"{l['id']}: Sympy ayrıştırma hatası: {e}")

    if dangling:
        print(f"[FAIL] 4. Asılı Sembol/Sözleşme Hataları ({len(dangling)} hata):\n  - " + "\n  - ".join(dangling[:5]))
        failures += 1
    else:
        print("[PASS] 4. Sembolik Sözleşme: 0 adet asılı/tanımsız sembol (Tüm girdiler bağlı).")

    # 5. NIST CODATA 2022 & 2019 SI Tam Sabitler Eşlik Denetimi
    try:
        import math as _math
        assert os.path.exists(NIST_PATH), "data/nist_constants.json bulunamadı!"
        with open(NIST_PATH, "r", encoding="utf-8") as f_nist:
            nist_db = json.load(f_nist)
        exact_pairs = [
            ("c", "speed of light in vacuum"),
            ("G", "Newtonian constant of gravitation"),
            ("h", "Planck constant"),
            ("k_B", "Boltzmann constant"),
            ("e", "elementary charge"),
            ("alpha", "fine-structure constant"),
            ("mu_0", "vacuum mag. permeability"),
            ("eps_0", "vacuum electric permittivity"),
            ("N_A", "Avogadro constant"),
            ("g0", "standard acceleration of gravity"),
        ]
        for attr, nist_key in exact_pairs:
            val = getattr(PhysicalConstants, attr)
            ref_val = float(nist_db[nist_key]["value"])
            assert val == ref_val, f"{attr}={val} != NIST CODATA 2022 ({nist_key}={ref_val})"
        assert abs(PhysicalConstants.hbar - PhysicalConstants.h / (2.0 * _math.pi)) < 1e-49, "hbar != h/(2*pi)"
        assert abs(PhysicalConstants.R_gas - PhysicalConstants.N_A * PhysicalConstants.k_B) < 1e-14, "R_gas != N_A*k_B"
        sigma_exact = (_math.pi ** 2 * (PhysicalConstants.k_B ** 4)) / (
            60.0 * (PhysicalConstants.hbar ** 3) * (PhysicalConstants.c ** 2)
        )
        assert abs(PhysicalConstants.sigma_SB - sigma_exact) / sigma_exact < 1e-15, "sigma_SB != 2019 SI exact"
        assert abs(PhysicalConstants.mu_0 * PhysicalConstants.eps_0 * (PhysicalConstants.c ** 2) - 1.0) < 2e-10
        qed_alpha_rel = (2.0 * PhysicalConstants.alpha * PhysicalConstants.h) / (
            PhysicalConstants.mu_0 * PhysicalConstants.c * (PhysicalConstants.e ** 2)
        )
        assert abs(qed_alpha_rel - 1.0) < 2e-10
        print("[PASS] 5. Temel Sabitler: NIST CODATA 2022 JSON eşliği, 2019 SI tam cebirsel ilişkiler ve Maxwell/QED vakum özdeşliği doğrulandı.")
    except Exception as e:
        print(f"[FAIL] 5. Temel Sabitler bütünlüğü bozulmuş: {e}")
        failures += 1

    # 6. RK4 Sayısal Entegratör Sağlamlık Testi
    try:
        sim = RK4TrajectorySimulator(dt_seconds=0.5)
        test_v = RocketVehicle("Test", 1000.0, 5000.0, 100000.0, 800.0, 1.0)
        res = sim.simulate(test_v, max_time_s=10.0)
        assert res["flight_duration_s"] > 0
        print("[PASS] 6. Sayısal Entegratör: 4. Derece Runge-Kutta (RK4) fizik motoru aktif.")
    except Exception as e:
        print(f"[FAIL] 6. Sayısal Entegratör hatası: {e}")
        failures += 1

    # 7. Malzeme Bilimi ve Dayanım Kütüphanesi Bütünlük Denetimi
    try:
        mdb = MaterialsDatabase(MAT_PATH)
        assert len(mdb.materials) >= 25, "Eksik malzeme kaydı!"
        cnt = mdb.get("mat_cnt_bundle")
        assert cnt.breaking_length_km > 1000.0, "CNT kopma uzunluğu fiziksel olarak hatalı!"
        print(f"[PASS] 7. Malzeme Bilimi Kütüphanesi: {len(mdb.materials)} sertifikalı havacılık/UHTC/TPS malzemesi doğrulandı.")
    except Exception as e:
        print(f"[FAIL] 7. Malzeme Bilimi hatası: {e}")
        failures += 1

    # 8. SSOT Sembol Ontolojisi (ONTOLOGY.json) ve Faz 1.3 Rejim/DomainBounds Denetimi
    try:
        kg = KnowledgeGraph()
        kg.load_canonical_database(DB_PATH)
        assert os.path.exists(ONT_PATH), "ONTOLOGY.json bulunamadı!"
        with open(ONT_PATH, "r", encoding="utf-8") as f_ont:
            ont = json.load(f_ont)
        assert set(ont.keys()) == set(kg.variable_dimensions.keys()), "ONTOLOGY.json ile kanonik sembol kümesi uyumsuz!"
        illegal_paths = kg.search_paths_to_target(
            {"P_pitot", "P_stat", "s_cone", "A_nose"}, "F_drag_hyp", max_depth=3
        )
        assert len(illegal_paths) == 0, "Sıkıştırılamaz Pitot -> Hipersonik sürükleme rejim çatışması budanmadı!"
        print(
            f"[PASS] 8. SSOT Ontoloji & Faz 1.3 Rejim Kalkanı: {len(ont)} tekil sembol ve {len(kg.summary()['regimes'])} fiziksel rejim doğrulandı."
        )
    except Exception as e:
        print(f"[FAIL] 8. SSOT Ontoloji / Rejim Kalkanı hatası: {e}")
        failures += 1

    # 9. Z^7 Buckingham-Pi Çekirdek Uzayı Keşfi ve NSGA-II Kesin Pareto Skyline Eşliği
    try:
        b_res = BuckinghamPiDiscovery.discover(
            [
                ("r", DIMENSIONS["length"]),
                ("E", DIMENSIONS["energy"]),
                ("rho", DIMENSIONS["density"]),
                ("t", DIMENSIONS["time"]),
            ],
            target_var="r",
        )
        assert b_res["dimensional_rank"] == 3 and b_res["num_pi_groups"] == 1
        assert b_res["target_identifiable"] is True
        p_res = MultiObjectivePareto.analyze(
            ["min", "max"],
            [("A", [1000.0, 1.4]), ("B", [1220.0, 2.1]), ("C", [1850.0, 2.45]), ("D", [1450.0, 1.85])],
        )
        assert p_res["skyline_ids"] == ["A", "B", "C"] and p_res["knee_id"] == "B"
        print(
            "[PASS] 9. Buckingham-Pi Z^7 Nullspace & NSGA-II Pareto Skyline: Tam kesirli Pi keşfi ve cephe tamlığı doğrulandı."
        )
    except Exception as e:
        print(f"[FAIL] 9. Buckingham-Pi / Pareto Skyline hatası: {e}")
        failures += 1

    # 10. USSA-1976 7-Katmanlı Hidrostatik Atmosfer Sürekliliği & Einstein Eşdeğerlik İlkesi (0.0 G Serbest Düşüş)
    try:
        rho0, p0, t0_k, a0_ms = AtmosphereModel.state_at(0.0)
        assert abs(t0_k - 288.15) < 1e-9 and abs(p0 - 101325.0) < 1e-6 and abs(a0_ms - 340.294) < 0.01
        # Tropopoz (Z = 11 km -> h = 11019.1 m) ve Stratopoz (Z = 47 km -> h = 47350.1 m)
        _, _, t_trop, _ = AtmosphereModel.state_at(11019.13)
        _, _, t_strat, _ = AtmosphereModel.state_at(47350.10)
        assert abs(t_trop - 216.65) < 1e-3 and abs(t_strat - 270.65) < 1e-3

        sim_eq = RK4TrajectorySimulator(dt_seconds=0.2)
        v_freefall = RocketVehicle(
            name="Weightless Vacuum Orbit",
            dry_mass_kg=1000.0,
            propellant_mass_kg=0.0,
            thrust_vacuum_n=0.0,
            isp_vacuum_s=300.0,
            cross_section_m2=1.0,
            drag_coeff=0.0,
            initial_velocity_ms=7725.0,
            initial_altitude_m=300000.0,
        )
        res_ff = sim_eq.simulate(v_freefall, max_time_s=20.0)
        assert res_ff["max_g_force"] == 0.0, f"Vakum serbest düşüşte sahte G-yükü: {res_ff['max_g_force']}"

        v_hover = RocketVehicle(
            name="Pad Hover 1G",
            dry_mass_kg=1000.0,
            propellant_mass_kg=100.0,
            thrust_vacuum_n=1100.0 * PhysicalConstants.g0,
            isp_vacuum_s=1e9,
            cross_section_m2=1.0,
            drag_coeff=0.0,
        )
        _, _, _, _, _, _, _, _, hover_g = sim_eq._derivatives(0.0, (0.0, 0.0, 0.0, 0.0, 1100.0), v_hover, 1, 0.0)
        assert abs(hover_g - 1.0) < 1e-12, f"Rampa askı (hover) öz ivmesi 1.0 G değil: {hover_g}"
        print(
            "[PASS] 10. USSA-1976 Hidrostatik Atmosfer & Öz İvme Eşdeğerliği: 7 katman sürekliliği, vakum 0.0 G ve askı 1.0 G kanıtlandı."
        )
    except Exception as e:
        print(f"[FAIL] 10. USSA-1976 / Öz İvme denetimi hatası: {e}")
        failures += 1

    # 11. Altın Sayısal Referans Çapaları (%100 Kapsam — Boyutsuz Katsayı Kayması Kalkanı)
    try:
        anchor_count = 0
        missing_anchors = []
        for law in laws:
            anchors = law.get("reference_checks", [])
            if not anchors:
                missing_anchors.append(law["id"])
                continue
            _, rhs_str = law["equation"].split("=", 1)
            local_syms = {name: sp.Symbol(name) for name in law.get("dimensions", {})}
            expr = safe_parse_math_expr(rhs_str.strip(), allowed_symbols=local_syms)
            for anc in anchors:
                subs_map = {local_syms[k]: float(v) for k, v in anc["inputs"].items()}
                actual = float(expr.subs(subs_map).evalf())
                expected = float(anc["expected"])
                rel_tol = float(anc.get("rel_tol", 1e-4))
                rel_err = abs(actual - expected) / max(abs(expected), 1e-300)
                assert rel_err <= rel_tol, (
                    f"{law['id']} [{anc.get('label','')}]: actual={actual:.8e}, expected={expected:.8e}, rel_err={rel_err:.2e} > {rel_tol:.2e}"
                )
                anchor_count += 1
        assert not missing_anchors, f"%100 çapa kapsamı ihlali — {len(missing_anchors)} yasada reference_checks eksik: {missing_anchors[:5]}"
        print(f"[PASS] 11. Altın Sayısal Referans Çapaları (%100 Kapsam): {anchor_count}/{len(laws)} yasa test vektörü doğrulandı.")
    except Exception as e:
        print(f"[FAIL] 11. Altın Sayısal Referans Çapaları hatası: {e}")
        failures += 1

    # 12. Çapraz-Yasa Fiziksel Kapanış & Asimptotik Klasik Limit Denetimi
    try:
        import math as _math
        laws_by_id = {l["id"]: l for l in laws}

        def _eval_l(lid: str, vals: dict) -> float:
            l_obj = laws_by_id[lid]
            _, r_s = l_obj["equation"].split("=", 1)
            lsyms = {k: sp.Symbol(k) for k in l_obj["dimensions"]}
            ex = safe_parse_math_expr(r_s.strip(), allowed_symbols=lsyms)
            return float(ex.subs({lsyms[k]: float(v) for k, v in vals.items()}).evalf())

        c, G, hbar, kB = PhysicalConstants.c, PhysicalConstants.G, PhysicalConstants.hbar, PhysicalConstants.k_B
        m_sun = 1.98847e30
        r_s = _eval_l("law_schwarzschild_radius", {"G_grav": G, "M_body": m_sun, "c": c})
        s_bh = _eval_l("law_bekenstein_hawking_entropy", {"k_B": kB, "c": c, "A": 4.0 * _math.pi * r_s * r_s, "G": G, "hbar": hbar})
        t_h = _eval_l("law_hawking_black_hole_temperature", {"hbar": hbar, "c": c, "G": G, "M_body": m_sun, "k_B": kB})
        assert abs(t_h * (2.0 * s_bh / m_sun) - c * c) / (c * c) < 1e-11, "Kara delik 1. yasa kapanışı bozuk!"

        k_b = _eval_l("law_bulk_modulus_isotropic", {"E_mod": 210e9, "nu_pois": 0.29})
        lam_e = _eval_l("law_lame_first_parameter", {"E_mod": 210e9, "nu_pois": 0.29})
        g_sh = _eval_l("law_isotropic_shear_modulus", {"E_mod": 210e9, "nu_pois": 0.29})
        assert abs(k_b - (lam_e + (2.0 / 3.0) * g_sh)) / k_b < 1e-12, "Elastisite K == lambda + 2/3*G kapanışı bozuk!"

        cop_hp = _eval_l("law_carnot_heat_pump_cop", {"T_h": 600.0, "T_c": 300.0})
        cop_r = _eval_l("law_carnot_refrigerator_cop", {"T_h": 600.0, "T_c": 300.0})
        assert abs((cop_hp - cop_r) - 1.0) < 1e-12, "Carnot COP_hp - COP_ref == 1 kapanışı bozuk!"

        v_low = 1e-4 * c
        g_low = _eval_l("law_lorentz_factor", {"v": v_low, "c": c})
        ek_r = _eval_l("law_relativistic_kinetic_energy", {"gamma_rel": g_low, "m": 10.0, "c": c})
        ek_c = _eval_l("law_kinetic_energy", {"m": 10.0, "v": v_low})
        assert abs(ek_r - ek_c) / ek_c < 1e-6, "Göreli -> Klasik kinetik enerji asimptotik limiti bozuk!"

        fanno_v = _eval_l("law_fanno_friction_parameter", {"f_darcy": 0.02, "L": 10.0, "D_pipe": 0.1})
        dp_dw = _eval_l("law_darcy_weisbach_pressure_drop", {"f_darcy": 0.02, "L": 10.0, "D_pipe": 0.1, "rho": 1000.0, "v": 2.0})
        q_dyn = _eval_l("law_dynamic_pressure", {"rho": 1000.0, "v": 2.0})
        assert abs(fanno_v - (dp_dw / q_dyn)) / fanno_v < 1e-12, "Fanno Darcy == dP_dw / q_dyn kapanışı bozuk!"
        print("[PASS] 12. Çapraz-Yasa Kapanış & Asimptotik Limitler: Kara delik 1. yasa, Elastisite, Carnot, Fanno-Darcy ve Göreli->Klasik limit doğrulandı.")
    except Exception as e:
        print(f"[FAIL] 12. Çapraz-Yasa Kapanış / Limit hatası: {e}")
        failures += 1

    # 13. Malzeme Ashby Fiziksel Zarfları, Isıl İşlem/Kırılma Modu ve >=3 Kaynak Uzlaşma Kalkanı
    try:
        mdb_env = MaterialsDatabase(MAT_PATH)
        valid_modes = {"ductile", "brittle", "composite_laminate", "ablative", "foam_insulator"}
        for m in mdb_env.materials.values():
            ys = m.yield_strength / 1e6
            uts = m.tensile_strength / 1e6
            assert m.density > 0 and uts >= ys > 0 and m.youngs_modulus > 0, f"{m.id}: geçersiz mekanik değer"
            assert m.condition.strip(), f"{m.id}: 'condition' eksik"
            assert m.failure_mode in valid_modes, f"{m.id}: geçersiz failure_mode={m.failure_mode}"
            assert len(m.sources) >= 3, f"{m.id}: <3 kaynak ({len(m.sources)})"
            assert m.yield_strength_range_mpa and len(m.yield_strength_range_mpa) == 2, f"{m.id}: yield_strength_range_mpa eksik"
            lo_y, hi_y = m.yield_strength_range_mpa
            assert lo_y <= ys + 1e-6 and ys <= hi_y + 1e-6 and lo_y > 0, f"{m.id}: nominal {ys} ∉ [{lo_y}, {hi_y}]"
        print(f"[PASS] 13. Malzeme Ashby Zarfları & 3-Kaynak Uzlaşması: {len(mdb_env.materials)} malzeme (durum, kırılma modu, aralık, >=3 kaynak) doğrulandı.")
    except Exception as e:
        print(f"[FAIL] 13. Malzeme Ashby Zarfı / Metadata hatası: {e}")
        failures += 1

    print("=" * 70)
    if failures == 0:
        print("🏆 SONUÇ: TÜM MÜHENDİSLİK DENETİMLERİ GEÇTİ (0 HATA). SİSTEM MÜHÜRLÜ.")
        print("=" * 70)
        return True
    else:
        print(f"❌ SONUÇ: {failures} ADET MÜHENDİSLİK KURAL İHLALİ TESPİT EDİLDİ!")
        print("=" * 70)
        return False

def main() -> None:
    """Console script entry point for `kmath` and `uvx --from kmath-mcp kmath`."""
    if len(sys.argv) > 1 and sys.argv[1] == "rules":
        cmd_rules()
    elif len(sys.argv) > 1 and sys.argv[1] == "stats":
        cmd_stats()
    elif len(sys.argv) > 1 and sys.argv[1] == "harvest":
        sub = sys.argv[2] if len(sys.argv) > 2 else "all"
        cmd_harvest(sub)
    elif len(sys.argv) > 1 and sys.argv[1] == "triage":
        cmd_triage()
    elif len(sys.argv) > 1 and sys.argv[1] == "simulate":
        sub = sys.argv[2] if len(sys.argv) > 2 else "compare"
        cmd_simulate(sub)
    elif len(sys.argv) > 1 and sys.argv[1] == "materials":
        cmd_materials()
    elif len(sys.argv) > 1 and sys.argv[1] == "stress":
        cmd_stress()
    elif len(sys.argv) > 1 and sys.argv[1] == "buckingham":
        extra = " ".join(shlex.quote(a) for a in sys.argv[2:])
        cmd_rust(f"buckingham {extra}".strip())
    elif len(sys.argv) > 1 and sys.argv[1] == "design":
        extra = " ".join(sys.argv[2:])
        cmd_rust(f"design {extra}".strip())
    elif len(sys.argv) > 1 and sys.argv[1] == "rust":
        sub = " ".join(sys.argv[2:]) if len(sys.argv) > 2 else "audit"
        cmd_rust(sub)
    else:
        success = cmd_audit()
        sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
