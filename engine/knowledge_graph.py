# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# Matematik Çekirdeği — Evrensel Bilgi, Fiziksel Rejim (DomainBounds) ve Yasalar Hipergrafı

"""
NetworkX tabanlı İki Parçalı (Bipartite) Fizik ve Matematik Bağıntı Hipergrafı.
Değişkenler ve Denklemler arasındaki rejim-uyumlu AND-OR türetim ağaçlarını ve
Faz 1.3 fiziksel geçerlilik bölgelerini (DomainBounds) yönetir.
"""

import json
import os
from typing import Dict, List, Optional, Set, Tuple, Any
import networkx as nx
from .dimensions import Dimension, DIMENSIONS


def are_regimes_compatible(r1: str, r2: str) -> bool:
    """İki fiziksel rejim etiketinin birbiriyle bağdaşıp bağdaşmadığını denetler."""
    if not r1 or not r2 or r1 == "universal" or r2 == "universal" or r1 == r2:
        return True

    flow_subsonic = {"incompressible_subsonic", "compressible_subsonic"}
    flow_high_mach = {"supersonic", "hypersonic_continuum"}
    if (r1 in flow_subsonic and r2 in flow_high_mach) or (r2 in flow_subsonic and r1 in flow_high_mach):
        return False

    if {r1, r2} == {"supersonic", "hypersonic_continuum"}:
        return False

    continuum_regimes = {
        "incompressible_subsonic",
        "compressible_subsonic",
        "supersonic",
        "hypersonic_continuum",
        "continuum_ideal_gas",
        "laminar_continuum",
    }
    if (r1 in continuum_regimes and r2 == "free_molecular_rarefied") or (
        r2 in continuum_regimes and r1 == "free_molecular_rarefied"
    ):
        return False

    if {r1, r2} == {"classical_nonrelativistic", "ultrarelativistic"}:
        return False

    clinical_regimes = {"clinical_outcome_scoring", "spine_biomechanics_quasistatic"}
    extreme_physics_regimes = {
        "hypersonic_continuum",
        "supersonic",
        "relativistic",
        "ultrarelativistic",
        "nuclear_relativistic",
        "free_molecular_rarefied",
    }
    if (r1 in clinical_regimes and r2 in extreme_physics_regimes) or (
        r2 in clinical_regimes and r1 in extreme_physics_regimes
    ):
        return False

    return True


def intersect_var_bounds(b1: Dict[str, Any], b2: Dict[str, Any], var_name: str) -> Tuple[bool, Dict[str, Any], str]:
    """İki değişken geçerlilik aralığının kesişimini hesaplar; boş küme ise (False, {}, hata) döndürür."""
    m1, m2 = b1.get("min"), b2.get("min")
    inc_min1 = bool(b1.get("min_inclusive", True))
    inc_min2 = bool(b2.get("min_inclusive", True))

    if m1 is not None and m2 is not None:
        if m1 > m2:
            new_min, new_min_inc = float(m1), inc_min1
        elif m2 > m1:
            new_min, new_min_inc = float(m2), inc_min2
        else:
            new_min, new_min_inc = float(m1), (inc_min1 and inc_min2)
    elif m1 is not None:
        new_min, new_min_inc = float(m1), inc_min1
    elif m2 is not None:
        new_min, new_min_inc = float(m2), inc_min2
    else:
        new_min, new_min_inc = None, True

    x1, x2 = b1.get("max"), b2.get("max")
    inc_max1 = bool(b1.get("max_inclusive", True))
    inc_max2 = bool(b2.get("max_inclusive", True))

    if x1 is not None and x2 is not None:
        if x1 < x2:
            new_max, new_max_inc = float(x1), inc_max1
        elif x2 < x1:
            new_max, new_max_inc = float(x2), inc_max2
        else:
            new_max, new_max_inc = float(x1), (inc_max1 and inc_max2)
    elif x1 is not None:
        new_max, new_max_inc = float(x1), inc_max1
    elif x2 is not None:
        new_max, new_max_inc = float(x2), inc_max2
    else:
        new_max, new_max_inc = None, True

    if new_min is not None and new_max is not None:
        if new_min > new_max or (abs(new_min - new_max) <= 1e-15 and (not new_min_inc or not new_max_inc)):
            return (
                False,
                {},
                f"Fiziksel geçerlilik bölgesi çakışması ({var_name}): [{m1}, {x1}] ∩ [{m2}, {x2}] = ∅",
            )

    n1 = b1.get("note", "")
    n2 = b2.get("note", "")
    merged_note = f"{n1}; {n2}" if (n1 and n2 and n1 != n2) else (n1 or n2)
    return (
        True,
        {
            "min": new_min,
            "max": new_max,
            "min_inclusive": new_min_inc,
            "max_inclusive": new_max_inc,
            "note": merged_note,
        },
        "",
    )


class KnowledgeGraph:
    """
    Fiziksel Kanunlar, Değişkenler, Geçerlilik Bölgeleri ve Korunum Kuralları Bilgi Grafı.
    Düğüm Tipleri:
      - 'variable': Fiziksel nicelik (örn: v_e, delta_v, F, m_0)
      - 'equation': Kanonik veya türetilmiş yasa (örn: law_tsiolkovsky)
    """

    def __init__(self):
        self.graph = nx.DiGraph()
        self.laws: Dict[str, Dict[str, Any]] = {}
        self.variable_dimensions: Dict[str, Dimension] = {}
        self.variable_first_law: Dict[str, str] = {}

    def add_variable(self, name: str, dim_name_or_obj: Any, description: str = "", law_id: str = ""):
        """Grafiğe değişken ekler ve SSOT sembol-boyut tekilliğini denetleyerek kaydeder."""
        if isinstance(dim_name_or_obj, str):
            if dim_name_or_obj not in DIMENSIONS:
                raise ValueError(f"Tanımsız boyut anahtarı ({law_id}): {name} -> {dim_name_or_obj}")
            dim = DIMENSIONS[dim_name_or_obj]
        elif isinstance(dim_name_or_obj, Dimension):
            dim = dim_name_or_obj
        else:
            dim = Dimension()

        if name in self.variable_dimensions and self.variable_dimensions[name] != dim:
            prev_law = self.variable_first_law.get(name, "?")
            raise ValueError(
                f"SSOT Ontoloji İhlali ({law_id}): Sembol '{name}' '{prev_law}' içinde "
                f"[{self.variable_dimensions[name]}] iken burada [{dim}] olarak bildirildi!"
            )

        self.variable_dimensions[name] = dim
        if name not in self.variable_first_law and law_id:
            self.variable_first_law[name] = law_id
        self.graph.add_node(name, node_type="variable", dimension=str(dim), desc=description)

    def add_law(self, law_data: Dict[str, Any]):
        """Kanonik bir fiziksel veya matematiksel denklemi grafiğe ekler."""
        law_id = law_data["id"]
        target = law_data["target"]
        inputs = law_data["inputs"]
        domain = law_data.get("domain", "general")
        regime = law_data.get("regime", "universal")

        self.laws[law_id] = law_data

        # Denklem Düğümü
        self.graph.add_node(
            law_id,
            node_type="equation",
            name=law_data.get("name", law_id),
            domain=domain,
            regime=regime,
            latex=law_data.get("latex", ""),
            equation=law_data.get("equation", ""),
            target=target,
            inputs=inputs,
        )

        # Hedef Değişken Düğümü ve Kenarı
        target_dim = law_data.get("dimensions", {}).get(target, "dimensionless")
        self.add_variable(target, target_dim, law_id=law_id)
        self.graph.add_edge(law_id, target, relation="produces")

        # Girdi Değişkenleri ve Kenarları
        for inp in inputs:
            inp_dim = law_data.get("dimensions", {}).get(inp, "dimensionless")
            self.add_variable(inp, inp_dim, law_id=law_id)
            self.graph.add_edge(inp, law_id, relation="requires")

    def validate_chain_bounds(self, plan: List[str]) -> Tuple[bool, List[str], Dict[str, Dict[str, Any]], str]:
        """Bir denklem zincirindeki tüm rejimlerin ve domain_bounds aralıklarının bağdaşlığını doğrular."""
        regimes: List[str] = []
        eff_bounds: Dict[str, Dict[str, Any]] = {}

        for eq_id in plan:
            law = self.laws.get(eq_id)
            if law is None:
                return False, [], {}, f"Tanımsız yasa ID: {eq_id}"
            r = law.get("regime", "universal") or "universal"
            for prev_r in regimes:
                if not are_regimes_compatible(prev_r, r):
                    return (
                        False,
                        [],
                        {},
                        f"Rejim çatışması ({eq_id}): '{prev_r}' ile '{r}' fiziksel olarak bağdaşmaz!",
                    )
            if r not in regimes:
                regimes.append(r)

            for var_name, bound in law.get("domain_bounds", {}).items():
                if var_name in eff_bounds:
                    ok, merged, err = intersect_var_bounds(eff_bounds[var_name], bound, var_name)
                    if not ok:
                        return False, [], {}, err
                    eff_bounds[var_name] = merged
                else:
                    eff_bounds[var_name] = dict(bound)

        return True, regimes, eff_bounds, ""

    def check_law_validity_bounds(self, law_id: str, bindings: Dict[str, float]) -> Tuple[bool, str]:
        """Verilen sayısal değerlerin bir yasanın domain_bounds sınırlarına uygunluğunu denetler."""
        law = self.laws.get(law_id)
        if law is None:
            return False, f"Yasa bulunamadı: {law_id}"
        regime = law.get("regime", "universal")
        for var_name, bound in law.get("domain_bounds", {}).items():
            if var_name not in bindings:
                continue
            val = float(bindings[var_name])
            lo = bound.get("min")
            hi = bound.get("max")
            inc_lo = bool(bound.get("min_inclusive", True))
            inc_hi = bool(bound.get("max_inclusive", True))
            note = bound.get("note", "")
            if lo is not None and (val < lo if inc_lo else val <= lo):
                op = ">=" if inc_lo else ">"
                return False, f"[{law_id} | rejim={regime}] {var_name}={val} alt sınırı ihlal ediyor ({var_name} {op} {lo}; {note})"
            if hi is not None and (val > hi if inc_hi else val >= hi):
                op = "<=" if inc_hi else "<"
                return False, f"[{law_id} | rejim={regime}] {var_name}={val} üst sınırı ihlal ediyor ({var_name} {op} {hi}; {note})"
        return True, f"[{law_id} | rejim={regime}] Tüm fiziksel geçerlilik sınırları sağlandı."

    def load_canonical_database(self, json_path: str):
        """Kanonik veritabanı dosyasını yükler."""
        if not os.path.exists(json_path):
            raise FileNotFoundError(f"Veritabanı bulunamadı: {json_path}")

        with open(json_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        for law in data:
            self.add_law(law)

    def find_producers(self, variable: str) -> List[str]:
        """Bir değişkeni hesaplayabilen denklemleri döndürür."""
        producers = []
        if variable not in self.graph:
            return producers
        for pred in self.graph.predecessors(variable):
            if self.graph.nodes[pred].get("node_type") == "equation":
                producers.append(pred)
        return producers

    def find_consumers(self, variable: str) -> List[str]:
        """Bir değişkeni girdi olarak kullanan denklemleri döndürür."""
        consumers = []
        if variable not in self.graph:
            return consumers
        for succ in self.graph.successors(variable):
            if self.graph.nodes[succ].get("node_type") == "equation":
                consumers.append(succ)
        return consumers

    def search_paths_to_target(
        self,
        available_inputs: Set[str],
        target_goal: str,
        max_depth: int = 4,
    ) -> List[List[str]]:
        """
        Eldeki girdilerden (available_inputs) hedef niceliğe (target_goal) ulaşan ve
        rejim/DomainBounds çelişkisi barındırmayan tüm geçerli AND-OR türetim zincirlerini bulur.
        """
        results: List[List[str]] = []

        def solve_conjunctive(
            goals: List[str],
            known: Set[str],
            plan: List[str],
            visiting_vars: Set[str],
            depth: int,
        ) -> List[Tuple[Set[str], List[str]]]:
            remaining = [g for g in goals if g not in known]
            if not remaining:
                ok_chain, _, _, _ = self.validate_chain_bounds(plan)
                return [(set(known), list(plan))] if ok_chain else []

            if depth > max_depth:
                return []

            current_goal = remaining[0]
            rest_goals = remaining[1:]

            if current_goal in visiting_vars:
                return []

            outcomes: List[Tuple[Set[str], List[str]]] = []
            next_visiting = visiting_vars | {current_goal}

            for eq_id in self.find_producers(current_goal):
                if eq_id in plan:
                    continue

                ok_cand, _, _, _ = self.validate_chain_bounds(plan + [eq_id])
                if not ok_cand:
                    continue

                eq_inputs = list(self.laws[eq_id]["inputs"])
                missing = [inp for inp in eq_inputs if inp not in known]

                if any(m in next_visiting for m in missing):
                    continue

                if not missing:
                    new_known = set(known)
                    new_known.add(current_goal)
                    new_plan = plan + [eq_id]
                    outcomes.extend(
                        solve_conjunctive(rest_goals, new_known, new_plan, visiting_vars, depth)
                    )
                else:
                    sub_solutions = solve_conjunctive(
                        missing, known, plan, next_visiting, depth + 1
                    )
                    for sub_known, sub_plan in sub_solutions:
                        if eq_id in sub_plan:
                            continue
                        merged_plan = sub_plan + [eq_id]
                        ok_merged, _, _, _ = self.validate_chain_bounds(merged_plan)
                        if not ok_merged:
                            continue
                        new_known = set(sub_known)
                        new_known.add(current_goal)
                        outcomes.extend(
                            solve_conjunctive(rest_goals, new_known, merged_plan, visiting_vars, depth)
                        )

            return outcomes

        solved = solve_conjunctive([target_goal], set(available_inputs), [], set(), 0)
        for _, p in solved:
            if p and p not in results:
                results.append(p)
        return results

    def summary(self) -> Dict[str, Any]:
        """Graf özet istatistiklerini verir."""
        vars_count = sum(1 for _, d in self.graph.nodes(data=True) if d.get("node_type") == "variable")
        eqs_count = sum(1 for _, d in self.graph.nodes(data=True) if d.get("node_type") == "equation")
        domains = sorted(set(d.get("domain") for _, d in self.graph.nodes(data=True) if "domain" in d))
        regimes = sorted(set(d.get("regime") for _, d in self.graph.nodes(data=True) if "regime" in d))
        return {
            "total_nodes": self.graph.number_of_nodes(),
            "total_edges": self.graph.number_of_edges(),
            "variables": vars_count,
            "equations": eqs_count,
            "domains": domains,
            "regimes": regimes,
        }
