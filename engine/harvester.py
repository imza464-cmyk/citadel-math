# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# Matematik Çekirdeği — Otomatize LaTeX Denklemi Hasat ve Eleme Motoru (LaTeX Harvester & Triage)

"""
İnternet kaynaklarından (Wikipedia, arXiv, açık bilim arşivleri) ve yerel belgelerden
LaTeX formatındaki matematiksel/fiziksel denklemleri otomatik ayıklayan,
SymPy AST ağacına dönüştüren ve Karşılaştırmalı Aday Havuzuna (Staging) aktaran motor.

AGENTS.md Değişmez 3 (Çift Katmanlı Egemenlik) uyarınca:
Taranan denklemler doğrudan çekirdeğe girmez; 'staging/harvested_equations.json'
içinde ham aday olarak depolanır ve boyut + rejim denetiminden sonra terfi edilir.
"""

import json
import os
import re
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, Dict, List, Optional, Tuple
import sympy as sp
from sympy.parsing.latex import parse_latex

STAGING_DIR = os.path.join(os.path.dirname(os.path.dirname(__file__)), "staging")


class LatexEquationHarvester:
    """LaTeX Denklemi Hasat, Normalizasyon ve Eleme (Triage) Motoru."""

    def __init__(self, staging_dir: str = STAGING_DIR):
        self.staging_dir = staging_dir
        os.makedirs(self.staging_dir, exist_ok=True)
        self.raw_store_path = os.path.join(self.staging_dir, "harvested_equations.json")
        self.triage_report_path = os.path.join(self.staging_dir, "triage_summary.json")

    def extract_and_clean_latex(self, raw_latex: str) -> Tuple[str, str]:
        """
        LaTeX bloğunu temizler, '\\text{...}' / '\\mathrm{...}' alt-indislerini ve '\\dot', '\\Delta'
        makrolarını kanonik sembol adlarına dönüştürür, '\\qquad' yan koşullarını ayırır.
        Döndürür: (temiz_denklem, yan_kosul_metni)
        """
        s = raw_latex.strip()
        if s.startswith("$$") and s.endswith("$$"):
            s = s[2:-2].strip()
        elif s.startswith("$") and s.endswith("$"):
            s = s[1:-1].strip()

        s = re.sub(r"\\begin\{[a-zA-Z*]+\}", "", s)
        s = re.sub(r"\\end\{[a-zA-Z*]+\}", "", s)
        s = re.sub(r"\\tag\{.*?\}", "", s)
        s = re.sub(r"\\label\{.*?\}", "", s)
        s = s.replace(r"\displaystyle", "").replace(r"\textstyle", "")
        s = s.replace(r"\left", "").replace(r"\right", "")

        # \qquad veya ', \text{where}' ile ayrılmış yan koşulları (örn. b \ne 0) denklemden ayır
        side_cond = ""
        cond_split = re.split(r"\\qquad|\\quad,\s*|,\s*\\text\{(?:where|for|if)\b", s, maxsplit=1)
        if len(cond_split) == 2:
            s = cond_split[0].strip()
            side_cond = cond_split[1].strip().rstrip(".,;")

        # Alt-indis içindeki \text{...}, \mathrm{...}, \mathit{...} bloklarını tekil tanımlayıcıya çevir
        def _clean_sub_text(match: re.Match) -> str:
            base = match.group(1)
            label = re.sub(r"[^a-zA-Z0-9_]", "", match.group(2))
            return f"{base}_{{{label}}}" if label else base

        s = re.sub(
            r"([A-Za-z0-9\}])\s*_\s*\{?\s*\\(?:text|mathrm|mathit|textrm|mathsf)\{([^}]*)\}\s*\}?",
            _clean_sub_text,
            s,
        )
        # Bağımsız \text{const} gibi kelimeleri de harf çarpımına dönüşmemesi için temizle
        s = re.sub(
            r"\\(?:text|mathrm|operatorname)\{([A-Za-z][A-Za-z0-9_]*)\}",
            lambda m: m.group(1),
            s,
        )

        # Zaman türevi nokta notasyonu: \dot{m} / \dot m -> m_dot, \ddot{x} -> x_ddot
        s = re.sub(r"\\ddot\s*\{?\s*([A-Za-z])\s*\}?", r"\1_{ddot}", s)
        s = re.sub(r"\\dot\s*\{?\s*([A-Za-z])\s*\}?", r"\1_{dot}", s)

        # \Delta v -> delta_v (ayrık çarpım Delta * v yerine tekil fark sembolü)
        s = re.sub(r"\\Delta\s*([A-Za-z])_\{([^}]+)\}", r"delta_\1_\2", s)
        s = re.sub(r"\\Delta\s*([A-Za-z])\b", r"delta_\1", s)

        # İnce boşluk makrolarını ve sonda kalan noktalama işaretlerini kaldır
        s = re.sub(r"\\[,;!]", " ", s)
        s = s.strip().rstrip(".,;")
        return s, side_cond

    def clean_latex_string(self, raw_latex: str) -> str:
        """Geriye dönük uyumlu temiz LaTeX döndürücü."""
        cleaned, _ = self.extract_and_clean_latex(raw_latex)
        return cleaned

    _GREEK_SUB_RE = re.compile(
        r"\\(alpha|beta|gamma|delta|epsilon|varepsilon|zeta|eta|theta|vartheta|"
        r"iota|kappa|lambda|mu|nu|xi|pi|rho|sigma|tau|upsilon|phi|varphi|chi|psi|omega|"
        r"Gamma|Delta|Theta|Lambda|Xi|Pi|Sigma|Phi|Psi|Omega)"
        r"\s*_\s*(?:\{([A-Za-z0-9_]+)\}|([A-Za-z0-9]))"
    )
    _ASCII_SUB_RE = re.compile(
        r"(?<![\\A-Za-z0-9_])([A-Za-z])\s*_\s*(?:\{([A-Za-z0-9_]+)\}|([A-Za-z0-9]))"
    )
    _DELTA_ID_RE = re.compile(r"\b(delta_[A-Za-z0-9_]+)\b")

    def _protect_compound_symbols(
        self, expr_str: str, canon_to_idx: Dict[str, int]
    ) -> str:
        """
        Alt-indisli ve çok-harfli kanonik sembolleri (örn. F_{thrust}, I_{sp}, g_0, \\rho_0, delta_v)
        SymPy parse_latex'in harf çarpımına (I_{p*s}) parçalamasını önlemek için atomik yer tutuculara sarar.
        """
        def _alloc_placeholder(canon_name: str) -> str:
            if canon_name not in canon_to_idx:
                canon_to_idx[canon_name] = 9000 + len(canon_to_idx)
            idx = canon_to_idx[canon_name]
            return f" z_{{{idx}}} "

        out = self._DELTA_ID_RE.sub(lambda m: _alloc_placeholder(m.group(1)), expr_str)
        out = self._GREEK_SUB_RE.sub(
            lambda m: _alloc_placeholder(f"{m.group(1)}_{m.group(2) or m.group(3)}"),
            out,
        )
        out = self._ASCII_SUB_RE.sub(
            lambda m: _alloc_placeholder(f"{m.group(1)}_{m.group(2) or m.group(3)}"),
            out,
        )
        return out

    def parse_equation(self, latex_str: str) -> Optional[Dict[str, Any]]:
        """
        Bir LaTeX denklem metnini SymPy AST'ye çevirir.
        LHS (hedef), RHS (girdiler) ve varsa ham yan koşulları ayrıştırır.
        """
        cleaned, side_cond = self.extract_and_clean_latex(latex_str)
        if not cleaned or "=" not in cleaned:
            return None

        parts = cleaned.split("=")
        if len(parts) != 2:
            return None

        lhs_raw = parts[0].strip().rstrip(",")
        rhs_raw = parts[1].strip().rstrip(",")
        if not lhs_raw or not rhs_raw:
            return None

        canon_to_idx: Dict[str, int] = {}
        lhs_protected = self._protect_compound_symbols(lhs_raw, canon_to_idx)
        rhs_protected = self._protect_compound_symbols(rhs_raw, canon_to_idx)

        try:
            lhs_expr = parse_latex(lhs_protected)
            rhs_expr = parse_latex(rhs_protected)
        except (SyntaxError, ValueError, TypeError, AttributeError, Exception):
            # LaTeX parser harici kütüphane hataları aday denklemi reddeder (Değişmez 3)
            return None

        if canon_to_idx:
            restore_map = {
                sp.Symbol(f"z_{{{idx}}}"): sp.Symbol(canon_name)
                for canon_name, idx in canon_to_idx.items()
            }
            lhs_expr = lhs_expr.xreplace(restore_map)
            rhs_expr = rhs_expr.xreplace(restore_map)

        lhs_symbols = sorted(str(s) for s in lhs_expr.free_symbols)
        rhs_symbols = sorted(str(s) for s in rhs_expr.free_symbols)
        if not lhs_symbols:
            return None

        target_var = str(lhs_expr)
        if not target_var.isidentifier():
            target_var = lhs_symbols[0]

        rec: Dict[str, Any] = {
            "raw_latex": cleaned,
            "parsed_eq": f"{lhs_expr} = {rhs_expr}",
            "target": target_var,
            "inputs": rhs_symbols,
            "sympy_lhs": str(lhs_expr),
            "sympy_rhs": str(rhs_expr),
            "status": "pending_verification",
        }
        if side_cond:
            rec["raw_side_condition"] = side_cond
        return rec

    def harvest_from_text(self, text: str, source_label: str = "local_doc") -> List[Dict[str, Any]]:
        """Bir metin veya Markdown belgesi içindeki tüm $$...$$ ve $...$ denklemlerini tarar."""
        harvested = []

        display_matches = re.findall(r"\$\$(.*?)\$\$", text, re.DOTALL)
        for raw in display_matches:
            parsed = self.parse_equation(raw)
            if parsed:
                parsed["source"] = source_label
                parsed["type"] = "display_math"
                harvested.append(parsed)

        inline_matches = re.findall(r"(?<!\$)\$(?!\$)(.*?)(?<!\$)\$(?!\$)", text)
        for raw in inline_matches:
            if "=" in raw:
                parsed = self.parse_equation(raw)
                if parsed:
                    parsed["source"] = source_label
                    parsed["type"] = "inline_math"
                    harvested.append(parsed)

        return harvested

    def harvest_from_file(self, file_path: str) -> List[Dict[str, Any]]:
        """Yerel bir dosyayı (örn. formuller_ve_denklemler.md) baştan sona tarar."""
        if not os.path.exists(file_path):
            return []

        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()

        base_name = os.path.basename(file_path)
        print(f"[+] Yerel dosya taranıyor: {base_name} ({len(content)} bayt)...")
        results = self.harvest_from_text(content, source_label=base_name)
        print(f"[✓] {len(results)} adet doğrulanabilir denklem başarıyla ayrıştırıldı.")
        return results

    def harvest_from_wikipedia(self, article_title: str) -> List[Dict[str, Any]]:
        """Wikipedia API üzerinden makalenin kaynak wikitext'ini çeker ve <math> etiketlerini hasat eder."""
        encoded_title = urllib.parse.quote(article_title.replace(" ", "_"))
        url = f"https://en.wikipedia.org/w/api.php?action=parse&page={encoded_title}&prop=wikitext&format=json"

        print(f"[+] Wikipedia taranıyor: {article_title}...")
        req = urllib.request.Request(url, headers={"User-Agent": "MatematikCekirdegiHarvester/1.1"})

        try:
            with urllib.request.urlopen(req, timeout=12) as resp:
                data = json.loads(resp.read().decode("utf-8"))
        except (urllib.error.URLError, TimeoutError, OSError, json.JSONDecodeError) as e:
            print(f"[-] Wikipedia bağlantı hatası ({article_title}): {e}")
            return []

        wikitext = data.get("parse", {}).get("wikitext", {}).get("*", "")
        if not wikitext:
            print(f"[-] Wikitext bulunamadı: {article_title}")
            return []

        math_tags = re.findall(r"<math(?:.*?)>(.*?)</math>", wikitext, re.DOTALL | re.IGNORECASE)
        print(f"    • {len(math_tags)} adet <math> etiketi tespit edildi. Ayrıştırılıyor...")

        harvested = []
        for raw_math in math_tags:
            parsed = self.parse_equation(raw_math)
            if parsed:
                parsed["source"] = f"wikipedia:{article_title}"
                parsed["type"] = "wiki_math"
                harvested.append(parsed)

        print(f"[✓] {len(harvested)} adet sembolik denklem başarıyla AST'ye dönüştürüldü.")
        return harvested

    def harvest_batch_topics(self, topics: List[str]) -> List[Dict[str, Any]]:
        """Toplu konu taraması yapar."""
        all_results = []
        for t in topics:
            all_results.extend(self.harvest_from_wikipedia(t))
        return all_results

    def save_to_staging(self, new_records: List[Dict[str, Any]]):
        """Taranan denklemleri ham aday havuzuna kaydeder (AGENTS.md Değişmez 3)."""
        existing: List[Dict[str, Any]] = []
        if os.path.exists(self.raw_store_path):
            with open(self.raw_store_path, "r", encoding="utf-8") as f:
                existing = json.load(f)

        seen_raw = {x.get("raw_latex") for x in existing}
        added_count = 0

        for r in new_records:
            if r.get("raw_latex") not in seen_raw:
                existing.append(r)
                seen_raw.add(r.get("raw_latex"))
                added_count += 1

        with open(self.raw_store_path, "w", encoding="utf-8") as f:
            json.dump(existing, f, indent=2, ensure_ascii=False)

        print(f"[✓] Staging Havuzu Güncellendi: +{added_count} yeni denklem eklendi. (Toplam: {len(existing)})")

    def triage_staging_pool(self, canonical_db_path: str) -> Dict[str, Any]:
        """
        Staging havuzundaki tüm ham aday denklemleri AGENTS.md Değişmez 3 uyarınca
        4 deterministik mühendislik kategorisine ayırır ve 'staging/triage_summary.json' üretir:
          1) promoted_or_covered_physics: Kanonik veritabanında karşılığı doğrulanmış fizik/sistem yasaları
          2) pure_algebraic_ring_axioms : PolyZ / MultiPolyZ / GuardedPoly ispat çekirdeği kapsamındaki saf cebir özdeşlikleri
          3) differential_integral_forms: Türev / İntegral / Limit / Toplam operatörü içeren analitik formlar
          4) quarantined_staging        : Eksik boyut bağlamı veya LaTeX artefaktı nedeniyle karantinada tutulan adaylar
        """
        if not os.path.exists(self.raw_store_path):
            raise FileNotFoundError(f"Staging dosyası bulunamadı: {self.raw_store_path}")

        with open(self.raw_store_path, "r", encoding="utf-8") as f:
            staged = json.load(f)

        canonical_laws = []
        if os.path.exists(canonical_db_path):
            with open(canonical_db_path, "r", encoding="utf-8") as f:
                canonical_laws = json.load(f)

        canonical_targets = {l["target"] for l in canonical_laws}
        canonical_sources = {l["domain"] for l in canonical_laws}

        counts = {
            "promoted_or_covered_physics": 0,
            "pure_algebraic_ring_axioms": 0,
            "differential_integral_forms": 0,
            "quarantined_staging": 0,
        }
        by_source: Dict[str, int] = {}

        calculus_markers = ("Derivative(", "Integral(", "Limit(", "Sum(", "\\int", "\\frac{d", "\\partial", "\\nabla")
        mangled_markers = ("_{text}", "_{mathrm}", "text*", "mathrm*")

        for rec in staged:
            src = rec.get("source", "unknown")
            by_source[src] = by_source.get(src, 0) + 1
            raw_ltx = rec.get("raw_latex", "")
            parsed_eq = rec.get("parsed_eq", "")
            sym_lhs = rec.get("sympy_lhs", "")
            target = rec.get("target", "")

            if any(m in parsed_eq or m in raw_ltx for m in mangled_markers):
                # Yeni temizleyici ile yeniden kurtarılabilir mi denetle
                reparsed = self.parse_equation(raw_ltx)
                if reparsed is None or any(m in reparsed.get("parsed_eq", "") for m in mangled_markers):
                    rec["triage_category"] = "quarantined_staging"
                    counts["quarantined_staging"] += 1
                    continue
                target = reparsed["target"]
                sym_lhs = reparsed["sympy_lhs"]
                parsed_eq = reparsed["parsed_eq"]

            if any(cm in parsed_eq or cm in raw_ltx for cm in calculus_markers):
                rec["triage_category"] = "differential_integral_forms"
                counts["differential_integral_forms"] += 1
            elif src == "formuller_ve_denklemler.md" and target in {"a", "x", "y", "z", "n", "S", "i", "e"}:
                rec["triage_category"] = "pure_algebraic_ring_axioms"
                counts["pure_algebraic_ring_axioms"] += 1
            elif sym_lhs.isidentifier() and (target in canonical_targets or src.startswith("wikipedia:")):
                rec["triage_category"] = "promoted_or_covered_physics"
                counts["promoted_or_covered_physics"] += 1
            else:
                rec["triage_category"] = "quarantined_staging"
                counts["quarantined_staging"] += 1

        summary = {
            "total_staging_candidates": len(staged),
            "canonical_core_laws": len(canonical_laws),
            "canonical_domains": sorted(canonical_sources),
            "triage_breakdown": counts,
            "sources_breakdown": by_source,
        }
        with open(self.triage_report_path, "w", encoding="utf-8") as f:
            json.dump(summary, f, indent=2, ensure_ascii=False)
        return summary
