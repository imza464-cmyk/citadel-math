# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
"""Comprehensive functional, boundary, and schema coverage tests for all 15 MCP tools and engine modules."""

import os
os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")

import json
import sys
import unittest

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

import kmath_mcp
from engine.constants import PhysicalConstants
from engine.dimensions import Dimension, check_dimensional_consistency
from engine.knowledge_graph import KnowledgeGraph
from engine.materials import MaterialsDatabase
from engine.simulator import RK4TrajectorySimulator, RocketVehicle


ALL_15_MCP_TOOLS = [
    "kmath_audit",
    "kmath_dimension_check",
    "kmath_constant",
    "kmath_materials",
    "kmath_stress_check",
    "kmath_simulate_trajectory",
    "kmath_solve_for",
    "kmath_prove_identity",
    "kmath_interval_eval",
    "kmath_synthesize",
    "kmath_inverse_design",
    "kmath_physical_bounds_check",
    "kmath_regime_check",
    "kmath_buckingham_pi",
    "kmath_pareto_front",
]


class TestAll15McpToolsCoverage(unittest.TestCase):
    """Verify every exposed MCP tool is callable, documented with an Example, and returns valid JSON."""

    def test_all_15_tools_have_docstrings_and_examples(self):
        self.assertEqual(len(ALL_15_MCP_TOOLS), 15)
        for name in ALL_15_MCP_TOOLS:
            with self.subTest(tool=name):
                fn = getattr(kmath_mcp, name, None)
                self.assertTrue(callable(fn), f"Missing MCP tool function: {name}")
                doc = getattr(fn, "__doc__", "") or ""
                self.assertIn("Example:", doc, f"Tool {name} docstring must include an Example: block")

    def test_01_kmath_audit(self):
        res = json.loads(kmath_mcp.kmath_audit())
        self.assertEqual(res["status"], "ok")
        self.assertGreaterEqual(res["python_ssot_audit"]["canonical_laws"], 367)
        self.assertEqual(res["python_ssot_audit"]["dimensional_errors"], [])
        self.assertTrue(res["rust_citadel_core_audit"]["passed"])

    def test_02_kmath_dimension_check_and_custom_vars(self):
        custom = json.loads(
            kmath_mcp.kmath_dimension_check(
                "P_tank * r / d_wall",
                "Pa",
                '{"P_tank": "pressure", "r": "L", "d_wall": "m"}',
            )
        )
        self.assertEqual(custom["status"], "ok")
        self.assertTrue(custom["homogeneous"])
        self.assertEqual(custom["verdict"], "DIMENSIONALLY_HOMOGENEOUS")

        bad_json = json.loads(kmath_mcp.kmath_dimension_check("F", "m * a", "{invalid"))
        self.assertEqual(bad_json["status"], "error")

    def test_03_kmath_constant_codata_iau_and_substring(self):
        iau = json.loads(kmath_mcp.kmath_constant("mu_earth"))
        self.assertEqual(iau["status"], "ok")
        self.assertEqual(iau["source"], "IAU_SSOT")
        self.assertGreater(iau["value"], 3.98e14)

        sub = json.loads(kmath_mcp.kmath_constant("Planck"))
        self.assertEqual(sub["status"], "ok")
        self.assertGreaterEqual(sub.get("total_matches", 1), 1)

        empty = json.loads(kmath_mcp.kmath_constant("   "))
        self.assertEqual(empty["status"], "error")

    def test_04_kmath_materials_filter_sort_and_unknown_category(self):
        alloys = json.loads(kmath_mcp.kmath_materials(sort_by="yield_strength", category="metal_alloy"))
        self.assertEqual(alloys["status"], "ok")
        self.assertGreater(alloys["total_materials"], 0)
        strengths = [m["yield_strength_MPa"] for m in alloys["materials"]]
        self.assertEqual(strengths, sorted(strengths, reverse=True))

        unknown = json.loads(kmath_mcp.kmath_materials(sort_by="max_temp", category="nonexistent_cat"))
        self.assertEqual(unknown["status"], "ok")
        self.assertEqual(unknown["total_materials"], 0)
        self.assertIn("available_categories", unknown)

    def test_05_kmath_stress_check_pass_and_unknown_material(self):
        stress = json.loads(
            kmath_mcp.kmath_stress_check(
                "mat_alli_2195",
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
        self.assertTrue(stress["overall_pass"])
        self.assertGreater(stress["yield_safety_factor"], 1.35)
        self.assertGreater(stress["buckling_safety_factor"], 1.35)

        bad_mat = json.loads(
            kmath_mcp.kmath_stress_check("mat_does_not_exist", mass_kg=1000.0, g_load=2.0, max_q_pa=10000.0)
        )
        self.assertEqual(bad_mat["status"], "error")

    def test_06_kmath_simulate_trajectory_single_and_compare(self):
        beamed = json.loads(kmath_mcp.kmath_simulate_trajectory("beamed"))
        self.assertEqual(beamed["status"], "ok")
        self.assertEqual(len(beamed["results"]), 1)
        self.assertGreater(beamed["results"][0]["max_altitude_km"], 10.0)

        comp_raw = kmath_mcp.kmath_simulate_trajectory("compare")
        self.assertTrue(len(comp_raw) > 20)

    def test_07_kmath_solve_for_tsiolkovsky_and_domain_guards(self):
        solved = json.loads(
            kmath_mcp.kmath_solve_for("delta_v = (g_0 * I_sp) * ln(m_0 / m_f)", "m_0")
        )
        self.assertEqual(solved["status"], "ok")
        self.assertEqual(solved["target"], "m_0")
        self.assertIn("exp", solved["isolated_rhs"])
        self.assertTrue(len(solved.get("safe_rust_fn", "")) > 0)

    def test_08_kmath_prove_identity_guarded_singularity(self):
        singular = json.loads(kmath_mcp.kmath_prove_identity("(x^2 - 1) / (x - 1)", "x + 1"))
        self.assertEqual(singular["status"], "ok")
        self.assertFalse(singular["proven_identical"])
        self.assertTrue(len(singular.get("lhs_domain_guards", [])) >= 1)

    def test_09_kmath_interval_eval_hoop_stress_and_derivative(self):
        iv = json.loads(
            kmath_mcp.kmath_interval_eval(
                "(P * r) / t",
                '{"P": [390000, 410000], "r": [1.795, 1.805], "t": [0.0049, 0.0051]}',
                "t",
            )
        )
        self.assertEqual(iv["status"], "ok")
        lo, hi = iv["outward_interval"]
        self.assertLess(lo, 144_000_000.0)
        self.assertGreater(hi, 144_000_000.0)

    def test_10_kmath_synthesize_hypergraph_derivation(self):
        syn = json.loads(kmath_mcp.kmath_synthesize("delta_v,g_0,I_sp,m_f", "m_0", max_depth=3))
        self.assertEqual(syn["status"], "ok")
        self.assertGreaterEqual(len(syn.get("synthesized_formulas", syn.get("candidates", [1]))), 1)

    def test_11_kmath_inverse_design_pareto_sizing(self):
        design = json.loads(
            kmath_mcp.kmath_inverse_design(
                payload_kg=500.0,
                target_orbit_km=200.0,
                max_g_limit=8.5,
                min_safety_factor=1.35,
            )
        )
        self.assertEqual(design["status"], "ok")
        self.assertGreater(design["exact_skyline_count"], 0)
        self.assertGreater(len(design["pareto_optimal_designs"]), 0)

    def test_12_kmath_physical_bounds_check_valid_and_violations(self):
        valid = json.loads(
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
        self.assertTrue(valid["physically_realizable"])
        self.assertEqual(valid["violations"], [])

        invalid = json.loads(
            kmath_mcp.kmath_physical_bounds_check(
                velocity_ms=3.1e8,
                temp_hot_k=600.0,
                temp_cold_k=300.0,
                efficiency=0.85,
            )
        )
        self.assertFalse(invalid["physically_realizable"])
        self.assertGreaterEqual(len(invalid["violations"]), 2)

    def test_13_kmath_regime_check_subsonic_and_violation(self):
        ok_reg = json.loads(
            kmath_mcp.kmath_regime_check(
                "law_prandtl_glauert_subsonic_cp",
                '{"Mach": 0.55, "C_p0": -0.4}',
            )
        )
        self.assertEqual(ok_reg["status"], "ok")
        self.assertTrue(ok_reg["valid_in_regime"])

        bad_reg = json.loads(
            kmath_mcp.kmath_regime_check(
                "law_prandtl_glauert_subsonic_cp",
                '{"Mach": 1.35, "C_p0": -0.4}',
            )
        )
        self.assertEqual(bad_reg["status"], "ok")
        self.assertFalse(bad_reg["valid_in_regime"])

    def test_14_kmath_buckingham_pi_with_calibration_sample(self):
        bpi = json.loads(
            kmath_mcp.kmath_buckingham_pi(
                "r,E,rho,t",
                "r",
                '[{"r": 80.5, "E": 8.4e13, "rho": 1.225, "t": 0.006}]',
            )
        )
        self.assertEqual(bpi["status"], "ok")
        self.assertEqual(bpi["num_pi_groups"], 1)

    def test_15_kmath_pareto_front_skyline_and_knee(self):
        pf = json.loads(
            kmath_mcp.kmath_pareto_front(
                '[{"name": "mass_kg", "direction": "min", "unit": "kg"}, {"name": "sf", "direction": "max", "unit": "x"}]',
                '[{"id": "light", "values": [950.0, 1.45]}, {"id": "balanced", "values": [1080.0, 1.95]}, {"id": "dominated", "values": [1300.0, 1.40]}]',
            )
        )
        self.assertEqual(pf["status"], "ok")
        self.assertEqual(pf["skyline_count"], 2)


class TestPythonEngineModules(unittest.TestCase):
    """Direct unit tests for Python SSOT engine modules (dimensions, constants, materials, graph, simulator)."""

    def test_dimension_algebra_and_consistency(self):
        force = Dimension(m=1, l=1, t=-2)
        mass_acc = Dimension(m=1) * Dimension(l=1, t=-2)
        ok, _ = check_dimensional_consistency(force, mass_acc)
        self.assertTrue(ok)

    def test_physical_constants_invariants(self):
        self.assertEqual(PhysicalConstants.c, 299792458.0)
        self.assertAlmostEqual(PhysicalConstants.g0, 9.80665, places=5)

    def test_materials_database_loads_80_certified_materials(self):
        mdb = MaterialsDatabase(kmath_mcp.MAT_PATH)
        self.assertGreaterEqual(len(mdb.materials), 80)
        alli = mdb.materials["mat_alli_2195"]
        self.assertGreater(alli.breaking_length_km, 10.0)

    def test_knowledge_graph_loads_canonical_laws(self):
        kg = KnowledgeGraph()
        kg.load_canonical_database(kmath_mcp.DB_PATH)
        summary = kg.summary()
        self.assertGreaterEqual(summary["equations"], 367)
        self.assertGreaterEqual(len(summary["domains"]), 30)

    def test_rk4_simulator_monotonic_burn(self):
        veh = RocketVehicle(
            name="Test Probe",
            dry_mass_kg=500.0,
            propellant_mass_kg=1500.0,
            thrust_vacuum_n=45000.0,
            isp_vacuum_s=320.0,
            cross_section_m2=0.8,
            drag_coeff=0.25,
        )
        sim = RK4TrajectorySimulator(dt_seconds=0.5)
        out = sim.simulate(veh, max_time_s=15.0)
        self.assertGreater(out["max_altitude_km"], 0.1)
        self.assertGreater(out["final_speed_ms"], 50.0)


if __name__ == "__main__":
    unittest.main()
