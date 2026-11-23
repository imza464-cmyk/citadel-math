# Contributing to Citadel-Math / Katkı Rehberi

We welcome bug reports, mathematical/physical soundness counterexamples, documentation improvements, benchmarks, and Pull Requests!

## 1. Engineering Invariants (Non-Negotiable)

Before opening a Pull Request, make sure your change respects the core invariants in [`AGENTS.md`](./AGENTS.md) and [`SECURITY.md`](./SECURITY.md):

1. **Dimensional Homogeneity ($\mathbb{Z}^7$):** Every equation added to `database/canonical_laws.json` must satisfy `dim(LHS) == dim(RHS)` and reference only symbols registered in `database/ONTOLOGY.json`.
2. **Zero `unsafe` Rust (`#![forbid(unsafe_code)]`):** `citadel_math_core` forbids `unsafe` code crate-wide and must compile with **zero Clippy warnings** (`-D warnings`).
3. **Zero-`eval` Python Parsing:** Never introduce `eval()`, `exec()`, `sympy.sympify()`, or `shell=True` subprocess calls on untrusted inputs. All expression parsing must flow through `engine/dimensions.py:safe_parse_math_expr()`.
4. **No Fake Simplifications:** Algebraic equivalence checks must preserve denominator domain guards (`!= 0`), and numerical checks must preserve IEEE 1788-2015 outward rounding.

## 2. Local Verification Checklist (Required Before PR)

Run the exact CI verification suite locally:

```bash
cargo clippy --manifest-path citadel_math_core/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path citadel_math_core/Cargo.toml
cargo build --release --manifest-path citadel_math_core/Cargo.toml
pip install -e .
python kmath.py audit
python kmath.py rust audit
python -m unittest discover -s tests -v
```

All checks must pass with `0` warnings and `0` failures.

## 3. Reporting Bugs & Soundness Issues

- Use [Bug Report](./.github/ISSUE_TEMPLATE/bug_report.md) for crashes, CLI/MCP errors, or packaging bugs.
- Use [Mathematical / Physical Soundness Issue](./.github/ISSUE_TEMPLATE/scientific_soundness.md) for dimensional mismatches, interval-bound leaks, or NIST/NASA reference discrepancies.
- For security vulnerabilities, follow [`SECURITY.md`](./SECURITY.md) (GitHub Security Advisories — do **not** open a public issue with an active exploit).

## 4. Licensing of Contributions

Citadel-Math is dual-licensed under **GNU AGPL-3.0-only** ([`LICENSE`](./LICENSE)) and a **Commercial License** ([`COMMERCIAL_LICENSE.md`](./COMMERCIAL_LICENSE.md)).

By submitting a Pull Request or patch to this repository, you confirm that:
- Your contribution is your original work (or you have the legal right to submit it), and
- You license your contribution under **GNU AGPL-3.0-only** and grant the project maintainer (Mekserina Karaosmanoğlu) the right to also include your contribution under Citadel-Math's dual commercial license model.

*(Türkçe özet: Bu depoya gönderdiğiniz Pull Request'ler GNU AGPL-3.0-only altında kabul edilir ve proje bakım sorumlusu tarafından çift lisanslama modeli kapsamında kullanılabilir. PR açmadan önce yukarıdaki Rust Clippy, `cargo test`, 26 denetim kapısı ve `tests/test_mcp_security.py` adımlarının tamamı yeşil geçmelidir.)*
