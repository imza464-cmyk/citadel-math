# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
# Matematik Çekirdeği — Çevrimiçi Veri ve Formül Çekici (Online Knowledge Ingestion)

"""
İnternet üzerinden NIST, bilimsel açık veri tabanları ve kanonik fizik kaynaklarından
formül, birim ve sabitleri otomatik indiren ve sisteme entegre eden modül.
"""

import os
import json
import re
import urllib.request
from typing import Dict, List, Any, Optional

NIST_ASCII_URL = "https://physics.nist.gov/cuu/Constants/Table/allascii.txt"

class OnlineKnowledgeFetcher:
    """Çevrimiçi fizik ve matematiksel bilgi çekici."""

    def __init__(self, data_dir: str):
        self.data_dir = data_dir
        os.makedirs(data_dir, exist_ok=True)

    def fetch_nist_constants(self) -> Dict[str, Any]:
        """
        NIST CODATA resmi sunucusundan (physics.nist.gov)
        tüm temel fiziksel sabitleri indirir ve yapılandırılmış JSON'a çevirir.
        """
        print(f"[+] NIST CODATA sabitleri indiriliyor: {NIST_ASCII_URL}...")
        req = urllib.request.Request(NIST_ASCII_URL, headers={"User-Agent": "MatematikCekirdegi/1.0"})

        try:
            with urllib.request.urlopen(req, timeout=15) as resp:
                raw_text = resp.read().decode("utf-8", errors="ignore")
        except Exception as e:
            print(f"[-] NIST indirme hatası: {e}")
            return {}

        constants = {}
        lines = raw_text.splitlines()

        # allascii.txt sabit kolon formatı:
        # Quantity (0..59), Value (60..84), Uncertainty (85..109), Unit (110..)
        for line in lines[10:]:
            if len(line) < 65 or line.startswith("---"):
                continue

            name = line[0:59].strip()
            val_str = line[60:85].strip().replace(" ", "").replace("...", "")
            unit = line[110:].strip() if len(line) > 110 else ""

            if not name or not val_str:
                continue

            try:
                # 'e-xx' ve bilimsel gösterimleri ayrıştır
                val = float(val_str)
                constants[name] = {
                    "value": val,
                    "unit": unit,
                    "raw_str": val_str
                }
            except ValueError:
                continue

        out_path = os.path.join(self.data_dir, "nist_constants.json")
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(constants, f, indent=2, ensure_ascii=False)

        print(f"[✓] {len(constants)} adet NIST sabiti başarıyla kaydedildi: {out_path}")
        return constants

    def fetch_wikipedia_physics_summary(self, topics: List[str]) -> Dict[str, Any]:
        """
        Wikipedia REST API üzerinden fizik ve uzay tahrik konularının
        resmi özetlerini ve denklemlerini çeker.
        """
        results = {}
        for topic in topics:
            clean_topic = topic.replace(" ", "_")
            url = f"https://en.wikipedia.org/api/rest_v1/page/summary/{clean_topic}"
            print(f"[+] Çevrimiçi veri çekiliyor: {topic}...")
            req = urllib.request.Request(url, headers={"User-Agent": "MatematikCekirdegi/1.0"})
            try:
                with urllib.request.urlopen(req, timeout=10) as resp:
                    data = json.loads(resp.read().decode("utf-8"))
                    results[topic] = {
                        "title": data.get("title"),
                        "extract": data.get("extract"),
                        "description": data.get("description"),
                        "page_url": data.get("content_urls", {}).get("desktop", {}).get("page")
                    }
            except Exception as e:
                print(f"[-] {topic} çekilemedi: {e}")

        out_path = os.path.join(self.data_dir, "online_physics_topics.json")
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(results, f, indent=2, ensure_ascii=False)

        print(f"[✓] {len(results)} konu kaydedildi: {out_path}")
        return results

    def fetch_all(self) -> Dict[str, Any]:
        """Tüm çevrimiçi veri kaynaklarını senkronize eder."""
        nist = self.fetch_nist_constants()

        space_topics = [
            "Tsiolkovsky_rocket_equation",
            "Beamed_energy_propulsion",
            "Mass_driver",
            "Skyhook_(structure)",
            "Specific_impulse",
            "Oberth_effect"
        ]
        wiki = self.fetch_wikipedia_physics_summary(space_topics)

        return {
            "nist_count": len(nist),
            "wiki_count": len(wiki)
        }
