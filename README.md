# Citadel-Math (`citadel_math_core` / `kmath-mcp`)

[![CI](https://github.com/imza464-cmyk/citadel-math/actions/workflows/ci.yml/badge.svg)](https://github.com/imza464-cmyk/citadel-math/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/imza464-cmyk/citadel-math?color=0ea5e9)](https://github.com/imza464-cmyk/citadel-math/releases)
[![License: AGPL-3.0 | Commercial](https://img.shields.io/badge/License-AGPL--3.0_%7C_Commercial-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-forbid%28unsafe__code%29-orange.svg)](./citadel_math_core/src/lib.rs)
[![MCP Server](https://img.shields.io/badge/MCP-15_Deterministic_Tools-10b981.svg)](./kmath_mcp.py)

**$\mathbb{Z}^7$ Dimensional Analysis Shield, Checked-`i64` Algebraic Equivalence Checker, and Interval/Physics Verification Engine (Pure-Rust Core + Python/SymPy MCP Server).**  
*$\mathbb{Z}^7$ Boyut Analizi Kalkanı, Taşma Korumalı `i64` Cebirsel Eşdeğerlik Denetleyicisi ve Aralık/Fizik Doğrulama Motoru (Saf-Rust Çekirdek + Python/SymPy MCP Sunucusu).*

---

## Why Citadel-Math? / Neden Citadel-Math?

Large Language Models hallucinate units, drop denominator domain guards, confuse classical/relativistic regimes, and leak IEEE-754 rounding errors into engineering safety factors.  
**Citadel-Math (`kmath-mcp`)** pairs a **pure-Rust (`#![forbid(unsafe_code)]`) numerical and algebraic core (`citadel_math_core`)** with a **Python + SymPy Model Context Protocol server (`kmath_mcp.py`)**, giving AI coding agents (Claude, Cursor, Windsurf, Antigravity) and engineers a deterministic verification layer backed by NIST CODATA 2022 constants, SI dimensional lattice checks, outward-rounded intervals, and a canonical physics equation graph.

Büyük Dil Modelleri (LLM) formül türetirken sıklıkla birim hatası yapar, payda sıfır koşullarını atlar ve kayan nokta yuvarlama hatalarını güvenlik katsayılarına sızdırır. **Citadel-Math**, saf Rust (`#![forbid(unsafe_code)]`) ile yazılmış deterministik hesaplama çekirdeğini (`citadel_math_core`) Python/SymPy tabanlı bir MCP sunucu katmanıyla (`kmath_mcp.py`) birleştirerek yapay zekâ ajanlarına birim, aralık ve cebirsel eşdeğerlik doğrulama araçları sunar.

```mermaid
flowchart LR
    LLM["LLM / MCP Client (Claude, Cursor, Windsurf)"] -->|"JSON-RPC (stdio)"| MCP["kmath_mcp.py (Python + Zero-Eval AST Guard)"]
    MCP -->|"Subprocess (shell=False)"| RUST["citadel_math_core (Pure Rust, forbid unsafe)"]
    MCP -->|"Read-only SSOT"| DB["database/ & data/ (214 Laws, 353 Symbols, CODATA 2022)"]
    RUST -->|"Verified Bounds & Proofs"| VERDICT["Deterministic JSON Verdict"]
    MCP --> VERDICT
```

---

## Quick Start / Hızlı Kurulum (3 Yol)

Choose the workflow that fits your environment — **Rust is NOT required** if you use pre-built release binaries or Docker.

### Option A — One-Command `uvx` + Pre-Built Binary (No Rust Compiler Required)
1. **Download the self-contained Rust core binary** (`1.45 MB` on Windows, embedded SSOT JSONs) from [GitHub Releases](https://github.com/imza464-cmyk/citadel-math/releases) and verify its SHA-256 checksum against `SHA256SUMS.txt`:
   - Linux x86_64: `citadel_math_core-linux-x86_64` (`chmod +x` and place on `PATH` or point `KMATH_RUST_BIN` to it)
   - Windows x86_64: `citadel_math_core-windows-x86_64.exe`
2. **Run CLI or MCP directly via [`uvx`](https://docs.astral.sh/uv/):**
   ```bash
   # Inspect ontology & canonical law metrics with zero manual cloning
   uvx --from git+https://github.com/imza464-cmyk/citadel-math.git kmath stats

   # Start the 15-tool stdio MCP server
   KMATH_RUST_BIN=/path/to/citadel_math_core uvx --from git+https://github.com/imza464-cmyk/citadel-math.git kmath-mcp
   ```
3. **MCP Client Configuration (`uvx`):**
   ```json
   {
     "mcpServers": {
       "kmath": {
         "command": "uvx",
         "args": ["--from", "git+https://github.com/imza464-cmyk/citadel-math.git", "kmath-mcp"],
         "env": {
           "KMATH_RUST_BIN": "/ABSOLUTE/PATH/TO/citadel_math_core",
           "OPENBLAS_NUM_THREADS": "1",
           "PYTHONUNBUFFERED": "1"
         }
       }
     }
   }
   ```

### Option B — Hardened Docker Container (Zero Host Rust/Python Dependencies)
Pre-built multi-stage images are published automatically to **GitHub Container Registry (GHCR)** (`ghcr.io/imza464-cmyk/citadel-math`), compiled from [`Dockerfile`](./Dockerfile) and verified under all 26 audit gates as **non-root UID `10001`** with a Python-native `HEALTHCHECK` and exec-form `ENTRYPOINT`.
```bash
# Run directly from GHCR with zero local build (read-only rootfs, zero capabilities, zero network)
docker run -i --rm \
  --read-only \
  --tmpfs /tmp:size=64M,noexec,nosuid \
  --cap-drop=ALL \
  --security-opt=no-new-privileges:true \
  --pids-limit=128 \
  --memory=512m \
  --network=none \
  ghcr.io/imza464-cmyk/citadel-math:0.2.0

# Or build locally from source
docker build -t citadel-math:0.2.0 .
```
See [`docker-compose.yml`](./docker-compose.yml) for the declarative hardened service definition. To attach Docker directly in Claude Desktop / Cursor:
```json
{
  "mcpServers": {
    "kmath": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm", "--read-only",
        "--tmpfs", "/tmp:size=64M,noexec,nosuid",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges:true",
        "--network=none",
        "ghcr.io/imza464-cmyk/citadel-math:0.2.0"
      ]
    }
  }
}
```

### Option C — Build From Source (Developers & Contributors)
```bash
git clone https://github.com/imza464-cmyk/citadel-math.git
cd citadel-math

# 1. Test & build release Rust core
cargo test --manifest-path citadel_math_core/Cargo.toml
cargo build --release --manifest-path citadel_math_core/Cargo.toml

# 2. Install editable Python package & console scripts (`kmath`, `kmath-mcp`)
pip install -e .

# 3. Verify all 26 audit gates + MCP security tests
kmath audit
kmath rust audit
python -m unittest discover -s tests -v
```
Copy [`mcp_config.example.json`](./mcp_config.example.json) into your MCP client config and set `/ABSOLUTE/PATH/TO/citadel-math/kmath_mcp.py` (the Rust binary in `citadel_math_core/target/release/` is discovered automatically).

---

## End-to-End Chained Agent Scenario / Uçtan Uca 5 Adımlı Ajan Senaryosu

**Prompt to AI Agent:** *"Design and audit a Phase-A cryogenic LOX tank (`45 t` gross mass, `4.5 G`, `q_max = 35 kPa`, `r = 1.8 m`, `L = 14 m`, `t = 5.0 mm`, `P = 4.0 bar`, peak ascent speed `2400 m/s`). Verify alloy selection, unit hygiene, structural buckling + thermal protection, manufacturing-tolerance hoop stress bounds, and propulsion physics."*

Below is the actual deterministic 5-tool chain executed live against `kmath_mcp.py` + `citadel_math_core`:

```mermaid
flowchart LR
    S1["1. kmath_materials<br/>Rank alloys -> Al-Li 2195"] --> S2["2. kmath_dimension_check<br/>Catch 1000x scale & symbol clash"]
    S2 --> S3["3. kmath_stress_check<br/>SP-8007 SF=2.49x + 26.3 mm PICA-X"]
    S3 --> S4["4. kmath_interval_eval<br/>[137.26, 151.03] MPa + dσ/dt"]
    S4 --> S5["5. kmath_physical_bounds_check<br/>Carnot 0.68 < 0.906, v << c -> PASS"]
```

| Step | MCP Tool Call | Verified Live Output / Caught Failure Mode |
| :---: | :--- | :--- |
| **1** | `kmath_materials(sort_by="yield_strength", category="metal_alloy")` | Compares `mat_maraging_300` (`2000 MPa`, `8000 kg/m³`, `L_b = 25.5 km`), `mat_ss_17_4ph_h900` (`1170 MPa`), `mat_ss_304l_cryo` (`960 MPa`, `806.9 °C`) and selects lightweight cryogenic **`mat_alli_2195`** (`550 MPa` yield, `146.85 °C` max wall limit). |
| **2** | `kmath_dimension_check("kN", "kg * m / s^2")` & `kmath_dimension_check("P * r / t", "Pa")` | **Catches two classic LLM bugs before computing:** (a) flags `1000.0×` scale error (`DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH`), and (b) flags implicit symbol collision where SSOT `P` = Power and `t` = Time (`[M L^3 T^-4] != [M L^-1 T^-2]`), forcing explicit `var_dimensions_json` disambiguation. |
| **3** | `kmath_stress_check("mat_alli_2195", mass_kg=45000, g_load=4.5, max_q_pa=35000, radius_m=1.8, length_m=14.0, wall_thickness_mm=5.0, speed_ms=2400, tank_pressure_bar=4.0)` | **Structural PASS + Auto-TPS Sizing:** Rust Von Mises `130.5 MPa` (`yield_safety_factor = 4.215×`), NASA SP-8007 buckling critical `97.32 MPa` (`buckling_safety_factor = 2.486×`). Bare wall hits `1951.5 °C` (> `146.85 °C` limit) → automatically sizes **`26.32 mm` PICA-X TPS (`173.60 kg`)**, lowering bondline to **`106.85 °C`** (`overall_pass: true`). |
| **4** | `kmath_interval_eval("(P * r) / t", '{"P":[390000,410000],"r":[1.795,1.805],"t":[0.0049,0.0051]}', "t")` | **Guaranteed Tolerance Bounds:** IEEE 1788-2015 outward-rounded hoop stress **`[137.26 MPa, 151.03 MPa]`** (midpoint `144.00 MPa`, width `13.77 MPa`), exact symbolic derivative `-(P * r) / t^2`, sensitivity **`∂σ/∂t = -2.88 × 10¹⁰ Pa/m`** (`-2.88 MPa` per `+0.1 mm` wall thickness). |
| **5** | `kmath_physical_bounds_check(mass_kg=45000, velocity_ms=2400, exhaust_velocity_ms=3400, mass_flow_kg_s=280, thrust_n=952000, temp_hot_k=3200, temp_cold_k=300, efficiency=0.68)` | **Thermodynamic & Causality Seal:** `carnot_max_efficiency = 0.90625` (`0.68 < 0.90625`), `lorentz_gamma = 1.000000000032` (`v ≪ c`), momentum `F = ṁ·v_e = 952 kN` exact → **`physically_realizable: true`**. |

### Additional Standalone Checks / Tekil Canlı Örnekler
- **Domain-Guard Preservation (`kmath_prove_identity`):** `kmath_prove_identity("(x^2 - 1) / (x - 1)", "x + 1")` returns `"proven_identical": false` with `"lhs_domain_guards": ["(x - 1) != 0"]`, rejecting singular cancellation at `x = 1`.
- **Exact Dimensionless Groups (`kmath_buckingham_pi`):** `kmath_buckingham_pi("r,E,rho,t", "r")` derives Taylor-Sedov blast scaling $r \propto (E t^2 / \rho)^{1/5}$ over exact `i128` rational $\mathbb{Z}^7$ nullspace.

---

## Performance & Memory Benchmarks / Performans ve Bellek Kıyaslamaları

Measured live on **Windows 11 x86_64** (Python 3.14, `OPENBLAS_NUM_THREADS=1`, release `citadel_math_core.exe` = **`1.45 MB`**):

| Operation / MCP Tool | Execution Path | `p50` Latency | `p95` Latency | Notes |
| :--- | :--- | :---: | :---: | :--- |
| Server cold start (`import kmath_mcp`) | Python (SymPy + NetworkX + MCP) | `876.3 ms` | — | One-time startup; `109.95 MB` working set (`97.18 MB` private) |
| `kmath_constant("c")` | In-process Python JSON | `0.28 ms` | `0.33 ms` | CODATA 2022 lookup |
| `kmath_dimension_check("kN", "kg * m / s^2")` | In-process Zero-Eval AST | `3.60 ms` | `4.81 ms` | Symbolic AST + $\mathbb{Z}^7$ scale check |
| `kmath_prove_identity` | Python → Rust subprocess (`shell=False`) | `5.30 ms` | `5.68 ms` | Includes full OS process spawn + JSON IPC |
| `kmath_interval_eval` | Python → Rust subprocess (`shell=False`) | `5.45 ms` | `5.72 ms` | IEEE 1788-2015 `Interval64` + `Dual64` |
| `kmath_stress_check` | Python + Rust subprocess | `5.56 ms` | `5.84 ms` | Von Mises + NASA SP-8007 + PICA-X TPS |
| `kmath_audit` (26 gates total) | 10 Python + 16 Rust gates | `45.65 ms` | `47.73 ms` | Native Rust 16-gate audit alone: `~13–17 ms` |

- **Steady-State Memory Footprint:** After running all tools and a full 26-gate audit, the MCP server stabilizes at **`113.16 MB` working set** (`113.21 MB` peak, `100.88 MB` private — only **`+3.2 MB`** delta above initial import).
- **Why Ephemeral Subprocess Isolation?** Each Rust tool call adds `~5.3–5.6 ms` of OS spawn/pipe overhead on Windows (`~1–2 ms` on Linux) in exchange for `shell=False` command-injection immunity, zero persistent FFI memory leaks, and clean timeout isolation (`20 s` hard cap).

---

## Core Architecture / Temel Mimari

```text
.
├── .github/
│   ├── ISSUE_TEMPLATE/             # Bug report & mathematical/physical soundness issue templates
│   └── workflows/
│       ├── ci.yml                  # Cross-platform CI (Linux/Windows): clippy, tests, wheel build & MCP security
│       └── release.yml             # Automated Linux/Windows release binary & SHA256SUMS publishing
├── AGENTS.md                       # Engineering invariants (dimensional homogeneity & SSOT rules)
├── CHANGELOG.md                    # Versioned release history (Keep a Changelog)
├── COMMERCIAL_LICENSE.md           # Commercial dual-license terms for proprietary / closed-SaaS use
├── CONTRIBUTING.md                 # Pull Request workflow, verification gates & coding standards
├── Dockerfile                      # Hardened multi-stage Rust + Python container (UID 10001, exec ENTRYPOINT)
├── docker-compose.yml              # Read-only, cap_drop: ALL, network_mode: none MCP service config
├── LICENSE                         # Official verbatim GNU Affero General Public License v3.0 (AGPL-3.0-only)
├── pyproject.toml                  # PEP 621 / Hatchling package (`kmath-mcp` + `kmath` CLI, bundled SSOT)
├── SECURITY.md                     # Threat model, zero-eval parser policy & vulnerability disclosure
├── kmath.py                        # Verification cockpit CLI (10 Python SSOT + 16 Rust audit gates)
├── kmath_mcp.py                    # 15-Tool Model Context Protocol (MCP) Server (zero-eval AST input parser)
├── mcp_config.example.json         # Portable local MCP client configuration example
├── tests/
│   └── test_mcp_security.py        # Zero-eval RCE rejection, binary resolution & live MCP bridge tests
├── citadel_math_core/              # Pure-Rust (#![forbid(unsafe_code)]) Engine & Native Binary
│   └── src/
│       ├── proof_kernel.rs         # Z[x] & MultiPolyZ (grevlex) algebraic equivalence + GuardedPoly guards
│       ├── interval.rs             # IEEE 1788-2015 outward-rounded Interval64 + Dual64 differentiation
│       ├── dimensions.rs           # Z^7 SI Dimensional Shield & ScaledUnit converter
│       ├── buckingham.rs           # Exact rational/integer Z^7 nullspace Buckingham-Pi discovery
│       ├── pareto.rs               # Exact Skyline & NSGA-II multi-objective Pareto optimizer
│       ├── graph.rs                # SSOT canonical hypergraph (214 laws, 544 inverse laws, 18 regimes)
│       ├── simulator.rs            # RK4 & adaptive Dormand-Prince RK4(5) integrator + USSA-1976 model
│       ├── astrodynamics.rs        # WGS84 J2 orbital precession & universal-variable Lambert solver
│       ├── materials.rs            # 30 aerospace/UHTC/TPS materials, NASA SP-8007 buckling & TPS checks
│       ├── cad.rs                  # 2D parametric CAD, Steiner/moment of inertia, Euler buckling & DXF/SVG
│       ├── designer.rs             # Closed-loop parametric preliminary sizing engine
│       ├── codegen.rs              # Verified zero-panic Result<f64, &'static str> Rust function generator
│       └── physical_bounds.rs      # Thermodynamics 1st/2nd law (Carnot), v < c causality & momentum pruning
├── engine/                         # Python symbolic AST, SymPy parity, harvester, RK4 & knowledge graph
├── database/
│   ├── canonical_laws.json         # 214 verified laws across 25+ domains (physics, aerospace, real-time, biomechanics)
│   ├── ONTOLOGY.json               # 353 unique SI symbol-dimension definitions (SSOT)
│   └── materials.json              # 30 certified structural & thermal protection materials
└── data/
    └── nist_constants.json         # NIST CODATA 2022 fundamental constants
```

---

## 15 Deterministic MCP Tools (`kmath_mcp.py`)

| Tool | Function |
| :--- | :--- |
| `kmath_audit` | Runs the Python (10-gate) & Rust (16-gate) engineering verification audits. |
| `kmath_dimension_check` | Verifies $\mathbb{Z}^7$ SI dimensional homogeneity (`dim(LHS) == dim(RHS)`) for expressions. |
| `kmath_prove_identity` | Proves polynomial/rational algebraic equivalence (`GuardedPoly`) with domain guards & overflow rejection. |
| `kmath_interval_eval` | Evaluates expressions using IEEE 1788-2015 outward-rounded `Interval64` + `Dual64` sensitivity. |
| `kmath_constant` | Fetches NIST CODATA 2022 fundamental constants and exact SI definitions. |
| `kmath_materials` | Queries and ranks 30 structural, cryogenic, UHTC, and ablative TPS materials. |
| `kmath_stress_check` | Checks cylindrical shell Von Mises stress, NASA SP-8007 buckling, and aerothermal TPS limits. |
| `kmath_simulate_trajectory` | Runs 4th-order Runge-Kutta (RK4) trajectory integration with USSA-1976 7-layer atmosphere. |
| `kmath_solve_for` | Symbolically solves canonical equations with automated SymPy/AST validation. |
| `kmath_synthesize` | Synthesizes valid candidate equations across the canonical AND-OR hypergraph with regime pruning. |
| `kmath_inverse_design` | Runs closed-loop parametric sweeps returning Pareto-filtered preliminary aerospace configurations. |
| `kmath_physical_bounds_check` | Prunes violations of 1st/2nd Thermodynamics Laws (Carnot), $v < c$, and momentum conservation. |
| `kmath_regime_check` | Validates variable compatibility across 18 physical validity regimes (`Mach`, relativistic, etc.). |
| `kmath_buckingham_pi` | Computes exact integer $\mathbb{Z}^7$ nullspace dimensionless $\Pi$-groups. |
| `kmath_pareto_front` | Computes exact Pareto Skyline (`Rank 0`), NSGA-II crowding distances, and Utopia knee points. |

---

## Troubleshooting / Sorun Giderme

### 1. How `kmath_mcp.py` & `kmath.py` Locate the Rust Binary (`resolve_rust_bin()`)
Every Rust-backed MCP tool resolves `citadel_math_core` dynamically in this exact priority order:
1. **`KMATH_RUST_BIN` environment variable** — Authoritative override. If set to an invalid path, the tool fails immediately with `rc=127` (never silently falls back to a stale build).
2. **`<repo>/citadel_math_core/target/release/citadel_math_core(.exe)`** — Default local release build.
3. **`<repo>/citadel_math_core/target/debug/citadel_math_core(.exe)`** — Local debug build fallback.
4. **`<repo>/bin/citadel_math_core(.exe)`** — Portable drop-in folder.
5. **System `PATH`** — Searches `citadel_math_core` as well as GitHub Release asset names (`citadel_math_core-linux-x86_64` / `citadel_math_core-windows-x86_64.exe`).

### 2. Missing Rust Binary Error (`rc=127`)
If you connect `kmath-mcp` without building Rust or configuring a pre-built release binary, pure-Python tools (`kmath_dimension_check`, `kmath_constant`, `kmath_materials`, `kmath_simulate_trajectory`, `kmath_solve_for`) continue working, while Rust-backed tools return a structured JSON error:
```json
{
  "status": "error",
  "message": "citadel_math_core Rust ikilisi bulunamadı / Rust binary not found. Aranan yollar / Searched: [...]. Çözüm / Fix: (1) Kaynaktan derleyin: `cargo build --release --manifest-path citadel_math_core/Cargo.toml`, veya (2) Hazır ikiliyi indirin: https://github.com/imza464-cmyk/citadel-math/releases ve `KMATH_RUST_BIN` ortam değişkenini (veya PATH'i) ayarlayın."
}
```
**Fix:** Either run `cargo build --release --manifest-path citadel_math_core/Cargo.toml`, or download the pre-built binary from [Releases](https://github.com/imza464-cmyk/citadel-math/releases) and set `KMATH_RUST_BIN`.

### 3. Packaged Wheel / `uvx` Data Resolution & Windows Memory Safety
- **Bundled SSOT Assets:** When installed via `uvx` or `pip install`, `_resolve_data_file()` automatically falls back to `engine/_ssot_data/` bundled inside the wheel (`database/`, `data/`, `AGENTS.md`).
- **Windows Virtual Memory (`OPENBLAS_NUM_THREADS=1`):** Both `kmath_mcp.py` and `kmath.py` enforce `OPENBLAS_NUM_THREADS=1` before importing numeric libraries to prevent ~640 MB OpenBLAS thread-pool commit-charge bloat on Windows.

---

## CLI Usage / Komut Satırı Kullanımı

```bash
kmath audit                   # Run 10-gate Python + SSOT engineering audit (or: python kmath.py audit)
kmath rust audit              # Run 16-gate pure-Rust proof & simulation audit
kmath simulate compare        # Compare Chemical vs. Beamed Laser vs. Kinetic launch trajectories
kmath materials               # Rank aerospace/TPS materials by breaking length & strength
kmath stress                  # Check cylindrical shell stress & heat flux limits
kmath stats                   # Show ontology & canonical law metrics
```

---

## Scope & Known Limitations / Kapsam ve Bilinen Sınırlar

- **Positioning / Amaç:** Citadel-Math is a deterministic MCP helper layer designed to catch unit, domain, and rounding errors in LLM-assisted workflows. It does **not** claim to replace battle-tested scientific ecosystems (`SymPy`, `SciPy`, `nalgebra`, `uom`, `GMAT`) or certified FEA/CFD suites for safety-critical production sign-off.
- **Algebraic Equivalence Checker (`PolyZ` / `MultiPolyZ`):** Checks polynomial and guarded rational equivalence under overflow-checked `i64` arithmetic (`GuardedPoly`); it is not a general theorem prover like Lean/Coq or an arbitrary-precision Gröbner basis solver over $\mathbb{C}$.
- **Astrodynamics & Trajectory (`RK4` / `RK4(5)`):** Models 2D point-mass trajectories, 7-layer USSA-1976 hydrostatic atmosphere, and secular WGS84 $J_2$ drift / two-body Lambert transfer; it does not model 6-DoF rigid-body aerodynamics or full $N$-body JPL ephemerides.
- **Structural & Thermal Sizing (`NASA SP-8007` & `Sutton-Graves`):** Cylindrical shell stress and ablative TPS margins provide Phase-A conceptual pre-sizing filters, not flight-certification 3D finite-element analysis.

---

## Reference Validation / Dış Referans Karşılaştırması

| Benchmark / Standard | Reference Value | Citadel-Math Check | Tolerance / Result |
| :--- | :--- | :--- | :--- |
| **NIST CODATA 2022** ($\mu_0 \varepsilon_0 c^2$) | `1.0` (Maxwell vacuum identity) | `constants.rs` / `kmath.py` | Relative drift `< 2e-10` (PASS) |
| **USSA-1976 Atmosphere** (0 / 11 / 47 km) | `288.15 K` / `216.65 K` / `270.65 K` | `AtmosphereModel::state_at` | `< 1e-3 K`, sea-level `a0 = 340.294 m/s` (PASS) |
| **WGS84 $J_2$ Critical Inclination** | `63.4349488°` ($\dot{\omega} = 0$) | `J2OrbitalModel::perigee_precession_rate_rad_s` | `|dω/dt| < 1e-16 rad/s` (PASS) |
| **WGS84 ISS LEO RAAN Drift** ($400\text{ km}, 51.64^\circ$) | `~ -4.998°/day` (NASA/NORAD) | `J2OrbitalModel::raan_precession_rate_rad_s` | `-4.9979°/day` (PASS) |
| **Lambert Orbital Transfer** (90° LEO) | Vis-viva energy & $\mathbf{h}$ conservation | `LambertSolver::solve` | Relative energy drift `< 1e-9` (PASS) |
| **IEEE 1788-2015 `Interval64`** (`1/3 * 3`) | Exact inclusion `1.0 ∈ [lo, hi]` | `Interval64::div` / `mul` | `0/1-ULP` outward rounding, `0 ∈ denom` rejected (PASS) |
| **Native Rust Audit Latency** (16 gates) | Sub-20 ms local execution | `citadel_math_core audit` | `~13–17 ms` release build |

---

## Contributing / Katkıda Bulunma

Community Pull Requests, new physically verified laws, bug reports, and security audits are welcome!  
Please read **[CONTRIBUTING.md](./CONTRIBUTING.md)** for the full engineering verification checklist (`cargo clippy -D warnings`, `cargo test`, `kmath audit`, `kmath rust audit`, `unittest`) and architectural invariants (`AGENTS.md`).

---

## License / Çift Lisans (Dual License)

Citadel-Math is **dual-licensed**:

1. **Open Source — GNU Affero General Public License v3.0 (`AGPL-3.0-only`):** Free to use, study, modify, contribute Pull Requests, and redistribute under the terms of [`LICENSE`](./LICENSE). Network/SaaS deployments exposing modified or unmodified Citadel-Math services must disclose corresponding source code under AGPL-3.0 Section 13.
2. **Commercial License:** Organizations wishing to embed Citadel-Math in proprietary closed-source software or commercial SaaS without AGPL-3.0 copyleft obligations may obtain a commercial license — see **[COMMERCIAL_LICENSE.md](./COMMERCIAL_LICENSE.md)**.

*Citadel-Math çift lisanslıdır: Açık kaynak kullanım ve topluluk katkıları için [GNU AGPL-3.0-only](./LICENSE); kapalı kaynak ticari ürün veya SaaS entegrasyonları için [Ticari Lisans](./COMMERCIAL_LICENSE.md) geçerlidir.*  
Copyright © 2026 Mekserina Karaosmanoğlu.
