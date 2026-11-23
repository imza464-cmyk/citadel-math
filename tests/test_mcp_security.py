# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
"""End-to-end security, packaging & functional tests for kmath_mcp.py and zero-eval AST parsing."""

import json
import os
import sys
import unittest
from unittest import mock

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

from engine.dimensions import safe_parse_math_expr
import kmath_mcp


class TestZeroEvalParserSecurity(unittest.TestCase):
    """Verify untrusted LLM inputs cannot trigger Python code execution (no sympify/eval)."""

    def test_blocks_dunder_import_and_eval_payloads(self):
        malicious_payloads = [
            "__import__('os').system('echo pwned')",
            "().__class__.__bases__[0].__subclasses__()",
            "(lambda x: x)(1)",
            "open('/etc/passwd').read()",
            "eval('1+1')",
            "exec('import os')",
            "getattr(1, '__class__')",
            "1; import os",
            "x.y + 1",
            "sin(x, evaluate=False)",
            "a" * 3000,
        ]
        for payload in malicious_payloads:
            with self.subTest(payload=payload[:40]):
                with self.assertRaises(ValueError):
                    safe_parse_math_expr(payload)

    def test_mcp_dimension_check_rejects_prompt_injection(self):
        raw = kmath_mcp.kmath_dimension_check(
            "__import__('os').system('echo pwned')",
            "m",
        )
        res = json.loads(raw)
        self.assertEqual(res["verdict"], "DIMENSIONAL_OR_SYNTAX_ERROR")
        self.assertFalse(res["homogeneous"])

        raw2 = kmath_mcp.kmath_dimension_check("().__class__.__bases__", "m")
        res2 = json.loads(raw2)
        self.assertEqual(res2["verdict"], "DIMENSIONAL_OR_SYNTAX_ERROR")
        self.assertFalse(res2["homogeneous"])


class TestMcpEndToEndTools(unittest.TestCase):
    """Smoke-test the live Python + Rust MCP tool bridge in CI."""

    def test_dimension_check_homogeneity_and_scale(self):
        ok_res = json.loads(kmath_mcp.kmath_dimension_check("F", "m * a"))
        self.assertEqual(ok_res["verdict"], "DIMENSIONALLY_HOMOGENEOUS")
        self.assertTrue(ok_res["homogeneous"])

        pi_res = json.loads(kmath_mcp.kmath_dimension_check("2 * pi * r", "length"))
        self.assertEqual(pi_res["verdict"], "DIMENSIONALLY_HOMOGENEOUS")

        scale_res = json.loads(kmath_mcp.kmath_dimension_check("kN", "kg * m / s^2"))
        self.assertEqual(
            scale_res["verdict"],
            "DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH",
        )
        self.assertAlmostEqual(scale_res["si_scale_ratio_lhs_over_rhs"], 1000.0, places=6)

        bad_res = json.loads(kmath_mcp.kmath_dimension_check("F", "m + a"))
        self.assertFalse(bad_res["homogeneous"])

    def test_constant_lookup_codata(self):
        res = json.loads(kmath_mcp.kmath_constant("c"))
        self.assertEqual(res["status"], "ok")
        self.assertIn("299792458", json.dumps(res))

    def test_rust_bridge_prove_interval_buckingham_pareto(self):
        poly = json.loads(kmath_mcp.kmath_prove_identity("(x + y)^2", "x^2 + 2*x*y + y^2"))
        self.assertEqual(poly["status"], "ok")
        self.assertTrue(poly["proven_identical"])

        iv = json.loads(
            kmath_mcp.kmath_interval_eval(
                "(x + 1) / y",
                '{"x": [1.0, 2.0], "y": [2.0, 4.0]}',
                "x",
            )
        )
        self.assertEqual(iv["status"], "ok")

        bpi = json.loads(kmath_mcp.kmath_buckingham_pi("r,E,rho,t", "r"))
        self.assertEqual(bpi["status"], "ok")

        pareto = json.loads(
            kmath_mcp.kmath_pareto_front(
                '[{"name": "mass", "direction": "min"}, {"name": "sf", "direction": "max"}]',
                '[{"id": "A", "values": [1000.0, 1.4]}, {"id": "B", "values": [1200.0, 2.1]}]',
            )
        )
        self.assertEqual(pareto["status"], "ok")


class TestPackagingAndBinaryResolution(unittest.TestCase):
    """Verify deterministic Rust binary discovery (KMATH_RUST_BIN/PATH) and wheel-safe SSOT resolution."""

    def test_resolve_data_file_finds_canonical_ssot_assets(self):
        laws_path = kmath_mcp._resolve_data_file("database", "canonical_laws.json")
        nist_path = kmath_mcp._resolve_data_file("data", "nist_constants.json")
        self.assertTrue(os.path.isfile(laws_path), f"Missing canonical laws: {laws_path}")
        self.assertTrue(os.path.isfile(nist_path), f"Missing NIST constants: {nist_path}")

    def test_resolve_rust_bin_default_finds_compiled_binary(self):
        resolved, searched = kmath_mcp.resolve_rust_bin()
        self.assertIsNotNone(resolved, f"Compiled Rust binary not found in {searched}")
        self.assertTrue(os.path.isfile(resolved))
        self.assertTrue(os.path.isabs(resolved), f"Resolved path must be absolute: {resolved}")

    def test_resolve_rust_bin_path_normalizes_relative_which_result(self):
        with mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("KMATH_RUST_BIN", None)
            with mock.patch("kmath_mcp.os.path.isfile", side_effect=lambda p: p == ".\\citadel_math_core.exe"), \
                 mock.patch("kmath_mcp.shutil.which", return_value=".\\citadel_math_core.exe"):
                resolved, _ = kmath_mcp.resolve_rust_bin()
                self.assertIsNotNone(resolved)
                self.assertTrue(os.path.isabs(resolved), f"Relative shutil.which result not normalized: {resolved}")

    def test_resolve_rust_bin_honors_explicit_env_override(self):
        default_bin, _ = kmath_mcp.resolve_rust_bin()
        self.assertIsNotNone(default_bin)
        with mock.patch.dict(os.environ, {"KMATH_RUST_BIN": default_bin}):
            resolved, searched = kmath_mcp.resolve_rust_bin()
            self.assertEqual(resolved, os.path.abspath(default_bin))
            self.assertEqual(searched, [os.path.abspath(default_bin)])

    def test_missing_env_override_returns_clear_troubleshooting_error(self):
        missing_path = os.path.join(ROOT_DIR, "nonexistent_citadel_math_core_bin")
        with mock.patch.dict(os.environ, {"KMATH_RUST_BIN": missing_path}):
            resolved, searched = kmath_mcp.resolve_rust_bin()
            self.assertIsNone(resolved)
            self.assertIn("KMATH_RUST_BIN", searched[0])

            rc, out, err = kmath_mcp._run_rust_cli(["audit"])
            self.assertEqual(rc, 127)
            self.assertEqual(out, "")
            self.assertIn("KMATH_RUST_BIN", err)
            self.assertIn("releases", err)

            tool_res = json.loads(kmath_mcp.kmath_prove_identity("x + 1", "1 + x"))
            self.assertEqual(tool_res["status"], "error")
            self.assertIn("KMATH_RUST_BIN", tool_res["message"])


if __name__ == "__main__":
    unittest.main()
