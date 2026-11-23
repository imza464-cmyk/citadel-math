#!/usr/bin/env python3
# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
# Matematik Çekirdeği — Egemen Model Context Protocol (MCP) Sunucusu (kmath_mcp)

"""
Citadel-Math Deterministik Matematik ve Fizik Doğrulama MCP Sunucusu.
Anayasal Kurallar (AGENTS.md):
  - Değişmez 1: Z^7 (7-Temel SI Boyutu) sembolik AST boyutsal homojenlik kalkanı.
  - Değişmez 2: Termodinamik 1./2. Yasa (Carnot), Görelilik (v < c) ve Momentum budama kalkanı.
  - Değişmez 4: Tanım kümesi yan koşulları (!= 0, > 0, >= 0), Interval64 ve RK4/RK45 entegrasyonu.
  - Değişmez 6: %100 Saf Tekillik Kaynağı (SSOT: canonical_laws.json, materials.json, nist_constants.json, citadel_math_core).
  - Sıfır Sessiz Hata Yutma: Hiçbir istisna 'except Exception: pass' ile gizlenmez.
"""

import functools
import inspect
import json
import os
import shutil
import subprocess
import sys
from typing import Annotated, Any, Dict, List, Optional, Tuple

os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")

import anyio
from pydantic import Field
import sympy as sp
try:
    from mcp.server.fastmcp import FastMCP
except ImportError:
    from mcp.server.mcpserver import MCPServer as FastMCP

ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

import engine
from engine.constants import PhysicalConstants
from engine.dimensions import (
    DIMENSIONS,
    Dimension,
    check_dimensional_consistency,
    infer_sympy_dimension,
    safe_parse_math_expr,
    verify_law_dimensional_homogeneity,
)
from engine.knowledge_graph import KnowledgeGraph
from engine.materials import MaterialsDatabase
from engine.simulator import RK4TrajectorySimulator, RocketVehicle


def _resolve_data_file(*parts: str) -> str:
    """Kaynak ağaçtaki (repo) veya paketlenmiş wheel (engine/_ssot_data) içindeki SSOT veri dosyasını çözer."""
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
    """Derlenmiş citadel_math_core ikilisini deterministik öncelik sırasıyla arar.

    Arama sırası:
      1. KMATH_RUST_BIN ortam değişkeni (tanımlıysa tek yetkili geçersiz kılmadır)
      2. <repo>/citadel_math_core/target/release/citadel_math_core(.exe)
      3. <repo>/citadel_math_core/target/debug/citadel_math_core(.exe)
      4. <repo>/bin/citadel_math_core(.exe)
      5. Sistem PATH'inde citadel_math_core ve GitHub Releases platform ikilileri
    """
    searched: List[str] = []
    env_override = os.environ.get("KMATH_RUST_BIN", "").strip()
    if env_override:
        expanded = os.path.abspath(os.path.expanduser(env_override))
        if os.path.isfile(expanded):
            return expanded, [expanded]
        return None, [f"KMATH_RUST_BIN={expanded} (bulunamadı / not found)"]

    bin_name = "citadel_math_core.exe" if os.name == "nt" else "citadel_math_core"
    local_candidates = [
        os.path.join(ROOT_DIR, "citadel_math_core", "target", "release", bin_name),
        os.path.join(ROOT_DIR, "citadel_math_core", "target", "debug", bin_name),
        os.path.join(ROOT_DIR, "bin", bin_name),
    ]
    for candidate in local_candidates:
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
_DEFAULT_RUST_BIN = os.path.join(
    ROOT_DIR,
    "citadel_math_core",
    "target",
    "release",
    "citadel_math_core.exe" if os.name == "nt" else "citadel_math_core",
)
RUST_BIN = resolve_rust_bin()[0] or _DEFAULT_RUST_BIN

mcp = FastMCP(
    "kmath",
    instructions=(
        "Citadel-Math Deterministik Matematik ve Fizik Doğrulama MCP Sunucusu. "
        "Olasılıksal tahmin yerine Z^7 sembolik AST boyut analizi, Z[x1..xn] kanonik polinom "
        "ve yan-koşul (GuardedPoly) ispat çekirdeği, IEEE-754 dışa yuvarlamalı Interval64 + Dual64 "
        "otomatik türev, 355 NIST CODATA sabiti, AND-OR hipergraf formül sentezi, RK4/RK45 yörünge "
        "simülasyonu, NASA SP-8007 burkulma + PICA-X TPS dayanım analizi ve 6.912 adaylı Pareto "
        "tersine mühendislik tasarım motorunu deterministik olarak sunar."
    ),
)


def _async_mcp_tool(fn):
    """MCP sunucusunda paralel çağrıların event-loop'u bloklamaması için aracı
    `anyio.to_thread.run_sync` üzerinde asenkron kaydeder; doğrudan Python çağrıları
    (unittest, Docker HEALTHCHECK) için senkron fonksiyon referansını korur."""
    sig = inspect.signature(fn)

    @functools.wraps(fn)
    async def _wrapper(*args: Any, **kwargs: Any) -> Any:
        return await anyio.to_thread.run_sync(functools.partial(fn, *args, **kwargs))

    _wrapper.__signature__ = sig  # type: ignore[attr-defined]
    mcp.tool(name=fn.__name__, description=fn.__doc__)(_wrapper)
    return fn


SI_AXIS_DIMENSIONS: Dict[str, Dimension] = {
    "M": Dimension(m=1),
    "L": Dimension(l=1),
    "T": Dimension(t=1),
    "I": Dimension(i=1),
    "Theta": Dimension(theta=1),
    "THETA": Dimension(theta=1),
    "N": Dimension(n=1),
    "J": Dimension(j=1),
    "1": Dimension(),
}

# Temel fiziksel sabitlerin ve SI birim sembollerinin Z^7 boyut ve ölçek eşlemeleri (SSOT uzantısı)
FUNDAMENTAL_SYMBOL_DIMENSIONS: Dict[str, Dimension] = {
    "G": Dimension(m=-1, l=3, t=-2),
    "c": Dimension(l=1, t=-1),
    "h": Dimension(m=1, l=2, t=-1),
    "hbar": Dimension(m=1, l=2, t=-1),
    "k_B": Dimension(m=1, l=2, t=-2, theta=-1),
    "e_charge": Dimension(t=1, i=1),
    "mu_0": Dimension(m=1, l=1, t=-2, i=-2),
    "eps_0": Dimension(m=-1, l=-3, t=4, i=2),
    "epsilon_0": Dimension(m=-1, l=-3, t=4, i=2),
    "sigma_SB": Dimension(m=1, t=-3, theta=-4),
    "N_A": Dimension(n=-1),
    "R_gas": Dimension(m=1, l=2, t=-2, theta=-1, n=-1),
    "g_0": Dimension(l=1, t=-2),
    "g0": Dimension(l=1, t=-2),
    # Yaygın genel fizik sembolleri (kanonik tablodaki sembollere ek varsayılanlar)
    "a": Dimension(l=1, t=-2),
    "m": Dimension(m=1),
    "m1": Dimension(m=1),
    "m2": Dimension(m=1),
    "M": Dimension(m=1),
    "r": Dimension(l=1),
    "x": Dimension(l=1),
    "t": Dimension(t=1),
    "v": Dimension(l=1, t=-1),
    "F": Dimension(m=1, l=1, t=-2),
    "E": Dimension(m=1, l=2, t=-2),
    "P": Dimension(m=1, l=2, t=-3),
    "T": Dimension(theta=1),
    "q": Dimension(t=1, i=1),
    "I": Dimension(i=1),
    "V": Dimension(m=1, l=2, t=-3, i=-1),
    "B": Dimension(m=1, t=-2, i=-1),
    "rho": Dimension(m=1, l=-3),
}

# SI Birim Sembolleri: (SI çarpanı, Z^7 Boyutu)
SI_UNIT_TABLE: Dict[str, Tuple[float, Dimension]] = {
    "1": (1.0, Dimension()),
    "kg": (1.0, Dimension(m=1)),
    "g": (1e-3, Dimension(m=1)),
    "ton": (1e3, Dimension(m=1)),
    "m": (1.0, Dimension(l=1)),
    "mm": (1e-3, Dimension(l=1)),
    "cm": (1e-2, Dimension(l=1)),
    "km": (1e3, Dimension(l=1)),
    "s": (1.0, Dimension(t=1)),
    "ms": (1e-3, Dimension(t=1)),
    "us": (1e-6, Dimension(t=1)),
    "ns": (1e-9, Dimension(t=1)),
    "A": (1.0, Dimension(i=1)),
    "K": (1.0, Dimension(theta=1)),
    "mol": (1.0, Dimension(n=1)),
    "cd": (1.0, Dimension(j=1)),
    "Hz": (1.0, Dimension(t=-1)),
    "N": (1.0, Dimension(m=1, l=1, t=-2)),
    "kN": (1e3, Dimension(m=1, l=1, t=-2)),
    "MN": (1e6, Dimension(m=1, l=1, t=-2)),
    "Pa": (1.0, Dimension(m=1, l=-1, t=-2)),
    "kPa": (1e3, Dimension(m=1, l=-1, t=-2)),
    "MPa": (1e6, Dimension(m=1, l=-1, t=-2)),
    "GPa": (1e9, Dimension(m=1, l=-1, t=-2)),
    "bar": (1e5, Dimension(m=1, l=-1, t=-2)),
    "J": (1.0, Dimension(m=1, l=2, t=-2)),
    "kJ": (1e3, Dimension(m=1, l=2, t=-2)),
    "MJ": (1e6, Dimension(m=1, l=2, t=-2)),
    "W": (1.0, Dimension(m=1, l=2, t=-3)),
    "kW": (1e3, Dimension(m=1, l=2, t=-3)),
    "MW": (1e6, Dimension(m=1, l=2, t=-3)),
    "GW": (1e9, Dimension(m=1, l=2, t=-3)),
    "C": (1.0, Dimension(t=1, i=1)),
    "V": (1.0, Dimension(m=1, l=2, t=-3, i=-1)),
    "Ohm": (1.0, Dimension(m=1, l=2, t=-3, i=-2)),
    "ohm": (1.0, Dimension(m=1, l=2, t=-3, i=-2)),
    "F": (1.0, Dimension(m=-1, l=-2, t=4, i=2)),
    "H": (1.0, Dimension(m=1, l=2, t=-2, i=-2)),
    "T": (1.0, Dimension(m=1, t=-2, i=-1)),
}

NIST_SYMBOL_ALIASES: Dict[str, str] = {
    "c": "speed of light in vacuum",
    "G": "Newtonian constant of gravitation",
    "h": "Planck constant",
    "hbar": "reduced Planck constant",
    "k_B": "Boltzmann constant",
    "e": "elementary charge",
    "m_e": "electron mass",
    "m_p": "proton mass",
    "m_n": "neutron mass",
    "mu_0": "vacuum mag. permeability",
    "eps_0": "vacuum electric permittivity",
    "epsilon_0": "vacuum electric permittivity",
    "sigma": "Stefan-Boltzmann constant",
    "sigma_SB": "Stefan-Boltzmann constant",
    "N_A": "Avogadro constant",
    "R": "molar gas constant",
    "R_gas": "molar gas constant",
    "g0": "standard acceleration of gravity",
    "g_0": "standard acceleration of gravity",
    "alpha": "fine-structure constant",
    "a_0": "Bohr radius",
    "R_inf": "Rydberg constant",
    "F": "Faraday constant",
    "u": "unified atomic mass unit",
}


def _run_rust_cli(args: List[str], timeout_s: float = 20.0) -> Tuple[int, str, str]:
    """Derlenmiş saf-Rust citadel_math_core ikilisini kabuk enjeksiyonsuz çalıştırır."""
    rust_bin, searched = resolve_rust_bin()
    if not rust_bin:
        searched_str = ", ".join(searched)
        return (
            127,
            "",
            (
                f"citadel_math_core Rust ikilisi bulunamadı / Rust binary not found. "
                f"Aranan yollar / Searched: [{searched_str}]. "
                f"Çözüm / Fix: (1) Kaynaktan derleyin: `cargo build --release --manifest-path citadel_math_core/Cargo.toml`, "
                f"veya (2) Hazır ikiliyi indirin: https://github.com/imza464-cmyk/citadel-math/releases "
                f"ve `KMATH_RUST_BIN` ortam değişkenini (veya PATH'i) ayarlayın."
            ),
        )
    proc = subprocess.run(
        [rust_bin, *args],
        cwd=ROOT_DIR if os.path.isdir(ROOT_DIR) else None,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout_s,
        check=False,
    )
    return proc.returncode, proc.stdout.strip(), proc.stderr.strip()


def _canonical_vehicles() -> Dict[str, RocketVehicle]:
    """Python RK4 simülatörü için kanonik referans araç tanımları (kmath.py ile birebir SSOT)."""
    return {
        "chemical": RocketVehicle(
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
            stage_2_isp_s=348.0,
        ),
        "beamed": RocketVehicle(
            name="2. Lazer Işınlamalı Termal Tahrik (Beamed Laser SSTO)",
            dry_mass_kg=1200.0,
            propellant_mass_kg=6000.0,
            thrust_vacuum_n=120000.0,
            isp_vacuum_s=950.0,
            cross_section_m2=1.5,
            drag_coeff=0.25,
        ),
        "kinetic": RocketVehicle(
            name="3. Kinetik Fırlatıcı + Üst Aşama Hibriti (SpinLaunch/Mass Driver)",
            dry_mass_kg=600.0,
            propellant_mass_kg=2400.0,
            thrust_vacuum_n=45000.0,
            isp_vacuum_s=335.0,
            cross_section_m2=0.8,
            drag_coeff=0.20,
            initial_velocity_ms=1800.0,
            initial_altitude_m=15000.0,
        ),
    }


def _parse_dimension_spec(spec: str) -> Optional[Dimension]:
    """Boyut kategorisi ('length'), SI birimi ('m/s'), SI temel ekseni ('L', 'M', 'T') veya
    boyutsal ifade ('M * L / T^2', 'M L T^-2') dizesini Z^7 Dimension nesnesine çevirir."""
    key = spec.strip()
    if not key:
        return None
    if key.lower() in DIMENSIONS:
        return DIMENSIONS[key.lower()]
    if key in SI_AXIS_DIMENSIONS:
        return SI_AXIS_DIMENSIONS[key]
    if key in SI_UNIT_TABLE:
        return SI_UNIT_TABLE[key][1]
    # Boşlukla ayrılmış SI boyut notasyonunu ('M L T^-2') çarpıma dönüştür
    tokens = key.split()
    expr_candidate = "*".join(tokens) if (len(tokens) > 1 and all(tok[0].isalpha() for tok in tokens)) else key
    expr_norm = expr_candidate.replace("^", "**")
    axis_syms = {k: sp.Symbol(k) for k in SI_AXIS_DIMENSIONS if k != "1"}
    try:
        parsed = safe_parse_math_expr(expr_norm, allowed_symbols=axis_syms)
        free_names = {str(s) for s in parsed.free_symbols}
        if free_names and free_names.issubset(axis_syms.keys()):
            return infer_sympy_dimension(parsed, SI_AXIS_DIMENSIONS)
    except (ValueError, TypeError, sp.SympifyError):
        pass
    unit_parsed = _try_parse_unit_expression(key)
    if unit_parsed is not None:
        return unit_parsed[1]
    return None


def _build_ssot_symbol_dimensions(extra_dims: Optional[Dict[str, str]] = None) -> Dict[str, Dimension]:
    """canonical_laws.json + temel fizik sabitleri + kullanıcı ek tanımlarından Z^7 sembol tablosu kurar."""
    var_dims: Dict[str, Dimension] = dict(FUNDAMENTAL_SYMBOL_DIMENSIONS)
    if os.path.exists(DB_PATH):
        kg = KnowledgeGraph()
        kg.load_canonical_database(DB_PATH)
        var_dims.update(kg.variable_dimensions)

    if extra_dims:
        for sym_name, dim_spec in extra_dims.items():
            resolved = _parse_dimension_spec(dim_spec)
            if resolved is not None:
                var_dims[sym_name] = resolved
            else:
                sample_cats = ", ".join(sorted(list(DIMENSIONS.keys()))[:12])
                raise ValueError(
                    f"Değişken '{sym_name}' için geçersiz boyut/birim anahtarı: '{dim_spec}'. "
                    f"Geçerli formatlar: SI boyut eksenleri (M, L, T, I, Theta, N, J veya 'M L T^-2'), "
                    f"SI birimleri (m, kg, s, N, Pa, kW...) veya boyut kategorileri ({sample_cats}, ...)."
                )
    return var_dims


def _try_parse_unit_expression(expr_str: str) -> Optional[Tuple[float, Dimension]]:
    """İfadenin tüm sembolleri SI_UNIT_TABLE birimlerinden oluşuyorsa (örn. 'km/s', 'kg*m/s^2', 'kPa')
    hem toplam SI ölçek çarpanını hem de Z^7 boyutunu hesaplar."""
    normalized = expr_str.strip().replace("^", "**")
    if normalized in SI_UNIT_TABLE:
        return SI_UNIT_TABLE[normalized]

    local_syms: Dict[str, sp.Symbol] = {u: sp.Symbol(u) for u in SI_UNIT_TABLE}
    try:
        parsed = safe_parse_math_expr(normalized, allowed_symbols=local_syms)
    except (sp.SympifyError, TypeError, ValueError):
        return None

    free_names = {str(s) for s in parsed.free_symbols}
    if not free_names or not free_names.issubset(SI_UNIT_TABLE.keys()):
        return None

    def eval_unit_node(node: sp.Expr) -> Tuple[float, Dimension]:
        if node.is_Number:
            return float(node), Dimension()
        if node.is_Symbol:
            return SI_UNIT_TABLE[str(node)]
        if node.is_Mul:
            scale_acc = 1.0
            dim_acc = Dimension()
            for arg in node.args:
                sc, dm = eval_unit_node(arg)
                scale_acc *= sc
                dim_acc = dim_acc * dm
            return scale_acc, dim_acc
        if node.is_Pow:
            base, exp = node.as_base_exp()
            b_scale, b_dim = eval_unit_node(base)
            rat = sp.nsimplify(exp, rational=True)
            if not rat.is_Rational:
                raise ValueError(f"Birim üssü rasyonel olmalıdır: {exp}")
            p, q = int(rat.p), int(rat.q)
            return (b_scale ** (p / q)), b_dim.rational_pow(p, q)
        raise ValueError(f"Birim ifadesinde desteklenmeyen işlem: {node}")

    try:
        return eval_unit_node(parsed)
    except ValueError:
        return None


def _resolve_expression_dimension(
    side_str: str, var_dims: Dict[str, Dimension]
) -> Tuple[Dimension, Optional[float], str]:
    """Bir tarafın (boyut adı, SI birim ifadesi veya sembolik AST denklemi) Z^7 boyutunu çıkarır."""
    cleaned = side_str.strip()
    lower = cleaned.lower()

    # 1. Doğrudan kanonik boyut kategorisi adı (örn. 'force', 'magnetic_field', 'power')
    if lower in DIMENSIONS:
        return DIMENSIONS[lower], None, "dimension_category"

    # 2. Yalnızca tekil sembol ise ve kanonik/kullanıcı değişken tablosunda tanımlıysa önce değişkeni seç
    if cleaned in var_dims:
        return var_dims[cleaned], None, "canonical_symbol"

    # 3. Ölçekli SI birim ifadesi (örn. 'kPa', 'km/s', 'kg*m/s^2')
    unit_parsed = _try_parse_unit_expression(cleaned)
    if unit_parsed is not None:
        scale, u_dim = unit_parsed
        return u_dim, scale, "si_unit_expression"

    # 4. Tam Sembolik AST İfadesi (örn. 'G * m1 * m2 / r**2', 'v_e * log(m_0 / m_f)')
    expr_norm = cleaned.replace("^", "**")
    local_symbols = {name: sp.Symbol(name) for name in var_dims}
    parsed = safe_parse_math_expr(expr_norm, allowed_symbols=local_symbols)
    inferred = infer_sympy_dimension(parsed, var_dims)
    return inferred, None, "symbolic_ast"


@_async_mcp_tool
def kmath_audit() -> str:
    """Matematik Çekirdeğinin hem 10 katmanlı Python SSOT denetimini hem de 16 katmanlı saf-Rust Citadel-Math kanıt denetimini koşturur."""
    if not os.path.exists(DB_PATH):
        return json.dumps(
            {"status": "error", "message": f"canonical_laws.json bulunamadı: {DB_PATH}"},
            ensure_ascii=False,
        )

    with open(DB_PATH, "r", encoding="utf-8") as f:
        laws = json.load(f)

    seen_ids = set()
    dup_ids = []
    dim_errors = []
    dangling_errors = []

    for law in laws:
        lid = law.get("id", "<missing>")
        if lid in seen_ids:
            dup_ids.append(lid)
        seen_ids.add(lid)

        ok_h, msg_h = verify_law_dimensional_homogeneity(law)
        if not ok_h:
            dim_errors.append(f"{lid}: {msg_h}")

        eq_str = law.get("equation", "")
        if "=" not in eq_str:
            dangling_errors.append(f"{lid}: '=' eksik")
            continue
        lhs_s, rhs_s = eq_str.split("=", 1)
        if lhs_s.strip() != law.get("target"):
            dangling_errors.append(f"{lid}: LHS '{lhs_s.strip()}' != target '{law.get('target')}'")
        local_syms = {name: sp.Symbol(name) for name in law.get("dimensions", {})}
        parsed_rhs = safe_parse_math_expr(rhs_s.strip(), allowed_symbols=local_syms)
        free_s = {str(s) for s in parsed_rhs.free_symbols}
        decl_s = set(law.get("inputs", []))
        if free_s != decl_s:
            dangling_errors.append(f"{lid}: AST {free_s} != inputs {decl_s}")

    with open(NIST_PATH, "r", encoding="utf-8") as f:
        nist_data = json.load(f)

    sim = RK4TrajectorySimulator(dt_seconds=0.5)
    test_veh = _canonical_vehicles()["beamed"]
    sim_res = sim.simulate(test_veh, max_time_s=10.0)
    rk4_ok = sim_res["flight_duration_s"] > 0.0

    mdb = MaterialsDatabase(MAT_PATH)
    kg = KnowledgeGraph()
    kg.load_canonical_database(DB_PATH)
    kg_summary = kg.summary()
    ont_count = 0
    if os.path.exists(ONT_PATH):
        with open(ONT_PATH, "r", encoding="utf-8") as f_ont:
            ont_count = len(json.load(f_ont))
    rc, rust_out, rust_err = _run_rust_cli(["audit"])

    return json.dumps(
        {
            "status": "ok"
            if (not dup_ids and not dim_errors and not dangling_errors and rk4_ok and rc == 0)
            else "fail",
            "python_ssot_audit": {
                "canonical_laws": len(laws),
                "ssot_ontology_symbols": ont_count,
                "domains": kg_summary["domains"],
                "regimes": kg_summary["regimes"],
                "duplicate_ids": dup_ids,
                "dimensional_errors": dim_errors,
                "dangling_symbol_errors": dangling_errors,
                "nist_codata_constants": len(nist_data),
                "rk4_integrator_active": rk4_ok,
                "certified_materials": len(mdb.materials),
            },
            "rust_citadel_core_audit": {
                "exit_code": rc,
                "passed": rc == 0,
                "report": rust_out if rc == 0 else rust_err,
            },
        },
        indent=2,
        ensure_ascii=False,
    )


@_async_mcp_tool
def kmath_dimension_check(
    lhs: Annotated[str, Field(description="Sol taraf ifadesi, SI birimi veya boyut adı (örn: 'F', 'kN', 'm * a', 'P * r / t', 'delta_v').")],
    rhs: Annotated[str, Field(description="Sağ taraf ifadesi, SI birimi veya boyut adı (örn: 'kg * m / s^2', 'Pa', 'G * m1 * m2 / r**2').")],
    var_dimensions_json: Annotated[str, Field(description="Opsiyonel özel değişken boyut sözlüğü JSON'u. SI ekseni ('L', 'M', 'T', 'M L T^-2'), kategori ('length', 'pressure') veya SI birimi ('Pa', 'm') kabul eder (örn: '{\"d\": \"L\", \"P\": \"pressure\"}').")] = "",
) -> str:
    """İki fiziksel ifadenin, SI biriminin veya denklemin Z^7 ([M, L, T, I, Theta, N, J]) boyutsal homojenliğini ve birim ölçek eşliğini denetler."""
    extra_dims: Optional[Dict[str, str]] = None
    if var_dimensions_json.strip():
        try:
            extra_dims = json.loads(var_dimensions_json)
        except json.JSONDecodeError as exc:
            return json.dumps(
                {"status": "error", "message": f"var_dimensions_json geçerli JSON değil: {exc}"},
                ensure_ascii=False,
            )

    try:
        var_dims = _build_ssot_symbol_dimensions(extra_dims)
        dim_l, scale_l, mode_l = _resolve_expression_dimension(lhs, var_dims)
        dim_r, scale_r, mode_r = _resolve_expression_dimension(rhs, var_dims)
    except (ValueError, sp.SympifyError, TypeError) as exc:
        return json.dumps(
            {
                "status": "error",
                "lhs": lhs,
                "rhs": rhs,
                "homogeneous": False,
                "verdict": "DIMENSIONAL_OR_SYNTAX_ERROR",
                "message": str(exc),
                "hint": "Tanımsız semboller için var_dimensions_json ile boyut tanımlayın (örn: '{\"d\": \"L\"}' veya '{\"d\": \"length\"}').",
            },
            indent=2,
            ensure_ascii=False,
        )

    homogeneous, msg = check_dimensional_consistency(dim_l, dim_r)
    scale_ratio: Optional[float] = None
    scale_match: Optional[bool] = None
    if homogeneous and scale_l is not None and scale_r is not None:
        scale_ratio = scale_l / scale_r
        scale_match = abs(scale_ratio - 1.0) < 1e-12

    verdict = "DIMENSIONALLY_HOMOGENEOUS" if homogeneous else "DIMENSIONAL_MISMATCH"
    if homogeneous and scale_match is False:
        verdict = "DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH"

    return json.dumps(
        {
            "status": "ok",
            "lhs": lhs,
            "rhs": rhs,
            "lhs_mode": mode_l,
            "rhs_mode": mode_r,
            "dim_lhs": repr(dim_l),
            "dim_rhs": repr(dim_r),
            "z7_tuple_lhs": list(dim_l.as_tuple()),
            "z7_tuple_rhs": list(dim_r.as_tuple()),
            "homogeneous": homogeneous,
            "si_scale_lhs": scale_l,
            "si_scale_rhs": scale_r,
            "si_scale_ratio_lhs_over_rhs": scale_ratio,
            "verdict": verdict,
            "detail": msg,
        },
        indent=2,
        ensure_ascii=False,
    )


@_async_mcp_tool
def kmath_constant(
    query: Annotated[str, Field(description="Sabit sembolü ('c', 'G', 'h', 'hbar', 'k_B', 'e', 'm_e', 'm_p', 'mu_0', 'eps_0', 'sigma_SB', 'N_A', 'R_gas', 'g0', 'alpha', 'M_earth', 'mu_earth') veya İngilizce NIST adı/alt-dizesi ('Planck', 'magnetic', 'proton').")],
) -> str:
    """355 resmi NIST CODATA fiziksel sabiti ve IAU gezegen parametreleri içerisinden kesin değer, birim ve tam duyarlılık sorgular."""
    q = query.strip()
    if not q:
        return json.dumps({"status": "error", "message": "Boş sabit sorgusu."}, ensure_ascii=False)

    if not os.path.exists(NIST_PATH):
        return json.dumps(
            {"status": "error", "message": f"nist_constants.json bulunamadı: {NIST_PATH}"},
            ensure_ascii=False,
        )

    with open(NIST_PATH, "r", encoding="utf-8") as f:
        nist_db: Dict[str, Dict[str, Any]] = json.load(f)

    # 1. Gezegen / IAU kanonik sabitleri kontrolü
    iau_map = {
        "M_earth": ("Earth Mass (IAU)", PhysicalConstants.M_earth, "kg"),
        "R_earth": ("Earth Mean Radius (IAU)", PhysicalConstants.R_earth, "m"),
        "mu_earth": ("Earth Standard Gravitational Parameter GM", PhysicalConstants.mu_earth, "m^3 s^-2"),
        "v_orbit_leo": ("Reference LEO Orbital Speed", PhysicalConstants.v_orbit_leo, "m s^-1"),
        "v_escape_earth": ("Earth Surface Escape Velocity", PhysicalConstants.v_escape_earth, "m s^-1"),
    }
    if q in iau_map:
        name, val, unit = iau_map[q]
        return json.dumps(
            {"status": "ok", "symbol": q, "name": name, "value": val, "unit": unit, "source": "IAU_SSOT"},
            indent=2,
            ensure_ascii=False,
        )

    # 2. Kanonik sembol takma adı -> tam NIST CODATA anahtarı
    target_key = NIST_SYMBOL_ALIASES.get(q)
    if target_key and target_key in nist_db:
        entry = nist_db[target_key]
        return json.dumps(
            {
                "status": "ok",
                "symbol": q,
                "nist_name": target_key,
                "value": entry["value"],
                "unit": entry["unit"],
                "raw_precision_str": entry["raw_str"],
                "source": "NIST_CODATA_355",
            },
            indent=2,
            ensure_ascii=False,
        )

    # 3. Tam NIST anahtarı eşleşmesi (büyük/küçük harf duyarsız)
    q_low = q.lower()
    for k, entry in nist_db.items():
        if k.lower() == q_low:
            return json.dumps(
                {
                    "status": "ok",
                    "nist_name": k,
                    "value": entry["value"],
                    "unit": entry["unit"],
                    "raw_precision_str": entry["raw_str"],
                    "source": "NIST_CODATA_355",
                },
                indent=2,
                ensure_ascii=False,
            )

    # 4. 355 NIST sabiti üzerinde alt-dize araması
    matches = [
        {
            "nist_name": k,
            "value": entry["value"],
            "unit": entry["unit"],
            "raw_precision_str": entry["raw_str"],
        }
        for k, entry in nist_db.items()
        if q_low in k.lower()
    ]
    if matches:
        return json.dumps(
            {
                "status": "ok",
                "query": q,
                "total_matches": len(matches),
                "matches": matches[:25],
                "source": "NIST_CODATA_355",
            },
            indent=2,
            ensure_ascii=False,
        )

    return json.dumps(
        {
            "status": "error",
            "message": f"'{q}' için 355 NIST CODATA sabiti içinde eşleşme bulunamadı.",
            "known_symbol_aliases": sorted(NIST_SYMBOL_ALIASES.keys()),
        },
        indent=2,
        ensure_ascii=False,
    )


@_async_mcp_tool
def kmath_materials(
    sort_by: Annotated[str, Field(description="Sıralama kriteri: 'breaking_length' (km), 'specific_strength', 'yield_strength' (MPa), veya 'max_temp' (K).")] = "breaking_length",
    category: Annotated[str, Field(description="Malzeme kategorisi: 'all', 'nanomaterial', 'composite', 'metal_alloy', 'superalloy', 'refractory_metal', 'copper_superalloy', 'ceramic_composite', 'uhtc_ceramic', 'ablative_tps', 'insulator'.")] = "all",
) -> str:
    """SSOT havacılık/uzay ve aşırı ortam malzeme veritabanını (30 sertifikalı malzeme) sorgular ve sıralar."""
    mdb = MaterialsDatabase(MAT_PATH)
    all_mats = list(mdb.materials.values())
    available_categories = sorted({m.category for m in all_mats})

    if category.strip().lower() != "all":
        cat_q = category.strip().lower()
        mat_list = [m for m in all_mats if m.category.lower() == cat_q]
    else:
        cat_q = "all"
        mat_list = list(all_mats)

    def _sort_mats(items: List[Any]) -> None:
        if sort_by == "breaking_length":
            items.sort(key=lambda m: m.breaking_length_km, reverse=True)
        elif sort_by == "specific_strength":
            items.sort(key=lambda m: m.specific_strength, reverse=True)
        elif sort_by == "yield_strength":
            items.sort(key=lambda m: m.yield_strength, reverse=True)
        elif sort_by == "max_temp":
            items.sort(key=lambda m: m.max_temperature, reverse=True)

    _sort_mats(mat_list)

    if not mat_list and cat_q != "all":
        suggested = [
            m
            for m in all_mats
            if cat_q in m.id.lower() or cat_q in m.name.lower() or cat_q in m.description.lower()
        ]
        _sort_mats(suggested)
        return json.dumps(
            {
                "status": "ok",
                "total_materials": 0,
                "requested_category": category,
                "available_categories": available_categories,
                "hint": f"'{category}' geçerli bir kategori değil. Geçerli kategoriler: {', '.join(available_categories)}.",
                "suggested_matches": [m.to_dict() for m in suggested],
                "materials": [],
            },
            indent=2,
            ensure_ascii=False,
        )

    return json.dumps(
        {
            "status": "ok",
            "total_materials": len(mat_list),
            "sorted_by": sort_by,
            "available_categories": available_categories,
            "materials": [m.to_dict() for m in mat_list],
        },
        indent=2,
        ensure_ascii=False,
    )


@_async_mcp_tool
def kmath_stress_check(
    material_id: Annotated[str, Field(description="Kanonik malzeme ID'si (örn: 'mat_alli_2195', 'mat_carbon_fiber_t1000g', 'mat_inconel_718', 'mat_titanium_ti6al4v', 'mat_sic_cmc').")],
    mass_kg: Annotated[float, Field(description="Toplam araç kütlesi (kg).")],
    g_load: Annotated[float, Field(description="Eksenel ivme yükü (G).")],
    max_q_pa: Annotated[float, Field(description="Maksimum aerodinamik basınç Max-Q (Pa).")],
    radius_m: Annotated[float, Field(description="Silindir gövde yarıçapı (m).")] = 1.85,
    length_m: Annotated[float, Field(description="Silindir gövde boyu (m, Euler/SP-8007 burkulma için).")] = 42.0,
    wall_thickness_mm: Annotated[float, Field(description="Cidar et kalınlığı (mm).")] = 5.0,
    speed_ms: Annotated[float, Field(description="Uçuş hızı (m/s, Sutton-Graves ısınma ve PICA-X TPS hesabı için).")] = 7800.0,
    tank_pressure_bar: Annotated[float, Field(description="İç tank basıncı (bar).")] = 3.0,
) -> str:
    """Tekil kaynaklı (SSOT) saf-Rust çekirdeğiyle ince cidarlı silindirik gövdede Biaxial Von Mises akma, NASA SP-8007 + Euler burkulma ve Sutton-Graves / PICA-X ablatif ısı kalkanı denetimini yürütür."""
    rc, rust_out, rust_err = _run_rust_cli(
        [
            "mcp",
            "stress_hull",
            material_id,
            str(mass_kg),
            str(g_load),
            str(max_q_pa),
            str(radius_m),
            str(length_m),
            str(wall_thickness_mm),
            str(speed_ms),
            str(tank_pressure_bar),
        ]
    )
    if rc != 0:
        return json.dumps({"status": "error", "message": rust_err or rust_out}, ensure_ascii=False)
    try:
        parsed = json.loads(rust_out)
        return json.dumps(parsed, indent=2, ensure_ascii=False)
    except json.JSONDecodeError:
        return rust_out


@_async_mcp_tool
def kmath_simulate_trajectory(
    vehicle_type: Annotated[str, Field(description="Simüle edilecek referans fırlatma aracı: 'compare' (3 mimari karşılaştırması), 'chemical' (2-kademeli LOX/RP-1), 'beamed' (Lazer SSTO) veya 'kinetic' (Kinetik hibrit).")] = "compare",
) -> str:
    """Gerçek 4-aşamalı Runge-Kutta (RK4, O(dt^4)) yörünge entegratörü ile 2D atmosferik uçuş, G-kısma (throttle), Kepler yörünge (enberi/enöte/e) ve canlı ısı yükü simülasyonunu koşturur."""
    if vehicle_type == "compare":
        rc, rust_out, _ = _run_rust_cli(["simulate"])
        if rc == 0 and rust_out:
            return rust_out

    vehicles = _canonical_vehicles()
    selected = (
        [vehicles[vehicle_type]]
        if vehicle_type in vehicles
        else list(vehicles.values())
    )
    sim = RK4TrajectorySimulator(dt_seconds=0.1)
    results = []
    for veh in selected:
        res = sim.simulate(veh, target_orbit_km=200.0)
        res.pop("telemetry_sample", None)
        results.append(res)

    return json.dumps(
        {"status": "ok", "integrator": "RK4 (dt=0.1s)", "results": results},
        indent=2,
        ensure_ascii=False,
    )


@_async_mcp_tool
def kmath_solve_for(
    equation: Annotated[str, Field(description="Terslenecek 'LHS = RHS' denklemi (örn: 'delta_v = (g_0 * I_sp) * ln(m_0 / m_f)' veya 'P_beam = (F * v_e) / (2 * eta)').")],
    target_var: Annotated[str, Field(description="Yalnız bırakılacak hedef değişken adı (örn: 'm_0', 'I_sp', 'eta', 'v').")],
) -> str:
    """Sembolik bir 'LHS = RHS' denklemini hedef değişken için cebirsel olarak tersler, zorunlu tanım kümesi yan koşullarını (!= 0, > 0, >= 0) ve ±sqrt dal uyarılarını çıkarır, sıfır-panik Rust kodu üretir."""
    rc, out, err = _run_rust_cli(["mcp", "solve_for", equation, target_var])
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_prove_identity(
    lhs: Annotated[str, Field(description="Sol cebirsel ifade (örn: '(x + y + z)^2' veya '(x^2 - 1) / (x - 1)').")],
    rhs: Annotated[str, Field(description="Sağ cebirsel ifade (örn: 'x^2 + y^2 + z^2 + 2*x*y + 2*x*z + 2*y*z' veya 'x + 1').")],
    max_degree: Annotated[int, Field(description="Maksimum polinom derece bütçesi (varsayılan: 16).")] = 16,
) -> str:
    """İki cebirsel ifadenin çok değişkenli kanonik polinom halkası Z[x_1, ..., x_n] (grevlex, checked_i64 taşma korumalı) üzerinde tam özdeşliğini ve tanım kümesi yan koşul korunumunu ispatlar."""
    rc, out, err = _run_rust_cli(["mcp", "prove_poly", lhs, rhs, str(max_degree)])
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_interval_eval(
    expression: Annotated[str, Field(description="Sembolik matematiksel ifade (örn: '(P * r) / t' veya '(g_0 * I_sp) * ln(m_0 / m_f)').")],
    intervals_json: Annotated[str, Field(description="Değişken aralık haritası JSON'u: '{\"x\": [lo, hi], ...}' (örn: '{\"P\": [390000, 410000], \"r\": [1.795, 1.805], \"t\": [0.0049, 0.0051]}').")],
    diff_var: Annotated[str, Field(description="Opsiyonel sembolik türev alınacak değişken adı (örn: 't', 'I_sp').")] = "",
) -> str:
    """IEEE-754 dışa yuvarlamalı Interval64, bağımlılık eleyen Taylor/Mean-Value aralığı, Dual64 tam gradyan ve sembolik türev hesaplar."""
    cmd = ["mcp", "interval_eval", expression, intervals_json]
    if diff_var.strip():
        cmd.append(diff_var.strip())
    rc, out, err = _run_rust_cli(cmd)
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_synthesize(
    inputs_csv: Annotated[str, Field(description="Virgülle ayrılmış bilinen SSOT değişkenleri (örn: 'F,m,t', 'g_0,I_sp,m_dot,eta' veya 'delta_v,g_0,I_sp,m_f').")],
    target_var: Annotated[str, Field(description="Sentezlenecek hedef fiziksel nicelik (örn: 'E_k', 'P_beam', 'delta_v', 'm_0').")],
    max_depth: Annotated[int, Field(description="Maksimum hipergraf zincirleme derinliği (varsayılan: 4).")] = 4,
) -> str:
    """Çift-yönlü AND-OR bilgi hipergrafı üzerinde eldeki girdilerden hedef fiziksel niceliği türeten tüm Z^7 doğrulamalı sembolik formülleri ve sıfır-panik Rust fonksiyonlarını sentezler."""
    rc, out, err = _run_rust_cli(["mcp", "synthesize", inputs_csv, target_var, str(max_depth)])
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_inverse_design(
    payload_kg: Annotated[float, Field(description="Hedef faydalı yük kütlesi (kg, örn: 500 veya 1000).")] = 500.0,
    target_orbit_km: Annotated[float, Field(description="Hedef LEO yörünge enberi irtifası (km, örn: 200 veya 400).")] = 200.0,
    max_g_limit: Annotated[float, Field(description="İzin verilen azami ivme yükü ve motor kısma sınırı (G, örn: 4.5 veya 8.5).")] = 8.5,
    min_safety_factor: Annotated[float, Field(description="Minimum akma ve NASA SP-8007 burkulma emniyet katsayısı (örn: 1.35).")] = 1.35,
) -> str:
    """Kapalı döngü tersine mühendislik ve Pareto optimizasyon motorunu (6.912 aday x G-kısmalı RK4 + sıkı Kepler enberi >= target_orbit_km + Von Mises + NASA SP-8007 burkulma + PICA-X TPS) çalıştırır."""
    rc, out, err = _run_rust_cli(
        [
            "mcp",
            "design",
            str(payload_kg),
            str(target_orbit_km),
            str(max_g_limit),
            str(min_safety_factor),
        ],
        timeout_s=30.0,
    )
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_physical_bounds_check(
    mass_kg: Annotated[Optional[float], Field(description="Kütle (kg, >= 0).")] = None,
    velocity_ms: Annotated[Optional[float], Field(description="Cisim hızı (m/s, |v| < c).")] = None,
    exhaust_velocity_ms: Annotated[Optional[float], Field(description="Egzoz hızı (m/s, |v_e| < c).")] = None,
    mass_flow_kg_s: Annotated[Optional[float], Field(description="Kütlesel debi m_dot (kg/s, >= 0).")] = None,
    thrust_n: Annotated[Optional[float], Field(description="İtki kuvveti F (N). m_dot=0 iken F <= 2*P_in/c denetlenir.")] = None,
    power_in_w: Annotated[Optional[float], Field(description="Giriş gücü P_in (W, >= 0).")] = None,
    power_out_w: Annotated[Optional[float], Field(description="Çıkış gücü P_out (W, <= P_in).")] = None,
    efficiency: Annotated[Optional[float], Field(description="Verim eta ([0, 1] ve <= Carnot 1 - T_c/T_h).")] = None,
    temp_hot_k: Annotated[Optional[float], Field(description="Sıcak kaynak sıcaklığı T_hot (K, > 0).")] = None,
    temp_cold_k: Annotated[Optional[float], Field(description="Soğuk kaynak sıcaklığı T_cold (K, > 0 ve <= T_hot).")] = None,
    entropy_delta_j_k: Annotated[Optional[float], Field(description="İzole sistem entropi değişimi dS (J/K, >= 0).")] = None,
) -> str:
    """AGENTS.md Değişmez 2 uyarınca bir fiziksel/mühendislik adayını Termodinamik 1./2. Yasa, Carnot sınırı, Özel Görelilik nedenselliği (v < c) ve Momentum korunumuna karşı erken çıkış yapmadan denetler ve tespit edilen TÜM ihlalleri raporlar."""
    payload = {
        k: v
        for k, v in {
            "mass_kg": mass_kg,
            "velocity_ms": velocity_ms,
            "exhaust_velocity_ms": exhaust_velocity_ms,
            "mass_flow_kg_s": mass_flow_kg_s,
            "thrust_n": thrust_n,
            "power_in_w": power_in_w,
            "power_out_w": power_out_w,
            "efficiency": efficiency,
            "temp_hot_k": temp_hot_k,
            "temp_cold_k": temp_cold_k,
            "entropy_delta_j_k": entropy_delta_j_k,
        }.items()
        if v is not None
    }
    rc, out, err = _run_rust_cli(["mcp", "check_bounds", json.dumps(payload)])
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_regime_check(
    law_id: Annotated[str, Field(description="Kanonik yasa ID'si (örn: 'law_prandtl_glauert_subsonic_cp', 'law_ackeret_supersonic_wave_cp', 'law_lorentz_factor', 'law_mm1_mean_response_time').")],
    bindings_json: Annotated[str, Field(description="Sayısal değişken değerleri JSON'u (örn: '{\"Mach\": 0.55, \"C_p0\": -0.4}' veya '{\"v\": 299792458.0, \"c\": 299792458.0}').")],
) -> str:
    """Faz 1.3 Fiziksel Geçerlilik Bölgesi (DomainBounds) kalkanı: Verilen sayısal girdilerin kanonik bir yasanın Mach, hız, basınç veya ergodik kararlılık rejim sınırları içinde kalıp kalmadığını denetler."""
    rc, out, err = _run_rust_cli(["mcp", "regime_check", law_id, bindings_json])
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_buckingham_pi(
    variables_spec: Annotated[str, Field(description="Virgülle ayrılmış SSOT değişkenleri veya 'ad:boyut' çiftleri ya da JSON nesnesi (örn: 'F_drag,rho,v,L_char,mu_visc' veya 'r,E,rho,t').")],
    target_var: Annotated[str, Field(description="Opsiyonel izole edilecek hedef değişken (örn: 'F_drag', 'r', 'T_orb').")] = "",
    samples_json: Annotated[str, Field(description="Opsiyonel deneysel/simülasyon örneklem dizisi JSON'u (örn: '[{\"r\": 80.5, \"E\": 8.4e13, \"rho\": 1.225, \"t\": 0.006}]').")] = "",
    repeating_csv: Annotated[str, Field(description="Opsiyonel tekrarlayan (pivot) taban değişkenleri öncelik listesi (örn: 'rho,v,L_char').")] = "",
) -> str:
    """Tam tamsayı Z^7 çekirdek uzayı (nullspace) üzerinde Buckingham-Pi boyutsuz benzerlik gruplarını keşfeder, hedef ölçekleme yasasını ve sıfır-panik Rust kodunu sentezler, örneklem verilirse kalibre eder."""
    cmd = [
        "mcp",
        "buckingham",
        variables_spec,
        target_var.strip(),
        samples_json.strip() if samples_json.strip() else "[]",
    ]
    if repeating_csv.strip():
        cmd.append(repeating_csv.strip())
    rc, out, err = _run_rust_cli(cmd)
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


@_async_mcp_tool
def kmath_pareto_front(
    objectives_json: Annotated[str, Field(description="Amaç tanım dizisi JSON'u (örn: '[{\"name\": \"mass_kg\", \"direction\": \"min\", \"unit\": \"kg\"}, {\"name\": \"safety_factor\", \"direction\": \"max\", \"unit\": \"x\"}]').")],
    candidates_json: Annotated[str, Field(description="Aday noktalar dizisi JSON'u (örn: '[{\"id\": \"A\", \"values\": [1000.0, 1.4]}, {\"id\": \"B\", \"values\": [1220.0, 2.1]}]').")],
) -> str:
    """Çok amaçlı kesin Pareto cephesi (Exact Skyline / NSGA-II Rank 0), kalabalık mesafesi, Ütopya/Nadir vektörleri, uzlaşma diz-noktası (Knee) ve 2B süpürme hiperhacmini hesaplar."""
    rc, out, err = _run_rust_cli(["mcp", "pareto", objectives_json, candidates_json])
    if rc != 0:
        return json.dumps({"status": "error", "message": err or out}, ensure_ascii=False)
    return out


def main() -> None:
    """Console script entry point for `kmath-mcp` and `uvx kmath-mcp`."""
    mcp.run()


if __name__ == "__main__":
    main()
