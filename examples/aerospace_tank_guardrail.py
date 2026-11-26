#!/usr/bin/env python3
# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
"""End-to-end 5-step Phase-A cryogenic LOX tank engineering guardrail pipeline."""

import os
os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")

import json
import sys

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

import kmath_mcp


def run_cryogenic_tank_guardrail() -> dict:
    # Step 1: Rank aerospace alloys by yield strength
    alloys = json.loads(kmath_mcp.kmath_materials(sort_by="yield_strength", category="metal_alloy"))
    alli_2195 = next(m for m in alloys["materials"] if m["id"] == "mat_alli_2195")

    # Step 2: Catch 1000x unit scale mismatch before structural sizing
    scale_guard = json.loads(kmath_mcp.kmath_dimension_check("kN", "kg * m / s^2"))
    assert scale_guard["verdict"] == "DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH"

    # Step 3: Biaxial Von Mises + NASA SP-8007 buckling + automatic PICA-X TPS sizing
    stress = json.loads(
        kmath_mcp.kmath_stress_check(
            material_id=alli_2195["id"],
            mass_kg=45000.0,
            g_load=4.5,
            max_q_pa=35000.0,
            radius_m=1.8,
            length_m=14.0,
            wall_thickness_mm=5.0,
            speed_ms=2400.0,
            tank_pressure_bar=4.0,
        )
    )
    assert stress["overall_pass"] is True

    # Step 4: IEEE 1788-2015 outward-rounded hoop stress bounds under manufacturing tolerances
    hoop_bounds = json.loads(
        kmath_mcp.kmath_interval_eval(
            "(P * r) / t",
            '{"P": [390000, 410000], "r": [1.795, 1.805], "t": [0.0049, 0.0051]}',
            "t",
        )
    )
    assert hoop_bounds["status"] == "ok"

    # Step 5: Thermodynamic (Carnot) & relativistic causality seal
    bounds_seal = json.loads(
        kmath_mcp.kmath_physical_bounds_check(
            mass_kg=45000.0,
            velocity_ms=2400.0,
            exhaust_velocity_ms=3400.0,
            mass_flow_kg_s=280.0,
            thrust_n=952000.0,
            temp_hot_k=3200.0,
            temp_cold_k=300.0,
            efficiency=0.68,
        )
    )
    assert bounds_seal["physically_realizable"] is True

    return {
        "selected_alloy": alli_2195["id"],
        "caught_scale_ratio": scale_guard["si_scale_ratio_lhs_over_rhs"],
        "yield_safety_factor": stress["yield_safety_factor"],
        "buckling_safety_factor": stress["buckling_safety_factor"],
        "hoop_stress_pa_interval": hoop_bounds["outward_interval"],
        "physically_realizable": bounds_seal["physically_realizable"],
    }


if __name__ == "__main__":
    summary = run_cryogenic_tank_guardrail()
    print(json.dumps(summary, indent=2))
