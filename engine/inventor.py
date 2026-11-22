# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# Matematik Çekirdeği — Sembolik İcat ve Keşif Motoru (Symbolic Invention Engine)

"""
Çok Disiplinli İcat Motoru (Cross-Domain Discovery Engine).
Fizik yasalarını birleştirir, sembolik sadeleştirme (Sympy) yapar,
boyut analiziyle denetler ve hedeflenen problem için yeni tasarım
yolları ve türetilmiş formüller üretir.
"""

from typing import List, Dict, Set, Any, Optional
import sympy as sp
from .knowledge_graph import KnowledgeGraph
from .dimensions import Dimension, check_dimensional_consistency, infer_sympy_dimension
from .constants import PhysicalConstants

class InventionCandidate:
    """Üretilen bir icat veya türetilmiş fiziksel tasarım adayı."""

    def __init__(
        self,
        name: str,
        goal: str,
        path: List[str],
        symbolic_expr: sp.Expr,
        free_symbols: List[str],
        dimensional_status: bool,
        notes: str = ""
    ):
        self.name = name
        self.goal = goal
        self.path = path
        self.symbolic_expr = symbolic_expr
        self.free_symbols = free_symbols
        self.dimensional_status = dimensional_status
        self.notes = notes

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "goal": self.goal,
            "chain": self.path,
            "equation": f"{self.goal} = {str(self.symbolic_expr)}",
            "latex": f"{self.goal} = {sp.latex(self.symbolic_expr)}",
            "parameters": self.free_symbols,
            "dimensional_verification": self.dimensional_status,
            "notes": self.notes,
        }

    def evaluate(self, values: Dict[str, float]) -> Optional[float]:
        """Verilen sayısal parametrelerle formülü hesaplar."""
        try:
            subs = {sp.Symbol(k): v for k, v in values.items()}
            res = self.symbolic_expr.evalf(subs=subs)
            return float(res)
        except Exception:
            return None


class InventionEngine:
    """Kural tabanlı ve sembolik icat sentezleyici."""

    def __init__(self, knowledge_graph: KnowledgeGraph):
        self.kg = knowledge_graph

    def synthesize(
        self,
        target_goal: str,
        available_inputs: Set[str],
        max_depth: int = 4
    ) -> List[InventionCandidate]:
        """
        Hedef bir nicelik (örn: 'delta_v', 'F', 'P_beam') ve elde bulunan
        kaynaklar/değişkenler için tüm olası türetim yollarını sentezler.
        """
        paths = self.kg.search_paths_to_target(available_inputs, target_goal, max_depth=max_depth)
        candidates: List[InventionCandidate] = []

        seen_signatures = set()

        local_symbols = {name: sp.Symbol(name) for name in self.kg.variable_dimensions}

        for path in paths:
            # Denklemleri adım adım sembolik olarak birleştir
            substitutions = {}
            eq_names = []

            for eq_id in path:
                law = self.kg.laws[eq_id]
                eq_names.append(law.get("name", eq_id))

                # Denklem metnini ayrıştır (sağ taraf)
                eq_str = law["equation"]
                lhs_str, rhs_str = eq_str.split("=")
                lhs_var = lhs_str.strip()
                rhs_expr = sp.sympify(rhs_str.strip(), locals=local_symbols, evaluate=False)

                # Önceki ara değişkenleri yerine koy
                substituted_rhs = rhs_expr.subs(substitutions)
                substitutions[sp.Symbol(lhs_var)] = substituted_rhs

            # Son hedef formülü
            goal_symbol = sp.Symbol(target_goal)
            final_expr = substitutions.get(goal_symbol)

            if final_expr is not None:
                final_expr = sp.simplify(final_expr)
                sig = str(final_expr)
                if sig in seen_signatures:
                    continue
                seen_signatures.add(sig)

                free_vars = [str(s) for s in final_expr.free_symbols]

                # Gerçek Sembolik AST Boyutsal Tutarlılık Kontrolü: dim(LHS) == dim(RHS)
                target_dim = self.kg.variable_dimensions.get(target_goal)
                if target_dim is None:
                    continue
                try:
                    inferred_rhs_dim = infer_sympy_dimension(final_expr, self.kg.variable_dimensions)
                    dim_status, _ = check_dimensional_consistency(target_dim, inferred_rhs_dim)
                except Exception:
                    dim_status = False

                if not dim_status:
                    continue

                candidate = InventionCandidate(
                    name=f"Tasarım: {' ➔ '.join(eq_names)}",
                    goal=target_goal,
                    path=path,
                    symbolic_expr=final_expr,
                    free_symbols=free_vars,
                    dimensional_status=dim_status,
                    notes=f"{len(path)} adımlı fiziksel çaprazlama ile elde edildi."
                )
                candidates.append(candidate)

        return candidates

    def analyze_space_launch_alternatives(self) -> Dict[str, Any]:
        """
        Uzaya düşük maliyetli çıkış (Low-Cost Space Launch) için kanonik
        fiziksel alternatiflerin karşılaştırmalı icat analizi.
        """
        # Hedef: LEO Yörünge Hızı (7.800 m/s) ve Kaçış Hızı (11.186 m/s)
        results = {
            "problem": "Uzaya Çıkış Maliyetini Düşürme (Escape Gravity Well)",
            "required_leo_delta_v_ms": 7800.0,
            "alternatives": []
        }

        # 1. Kimyasal Roket (Tsiolkovsky) Kısıtı
        # Isp = 320 s (kerosen/LOX), Isp = 450 s (hidrojen/LOX)
        v_e_kerosene = PhysicalConstants.g0 * 320.0  # ~3138 m/s
        mass_ratio_chem = sp.exp(7800.0 / v_e_kerosene).evalf()
        results["alternatives"].append({
            "name": "1. Geleneksel Kimyasal Roket (Tsiolkovsky Tiranlığı)",
            "exhaust_velocity_ms": float(v_e_kerosene),
            "payload_fraction_percent": float((1.0 / mass_ratio_chem) * 100),
            "fuel_mass_ratio": float(mass_ratio_chem),
            "verdict": "Roketin %92'si sadece yakıttır; kütle çarpanı nedeniyle masraflıdır."
        })

        # 2. Işınlamalı Termal İtki (Beamed Thermal Propulsion - Laser/Microwave)
        # Isp = 900 s (saf hidrojen ısıtma), yakıt yanmaz, enerji yerden ışınlanır
        v_e_beamed = PhysicalConstants.g0 * 900.0   # ~8826 m/s
        mass_ratio_beamed = sp.exp(7800.0 / v_e_beamed).evalf()
        results["alternatives"].append({
            "name": "2. Yerden Lazer/Mikrodalga Işınlamalı Termal Tahrik (Beamed Energy)",
            "exhaust_velocity_ms": float(v_e_beamed),
            "payload_fraction_percent": float((1.0 / mass_ratio_beamed) * 100),
            "fuel_mass_ratio": float(mass_ratio_beamed),
            "verdict": "Roket yakıt motoru taşımaz, sadece hidrojen taşır; kütle fraksiyonu %41'e çıkar!"
        })

        # 3. Kinetik Döner Fırlatıcı (SpinLaunch) + Üst Aşama Hibriti
        # v = 2000 m/s kinetik çıkış, kalan 5800 m/s roketle
        mass_ratio_hybrid = sp.exp((7800.0 - 2000.0) / v_e_kerosene).evalf()
        results["alternatives"].append({
            "name": "3. Kinetik Döner/Manyetik Hibrit Fırlatıcı (Mass Driver + Roket)",
            "ground_launch_velocity_ms": 2000.0,
            "remaining_delta_v_ms": 5800.0,
            "fuel_mass_ratio": float(mass_ratio_hybrid),
            "payload_fraction_percent": float((1.0 / mass_ratio_hybrid) * 100),
            "verdict": "İlk aşama sıfır yakıtla (elektrikle) aşılır, roket boyutu %60 küçülür."
        })

        # 4. Gökyüzü Kancası (Momentum Skyhook Tether)
        # 1000 km kablo, v_tip = 1500 m/s
        results["alternatives"].append({
            "name": "4. Dönen Yörünge Kancası (Skyhook Momentum Tether)",
            "tether_tip_velocity_ms": 1500.0,
            "breaking_length_carbon_km": 1000.0,
            "verdict": "Yörüngedeki dönen kablo atmosferin üstünden aracı yakalayıp savurur."
        })

        return results
