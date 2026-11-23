# Changelog

All notable changes to **Citadel-Math (`citadel_math_core` + `kmath-mcp`)** are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed
- **Dual-License Model (GNU AGPL-3.0-only + Commercial):** Replaced the restrictive read-only source-available license with verbatim [GNU AGPL-3.0-only](./LICENSE) paired with an optional [Commercial License](./COMMERCIAL_LICENSE.md), enabling open-source community Pull Requests while protecting against closed-source commercial SaaS appropriation.
- **Smart Rust Binary & Wheel-Safe SSOT Discovery (`kmath_mcp.py`, `kmath.py`):** Added deterministic `resolve_rust_bin()` lookup (`KMATH_RUST_BIN` -> `target/release` -> `target/debug` -> `bin/` -> `PATH` including GitHub Releases asset names) with clear troubleshooting errors (`rc=127`), plus `_resolve_data_file()` fallback to packaged wheel SSOT assets (`engine/_ssot_data/`).

### Added
- **Zero-Friction Packaging (`pyproject.toml`):** Added PEP 621 / Hatchling package metadata exposing `kmath-mcp` and `kmath` console scripts for `uvx` and `pip` installation, bundling `database/`, `data/`, and `AGENTS.md` inside the wheel.
- **Hardened Container Distribution (`Dockerfile`, `.dockerignore`, `docker-compose.yml`):** Multi-stage Rust + Python build running as non-root UID `10001`, exec-form `ENTRYPOINT`, Python-native `HEALTHCHECK`, `read_only: true`, `cap_drop: [ALL]`, `no-new-privileges:true`, and `network_mode: none`.
- **Community & DX Documentation (`CONTRIBUTING.md`, `README.md`):** Added PR/contribution guidelines and upgraded `README.md` with 3 install paths (`uvx` + pre-built GitHub Releases binaries, Docker, from source), a 5-step end-to-end chained AI agent engineering scenario, measured Python-to-Rust subprocess latency & RSS memory benchmarks, and a Troubleshooting section.
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
