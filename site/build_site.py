#!/usr/bin/env python3
# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
"""Build site/data/ssot_bundle.js directly from Citadel-Math SSOT JSON databases."""

import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SITE_DIR = os.path.dirname(os.path.abspath(__file__))
OUT_PATH = os.path.join(SITE_DIR, "data", "ssot_bundle.js")
INDEX_HTML_PATH = os.path.join(SITE_DIR, "index.html")
KWEB_GATE = r"C:\Users\serit\projelerim\KALEOS\scripts\kweb_gate.py"


def stamp_asset_hashes() -> dict:
    """Stamp content-hash ?v=<sha256_10> onto local CSS/JS references in site/index.html to prevent CDN max-age=600 desync."""
    hashes = {}
    for rel in ("styles.css", "data/ssot_bundle.js", "app.js"):
        p = os.path.join(SITE_DIR, *rel.split("/"))
        if os.path.isfile(p):
            with open(p, "rb") as f:
                hashes[rel] = hashlib.sha256(f.read()).hexdigest()[:10]
    if os.path.isfile(INDEX_HTML_PATH) and hashes:
        with open(INDEX_HTML_PATH, "r", encoding="utf-8") as f:
            html = f.read()
        updated = html
        for rel, digest in hashes.items():
            pattern = re.compile(r'(\./' + re.escape(rel) + r')(?:\?v=[0-9a-fA-F]+)?')
            updated = pattern.sub(rf"\1?v={digest}", updated)
        if updated != html:
            with open(INDEX_HTML_PATH, "w", encoding="utf-8") as f:
                f.write(updated)
    return hashes


def load_json(rel_path: str):
    full = os.path.join(ROOT_DIR, rel_path)
    with open(full, "r", encoding="utf-8") as f:
        return json.load(f)


def main() -> None:
    laws = load_json(os.path.join("database", "canonical_laws.json"))
    ontology = load_json(os.path.join("database", "ONTOLOGY.json"))
    materials = load_json(os.path.join("database", "materials.json"))
    nist_raw = load_json(os.path.join("data", "nist_constants.json"))

    constants = []
    for name, entry in sorted(nist_raw.items(), key=lambda kv: kv[0].lower()):
        constants.append(
            {
                "name": name,
                "value": entry.get("value"),
                "unit": entry.get("unit", ""),
                "raw_str": entry.get("raw_str", str(entry.get("value", ""))),
            }
        )

    domains = sorted({law.get("domain", "unknown") for law in laws})
    regimes = sorted({law.get("regime", "universal") for law in laws})
    categories = sorted({mat.get("category", "other") for mat in materials})

    bundle = {
        "meta": {
            "version": "0.2.2",
            "generated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "law_count": len(laws),
            "symbol_count": len(ontology),
            "material_count": len(materials),
            "constant_count": len(constants),
            "domain_count": len(domains),
            "regime_count": len(regimes),
            "domains": domains,
            "regimes": regimes,
            "material_categories": categories,
        },
        "laws": laws,
        "ontology": ontology,
        "materials": materials,
        "constants": constants,
    }

    os.makedirs(os.path.dirname(OUT_PATH), exist_ok=True)
    payload = json.dumps(bundle, ensure_ascii=False, separators=(",", ":"))
    with open(OUT_PATH, "w", encoding="utf-8") as f:
        f.write("// Auto-generated from SSOT JSON databases by site/build_site.py\n")
        f.write(f"window.CITADEL_SSOT = {payload};\n")

    hashes = stamp_asset_hashes()
    size_kb = os.path.getsize(OUT_PATH) / 1024.0
    print(
        f"[OK] Generated {OUT_PATH} ({size_kb:.1f} KB) | "
        f"{len(laws)} laws, {len(ontology)} symbols, {len(materials)} materials, {len(constants)} constants | "
        f"hashes={hashes}"
    )

    if os.path.isfile(KWEB_GATE) and "--skip-gate" not in sys.argv:
        res = subprocess.run([sys.executable, KWEB_GATE, ROOT_DIR], check=False)
        if res.returncode != 0:
            sys.exit(res.returncode)


if __name__ == "__main__":
    main()
