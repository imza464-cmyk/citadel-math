# Citadel-Math Launch & Discovery Kit (MCP Directories, PyPI/crates.io & Communities)

Hazır, abartısız, doğrudan problemi ve gerçek mühendislik hikâyesini anlatan dağıtım ve başvuru metinleri.

---

## 1. MCP Dizinleri Başvuru Checklist'i

### A. Official MCP Registry (`registry.modelcontextprotocol.io`)
Depo kökündeki [`server.json`](../server.json) ve `Dockerfile` içindeki `io.modelcontextprotocol.server.name` OCI etiketi hazırdır.
```bash
# mcp-publisher CLI ile yayınlama (GitHub hesabıyla tek seferlik OAuth):
mcp-publisher login github
mcp-publisher publish
```
> Not: `v0.2.2` etiketini attığında `server.json` içindeki `version` ve `ghcr.io/...:0.2.2` alanlarını da `0.2.2` yapıp tekrar `mcp-publisher publish` çalıştırabilirsin.

### B. Smithery (`smithery.ai`)
Depo kökündeki [`smithery.yaml`](../smithery.yaml) (sıfır API anahtarlı Docker `stdio`) hazırdır. `https://smithery.ai/new` üzerinden doğrudan `https://github.com/imza464-cmyk/citadel-math` bağlantısını ekle.

### C. `awesome-mcp-servers` (GitHub PR Satırı)
`Science & Math` veya `Developer Tools` bölümüne eklenecek tek satırlık PR metni:
```markdown
- [imza464-cmyk/citadel-math](https://github.com/imza464-cmyk/citadel-math) 🦀 🐍 🏠 - Offline deterministic Z⁷ SI dimensional shield, checked-`i64` algebraic identity checker, IEEE 1788-2015 outward-rounded `Interval64` engine, and 15-tool physics/engineering guardrail (`#![forbid(unsafe_code)]`, zero-`eval` AST).
```

### D. PulseMCP (`pulsemcp.com/submit`) & `mcp.so`
- **Name:** `Citadel-Math (kmath-mcp)`
- **Short Pitch:** Offline Rust + Python MCP guardrail that catches LLM unit-scale bugs (`kN` vs `N`), dropped singularities (`x != 1`), and IEEE-754 floating-point drift in ~5 ms (`85/85` Rust tests, `25,000+` exact dyadic + `80`-digit `Decimal` interval oracles, zero-`eval` AST).
- **Repo:** `https://github.com/imza464-cmyk/citadel-math`
- **Docs/Live Demo:** `https://imza464-cmyk.github.io/citadel-math/`

---

## 2. PyPI & crates.io Yayın Adımları (v0.2.2 için)

```bash
# 1. PyPI (pip install kmath-mcp / uvx kmath-mcp)
pip install build twine
python -m build
twine check dist/*
twine upload dist/*   # PyPI API token veya GitHub Actions OIDC Trusted Publisher

# 2. crates.io (citadel_math_core)
cargo publish --manifest-path citadel_math_core/Cargo.toml --dry-run
cargo publish --manifest-path citadel_math_core/Cargo.toml
```
*(PyPI'a yüklendikten sonra `README.md` başındaki `<!-- mcp-name: io.github.imza464-cmyk/citadel-math -->` etiketi sayesinde `server.json` içine `registryType: "pypi"` paketi de doğrudan eklenebilir.)*

---

## 3. Show HN / `r/rust` / `r/mcp` Paylaşım Taslağı (İngilizce — Gerçek Mühendislik Hikâyesi)

**Başlık (Show HN / r/rust):**
> Show HN: I built a zero-unsafe Rust MCP server to catch LLM unit/interval bugs — then an 80-digit oracle caught a 1-ULP `serde_json` bug in my own server

**Gövde:**
```text
LLMs are surprisingly bad at silent math/engineering edge cases: mixing kN and N (same Z^7 dimension, 1000x off), simplifying (x^2 - 1)/(x - 1) to x + 1 while dropping the x != 1 hole, or leaking IEEE-754 rounding drift into tolerances.

I built Citadel-Math (kmath-mcp), an offline pure-Rust (#![forbid(unsafe_code)]) + Python zero-eval MCP server that acts as a ~5 ms deterministic guardrail for AI agents:
- Z^7 SI dimensional homogeneity & unit-scale mismatch checker
- Checked-i64 guarded rational/polynomial identity checker (surfaces denominator guards like x != 1)
- IEEE 1788-2015 outward-rounded Interval64 (Knuth TwoSum, TwoProdFMA, Sterbenz-Boldo-Melquiond FMA division) + Dual64 AD
- Exact rational Buckingham-Pi nullspace solver, NSGA-II/Skyline Pareto, and 367 golden-anchored physics laws / 80 materials

The best part happened after an honest code review pushed me to write independent high-precision oracles (25,000+ exact BigInt dyadic vectors in Rust + an 80-digit Python Decimal oracle against the live binary). Those tests immediately caught 5 real numerical bugs in my own engine:
1. `serde_json` without `features = ["float_roundtrip"]` shifted `13.769785044591707` by +1 ULP on input, which `exp(x)` amplified into a 15-ULP interval containment violation.
2. `Interval64::pow` had a `< 1e-12` integer-snapping shortcut that broke containment on `1000^(2 + 5e-13)` by ~30,000 ULPs.
3. Scaled TwoSum in `add_directed` underflowed `5e-324 + 1e307` to `[1e307, 1e307]`.
4. Stumpff functions in our Lambert solver lost 9–12 digits to catastrophic cancellation near parabolic orbits (`1e-5 < |z| < 0.05`).
5. A 4x Darcy-vs-Fanning friction factor bug in Fanno flow had slipped right past dimensional analysis because `4` is dimensionless.

Repo (AGPL-3.0 / Commercial): https://github.com/imza464-cmyk/citadel-math
Interactive Z^7 browser preview & benchmarks: https://imza464-cmyk.github.io/citadel-math/

I'd especially appreciate feedback on `interval.rs` and whether a smaller standalone crate focused just on `dimensions + interval` would be more useful to Rust/embedded folks.
```

---

## 4. TEKNOFEST / Üniversite Roket & İHA Takımları + Türkçe Geliştirici Toplulukları

**Başlık:**
> LLM'lerin birim (`kN` vs `N`), burkulma ve yuvarlama hatalarını 5 ms'de yakalayan çevrimdışı Rust + MCP doğrulama motoru (Citadel-Math)

**Gövde:**
```text
Yapay zekâ araçlarına (Claude, Cursor, Windsurf) fizik veya mühendislik hesabı yaptırırken en sık başa gelen dert: formül düzgün görünüyor ama arada 1000× birim ölçek hatası (kN/N), atlanmış payda sıfırı (x != 1) ya da yuvarlama sapması sızıyor.

Bunu çevrimdışı ve deterministik olarak yakalamak için saf Rust (`#![forbid(unsafe_code)]`) çekirdekli 15 araçlı bir MCP sunucusu geliştirdim: Citadel-Math (`kmath-mcp`).

Öne çıkanlar:
- Z^7 SI boyut ve birim ölçek kalkanı (`kN` vs `kg·m/s²` gibi 1000× hataları ~4 ms'de yakalar)
- IEEE 1788-2015 dışa yuvarlamalı Aralık Aritmetiği (toleranslar altında kesin en kötü durum aralığı; 25.000+ tam ikili BigInt + 80-basamaklı Decimal kahini ile doğrulandı)
- Faz-A ön eleme araçları: 80 malzeme (Al-Li 2195, Ti-6Al-4V, PICA-X, S355, Viton, CoCrMo...), NASA SP-8007 kabuk burkulması + Von Mises + ablatif TPS boyutlandırma, RK4 yörünge/Lambert (Curtis Örnek 5.2 doğrulamalı)
- 367 kanonik yasa (%100 altın sayısal çapa ve >=3 kaynak), 85/85 Rust birim testi, 0 `eval()` (11 RCE testi PASS)
- Tek komutla `--network=none` Docker / GHCR üzerinden tamamen çevrimdışı çalışır.

ANSYS/GMAT yerine geçmez; ağır analize geçmeden önce ajanların ve hızlı ön tasarımların önüne koyabileceğiniz deterministik bir filtre.

Canlı tarayıcı önizlemesi & katalog: https://imza464-cmyk.github.io/citadel-math/
GitHub: https://github.com/imza464-cmyk/citadel-math

Özellikle roket/İHA/gömülü ekiplerden "şu araç veya şu doğrulama eksik" şeklinde gerçek geri bildirimlere çok açığım.
```
