# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# Matematik Çekirdeği — Tam Kesirli Z^7 Buckingham-Pi Keşif ve Çok Amaçlı NSGA-II Pareto Motoru (Python Eşliği)

from __future__ import annotations

from fractions import Fraction
import math
from typing import Any, Dict, List, Optional, Sequence, Tuple

from engine.dimensions import DIMENSIONS, Dimension


class BuckinghamPiDiscovery:
    """7-Temel SI boyut kafesi (Z^7) üzerinde tam kesirli (Fraction) RREF ile boyutsuz Pi gruplarını keşfeder."""

    @staticmethod
    def _dim_vector(d: Dimension) -> Tuple[int, int, int, int, int, int, int]:
        return (d.m, d.l, d.t, d.i, d.theta, d.n, d.j)

    @classmethod
    def _nnz_and_l1(cls, d: Dimension) -> Tuple[int, int]:
        comps = cls._dim_vector(d)
        nnz = sum(1 for c in comps if c != 0)
        l1 = sum(abs(c) for c in comps)
        return nnz, l1

    @classmethod
    def discover(
        cls,
        variables: Sequence[Tuple[str, Dimension]],
        target_var: Optional[str] = None,
        repeating_hint: Sequence[str] = (),
    ) -> Dict[str, Any]:
        if not variables:
            raise ValueError("En az bir fiziksel değişken belirtilmelidir!")
        names = [nm for nm, _ in variables]
        if target_var and target_var not in names:
            raise ValueError(f"Hedef değişken '{target_var}' verilen değişken kümesinde bulunamadı!")

        n = len(variables)
        hint_index = {nm: i for i, nm in enumerate(repeating_hint)}

        def col_key(idx: int) -> Tuple[int, int, int, int]:
            nm, dm = variables[idx]
            if target_var and nm == target_var:
                return (3, 999, 999, idx)
            if nm in hint_index:
                return (0, hint_index[nm], 0, idx)
            if dm.is_dimensionless():
                return (2, 0, 0, idx)
            nnz, l1 = cls._nnz_and_l1(dm)
            return (1, nnz, l1, idx)

        perm = sorted(range(n), key=col_key)
        mat: List[List[Fraction]] = [[Fraction(0) for _ in range(n)] for _ in range(7)]
        for col_pos, orig_idx in enumerate(perm):
            comps = cls._dim_vector(variables[orig_idx][1])
            for row_idx, c in enumerate(comps):
                mat[row_idx][col_pos] = Fraction(c)

        # Tam kesirli Gauss-Jordan RREF
        pivot_cols: List[int] = []
        piv_row = 0
        for col in range(n):
            if piv_row >= 7:
                break
            best_r: Optional[int] = None
            for r in range(piv_row, 7):
                if mat[r][col] != 0:
                    if best_r is None or abs(mat[r][col]) > abs(mat[best_r][col]):
                        best_r = r
            if best_r is None:
                continue
            mat[piv_row], mat[best_r] = mat[best_r], mat[piv_row]
            piv_val = mat[piv_row][col]
            for c_idx in range(col, n):
                mat[piv_row][c_idx] /= piv_val
            for r in range(7):
                if r != piv_row and mat[r][col] != 0:
                    factor = mat[r][col]
                    for c_idx in range(col, n):
                        mat[r][c_idx] -= factor * mat[piv_row][c_idx]
            pivot_cols.append(col)
            piv_row += 1

        rank = len(pivot_cols)
        repeating_basis = [variables[perm[c]][0] for c in pivot_cols]
        pivot_set = set(pivot_cols)
        free_cols = [c for c in range(n) if c not in pivot_set]
        target_orig_idx = names.index(target_var) if target_var else None

        raw_vectors: List[List[int]] = []
        for f_col in free_cols:
            vec_perm = [Fraction(0) for _ in range(n)]
            vec_perm[f_col] = Fraction(1)
            for p_row, p_col in enumerate(pivot_cols):
                vec_perm[p_col] = -mat[p_row][f_col]

            lcm_den = 1
            for q in vec_perm:
                if q != 0:
                    lcm_den = math.lcm(lcm_den, q.denominator)

            exps = [0] * n
            for col_pos, q in enumerate(vec_perm):
                if q != 0:
                    exps[perm[col_pos]] = int(q.numerator * (lcm_den // q.denominator))

            nz = [abs(e) for e in exps if e != 0]
            g_all = math.gcd(*nz) if nz else 1
            if g_all > 1:
                exps = [e // g_all for e in exps]

            if target_orig_idx is not None and exps[target_orig_idx] < 0:
                exps = [-e for e in exps]

            # Z^7 boyutsuzluk doğrulaması (Değişmez 1)
            acc = Dimension()
            for (_, dm), e in zip(variables, exps):
                if e != 0:
                    acc = acc * (dm ** e)
            if not acc.is_dimensionless():
                raise RuntimeError(f"Z^7 içsel hata: Pi grubu boyutsuz değil ({acc})!")

            raw_vectors.append(exps)

        if target_orig_idx is not None:
            for i, v in enumerate(raw_vectors):
                if v[target_orig_idx] != 0 and i != 0:
                    raw_vectors[0], raw_vectors[i] = raw_vectors[i], raw_vectors[0]
                    break

        pi_groups: List[Dict[str, Any]] = []
        for idx, exps in enumerate(raw_vectors, 1):
            pairs = [(nm, e) for (nm, _), e in zip(variables, exps) if e != 0]
            num_parts = [nm if e == 1 else f"{nm}**{e}" for nm, e in pairs if e > 0]
            den_parts = [nm if -e == 1 else f"{nm}**{-e}" for nm, e in pairs if e < 0]
            num_s = " * ".join(num_parts) if num_parts else "1"
            expr_s = num_s if not den_parts else f"({num_s}) / ({' * '.join(den_parts)})"
            pi_groups.append(
                {
                    "index": idx,
                    "symbol": f"Pi_{idx}",
                    "exponents": pairs,
                    "expression": expr_s,
                    "verified_dimension": "1 (Boyutsuz)",
                }
            )

        target_identifiable = bool(
            target_orig_idx is not None and any(v[target_orig_idx] != 0 for v in raw_vectors)
        )
        return {
            "num_variables": n,
            "dimensional_rank": rank,
            "num_pi_groups": n - rank,
            "repeating_basis_variables": repeating_basis,
            "pi_groups": pi_groups,
            "target_variable": target_var,
            "target_identifiable": target_identifiable,
        }


class MultiObjectivePareto:
    """Kesin kısmi sıra (strict partial order) üzerinde NSGA-II baskılanmamış cephe ve Skyline sentezleyicisi."""

    @staticmethod
    def dominates(canon_a: Sequence[float], canon_b: Sequence[float]) -> bool:
        if len(canon_a) != len(canon_b) or not canon_a:
            return False
        strictly_better = False
        for va, vb in zip(canon_a, canon_b):
            if not (math.isfinite(va) and math.isfinite(vb)):
                return False
            if va > vb:
                return False
            if va < vb:
                strictly_better = True
        return strictly_better

    @classmethod
    def analyze(
        cls,
        directions: Sequence[str],
        candidates: Sequence[Tuple[str, Sequence[float]]],
    ) -> Dict[str, Any]:
        if not directions:
            raise ValueError("En az bir amaç yönü ('min'/'max') gereklidir!")
        signs = [1.0 if d.lower().startswith("min") else -1.0 for d in directions]
        n = len(candidates)
        if n == 0:
            return {"skyline_ids": [], "fronts": [], "knee_id": None}

        canon = [[s * float(v) for s, v in zip(signs, vals)] for _, vals in candidates]
        dom_sets: List[List[int]] = [[] for _ in range(n)]
        dom_counts = [0] * n

        for p in range(n):
            for q in range(p + 1, n):
                if cls.dominates(canon[p], canon[q]):
                    dom_sets[p].append(q)
                    dom_counts[q] += 1
                elif cls.dominates(canon[q], canon[p]):
                    dom_sets[q].append(p)
                    dom_counts[p] += 1

        fronts: List[List[int]] = []
        curr = [p for p in range(n) if dom_counts[p] == 0]
        while curr:
            fronts.append(curr)
            nxt: List[int] = []
            for p in curr:
                for q in dom_sets[p]:
                    dom_counts[q] -= 1
                    if dom_counts[q] == 0:
                        nxt.append(q)
            curr = nxt

        f0 = fronts[0]
        m_obj = len(signs)
        utopia_c = [min(canon[p][m] for p in f0) for m in range(m_obj)]
        nadir_c = [max(canon[p][m] for p in f0) for m in range(m_obj)]

        def utopia_dist(p: int) -> float:
            acc = 0.0
            for m in range(m_obj):
                span = max(nadir_c[m] - utopia_c[m], 1e-12)
                d = (canon[p][m] - utopia_c[m]) / span
                acc += d * d
            return math.sqrt(acc)

        knee_idx = min(f0, key=lambda p: (utopia_dist(p), p))
        return {
            "skyline_ids": [candidates[p][0] for p in f0],
            "fronts": [[candidates[p][0] for p in fr] for fr in fronts],
            "knee_id": candidates[knee_idx][0],
            "utopia_point": [s * u for s, u in zip(signs, utopia_c)],
            "nadir_point": [s * nd for s, nd in zip(signs, nadir_c)],
        }


__all__ = ["BuckinghamPiDiscovery", "MultiObjectivePareto", "DIMENSIONS"]
