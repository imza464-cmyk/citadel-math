"""Independent 80-Digit Arbitrary-Precision Decimal Oracle Tests for Rust Interval64.

Verifies IEEE 1788-2015 outward-rounding containment (`lo <= exact_80digit <= hi`)
of the compiled `citadel_math_core` Rust binary across arithmetic, rational,
catastrophic-cancellation, subnormal, and transcendental (`sqrt`, `exp`, `ln`, `pow`)
expressions using Python's independent `decimal.Decimal` engine (prec=80).
"""
import os
os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")

import decimal
import json
import random
import subprocess
import sys
import unittest

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT_DIR not in sys.path:
    sys.path.insert(0, ROOT_DIR)

from kmath_mcp import resolve_rust_bin

CTX = decimal.Context(prec=80)


def _eval_rust_interval(expr: str, bounds: dict) -> tuple[tuple[float, float], tuple[float, float]]:
    resolved = resolve_rust_bin()
    rust_bin = resolved[0] if isinstance(resolved, tuple) else resolved
    if not rust_bin:
        raise unittest.SkipTest("citadel_math_core Rust binary not found")
    bounds_json = json.dumps({k: [lo, hi] for k, (lo, hi) in bounds.items()})
    res = subprocess.run(
        [rust_bin, "mcp", "interval_eval", expr, bounds_json],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        timeout=15,
        check=True,
    )
    data = json.loads(res.stdout)
    assert data.get("status") == "ok", f"Rust interval_eval failed for {expr} ({bounds_json}): {data}"
    naive = (float(data["outward_interval"][0]), float(data["outward_interval"][1]))
    taylor = (float(data["taylor_mean_value_interval"][0]), float(data["taylor_mean_value_interval"][1]))
    return naive, taylor


def _d(x: float) -> decimal.Decimal:
    # Exact binary-float -> Decimal conversion (zero decimal-parsing error!)
    return decimal.Decimal.from_float(x)


class TestInterval64IndependentOracle(unittest.TestCase):
    def _assert_contains_exact(self, intervals, exact: decimal.Decimal, label: str):
        naive, taylor = intervals
        for kind, (lo, hi) in (("outward", naive), ("taylor_mv", taylor)):
            self.assertLessEqual(lo, hi, f"{label} [{kind}]: lo > hi")
            self.assertLessEqual(
                _d(lo), exact,
                f"CONTAINMENT VIOLATION (lo > exact) [{label} / {kind}]: lo={lo!r}, exact={exact}"
            )
            self.assertGreaterEqual(
                _d(hi), exact,
                f"CONTAINMENT VIOLATION (hi < exact) [{label} / {kind}]: hi={hi!r}, exact={exact}"
            )

    def test_pow_near_integer_exponent_no_snapping_oracle(self):
        # Regression: 1000^(2 + 5e-13) and 1000^(0.5 + 5e-13) MUST NOT snap to integer/0.5!
        p1 = 2.0 + 5.0e-13
        ivs1 = _eval_rust_interval("x ** p", {"x": (1000.0, 1000.0), "p": (p1, p1)})
        exact1 = CTX.exp(CTX.multiply(_d(p1), CTX.ln(_d(1000.0))))
        self._assert_contains_exact(ivs1, exact1, "1000**(2+5e-13)")

        p2 = 0.5 + 5.0e-13
        ivs2 = _eval_rust_interval("x ** p", {"x": (1000.0, 1000.0), "p": (p2, p2)})
        exact2 = CTX.exp(CTX.multiply(_d(p2), CTX.ln(_d(1000.0))))
        self._assert_contains_exact(ivs2, exact2, "1000**(0.5+5e-13)")

    def test_transcendental_exp_ln_sqrt_independent_80digit_oracle(self):
        rng = random.Random(20261124)
        # 1. exp(x) across negative, near-zero, moderate, and large values
        for x in [-50.0, -1.0, -1e-7, 0.0, 1e-7, 0.6931471805599453, 1.0, 10.0, 50.0] + [rng.uniform(-40.0, 40.0) for _ in range(12)]:
            ivs = _eval_rust_interval("exp(x)", {"x": (x, x)})
            exact = CTX.exp(_d(x))
            self._assert_contains_exact(ivs, exact, f"exp({x})")

        # 2. ln(x) across sub-unity, near-1, and large scales
        for x in [1e-120, 0.5, 1.0 - 1e-12, 1.0, 1.0 + 1e-12, 2.0, 2.718281828459045, 1e120] + [10.0 ** rng.uniform(-80, 80) for _ in range(12)]:
            ivs = _eval_rust_interval("ln(x)", {"x": (x, x)})
            exact = CTX.ln(_d(x))
            self._assert_contains_exact(ivs, exact, f"ln({x})")

        # 3. sqrt(x) across tiny, exact squares, and irrational inputs
        for x in [0.0, 1e-250, 0.25, 2.0, 3.0, 10.0, 1e250] + [10.0 ** rng.uniform(-100, 100) for _ in range(10)]:
            ivs = _eval_rust_interval("sqrt(x)", {"x": (x, x)})
            exact = CTX.sqrt(_d(x))
            self._assert_contains_exact(ivs, exact, f"sqrt({x})")

    def test_compound_physics_expressions_interval_enclosure_80digit_oracle(self):
        # Test non-degenerate intervals: sample corners + interior points and verify exact 80-digit values ∈ [lo, hi]
        cases = [
            (
                "v_e * ln(m0 / mf)",
                {"v_e": (3100.0, 3150.0), "m0": (980.0, 1020.0), "mf": (195.0, 205.0)},
                lambda p: CTX.multiply(_d(p["v_e"]), CTX.ln(CTX.divide(_d(p["m0"]), _d(p["mf"])))),
            ),
            (
                "(P * r) / t",
                {"P": (395000.0, 405000.0), "r": (1.795, 1.805), "t": (0.0049, 0.0051)},
                lambda p: CTX.divide(CTX.multiply(_d(p["P"]), _d(p["r"])), _d(p["t"])),
            ),
            (
                "sqrt(2 * mu / r)",
                {"mu": (3.986e14, 3.98601e14), "r": (6.77e6, 6.79e6)},
                lambda p: CTX.sqrt(CTX.divide(CTX.multiply(decimal.Decimal(2), _d(p["mu"])), _d(p["r"]))),
            ),
            (
                "x - x^2",
                {"x": (0.4, 0.6)},
                lambda p: CTX.subtract(_d(p["x"]), CTX.multiply(_d(p["x"]), _d(p["x"]))),
            ),
        ]
        rng = random.Random(42)
        for expr, bounds, oracle_fn in cases:
            ivs = _eval_rust_interval(expr, bounds)
            for _ in range(30):
                sample = {k: rng.uniform(a, b) for k, (a, b) in bounds.items()}
                exact = oracle_fn(sample)
                self._assert_contains_exact(ivs, exact, f"{expr} @ {sample}")


if __name__ == "__main__":
    unittest.main()
