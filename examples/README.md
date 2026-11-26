# Citadel-Math Runnable Examples / Çalıştırılabilir Örnekler

All examples run 100% offline against the local `kmath_mcp.py` server and `citadel_math_core` pure-Rust engine.

## 1. Python MCP Examples (`examples/`)

```bash
# Run all 15 deterministic MCP tools end-to-end
python examples/quickstart_all_15_tools.py

# Run the 5-step Phase-A cryogenic LOX tank guardrail workflow
python examples/aerospace_tank_guardrail.py
```

## 2. Pure-Rust Core Example (`citadel_math_core/examples/`)

```bash
cargo run --manifest-path citadel_math_core/Cargo.toml --example interval_and_proof
```
