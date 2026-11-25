# Changelog

All notable changes to **Citadel-Math (`citadel_math_core` + `kmath-mcp`)** are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.2] - 2026-11-25

### Fixed (6 Real Physics & Numerical Bugs Caught by Independent Oracles & Anchors)
- **JSON Float Input Roundtrip (`Cargo.toml`):** Enabled `serde_json = { version = "1.0", features = ["float_roundtrip"] }`. Previously, default `serde_json` shifted inputs like `13.769785044591707` by `+1 ULP` to `13.769785044591709`, causing a 15-ULP containment breach in `exp(x)` caught by our 80-digit `Decimal` oracle.
- **`Interval64::pow` & `Dual64::pow` Integer-Snapping Bug (`interval.rs`):** Replaced sloppy `< 1e-12` tolerance with exact IEEE-754 `p.fract() == 0.0` / `p == 0.5`, preventing ~30,000-ULP containment violations on near-integer exponents such as `1000^(2 + 5e-13)`.
- **`Interval64::add_directed` Extreme & Subnormal Boundary Bug (`interval.rs`):** Fixed spurious `+Inf` on `f64::MAX + (-1.0)` and subnormal underflow on `5e-324 + 1e307` via a 3-regime exact error-free transform (`|a|,|b|<1e307` TwoSum, `|a|,|b|>=1e290` half-scaled TwoSum, and direct sign-side 1-ULP rule for huge+small).
- **`Interval64::from_rel_uncertainty` & `from_pm` (`interval.rs`):** Replaced round-to-nearest radius multiplication with outward-rounded `mul_directed(...).1` + Knuth `add_directed`, preventing subnormal uncertainty underflow.
- **Near-Parabolic Stumpff Catastrophic Cancellation (`astrodynamics.rs`):** Raised Taylor threshold in `stumpff_c/s/c_prime/s_prime` from `1e-5` to `0.1` with exact 7-term series (`< 5e-21` truncation error), recovering 9–12 lost digits on near-parabolic Lambert orbits (`1e-5 < |z| < 0.05`).
- **4× Darcy-vs-Fanning Fanno Flow Bug (`database/canonical_laws.json`):** Fixed `law_fanno_friction_parameter` from `(4 * f_darcy * L) / D_pipe` to `(f_darcy * L) / D_pipe` (since $f_D = 4 f_{\text{Fanning}}$), locked permanently by Cross-Law Closure #15 (`fanno_param == delta_P_dw / q_dyn`), and lowered Planck-scale relative-error denominator floor from `1e-30` to `1e-300`.

### Added
- **4-Layer Epistemic Self-Verification Shield (`epistemic_audit.rs`, `kmath.py` — 33 Total Audit Gates: 20 Rust + 13 Python):**
  - **100% (`367/367`) Golden Numerical Reference Anchors** + **100% (`367/367`) `≥3` Authoritative Sources** + **Universal `Dual64` Jacobian Non-Degeneracy Shield (`1,031/1,031` finite non-zero partial derivatives)**.
  - **25 Cross-Law Algebraic Closures** (Black Hole 1st Law, Elasticity triangle, Maxwell/Planck triads, Carnot COP, Fanno–Darcy, Nuclear half-life/HVL, Thermoelectric `ZT = S²/L`, Semiconductor `V_bi = 10 V_T`, Griffith fracture duality, Photometry `J`-axis, etc.) and **28 Asymptotic Limit + `Dual64` Monotonicity Checks**.
  - **17-Category Ashby Material Physical Envelopes** (`80/80` materials verified with `condition`, `failure_mode`, `yield_strength_range_mpa`, and `≥3` independent sources).
- **3-Source-Verified SSOT Expansion:** Expanded from `217 → 367` canonical laws (`37` domains, **all 7/7 SI base axes** `M, L, T, I, Θ, N, J` active across `618` unique symbols) and `30 → 80` certified materials (`+civil_structural`, `+elastomer_seal`, `+electrical_magnetic`, `+biomedical_alloy`, `+technical_ceramic`, `+semiconductor_optical`, `+polymer`).
- **Independent High-Precision Oracles & Balanced Module Unit Tests (`34 → 85` Rust tests across `16/18` modules + `13/13` Python tests):**
  - Self-contained exact infinite-precision `BigInt` dyadic oracle (`25,000+` vectors in `interval.rs`) + live 80-digit `decimal.Decimal` oracle (`tests/test_interval_oracle.py`).
  - Independent textbook/analytical module unit tests in `interval`, `dimensions`, `ast`, `buckingham` (Reynolds, Taylor-Sedov `1/5`), `pareto` (exact 2D hypervolume), `astrodynamics` (Curtis Ex. 5.2 Lambert, SMAD SSO, ISS RAAN, Molniya), `materials`, `physical_bounds`, `codegen`, `graph`, and `constants`.
- **Governance, Discovery & Docs:** Added bilingual `DISCLAIMER.md` (AGPL-3.0 §§ 15–16 voluntary use & AI/MCP liability waiver), `server.json` (Official MCP Registry manifest), `smithery.yaml`, README Before/After guardrail table + animated SVG demo (`docs/demo_guardrail.svg`), `docs/LAUNCH_KIT.md`, and aligned website latency/terminology/oracle transparency.

## [0.2.1] - 2026-11-23

### Fixed
- **Honest Dimension & Branch Reporting (`kmath_solve_for`, `codegen.rs`, `main.rs`):** Eliminated false `[1 (Boyutsuz)]` Z^7 proof stamps when target variables are outside the SSOT ontology (`dimension_verified: false` + explicit `[Bilinmiyor / Doğrulanmadı — SSOT Ontoloji Dışı Sembol]` header), and added explicit `±sqrt` principal-branch warnings on even-root inversions.
- **Single-SSOT Structural & TPS Audit (`kmath_stress_check`, `materials.rs`, `kmath_mcp.py`):** Removed divergent duplicate Python Von Mises / thermal calculations so `kmath_stress_check` reports directly from `citadel_math_core` with active `tank_pressure_bar` coupling and explicit separation between `bare_overall_pass` and `tps_protected_overall_pass`.
- **Full Multi-Violation Physical Bounds Collection (`kmath_physical_bounds_check`, `physical_bounds.rs`):** Removed early short-circuit returns so concurrent violations (e.g. causality `|v| >= c` + Carnot bound `eta > eta_Carnot`) are all collected and reported in `violations[]` with `violation_count`.
- **Orbital Guidance & Inverse Design Realism (`simulator.rs`, `engine/simulator.py`, `designer.rs`):** Implemented closed-loop Hohmann coast-to-apogee (`t_apo > 0.55 * t_circ + 4.0`) + apogee circularization (`vh -> 0`) + `4.5 G` throttling and strict Kepler SECO (`perigee >= target_orbit_km && e < 0.08`). Both chemical (`e = 0.0003–0.0009`) and beamed-laser SSTO (`e = 0.0007`) achieve circular LEO without hyperbolic overshoot, and `kmath_inverse_design(1000 kg, 400 km, 4.5 G)` scales hull geometry with payload to discover `765` structurally/thermally feasible designs (`min_perigee >= 402.6 km`).

### Added
- **FastMCP AI Ergonomics & Non-Blocking Concurrency (`kmath_mcp.py`, `engine/materials.py`):** Added `Annotated[..., Field(description=...)]` schemas across all 15 MCP tools, async thread-pool execution (`_async_mcp_tool`, 9-way concurrent calls in `~0.03s`), SI axis dimension parsing (`L`, `M L T^-2`), `available_categories` + `suggested_matches` hints on unknown material categories, and clean rounded Celsius temperatures.
- **Kinematics & Work Hypergraph Bridges (`database/canonical_laws.json`, `database/ONTOLOGY.json`):** Added 3 canonical laws (`217` laws, `550` inverse laws, `356` symbols) enabling multi-hop synthesis of `F, m, t -> E_k` and `F, m, t -> W`, plus `test_hohmann_circular_orbit_400km_45g_and_beamed_200km` (`34/34` Rust unit tests).

## [0.2.0] - 2026-11-23

### Changed
- **Dual-License Model (GNU AGPL-3.0-only + Commercial):** Replaced the restrictive read-only source-available license with verbatim [GNU AGPL-3.0-only](./LICENSE) paired with an optional [Commercial License](./COMMERCIAL_LICENSE.md), enabling open-source community Pull Requests while protecting against closed-source commercial SaaS appropriation.
- **Smart Rust Binary & Wheel-Safe SSOT Discovery (`kmath_mcp.py`, `kmath.py`):** Added deterministic `resolve_rust_bin()` lookup (`KMATH_RUST_BIN` -> `target/release` -> `target/debug` -> `bin/` -> `PATH` including GitHub Releases asset names) with clear troubleshooting errors (`rc=127`), plus `_resolve_data_file()` fallback to packaged wheel SSOT assets (`engine/_ssot_data/`).

### Added
- **Zero-Friction Packaging (`pyproject.toml`):** Added PEP 621 / Hatchling package metadata exposing `kmath-mcp` and `kmath` console scripts for `uvx` and `pip` installation, bundling `database/`, `data/`, and `AGENTS.md` inside the wheel.
- **Hardened Container Distribution & GHCR Publishing (`Dockerfile`, `.dockerignore`, `docker-compose.yml`, `.github/workflows/release.yml`):** Multi-stage Rust + Python build running as non-root UID `10001`, exec-form `ENTRYPOINT`, Python-native `HEALTHCHECK`, `read_only: true`, `cap_drop: [ALL]`, `no-new-privileges:true`, and `network_mode: none`. Tagged releases automatically smoke-test all 26 audit gates inside the hardened container and publish to `ghcr.io/imza464-cmyk/citadel-math`, alongside Linux/Windows binaries, Python `.whl`/`.tar.gz` artifacts, `SHA256SUMS.txt`, and an opt-in PyPI OIDC Trusted Publisher job.
- **Community & DX Documentation (`CONTRIBUTING.md`, `README.md`):** Added PR/contribution guidelines and upgraded `README.md` with 3 install paths (`uvx` + pre-built GitHub Releases binaries, GHCR/Docker, from source), a 5-step end-to-end chained AI agent engineering scenario, measured Python-to-Rust subprocess latency & RSS memory benchmarks, and a Troubleshooting section.
- **CI & Security Tests:** Added wheel-packaging verification in `.github/workflows/ci.yml` and `TestPackagingAndBinaryResolution` unit tests in `tests/test_mcp_security.py`.

## [0.1.0] - 2026-11-23

### Added
- **Pure-Rust Core (`citadel_math_core`, `#![forbid(unsafe_code)]`):**
  - 7-dimensional SI lattice ($\mathbb{Z}^7$) and `ScaledUnit` converter (`dimensions.rs`).
  - NIST CODATA 2022 fundamental constants (`constants.rs`, `data/nist_constants.json`).
  - IEEE Std 1788-2015 `tightest` outward-rounded `Interval64` (Knuth `TwoSum` + FMA error-free transforms) and forward-mode `Dual64` differentiation (`interval.rs`).
  - Symbolic AST parser (`ast.rs`) and overflow-checked `i64` `PolyZ` / `MultiPolyZ` (grevlex) algebraic equivalence checker with denominator domain-guard preservation (`GuardedPoly`, `proof_kernel.rs`).
  - Exact integer (`Rat128` Gauss-Jordan) $\mathbb{Z}^7$ nullspace Buckingham-$\Pi$ discovery (`buckingham.rs`).
  - Exact Pareto Skyline (`Rank 0`) and NSGA-II crowding-distance / Utopia knee optimizer (`pareto.rs`).
  - SSOT canonical physics hypergraph (`214` equations, `544` inverse equations, `353` ontology symbols, `18` physical regimes, `30` structural/TPS materials — `graph.rs`, `materials.rs`).
  - 4th-order Runge-Kutta (`RK4`) and adaptive Dormand-Prince `RK4(5)` trajectory integrator with 7-layer USSA-1976 hydrostatic atmosphere (`simulator.rs`), WGS84 $J_2$ secular precession & universal-variable Lambert solver (`astrodynamics.rs`), and 2D parametric CAD DXF/SVG export (`cad.rs`).
- **Python Symbolic Layer & 15-Tool MCP Server (`kmath.py`, `kmath_mcp.py`):**
  - 15 offline Model Context Protocol tools bridging Claude/Cursor/Windsurf agents to `citadel_math_core` and SSOT databases.
  - Zero-`eval` AST math parser (`safe_parse_math_expr` in `engine/dimensions.py`) eliminating `sympy.sympify()` / `eval()` on untrusted LLM inputs.
  - Compatibility shim supporting both `mcp` v1 (`FastMCP`) and v2 (`MCPServer`) with pinned major bounds in `requirements.txt`.
- **Verification, Security & CI:**
  - `33` Rust unit tests (`cargo test`), `0` clippy warnings (`cargo clippy --all-targets -- -D warnings`), `16`-gate Rust audit, `10`-gate Python SSOT audit, and `tests/test_mcp_security.py` (11 RCE/injection payload tests + live MCP bridge checks).
  - Cross-platform GitHub Actions CI (`ubuntu-latest` and `windows-latest`) and automated release binary workflow (`.github/workflows/release.yml`).
  - Threat model and disclosure policy (`SECURITY.md`) and Citadel-Math Source-Available License v1.0 (`LICENSE`).
