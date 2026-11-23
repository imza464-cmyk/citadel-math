# Security Policy / Güvenlik Politikası

## Security Architecture & Threat Model / Güvenlik Mimarisi

Citadel-Math (`citadel_math_core` + `kmath_mcp.py`) is designed to accept untrusted mathematical expressions from LLM/MCP clients without executing arbitrary code:

1. **Zero `unsafe` Rust Core (`#![forbid(unsafe_code)]`):**
   The native engine in `citadel_math_core/` is compiled with crate-wide `#![forbid(unsafe_code)]`, eliminating memory-safety UB classes and enforcing overflow-checked `i64` arithmetic in the polynomial equivalence checker.
2. **Zero-`eval` Python Input Parsing (`safe_parse_math_expr`):**
   Neither `kmath_mcp.py` nor `engine/dimensions.py` calls Python `eval()`, `exec()`, or `sympy.sympify()` on incoming strings. All expressions are parsed via Python's non-executing `ast.parse(..., mode="eval")` and recursively translated into SymPy nodes through a strict allowlist of arithmetic operators and 15 elementary math functions. Attribute access (`.`), dunder identifiers (`__`), `lambda`, semicolons, and overlong inputs (`> 2048` chars) are rejected before evaluation.
3. **No Shell Injection:**
   The Python-to-Rust bridge (`_run_rust_cli` in `kmath_mcp.py`) invokes the compiled `citadel_math_core` binary directly via `subprocess.run([RUST_BIN, *args], shell=False)` with explicit timeouts.
4. **Offline MCP Runtime:**
   All 15 MCP tools in `kmath_mcp.py` operate strictly offline against local JSON datasets (`database/`, `data/`) and the compiled Rust binary. No outbound network requests are made during MCP tool execution.
5. **Continuous Verification:**
   Cross-platform CI (`.github/workflows/ci.yml` on Linux & Windows) runs `cargo clippy --all-targets -- -D warnings`, `cargo test`, the 26-gate Rust + Python audit suite, and `tests/test_mcp_security.py` (which tests 11 prompt-injection / RCE payloads) on every push to `main`.

---

## Reporting a Vulnerability / Güvenlik Açığı Bildirimi

If you discover a security vulnerability (such as an AST parser bypass, denial-of-service vector, or arithmetic soundness bug):

- **Do not** open a public issue with an active exploit payload.
- Report privately via **GitHub Security Advisories** (`Security` -> `Advisories` -> `Report a vulnerability` on this repository).
- Include reproduction steps, the offending expression/payload, and the affected version/commit hash.

Bir güvenlik açığı tespit ederseniz (AST ayrıştırıcı aşımı, servis dışı bırakma veya aritmetik güvenlik ihlali), istismar kodunu herkese açık Issue olarak paylaşmak yerine bu deponun **GitHub Security Advisories** sekmesinden gizli bildirim açabilirsiniz.
