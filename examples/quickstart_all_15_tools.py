#!/usr/bin/env python3
# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
"""Runnable quickstart demonstrating all 15 deterministic Citadel-Math MCP tools offline."""

import os
os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")

import json
import sys

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

import kmath_mcp


def main() -> None:
    checks = [
        ("1. kmath_dimension_check", json.loads(kmath_mcp.kmath_dimension_check("kN", "kg * m / s^2"))["verdict"]),
        ("2. kmath_constant", json.loads(kmath_mcp.kmath_constant("c"))["value"]),
        ("3. kmath_materials", json.loads(kmath_mcp.kmath_materials("yield_strength", "metal_alloy"))["total_materials"]),
        (
            "4. kmath_stress_check",
            json.loads(
                kmath_mcp.kmath_stress_check(
                    "mat_alli_2195", 45000.0, 4.5, 35000.0, 1.8, 14.0, 5.0, 2400.0, 4.0
                )
            )["overall_pass"],
        ),
        (
            "5. kmath_simulate_trajectory",
            json.loads(kmath_mcp.kmath_simulate_trajectory("beamed"))["results"][0]["max_altitude_km"],
        ),
        (
            "6. kmath_solve_for",
            json.loads(kmath_mcp.kmath_solve_for("delta_v = (g_0 * I_sp) * ln(m_0 / m_f)", "m_0"))["isolated_rhs"],
        ),
        (
            "7. kmath_prove_identity",
            json.loads(kmath_mcp.kmath_prove_identity("(x^2 - 1) / (x - 1)", "x + 1"))["proven_identical"],
        ),
        (
            "8. kmath_interval_eval",
            json.loads(
                kmath_mcp.kmath_interval_eval(
                    "(P * r) / t",
                    '{"P": [390000, 410000], "r": [1.795, 1.805], "t": [0.0049, 0.0051]}',
                    "t",
                )
            )["outward_interval"],
        ),
        (
            "9. kmath_synthesize",
            json.loads(kmath_mcp.kmath_synthesize("delta_v,g_0,I_sp,m_f", "m_0", 3))["status"],
        ),
        (
            "10. kmath_inverse_design",
            json.loads(kmath_mcp.kmath_inverse_design(500.0, 200.0, 8.5, 1.35))["exact_skyline_count"],
        ),
        (
            "11. kmath_physical_bounds_check",
            json.loads(
                kmath_mcp.kmath_physical_bounds_check(
                    velocity_ms=2400.0, temp_hot_k=3200.0, temp_cold_k=300.0, efficiency=0.68
                )
            )["physically_realizable"],
        ),
        (
            "12. kmath_regime_check",
            json.loads(
                kmath_mcp.kmath_regime_check("law_prandtl_glauert_subsonic_cp", '{"Mach": 0.55, "C_p0": -0.4}')
            )["valid_in_regime"],
        ),
        ("13. kmath_buckingham_pi", json.loads(kmath_mcp.kmath_buckingham_pi("r,E,rho,t", "r"))["num_pi_groups"]),
        (
            "14. kmath_pareto_front",
            json.loads(
                kmath_mcp.kmath_pareto_front(
                    '[{"name": "mass", "direction": "min"}, {"name": "sf", "direction": "max"}]',
                    '[{"id": "A", "values": [1000.0, 1.5]}, {"id": "B", "values": [1200.0, 2.1]}]',
                )
            )["skyline_count"],
        ),
        ("15. kmath_audit", json.loads(kmath_mcp.kmath_audit())["status"]),
    ]

    print("=== Citadel-Math 15-Tool Deterministic Quickstart ===")
    for label, summary in checks:
        print(f"  [PASS] {label:<30} -> {summary}")


if __name__ == "__main__":
    main()
