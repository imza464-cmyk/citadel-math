// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Citadel-Math v0.2.2 — Precision Engineering Studio, Zero-Eval Z^7 AST Engine, SVG Ashby Plot & ⌘K Global Palette

(() => {
  "use strict";

  const SSOT = window.CITADEL_SSOT;
  if (!SSOT) {
    document.body.innerHTML =
      "<p style='padding:24px;font-family:monospace;color:#f87171;'>SSOT veri paketi (site/data/ssot_bundle.js) yüklenemedi. Önce 'python site/build_site.py' çalıştırın.</p>";
    return;
  }

  const G0 = 9.80665;
  const Z7_AXES = ["M", "L", "T", "I", "Θ", "N", "J"];
  const ZERO_DIM = [0, 0, 0, 0, 0, 0, 0];

  const DIMENSION_CATEGORIES = {
    dimensionless: [0, 0, 0, 0, 0, 0, 0],
    mass: [1, 0, 0, 0, 0, 0, 0],
    length: [0, 1, 0, 0, 0, 0, 0],
    time: [0, 0, 1, 0, 0, 0, 0],
    current: [0, 0, 0, 1, 0, 0, 0],
    temperature: [0, 0, 0, 0, 1, 0, 0],
    amount: [0, 0, 0, 0, 0, 1, 0],
    luminous: [0, 0, 0, 0, 0, 0, 1],
    area: [0, 2, 0, 0, 0, 0, 0],
    volume: [0, 3, 0, 0, 0, 0, 0],
    velocity: [0, 1, -1, 0, 0, 0, 0],
    acceleration: [0, 1, -2, 0, 0, 0, 0],
    force: [1, 1, -2, 0, 0, 0, 0],
    momentum: [1, 1, -1, 0, 0, 0, 0],
    energy: [1, 2, -2, 0, 0, 0, 0],
    power: [1, 2, -3, 0, 0, 0, 0],
    pressure: [1, -1, -2, 0, 0, 0, 0],
    stress: [1, -1, -2, 0, 0, 0, 0],
    density: [1, -3, 0, 0, 0, 0, 0],
    frequency: [0, 0, -1, 0, 0, 0, 0],
    voltage: [1, 2, -3, -1, 0, 0, 0],
    magnetic_field: [1, 0, -2, -1, 0, 0, 0],
    electric_charge: [0, 0, 1, 1, 0, 0, 0],
    resistance: [1, 2, -3, -2, 0, 0, 0],
  };

  const DEFAULT_PHYSICS_SYMBOLS = {
    a: [0, 1, -2, 0, 0, 0, 0],
    m: [1, 0, 0, 0, 0, 0, 0],
    m1: [1, 0, 0, 0, 0, 0, 0],
    m2: [1, 0, 0, 0, 0, 0, 0],
    M: [1, 0, 0, 0, 0, 0, 0],
    r: [0, 1, 0, 0, 0, 0, 0],
    x: [0, 1, 0, 0, 0, 0, 0],
    t: [0, 0, 1, 0, 0, 0, 0],
    v: [0, 1, -1, 0, 0, 0, 0],
    F: [1, 1, -2, 0, 0, 0, 0],
    E: [1, 2, -2, 0, 0, 0, 0],
    P: [1, 2, -3, 0, 0, 0, 0],
    T: [0, 0, 0, 0, 1, 0, 0],
    q: [0, 0, 1, 1, 0, 0, 0],
    I: [0, 0, 0, 1, 0, 0, 0],
    V: [1, 2, -3, -1, 0, 0, 0],
    B: [1, 0, -2, -1, 0, 0, 0],
    rho: [1, -3, 0, 0, 0, 0, 0],
    g_0: [0, 1, -2, 0, 0, 0, 0],
    g0: [0, 1, -2, 0, 0, 0, 0],
    c: [0, 1, -1, 0, 0, 0, 0],
  };

  const SI_UNIT_TABLE = {
    kg: [1.0, [1, 0, 0, 0, 0, 0, 0]],
    g: [1e-3, [1, 0, 0, 0, 0, 0, 0]],
    ton: [1e3, [1, 0, 0, 0, 0, 0, 0]],
    m: [1.0, [0, 1, 0, 0, 0, 0, 0]],
    mm: [1e-3, [0, 1, 0, 0, 0, 0, 0]],
    cm: [1e-2, [0, 1, 0, 0, 0, 0, 0]],
    km: [1e3, [0, 1, 0, 0, 0, 0, 0]],
    s: [1.0, [0, 0, 1, 0, 0, 0, 0]],
    ms: [1e-3, [0, 0, 1, 0, 0, 0, 0]],
    us: [1e-6, [0, 0, 1, 0, 0, 0, 0]],
    ns: [1e-9, [0, 0, 1, 0, 0, 0, 0]],
    A: [1.0, [0, 0, 0, 1, 0, 0, 0]],
    K: [1.0, [0, 0, 0, 0, 1, 0, 0]],
    mol: [1.0, [0, 0, 0, 0, 0, 1, 0]],
    cd: [1.0, [0, 0, 0, 0, 0, 0, 1]],
    Hz: [1.0, [0, 0, -1, 0, 0, 0, 0]],
    N: [1.0, [1, 1, -2, 0, 0, 0, 0]],
    kN: [1e3, [1, 1, -2, 0, 0, 0, 0]],
    MN: [1e6, [1, 1, -2, 0, 0, 0, 0]],
    Pa: [1.0, [1, -1, -2, 0, 0, 0, 0]],
    kPa: [1e3, [1, -1, -2, 0, 0, 0, 0]],
    MPa: [1e6, [1, -1, -2, 0, 0, 0, 0]],
    GPa: [1e9, [1, -1, -2, 0, 0, 0, 0]],
    bar: [1e5, [1, -1, -2, 0, 0, 0, 0]],
    J: [1.0, [1, 2, -2, 0, 0, 0, 0]],
    kJ: [1e3, [1, 2, -2, 0, 0, 0, 0]],
    MJ: [1e6, [1, 2, -2, 0, 0, 0, 0]],
    W: [1.0, [1, 2, -3, 0, 0, 0, 0]],
    kW: [1e3, [1, 2, -3, 0, 0, 0, 0]],
    MW: [1e6, [1, 2, -3, 0, 0, 0, 0]],
    GW: [1e9, [1, 2, -3, 0, 0, 0, 0]],
    C: [1.0, [0, 0, 1, 1, 0, 0, 0]],
    V: [1.0, [1, 2, -3, -1, 0, 0, 0]],
    Ohm: [1.0, [1, 2, -3, -2, 0, 0, 0]],
    ohm: [1.0, [1, 2, -3, -2, 0, 0, 0]],
    F: [1.0, [-1, -2, 4, 2, 0, 0, 0]],
    H: [1.0, [1, 2, -2, -2, 0, 0, 0]],
    T: [1.0, [1, 0, -2, -1, 0, 0, 0]],
  };

  const MCP_TOOLS = [
    { name: "kmath_audit", layer: "Python (10) + Rust (16)", desc: "26 katmanlı SSOT ontoloji, boyutsal homojenlik, CODATA ve cebirsel sağlamlık denetimini çalıştırır." },
    { name: "kmath_dimension_check", layer: "Zero-Eval AST + Z⁷", desc: "İki ifadenin Z⁷ SI boyut eşitliğini (dim(LHS) == dim(RHS)) ve ölçekli birim çarpan oranını denetler." },
    { name: "kmath_prove_identity", layer: "Rust · PolyZ / MultiPolyZ", desc: "Taşma korumalı checked_i64 polinom halkasında cebirsel özdeşliği ve payda yan koşullarını (GuardedPoly) doğrular." },
    { name: "kmath_interval_eval", layer: "Rust · Interval64 + Dual64", desc: "IEEE 1788-2015 0/1-ULP dışa yuvarlamalı aralık, Taylor kapanımı, Dual64 gradyan ve sembolik türev hesaplar." },
    { name: "kmath_constant", layer: "SSOT · NIST CODATA 2022", desc: "355 resmi NIST CODATA 2022 sabiti ve IAU gezegen parametrelerinden kesin değer ve birim getirir." },
    { name: "kmath_materials", layer: "SSOT · 30 Malzeme", desc: "30 yapısal, kriyojenik, UHTC ve ablatif TPS malzemesini kopma uzunluğu ve sıcaklık sınırına göre sıralar." },
    { name: "kmath_stress_check", layer: "Rust · NASA SP-8007", desc: "İnce cidarlı silindirik kabuk Von Mises gerilmesi, eksenel burkulma ve Sutton-Graves TPS dayanımını denetler." },
    { name: "kmath_simulate_trajectory", layer: "Python + Rust · RK4 / USSA-1976", desc: "7 katmanlı USSA-1976 hidrostatik atmosfer altında 4. derece Runge-Kutta yörünge entegrasyonu koşturur." },
    { name: "kmath_solve_for", layer: "Rust · Symbolic Inverter", desc: "Kanonik bir denklemi hedef değişken için cebirsel tersler, yan koşulları çıkarır ve sıfır-panik Rust kodu üretir." },
    { name: "kmath_synthesize", layer: "Rust · AND-OR Hypergraph", desc: "Çift-yönlü bilgi hipergrafında geçerlilik rejimi uyumlu yeni denklem zincirleri sentezler." },
    { name: "kmath_inverse_design", layer: "Rust · Pareto Sweeper", desc: "Kapalı döngü parametrik taramayla (RK4 + SP-8007 + PICA-X) Pareto-optimal ön tasarım adaylarını filtreler." },
    { name: "kmath_physical_bounds_check", layer: "Rust · Conservation Shield", desc: "Termodinamik 1./2. yasa (Carnot), nedensellik (v < c) ve momentum korunumu ihlallerini budar." },
    { name: "kmath_regime_check", layer: "Rust · DomainBounds", desc: "Sayısal girdilerin 18 fiziksel rejim (Mach, görelilik, ergodiklik) sınırı içinde kalıp kalmadığını denetler." },
    { name: "kmath_buckingham_pi", layer: "Rust · Rat128 Nullspace", desc: "Tam tamsayı Z⁷ sıfır uzayı (Gauss-Jordan) üzerinden boyutsuz Buckingham-Π gruplarını keşfeder." },
    { name: "kmath_pareto_front", layer: "Rust · NSGA-II / Skyline", desc: "Kesin Pareto Skyline (Rank 0), kalabalık mesafesi, Ütopya/Nadir vektörleri ve diz-noktasını hesaplar." },
  ];

  const QUICKRUN_COMMANDS = {
    docker: "docker run -i --rm --read-only --tmpfs /tmp:size=64M,noexec,nosuid --cap-drop=ALL --security-opt=no-new-privileges:true --network=none ghcr.io/imza464-cmyk/citadel-math:0.2.2",
    pip: "pip install https://github.com/imza464-cmyk/citadel-math/releases/download/v0.2.2/kmath_mcp-0.2.2-py3-none-any.whl",
    uvx: "uvx --from https://github.com/imza464-cmyk/citadel-math/releases/download/v0.2.2/kmath_mcp-0.2.2-py3-none-any.whl kmath stats",
  };

  const LATENCY_BENCHMARKS = [
    { name: 'kmath_constant("c")', p50: 0.28, p95: 0.33, cls: "py", note: "Python JSON" },
    { name: "kmath_dimension_check", p50: 3.60, p95: 4.81, cls: "py", note: "Zero-Eval AST" },
    { name: "kmath_prove_identity", p50: 5.30, p95: 5.68, cls: "rust", note: "Rust Subprocess" },
    { name: "kmath_interval_eval", p50: 5.45, p95: 5.72, cls: "rust", note: "Rust Subprocess" },
    { name: "kmath_stress_check", p50: 5.56, p95: 5.84, cls: "rust", note: "Py + Rust" },
    { name: "kmath_audit (26 kapı)", p50: 45.65, p95: 47.73, cls: "audit", note: "10 Py + 16 Rust" },
  ];

  const SYMBOL_DIMS = { ...DEFAULT_PHYSICS_SYMBOLS };
  for (const [sym, info] of Object.entries(SSOT.ontology)) {
    if (Array.isArray(info.z7_vector) && info.z7_vector.length === 7) {
      SYMBOL_DIMS[sym] = info.z7_vector;
    }
  }

  const $ = (sel) => document.querySelector(sel);
  const $$ = (sel) => Array.from(document.querySelectorAll(sel));

  function readStorage(key, fallback) {
    try {
      const val = window.localStorage ? window.localStorage.getItem(key) : null;
      return val || fallback;
    } catch (_) {
      return fallback;
    }
  }

  function writeStorage(key, value) {
    try {
      if (window.localStorage) window.localStorage.setItem(key, value);
    } catch (_) {}
  }

  function esc(str) {
    return String(str ?? "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function dimEq(a, b) {
    for (let i = 0; i < 7; i++) if (a[i] !== b[i]) return false;
    return true;
  }

  function isDimensionless(d) {
    return dimEq(d, ZERO_DIM);
  }

  function dimAdd(a, b) { return a.map((v, i) => v + b[i]); }
  function dimSub(a, b) { return a.map((v, i) => v - b[i]); }

  function dimScale(a, exp) {
    const out = [];
    for (let i = 0; i < 7; i++) {
      const val = a[i] * exp;
      const rounded = Math.round(val);
      if (Math.abs(val - rounded) > 1e-9) {
        throw new Error(`Kesirli boyut hatası: [${formatDim(a)}]^(${exp}) Z^7 kafesinde tamsayı değil.`);
      }
      out.push(rounded);
    }
    return out;
  }

  function formatDim(d) {
    const parts = [];
    for (let i = 0; i < 7; i++) {
      if (d[i] === 1) parts.push(Z7_AXES[i]);
      else if (d[i] !== 0) parts.push(`${Z7_AXES[i]}^${d[i]}`);
    }
    return parts.length ? parts.join(" ") : "1 (boyutsuz)";
  }

  function showToast(msg) {
    const el = $("#toast");
    if (!el) return;
    el.textContent = msg;
    el.classList.add("show");
    clearTimeout(showToast._t);
    showToast._t = setTimeout(() => el.classList.remove("show"), 1900);
  }

  function copyText(text, label = "Kopyalandı") {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(() => showToast(label)).catch(() => fallbackCopy(text, label));
    } else {
      fallbackCopy(text, label);
    }
  }

  function fallbackCopy(text, label) {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
    showToast(label);
  }

  // =========================================================================
  // Zero-Eval Client-Side Z^7 Dimensional & SI Unit Scale Parser
  // =========================================================================
  const SAFE_FUNCS = new Set([
    "sin", "cos", "tan", "asin", "acos", "atan",
    "sinh", "cosh", "tanh", "exp", "log", "ln", "sqrt", "abs", "Abs",
  ]);
  const BLOCKED_IDENTS = new Set([
    "eval", "exec", "open", "compile", "getattr", "setattr", "delattr",
    "globals", "locals", "vars", "dir", "type", "object", "input",
    "breakpoint", "help", "exit", "quit", "lambda", "import", "Function", "window",
  ]);

  function tokenizeMath(raw) {
    if (!raw || !raw.trim()) throw new Error("Boş ifade ayrıştırılamaz.");
    const s = raw.trim();
    if (s.length > 2048) throw new Error("SECURITY_BLOCKED: İfade uzunluğu 2048 karakter sınırını aşıyor.");
    if (s.includes("__") || s.includes(";") || s.includes("\n") || s.includes("\r") || s.includes("'") || s.includes('"')) {
      throw new Error("SECURITY_BLOCKED: Yasaklı karakter, tırnak veya dunder (__) dizisi tespit edildi (zero-eval AST kalkanı).");
    }

    const tokens = [];
    let i = 0;
    while (i < s.length) {
      const ch = s[i];
      if (/\s/.test(ch)) { i++; continue; }
      if (ch === "*" && s[i + 1] === "*") { tokens.push({ t: "POW", v: "**" }); i += 2; continue; }
      if ("+-*/^(),".includes(ch)) {
        const map = { "+": "PLUS", "-": "MINUS", "*": "MUL", "/": "DIV", "^": "POW", "(": "LP", ")": "RP", ",": "COMMA" };
        tokens.push({ t: map[ch], v: ch });
        i++;
        continue;
      }
      if (/[0-9]/.test(ch) || (ch === "." && /[0-9]/.test(s[i + 1] || ""))) {
        let j = i;
        while (j < s.length && /[0-9.]/.test(s[j])) j++;
        if ((s[j] === "e" || s[j] === "E") && /[0-9+-]/.test(s[j + 1] || "")) {
          j++;
          if (s[j] === "+" || s[j] === "-") j++;
          while (j < s.length && /[0-9]/.test(s[j])) j++;
        }
        const num = Number(s.slice(i, j));
        if (!Number.isFinite(num)) throw new Error(`Geçersiz sayı sabiti: '${s.slice(i, j)}'`);
        tokens.push({ t: "NUM", v: num });
        i = j;
        continue;
      }
      if (/[A-Za-z_]/.test(ch)) {
        let j = i;
        while (j < s.length && /[A-Za-z0-9_]/.test(s[j])) j++;
        const id = s.slice(i, j);
        if (s[j] === ".") throw new Error(`SECURITY_BLOCKED: Öznitelik erişimi '${id}.*' zero-eval AST duvarınca reddedildi.`);
        if (id.startsWith("_") || BLOCKED_IDENTS.has(id)) throw new Error(`SECURITY_BLOCKED: Yasaklı tanımlayıcı '${id}' reddedildi.`);
        tokens.push({ t: "ID", v: id });
        i = j;
        continue;
      }
      throw new Error(`SECURITY_BLOCKED: Desteklenmeyen karakter '${ch}'.`);
    }
    return tokens;
  }

  function parseAst(tokens) {
    let pos = 0;
    const peek = () => tokens[pos];
    const next = () => tokens[pos++];

    function parseExpr() {
      let node = parseTerm();
      while (peek() && (peek().t === "PLUS" || peek().t === "MINUS")) {
        const op = next().t;
        node = { type: "bin", op, left: node, right: parseTerm() };
      }
      return node;
    }

    function parseTerm() {
      let node = parseUnary();
      while (peek() && (peek().t === "MUL" || peek().t === "DIV")) {
        const op = next().t;
        node = { type: "bin", op, left: node, right: parseUnary() };
      }
      return node;
    }

    function parseUnary() {
      if (peek() && (peek().t === "PLUS" || peek().t === "MINUS")) {
        const op = next().t;
        return { type: "unary", op, arg: parseUnary() };
      }
      return parsePow();
    }

    function parsePow() {
      let base = parsePrimary();
      if (peek() && peek().t === "POW") {
        next();
        return { type: "bin", op: "POW", left: base, right: parseUnary() };
      }
      return base;
    }

    function parsePrimary() {
      const tok = peek();
      if (!tok) throw new Error("Beklenmeyen ifade sonu.");
      if (tok.t === "NUM") { next(); return { type: "num", val: tok.v }; }
      if (tok.t === "ID") {
        next();
        if (peek() && peek().t === "LP") {
          if (!SAFE_FUNCS.has(tok.v)) throw new Error(`SECURITY_BLOCKED: İzin listesinde olmayan fonksiyon çağrısı '${tok.v}()'.`);
          next();
          const args = [];
          if (peek() && peek().t !== "RP") {
            args.push(parseExpr());
            while (peek() && peek().t === "COMMA") { next(); args.push(parseExpr()); }
          }
          if (!peek() || peek().t !== "RP") throw new Error("Kapanmamış parantez ')'.");
          next();
          return { type: "call", name: tok.v, args };
        }
        return { type: "id", name: tok.v };
      }
      if (tok.t === "LP") {
        next();
        const inner = parseExpr();
        if (!peek() || peek().t !== "RP") throw new Error("Kapanmamış parantez ')'.");
        next();
        return inner;
      }
      throw new Error(`Beklenmeyen jeton: '${tok.v}'`);
    }

    const ast = parseExpr();
    if (pos < tokens.length) throw new Error(`İfade sonunda beklenmeyen jeton: '${tokens[pos].v}'`);
    return ast;
  }

  function evalNumericConst(node) {
    if (node.type === "num") return node.val;
    if (node.type === "id" && node.name === "pi") return Math.PI;
    if (node.type === "unary") {
      const v = evalNumericConst(node.arg);
      return v === null ? null : (node.op === "MINUS" ? -v : v);
    }
    if (node.type === "bin") {
      const l = evalNumericConst(node.left);
      const r = evalNumericConst(node.right);
      if (l === null || r === null) return null;
      if (node.op === "PLUS") return l + r;
      if (node.op === "MINUS") return l - r;
      if (node.op === "MUL") return l * r;
      if (node.op === "DIV") return r === 0 ? null : l / r;
      if (node.op === "POW") return Math.pow(l, r);
    }
    return null;
  }

  function collectIds(node, set) {
    if (node.type === "id") set.add(node.name);
    else if (node.type === "unary") collectIds(node.arg, set);
    else if (node.type === "bin") { collectIds(node.left, set); collectIds(node.right, set); }
    else if (node.type === "call") node.args.forEach((a) => collectIds(a, set));
  }

  function resolveSide(raw) {
    const cleaned = raw.trim();
    const lower = cleaned.toLowerCase();
    if (DIMENSION_CATEGORIES[lower]) return { dim: DIMENSION_CATEGORIES[lower], scale: null, mode: "dimension_category" };
    if (SYMBOL_DIMS[cleaned] && !SI_UNIT_TABLE[cleaned]) return { dim: SYMBOL_DIMS[cleaned], scale: null, mode: "canonical_symbol" };

    const ast = parseAst(tokenizeMath(cleaned));
    if (ast.type === "id") {
      if (SI_UNIT_TABLE[ast.name] && !SYMBOL_DIMS[ast.name]) {
        const [sc, dm] = SI_UNIT_TABLE[ast.name];
        return { dim: dm, scale: sc, mode: "si_unit_expression" };
      }
      if (SYMBOL_DIMS[ast.name]) return { dim: SYMBOL_DIMS[ast.name], scale: null, mode: "canonical_symbol" };
    }

    const ids = new Set();
    collectIds(ast, ids);
    const allUnits =
      ids.size > 0 &&
      Array.from(ids).every((id) => Boolean(SI_UNIT_TABLE[id])) &&
      Array.from(ids).some((id) => Boolean(SI_UNIT_TABLE[id]) && !SYMBOL_DIMS[id]);
    if (allUnits) {
      try {
        const evalUnit = (node) => {
          if (node.type === "num") return [node.val, ZERO_DIM];
          if (node.type === "id") return SI_UNIT_TABLE[node.name];
          if (node.type === "bin" && node.op === "MUL") { const [ls, ld] = evalUnit(node.left); const [rs, rd] = evalUnit(node.right); return [ls * rs, dimAdd(ld, rd)]; }
          if (node.type === "bin" && node.op === "DIV") { const [ls, ld] = evalUnit(node.left); const [rs, rd] = evalUnit(node.right); return [ls / rs, dimSub(ld, rd)]; }
          if (node.type === "bin" && node.op === "POW") {
            const [bs, bd] = evalUnit(node.left);
            const expVal = evalNumericConst(node.right);
            if (expVal === null) throw new Error("Birim üssü sabit sayı olmalıdır.");
            return [Math.pow(bs, expVal), dimScale(bd, expVal)];
          }
          throw new Error("NOT_PURE_UNIT");
        };
        const [scale, dim] = evalUnit(ast);
        return { dim, scale, mode: "si_unit_expression" };
      } catch (e) {
        if (e.message !== "NOT_PURE_UNIT") throw e;
      }
    }

    const infer = (node) => {
      if (node.type === "num") return ZERO_DIM;
      if (node.type === "id") {
        if (node.name === "pi") return ZERO_DIM;
        if (SYMBOL_DIMS[node.name]) return SYMBOL_DIMS[node.name];
        if (SI_UNIT_TABLE[node.name]) return SI_UNIT_TABLE[node.name][1];
        throw new Error(`Sembol '${node.name}' SSOT ONTOLOGY.json içinde tanımlı değil.`);
      }
      if (node.type === "unary") return infer(node.arg);
      if (node.type === "bin") {
        const ld = infer(node.left);
        const rd = infer(node.right);
        if (node.op === "PLUS" || node.op === "MINUS") {
          if (!dimEq(ld, rd)) throw new Error(`Toplamsal boyut homojenliği ihlali: [${formatDim(ld)}] != [${formatDim(rd)}]`);
          return ld;
        }
        if (node.op === "MUL") return dimAdd(ld, rd);
        if (node.op === "DIV") return dimSub(ld, rd);
        if (node.op === "POW") {
          if (!isDimensionless(rd)) throw new Error(`Üs ifadesi boyutsuz olmalıdır, bulunan: [${formatDim(rd)}]`);
          if (isDimensionless(ld)) return ZERO_DIM;
          const expVal = evalNumericConst(node.right);
          if (expVal === null) throw new Error("Boyutlu tabanın üssü rasyonel sabit olmalıdır.");
          return dimScale(ld, expVal);
        }
      }
      if (node.type === "call") {
        const fn = node.name.toLowerCase();
        if (fn === "sqrt") { if (node.args.length !== 1) throw new Error("sqrt() 1 argüman alır."); return dimScale(infer(node.args[0]), 0.5); }
        if (fn === "abs") { if (node.args.length !== 1) throw new Error("abs() 1 argüman alır."); return infer(node.args[0]); }
        for (const a of node.args) {
          const ad = infer(a);
          if (!isDimensionless(ad)) throw new Error(`Aşkın fonksiyon '${node.name}(...)' argümanı boyutsuz olmalıdır, bulunan: [${formatDim(ad)}]`);
        }
        return ZERO_DIM;
      }
      throw new Error("Desteklenmeyen AST düğümü.");
    };
    return { dim: infer(ast), scale: null, mode: "symbolic_ast" };
  }

  function evaluateZ7Pair(lhs, rhs) {
    const t0 = performance.now();
    try {
      const l = resolveSide(lhs);
      const r = resolveSide(rhs);
      const homogeneous = dimEq(l.dim, r.dim);
      let scaleRatio = null;
      let scaleMatch = null;
      if (homogeneous && l.scale !== null && r.scale !== null) {
        scaleRatio = l.scale / r.scale;
        scaleMatch = Math.abs(scaleRatio - 1.0) < 1e-12;
      }
      let verdict = homogeneous ? "DIMENSIONALLY_HOMOGENEOUS" : "DIMENSIONAL_MISMATCH";
      let cls = homogeneous ? "ok" : "err";
      if (homogeneous && scaleMatch === false) {
        verdict = "DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH";
        cls = "warn";
      }
      return {
        ok: true,
        cls,
        verdict,
        ms: (performance.now() - t0).toFixed(2),
        l,
        r,
        scaleRatio,
        payload: {
          status: "ok",
          engine: "browser_js_preview",
          lhs: lhs.trim(),
          rhs: rhs.trim(),
          dim_lhs: formatDim(l.dim),
          dim_rhs: formatDim(r.dim),
          z7_tuple_lhs: l.dim,
          z7_tuple_rhs: r.dim,
          homogeneous,
          si_scale_ratio_lhs_over_rhs: scaleRatio,
          verdict,
        },
      };
    } catch (err) {
      const isSec = String(err.message).startsWith("SECURITY_BLOCKED");
      return {
        ok: false,
        cls: "err",
        verdict: isSec ? "SECURITY_BLOCKED_ZERO_EVAL" : "DIMENSIONAL_OR_SYNTAX_ERROR",
        ms: (performance.now() - t0).toFixed(2),
        isSec,
        message: err.message,
        payload: {
          status: "error",
          engine: "browser_js_preview",
          lhs: lhs.trim(),
          rhs: rhs.trim(),
          homogeneous: false,
          verdict: isSec ? "SECURITY_BLOCKED_ZERO_EVAL" : "DIMENSIONAL_OR_SYNTAX_ERROR",
          message: err.message,
        },
      };
    }
  }

  function renderZ7Matrix(lDim, rDim) {
    return `
      <div class="z7-grid" aria-label="Z7 Boyut Vektörü Karşılaştırması">
        ${Z7_AXES.map((ax, idx) => {
          const lv = lDim[idx];
          const rv = rDim[idx];
          const diff = lv !== rv;
          const active = !diff && lv !== 0;
          const cls = diff ? "mismatch" : active ? "active-dim" : "";
          const disp = diff ? `${lv}≠${rv}` : String(lv);
          return `<div class="z7-cell ${cls}"><span class="z7-axis">${ax}</span><span class="z7-vals">${disp}</span></div>`;
        }).join("")}
      </div>
    `;
  }

  function runHeroConsole() {
    const lhsEl = $("#heroLhs");
    const rhsEl = $("#heroRhs");
    const out = $("#heroConsoleOutput");
    if (!lhsEl || !rhsEl || !out) return;

    const res = evaluateZ7Pair(lhsEl.value, rhsEl.value);
    const timingEl = $("#heroTiming");
    if (timingEl) timingEl.textContent = `Tarayıcı JS: ${res.ms} ms · Canlı MCP p50: 5.30 ms`;

    if (res.ok) {
      const note = res.scaleRatio !== null ? `${res.scaleRatio}× SI ölçek` : `${formatDim(res.l.dim)} vs ${formatDim(res.r.dim)}`;
      const scaleL = res.l.scale !== null ? ` · ölçek ${res.l.scale}×` : "";
      const scaleR = res.r.scale !== null ? ` · ölçek ${res.r.scale}×` : "";
      out.innerHTML = `
        <div class="verdict-line ${res.cls}">
          <span>${esc(res.verdict)}</span>
          <span>${esc(note)}</span>
        </div>
        ${renderZ7Matrix(res.l.dim, res.r.dim)}
        <div class="mono" style="font-size:0.72rem;color:var(--text-muted);background:var(--bg-elevated);border:1px solid var(--border-subtle);border-radius:6px;padding:8px 10px;line-height:1.55;">
          <div><span style="color:var(--text-dim);">LHS</span> → <strong>[${esc(formatDim(res.l.dim))}]</strong> <span style="color:var(--text-dim);">(${esc(res.l.mode)}${scaleL})</span></div>
          <div><span style="color:var(--text-dim);">RHS</span> → <strong>[${esc(formatDim(res.r.dim))}]</strong> <span style="color:var(--text-dim);">(${esc(res.r.mode)}${scaleR})</span></div>
        </div>
      `;
    } else {
      out.innerHTML = `
        <div class="verdict-line err">
          <span>${esc(res.verdict)}</span>
          <span>${res.isSec ? "AST Güvenlik Duvarı" : "Reddedildi"}</span>
        </div>
        <div class="mono" style="font-size:0.75rem;color:var(--text-muted);padding:4px 2px;">${esc(res.message)}</div>
      `;
    }
  }

  function runSandbox() {
    const out = $("#sandboxOutput");
    if (!out) return;
    const res = evaluateZ7Pair($("#lhsInput").value, $("#rhsInput").value);
    if (res.ok) {
      const note = res.scaleRatio !== null ? `SI Ölçek Oranı: ${res.scaleRatio}×` : `${res.l.mode} / ${res.r.mode}`;
      out.innerHTML = `
        <div class="verdict-line ${res.cls}">
          <span>${esc(res.verdict)}</span>
          <span>${esc(note)} · ${res.ms} ms</span>
        </div>
        ${renderZ7Matrix(res.l.dim, res.r.dim)}
        <pre class="code-block">${esc(JSON.stringify(res.payload, null, 2))}</pre>
      `;
    } else {
      out.innerHTML = `
        <div class="verdict-line err">
          <span>${esc(res.verdict)}</span>
          <span>${res.isSec ? "Zero-Eval AST Duvarı" : "Reddedildi"} · ${res.ms} ms</span>
        </div>
        <pre class="code-block">${esc(JSON.stringify(res.payload, null, 2))}</pre>
      `;
    }
  }

  // =========================================================================
  // Sync Counts & Latency Chart
  // =========================================================================
  function syncCounts() {
    const setTxt = (sel, val) => { const el = $(sel); if (el) el.textContent = String(val); };
    setTxt("#tocLawCount", SSOT.meta.law_count);
    setTxt("#tocOntCount", SSOT.meta.symbol_count);
    setTxt("#tocNistCount", SSOT.meta.constant_count);
    setTxt("#tocMatCount", SSOT.meta.material_count);
    setTxt("#kpiLawsSym", `${SSOT.meta.law_count} / ${SSOT.meta.symbol_count}`);
  }

  function renderLatencyChart() {
    const el = $("#latencyChart");
    if (!el) return;
    const maxMs = Math.max(...LATENCY_BENCHMARKS.map((b) => b.p95));
    el.innerHTML = LATENCY_BENCHMARKS.map((b) => {
      const pct = Math.max(2.5, (b.p50 / maxMs) * 100);
      return `
        <div class="lat-row">
          <div>
            <strong>${esc(b.name)}</strong>
            <span style="color:var(--text-dim);font-size:0.7rem;display:block;">${esc(b.note)}</span>
          </div>
          <div class="lat-track"><div class="lat-fill ${b.cls}" style="width:${pct.toFixed(1)}%"></div></div>
          <div style="text-align:right;font-variant-numeric:tabular-nums;">
            <strong>${b.p50.toFixed(2)} ms</strong>
            <span style="color:var(--text-dim);font-size:0.7rem;display:block;">p95: ${b.p95.toFixed(2)} ms</span>
          </div>
        </div>
      `;
    }).join("");
  }

  // =========================================================================
  // Interactive 5-Step End-to-End Agent Scenario
  // =========================================================================
  const AGENT_SCENARIO_STEPS = [
    {
      num: "01",
      tool: "kmath_materials",
      shortTitle: "Alaşım Seçimi",
      badge: "PASS · 30 Malzeme",
      badgeClass: "ok",
      call: 'kmath_materials(sort_by="yield_strength", category="metal_alloy")',
      summary: "Metal alaşımları akma dayanımına göre sıralanır: Maraging C-300 (2000 MPa, 8000 kg/m³), 17-4 PH (1170 MPa), AISI 304L Cryo (960 MPa, 806.9 °C) karşılaştırılır; kriyojenik hafif tank için Al-Li 2195-T8 (550 MPa akma, 146.85 °C tavan) seçilir.",
      json: `{
  "selected": "mat_alli_2195 (Al-Li 2195-T8)",
  "top3_yield_comparison": [
    {"id": "mat_maraging_300", "yield_strength_MPa": 2000.0, "breaking_length_km": 25.5},
    {"id": "mat_ss_17_4ph_h900", "yield_strength_MPa": 1170.0, "breaking_length_km": 15.3},
    {"id": "mat_ss_304l_cryo", "yield_strength_MPa": 960.0, "max_temperature_C": 806.85}
  ]
}`,
    },
    {
      num: "02",
      tool: "kmath_dimension_check",
      shortTitle: "Birim & Sembol Kalkanı",
      badge: "2 HATA YAKALANDI",
      badgeClass: "warn",
      call: 'kmath_dimension_check("kN", "kg * m / s^2") && kmath_dimension_check("P * r / t", "Pa")',
      summary: "Hesaplamadan önce iki klasik LLM hatası yakalanır: (a) kN ile kg·m/s² arasında 1000× SI ölçek farkı; (b) P·r/t ifadesinde P=Güç ve t=Zaman çakışması ([M L³ T⁻⁴] ≠ [M L⁻¹ T⁻²]). Açık var_dimensions_json ayrımı zorunlu kılınır.",
      json: `{
  "unit_scale_check": {"lhs": "kN", "rhs": "kg * m / s^2", "si_scale_ratio_lhs_over_rhs": 1000.0, "verdict": "DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH"},
  "symbol_collision_check": {"lhs": "P * r / t", "dim_lhs": "M L^3 T^-4", "dim_rhs": "M L^-1 T^-2", "verdict": "DIMENSIONAL_MISMATCH"}
}`,
      sandboxLhs: "P * r / t",
      sandboxRhs: "Pa",
    },
    {
      num: "03",
      tool: "kmath_stress_check",
      shortTitle: "SP-8007 + PICA-X TPS",
      badge: "PASS · 26.32 mm TPS",
      badgeClass: "ok",
      call: 'kmath_stress_check("mat_alli_2195", mass_kg=45000, g_load=4.5, max_q_pa=35000, radius_m=1.8, length_m=14.0, wall_thickness_mm=5.0, speed_ms=2400, tank_pressure_bar=4.0)',
      summary: "Yapısal dayanım ve NASA SP-8007 burkulma geçer (Von Mises 130.5 MPa, SF 4.215×; burkulma SF 2.486×). Çıplak duvar sıcaklığı 1951.5 °C ile 146.85 °C sınırını aşar; motor otomatik 26.32 mm PICA-X TPS (173.60 kg) boyutlandırır ve yapışma hattını 106.85 °C'ye indirir.",
      json: `{
  "von_mises_mpa": 130.5, "yield_safety_factor": 4.215,
  "sp8007_buckling_critical_mpa": 97.32, "buckling_safety_factor": 2.486,
  "bare_wall_temp_c": 1951.53, "pica_x_thickness_mm": 26.32,
  "pica_x_mass_kg": 173.60, "protected_bondline_temp_c": 106.85, "overall_pass": true
}`,
    },
    {
      num: "04",
      tool: "kmath_interval_eval",
      shortTitle: "Interval64 Tolerans",
      badge: "IEEE 1788-2015 PASS",
      badgeClass: "ok",
      call: 'kmath_interval_eval("(P * r) / t", \'{"P":[390000,410000],"r":[1.795,1.805],"t":[0.0049,0.0051]}\', "t")',
      summary: "İmalat toleransları (P ±10 kPa, r ±5 mm, t ±0.1 mm) altında çember gerilmesi garantili dışa yuvarlamalı [137.26 MPa, 151.03 MPa] aralığına kilitlenir; sembolik türev -(P·r)/t² ve kesin et kalınlığı duyarlılığı ∂σ/∂t = -2.88×10¹⁰ Pa/m hesaplanır.",
      json: `{
  "outward_interval_Pa": [137264705.88, 151030612.24],
  "midpoint_Pa": 144000000.0,
  "symbolic_derivative": "(-((P * r) / t^2))",
  "gradient": {"P": 360.0, "r": 80000000.0, "t": -28800000000.0}, "status": "ok"
}`,
    },
    {
      num: "05",
      tool: "kmath_physical_bounds_check",
      shortTitle: "Carnot & Nedensellik",
      badge: "FİZİKSEL GEÇERLİ",
      badgeClass: "ok",
      call: "kmath_physical_bounds_check(mass_kg=45000, velocity_ms=2400, exhaust_velocity_ms=3400, mass_flow_kg_s=280, thrust_n=952000, temp_hot_k=3200, temp_cold_k=300, efficiency=0.68)",
      summary: "Carnot tavanı 0.90625 (0.68 < 0.90625), Lorentz faktörü γ = 1.000000000032 (v ≪ c) ve momentum korunumu F = ṁ·v_e = 952 kN tam doğrulanır; tasarım fiziksel olarak gerçekleştirilebilir mührü alır.",
      json: `{
  "carnot_max_efficiency": 0.90625,
  "lorentz_gamma": 1.0000000000320444,
  "physically_realizable": true,
  "verdict": "FİZİKSEL OLARAK GEÇERLİ (Termodinamik 1./2. Yasa, Nedensellik ve Momentum Korunumu Doğrulandı)"
}`,
    },
  ];

  let activeScenarioIdx = 0;

  function renderScenario(idx) {
    const bar = $("#scenarioStepper");
    const card = $("#scenarioCard");
    if (!bar || !card) return;
    activeScenarioIdx = Math.max(0, Math.min(AGENT_SCENARIO_STEPS.length - 1, idx));

    bar.innerHTML = AGENT_SCENARIO_STEPS.map(
      (st, i) => `
        <button class="step-btn${i === activeScenarioIdx ? " active" : ""}" role="tab" aria-selected="${i === activeScenarioIdx}" data-scenario-step="${i}">
          <span class="step-num">${esc(st.num)} · ${esc(st.tool)}</span>
          ${esc(st.shortTitle)}
        </button>
      `
    ).join("");

    const cur = AGENT_SCENARIO_STEPS[activeScenarioIdx];
    const sandboxAction = cur.sandboxLhs
      ? `<button class="mini-btn" data-test-lhs="${esc(cur.sandboxLhs)}" data-test-rhs="${esc(cur.sandboxRhs)}">Z⁷ Stüdyoda Aç →</button>`
      : "";
    card.innerHTML = `
      <div class="step-header">
        <div class="step-title">Adım ${esc(cur.num)} · <code>${esc(cur.tool)}</code> — ${esc(cur.shortTitle)}</div>
        <span class="tag ${esc(cur.badgeClass)}">${esc(cur.badge)}</span>
      </div>
      <p>${esc(cur.summary)}</p>
      <div class="code-head">
        <span>${esc(cur.tool)} canlı doğrulama kaydı</span>
        <button class="mini-btn" data-copy="${esc(cur.call)}">Çağrıyı Kopyala</button>
      </div>
      <pre class="code-block">&gt; ${esc(cur.call)}

${esc(cur.json)}</pre>
      <div class="step-actions">
        <div>${sandboxAction}</div>
        <div style="display:flex;gap:6px;">
          <button class="mini-btn" data-scenario-step="${activeScenarioIdx - 1}"${activeScenarioIdx === 0 ? " disabled" : ""}>← Önceki</button>
          <button class="mini-btn" data-scenario-step="${activeScenarioIdx + 1}"${activeScenarioIdx === AGENT_SCENARIO_STEPS.length - 1 ? " disabled" : ""}>Sonraki →</button>
        </div>
      </div>
    `;
  }

  // =========================================================================
  // Canonical Laws Catalog
  // =========================================================================
  const lawState = { q: "", domain: "", regime: "", page: 1, perPage: 20, highlightId: "" };

  function initLawFilters() {
    const domSel = $("#lawDomain");
    SSOT.meta.domains.forEach((d) => {
      const opt = document.createElement("option");
      opt.value = d; opt.textContent = d; domSel.appendChild(opt);
    });
    const regSel = $("#lawRegime");
    SSOT.meta.regimes.forEach((r) => {
      const opt = document.createElement("option");
      opt.value = r; opt.textContent = r; regSel.appendChild(opt);
    });
  }

  function renderLatexNode(el, tex, fallbackEq) {
    if (window.katex && tex) {
      try {
        window.katex.render(tex, el, { displayMode: false, throwOnError: false });
        return;
      } catch (_) {}
    }
    el.innerHTML = `<code>${esc(fallbackEq || tex)}</code>`;
  }

  function renderLaws() {
    const q = lawState.q.trim().toLowerCase();
    const filtered = SSOT.laws.filter((law) => {
      if (lawState.domain && law.domain !== lawState.domain) return false;
      if (lawState.regime && law.regime !== lawState.regime) return false;
      if (!q) return true;
      return (
        (law.id && law.id.toLowerCase().includes(q)) ||
        (law.name && law.name.toLowerCase().includes(q)) ||
        (law.equation && law.equation.toLowerCase().includes(q)) ||
        (law.description && law.description.toLowerCase().includes(q)) ||
        (law.domain && law.domain.toLowerCase().includes(q)) ||
        (law.inputs && law.inputs.some((x) => x.toLowerCase().includes(q)))
      );
    });

    const totalPages = Math.max(1, Math.ceil(filtered.length / lawState.perPage));
    if (lawState.page > totalPages) lawState.page = totalPages;
    const start = (lawState.page - 1) * lawState.perPage;
    const slice = filtered.slice(start, start + lawState.perPage);

    $("#lawCountLabel").textContent = `${filtered.length} / ${SSOT.laws.length} yasa gösteriliyor (Sayfa ${lawState.page} / ${totalPages})`;
    const list = $("#lawsList");
    if (!slice.length) {
      list.innerHTML = `<p style="padding:12px 0;">Aramanızla eşleşen kanonik yasa bulunamadı.</p>`;
      $("#lawsPager").innerHTML = "";
      return;
    }

    list.innerHTML = slice.map((law) => {
      const dims = law.dimensions || {};
      const bounds = law.domain_bounds || {};
      const boundEntries = Object.entries(bounds);
      const varsText = Object.entries(dims).map(([k, v]) => `<strong>${esc(k)}</strong>: ${esc(v)}`).join(" · ");
      let boundsHtml = "";
      if (boundEntries.length) {
        const items = boundEntries.map(([v, b]) => {
          const parts = [];
          if (b.min !== undefined) parts.push(`${b.min_inclusive === false ? ">" : ">="} ${b.min}`);
          if (b.max !== undefined) parts.push(`${b.max_inclusive === false ? "<" : "<="} ${b.max}`);
          return `<li><strong>${esc(v)}</strong>: ${parts.join(", ") || "sonlu reel"}${b.note ? ` — ${esc(b.note)}` : ""}</li>`;
        }).join("");
        boundsHtml = `<details class="law-bounds"><summary>Tanım kümesi sınırları (${boundEntries.length} değişken)</summary><ul>${items}</ul></details>`;
      }
      const eqParts = (law.equation || "").split("=");
      const rhsExpr = eqParts.length > 1 ? eqParts.slice(1).join("=").trim() : "";
      const sourcesList = Array.isArray(law.sources) ? law.sources : [];
      const sourcesHtml = sourcesList.length
        ? `<details class="law-bounds"><summary>Doğrulama kaynakları (${sourcesList.length} bağımsız referans)</summary><ul>${sourcesList.map((s) => `<li>${esc(s)}</li>`).join("")}</ul></details>`
        : "";
      const isHi = lawState.highlightId && law.id === lawState.highlightId;
      return `
        <article class="law-entry${isHi ? " highlight" : ""}" data-law-id="${esc(law.id)}">
          <div class="law-head">
            <div class="law-name">${esc(law.name)}</div>
            <div class="law-meta">
              <code>${esc(law.id)}</code>
              <span class="tag accent">${esc(law.domain)}</span>
              <span class="tag">${esc(law.regime || "universal")}</span>
              ${sourcesList.length >= 3 ? `<span class="tag ok">${sourcesList.length} Kaynak</span>` : ""}
            </div>
          </div>
          <div class="formula-line">
            <div>
              <div class="katex-slot" data-tex="${esc(law.latex || law.equation)}" data-eq="${esc(law.equation)}"><code>${esc(law.equation)}</code></div>
              <div class="mono" style="font-size:0.72rem;color:var(--text-dim);margin-top:4px;">ASCII: ${esc(law.equation)}</div>
            </div>
            <button class="mini-btn" data-test-lhs="${esc(law.target)}" data-test-rhs="${esc(rhsExpr)}">Z⁷ Stüdyoda Test Et</button>
          </div>
          ${law.description ? `<p style="font-size:0.85rem;margin:6px 0;">${esc(law.description)}</p>` : ""}
          <div class="law-vars">Semboller: ${varsText}</div>
          ${boundsHtml}
          ${sourcesHtml}
        </article>
      `;
    }).join("");

    $$("#lawsList .katex-slot").forEach((slot) => renderLatexNode(slot, slot.dataset.tex, slot.dataset.eq));
    renderPager("#lawsPager", lawState.page, totalPages, (p) => {
      lawState.page = p; renderLaws(); window.scrollTo({ top: 0, behavior: "auto" });
    });
  }

  function renderPager(selector, page, total, onPage) {
    const el = $(selector);
    if (total <= 1) { el.innerHTML = ""; return; }
    el.innerHTML = `
      <button class="btn" ${page <= 1 ? "disabled" : ""} data-p="${page - 1}">← Önceki</button>
      <span>Sayfa ${page} / ${total}</span>
      <button class="btn" ${page >= total ? "disabled" : ""} data-p="${page + 1}">Sonraki →</button>
    `;
    el.querySelectorAll("button[data-p]").forEach((btn) => btn.addEventListener("click", () => onPage(Number(btn.dataset.p))));
  }

  // =========================================================================
  // Ontology & CODATA
  // =========================================================================
  const ontList = Object.entries(SSOT.ontology)
    .map(([sym, info]) => ({ sym, ...info }))
    .sort((a, b) => a.sym.localeCompare(b.sym));
  const ontState = { q: "", dim: "", page: 1, perPage: 40 };

  function initOntologyFilter() {
    const dims = Array.from(new Set(ontList.map((o) => o.dimension_key))).sort();
    const sel = $("#ontDimFilter");
    dims.forEach((d) => { const opt = document.createElement("option"); opt.value = d; opt.textContent = d; sel.appendChild(opt); });
  }

  function renderOntology() {
    const q = ontState.q.trim().toLowerCase();
    const filtered = ontList.filter((o) => {
      if (ontState.dim && o.dimension_key !== ontState.dim) return false;
      if (!q) return true;
      return (
        o.sym.toLowerCase().includes(q) ||
        o.dimension_key.toLowerCase().includes(q) ||
        (o.si_representation || "").toLowerCase().includes(q) ||
        (o.produced_by || []).some((l) => l.toLowerCase().includes(q)) ||
        (o.consumed_by || []).some((l) => l.toLowerCase().includes(q))
      );
    });
    const totalPages = Math.max(1, Math.ceil(filtered.length / ontState.perPage));
    if (ontState.page > totalPages) ontState.page = totalPages;
    const slice = filtered.slice((ontState.page - 1) * ontState.perPage, ontState.page * ontState.perPage);
    $("#ontCountLabel").textContent = `${filtered.length} / ${ontList.length} SSOT sembolü`;
    $("#ontTableBody").innerHTML = slice.map((o) => {
      const allLaws = Array.from(new Set([...(o.produced_by || []), ...(o.consumed_by || [])]));
      const links = allLaws.slice(0, 5).map((id) => `<button class="ref-link" data-jump-law="${esc(id)}">${esc(id)}</button>`).join("");
      const more = allLaws.length > 5 ? `<span class="mono" style="font-size:0.72rem;color:var(--text-dim);">+${allLaws.length - 5}</span>` : "";
      return `<tr><td class="mono"><strong>${esc(o.sym)}</strong></td><td class="mono">${esc(o.dimension_key)}</td><td class="mono">${esc(o.si_representation)}</td><td class="mono" style="color:var(--text-muted);">[${(o.z7_vector || ZERO_DIM).join(", ")}]</td><td>${links}${more}</td></tr>`;
    }).join("");
    renderPager("#ontPager", ontState.page, totalPages, (p) => { ontState.page = p; renderOntology(); window.scrollTo({ top: 0, behavior: "auto" }); });
  }

  const nistState = { q: "", page: 1, perPage: 40 };
  function renderNist() {
    const q = nistState.q.trim().toLowerCase();
    const filtered = SSOT.constants.filter((c) => !q || c.name.toLowerCase().includes(q) || (c.unit || "").toLowerCase().includes(q));
    const totalPages = Math.max(1, Math.ceil(filtered.length / nistState.perPage));
    if (nistState.page > totalPages) nistState.page = totalPages;
    const slice = filtered.slice((nistState.page - 1) * nistState.perPage, nistState.page * nistState.perPage);
    $("#nistCountLabel").textContent = `${filtered.length} / ${SSOT.constants.length} NIST CODATA 2022 sabiti`;
    $("#nistTableBody").innerHTML = slice.map((c) => `
      <tr>
        <td>${esc(c.name)}</td>
        <td class="mono"><strong>${esc(c.raw_str)}</strong></td>
        <td class="mono" style="color:var(--text-muted);">${esc(c.unit || "1 (boyutsuz)")}</td>
        <td><button class="mini-btn" data-copy="${esc(c.raw_str)}">Kopyala</button></td>
      </tr>
    `).join("");
    renderPager("#nistPager", nistState.page, totalPages, (p) => { nistState.page = p; renderNist(); window.scrollTo({ top: 0, behavior: "auto" }); });
  }

  // =========================================================================
  // Aerospace & TPS Materials + Interactive SVG Ashby Scatter Plot
  // =========================================================================
  const materialsEnriched = SSOT.materials.map((m) => ({
    ...m,
    breaking_length_km: (m.tensile_strength_mpa * 1e6) / (m.density_kg_m3 * G0 * 1000.0),
  }));

  const CAT_COLORS = {
    metal_alloy: "oklch(0.70 0.16 244)",
    composite: "oklch(0.74 0.16 155)",
    nanomaterial: "oklch(0.76 0.14 195)",
    superalloy: "oklch(0.72 0.16 305)",
    copper_superalloy: "oklch(0.75 0.16 55)",
    refractory_metal: "oklch(0.76 0.11 265)",
    uhtc_ceramic: "oklch(0.81 0.16 82)",
    ceramic_composite: "oklch(0.78 0.15 115)",
    technical_ceramic: "oklch(0.79 0.14 96)",
    semiconductor_optical: "oklch(0.76 0.15 215)",
    polymer: "oklch(0.74 0.14 330)",
    elastomer_seal: "oklch(0.72 0.15 355)",
    civil_structural: "oklch(0.71 0.08 75)",
    electrical_magnetic: "oklch(0.77 0.16 275)",
    biomedical_alloy: "oklch(0.76 0.14 175)",
    ablative_tps: "oklch(0.69 0.19 25)",
    insulator: "oklch(0.73 0.13 345)",
  };

  let ashbyYMode = "yield";
  let selectedMatId = "mat_alli_2195";

  function initMaterialsFilter() {
    const sel = $("#matCategory");
    SSOT.meta.material_categories.forEach((cat) => {
      const opt = document.createElement("option");
      opt.value = cat; opt.textContent = cat; sel.appendChild(opt);
    });
  }

  function renderAshbyHud(mat) {
    const hud = $("#ashbyHud");
    if (!hud || !mat) return;
    const tempC = Math.round(mat.max_temperature_k - 273.15);
    const col = CAT_COLORS[mat.category] || "var(--accent)";
    const srcArr = Array.isArray(mat.sources) ? mat.sources : [];
    const srcHtml = srcArr.length
      ? `<div style="margin-top:8px;padding-top:8px;border-top:1px solid var(--border-subtle);font-size:0.7rem;color:var(--text-dim);line-height:1.4;"><strong style="color:var(--ok);">✓ ${srcArr.length} Kaynak:</strong> ${srcArr.map((s) => esc(s)).join(" · ")}</div>`
      : "";
    hud.innerHTML = `
      <div>
        <div style="display:flex;align-items:center;gap:6px;margin-bottom:6px;">
          <span style="width:9px;height:9px;border-radius:50%;background:${col};display:inline-block;"></span>
          <span class="mono" style="font-size:0.72rem;color:var(--text-muted);">${esc(mat.category)}</span>
        </div>
        <div style="font-weight:650;font-size:0.9rem;color:var(--text);line-height:1.3;">${esc(mat.name)}</div>
        <div class="mono" style="font-size:0.7rem;color:var(--text-dim);margin-top:3px;">${esc(mat.id)}${mat.condition ? ` · ${esc(mat.condition)}` : ""}${mat.failure_mode ? ` · ${esc(mat.failure_mode)}` : ""}</div>
        <p style="font-size:0.78rem;margin:8px 0 0;line-height:1.45;">${esc(mat.description || "")}</p>
        ${srcHtml}
      </div>
      <div class="mono" style="font-size:0.75rem;display:grid;grid-template-columns:1fr 1fr;gap:6px;padding-top:10px;border-top:1px solid var(--border-subtle);">
        <div><span style="color:var(--text-dim);display:block;font-size:0.66rem;">Yoğunluk</span><strong>${mat.density_kg_m3} kg/m³</strong></div>
        <div><span style="color:var(--text-dim);display:block;font-size:0.66rem;">Akma</span><strong>${mat.yield_strength_mpa} MPa</strong></div>
        <div><span style="color:var(--text-dim);display:block;font-size:0.66rem;">T_max</span><strong>${mat.max_temperature_k.toFixed(0)} K (${tempC}°C)</strong></div>
        <div><span style="color:var(--text-dim);display:block;font-size:0.66rem;">Kopma L_b</span><strong>${mat.breaking_length_km.toFixed(1)} km</strong></div>
      </div>
    `;
  }

  function renderAshbyPlot(filtered) {
    const svg = $("#ashbySvg");
    if (!svg) return;
    const W = 680, H = 310, padL = 56, padR = 20, padT = 22, padB = 38;
    const plotW = W - padL - padR;
    const plotH = H - padT - padB;

    const minX = Math.log10(150);
    const maxX = Math.log10(22000);
    const xPos = (rho) => padL + ((Math.log10(Math.max(150, rho)) - minX) / (maxX - minX)) * plotW;

    const isYield = ashbyYMode === "yield";
    const minY = isYield ? Math.log10(1) : 250;
    const maxY = isYield ? Math.log10(200000) : 4400;
    const yPos = (m) => {
      const val = isYield ? Math.log10(Math.max(1, m.yield_strength_mpa)) : Math.min(4400, Math.max(250, m.max_temperature_k));
      return padT + plotH - ((val - minY) / (maxY - minY)) * plotH;
    };

    const xTicks = [200, 500, 1000, 2000, 5000, 10000, 20000];
    const yTicks = isYield ? [10, 100, 500, 1000, 5000, 50000, 150000] : [500, 1000, 2000, 3000, 4000];

    const gridLines = [
      ...xTicks.map((v) => {
        const x = xPos(v).toFixed(1);
        return `<line x1="${x}" y1="${padT}" x2="${x}" y2="${padT + plotH}" stroke="var(--border-subtle)" stroke-dasharray="2 3"/>
                <text x="${x}" y="${H - 16}" text-anchor="middle" fill="var(--text-dim)" font-family="JetBrains Mono,monospace" font-size="10">${v >= 1000 ? `${v / 1000}k` : v}</text>`;
      }),
      ...yTicks.map((v) => {
        const y = isYield
          ? (padT + plotH - ((Math.log10(v) - minY) / (maxY - minY)) * plotH).toFixed(1)
          : (padT + plotH - ((v - minY) / (maxY - minY)) * plotH).toFixed(1);
        return `<line x1="${padL}" y1="${y}" x2="${W - padR}" y2="${y}" stroke="var(--border-subtle)" stroke-dasharray="2 3"/>
                <text x="${padL - 8}" y="${Number(y) + 3}" text-anchor="end" fill="var(--text-dim)" font-family="JetBrains Mono,monospace" font-size="10">${v >= 1000 ? `${v / 1000}k` : v}</text>`;
      }),
    ].join("");

    const dots = filtered.map((m) => {
      const cx = xPos(m.density_kg_m3).toFixed(1);
      const cy = yPos(m).toFixed(1);
      const col = CAT_COLORS[m.category] || "var(--accent)";
      const sel = m.id === selectedMatId;
      return `
        <g class="ashby-dot" data-ashby-id="${esc(m.id)}" tabindex="0" role="button" aria-label="${esc(m.name)}">
          ${sel ? `<circle cx="${cx}" cy="${cy}" r="10" fill="${col}" fill-opacity="0.22" stroke="${col}" stroke-width="1.5"/>` : ""}
          <circle cx="${cx}" cy="${cy}" r="${sel ? 5.5 : 4.5}" fill="${col}" stroke="var(--bg-code)" stroke-width="1.2"/>
        </g>
      `;
    }).join("");

    svg.innerHTML = `
      ${gridLines}
      <text x="${padL + plotW / 2}" y="${H - 3}" text-anchor="middle" fill="var(--text-muted)" font-family="JetBrains Mono,monospace" font-size="10">Yoğunluk ρ (kg/m³, log ölçek)</text>
      <text x="14" y="${padT + plotH / 2}" text-anchor="middle" transform="rotate(-90 14 ${padT + plotH / 2})" fill="var(--text-muted)" font-family="JetBrains Mono,monospace" font-size="10">${isYield ? "Akma σ_y (MPa, log)" : "T_max (K)"}</text>
      ${dots}
    `;

    const current = filtered.find((m) => m.id === selectedMatId) || filtered[0] || materialsEnriched[0];
    if (current) renderAshbyHud(current);
  }

  function renderMaterials() {
    const q = $("#matSearch").value.trim().toLowerCase();
    const cat = $("#matCategory").value;
    const sortBy = $("#matSort").value;

    const filtered = materialsEnriched.filter((m) => {
      if (cat && m.category !== cat) return false;
      if (!q) return true;
      return m.name.toLowerCase().includes(q) || m.id.toLowerCase().includes(q) || (m.description || "").toLowerCase().includes(q) || (m.category || "").toLowerCase().includes(q);
    });

    filtered.sort((a, b) => {
      if (sortBy === "breaking_desc") return b.breaking_length_km - a.breaking_length_km;
      if (sortBy === "temp_desc") return b.max_temperature_k - a.max_temperature_k;
      if (sortBy === "yield_desc") return b.yield_strength_mpa - a.yield_strength_mpa;
      if (sortBy === "modulus_desc") return b.youngs_modulus_gpa - a.youngs_modulus_gpa;
      if (sortBy === "density_asc") return a.density_kg_m3 - b.density_kg_m3;
      return 0;
    });

    renderAshbyPlot(filtered);

    const maxLogBreaking = Math.max(...materialsEnriched.map((m) => Math.log10(m.breaking_length_km + 1)));
    $("#matCountLabel").textContent = `${filtered.length} / ${materialsEnriched.length} malzeme`;
    $("#matTableBody").innerHTML = filtered.map((m) => {
      const pct = Math.min(100, Math.max(4, (Math.log10(m.breaking_length_km + 1) / maxLogBreaking) * 100));
      const tempC = Math.round(m.max_temperature_k - 273.15);
      const hi = m.id === selectedMatId ? ' class="row-highlight"' : "";
      return `
        <tr data-mat-row="${esc(m.id)}"${hi}>
          <td><div style="font-weight:600;color:var(--text);">${esc(m.name)}</div><div style="font-size:0.78rem;color:var(--text-muted);margin-top:2px;">${esc(m.description || "")}</div></td>
          <td><span class="tag">${esc(m.category)}</span></td>
          <td class="mono">${m.density_kg_m3.toLocaleString("en-US", { maximumFractionDigits: 1 })}</td>
          <td class="mono">${m.yield_strength_mpa.toLocaleString("en-US")} / ${m.tensile_strength_mpa.toLocaleString("en-US")}</td>
          <td class="mono">${m.youngs_modulus_gpa.toLocaleString("en-US", { maximumFractionDigits: 1 })}</td>
          <td class="mono">${m.max_temperature_k.toFixed(0)} K <span style="color:var(--text-dim);">(${tempC}°C)</span></td>
          <td><div class="mono"><strong>${m.breaking_length_km.toLocaleString("en-US", { maximumFractionDigits: 1 })} km</strong></div><div class="meter"><div class="meter-fill" style="width:${pct.toFixed(1)}%"></div></div></td>
        </tr>
      `;
    }).join("");
  }

  function renderMcpTools() {
    $("#mcpToolsBody").innerHTML = MCP_TOOLS.map((t) => `
      <tr>
        <td class="mono"><strong>${esc(t.name)}</strong></td>
        <td class="mono" style="color:var(--text-muted);white-space:nowrap;">${esc(t.layer)}</td>
        <td>${esc(t.desc)}</td>
      </tr>
    `).join("");
  }

  // =========================================================================
  // Native HTML5 <dialog> Global Command Palette (967 SSOT Entries)
  // =========================================================================
  const CMD_INDEX = [
    ...MCP_TOOLS.map((t) => ({ kind: "MCP", title: t.name, sub: t.desc, sec: "mcp", key: t.name })),
    ...SSOT.laws.map((l) => ({ kind: "YASA", title: l.name, sub: `${l.id} · ${l.equation}`, sec: "laws", key: l.id })),
    ...materialsEnriched.map((m) => ({ kind: "MALZEME", title: m.name, sub: `${m.id} · ${m.yield_strength_mpa} MPa · ${m.max_temperature_k} K`, sec: "materials", key: m.id })),
    ...ontList.map((o) => ({ kind: "SEMBOL", title: o.sym, sub: `${o.dimension_key} · ${o.si_representation}`, sec: "ontology", key: o.sym })),
    ...SSOT.constants.map((c) => ({ kind: "CODATA", title: c.name, sub: `${c.raw_str} ${c.unit || ""}`, sec: "codata", key: c.name })),
  ];

  let cmdMatches = [];
  let cmdSelIdx = 0;

  function openCmdPalette() {
    const dlg = $("#cmdPalette");
    if (!dlg) return;
    if (!dlg.open) dlg.showModal();
    $("#cmdInput").value = "";
    renderCmdResults("");
    $("#cmdInput").focus();
  }

  function renderCmdResults(raw) {
    const q = raw.trim().toLowerCase();
    cmdMatches = q
      ? CMD_INDEX.filter((it) => it.title.toLowerCase().includes(q) || it.sub.toLowerCase().includes(q) || it.kind.toLowerCase().includes(q)).slice(0, 24)
      : CMD_INDEX.slice(0, 14);
    cmdSelIdx = 0;

    const box = $("#cmdResults");
    $("#cmdCount").textContent = q ? `${cmdMatches.length} sonuç · ${CMD_INDEX.length} kayıt` : `${CMD_INDEX.length} SSOT kaydı indekslendi`;

    if (!cmdMatches.length) {
      box.innerHTML = `<div style="padding:18px;text-align:center;color:var(--text-dim);font-size:0.84rem;">Eşleşen SSOT kaydı bulunamadı.</div>`;
      return;
    }
    box.innerHTML = cmdMatches.map((it, idx) => `
      <button class="cmd-item${idx === 0 ? " selected" : ""}" role="option" data-cmd-idx="${idx}">
        <div style="min-width:0;">
          <div style="font-weight:600;">${esc(it.title)}</div>
          <div class="cmd-item-sub">${esc(it.sub)}</div>
        </div>
        <span class="tag">${esc(it.kind)}</span>
      </button>
    `).join("");
  }

  function executeCmdItem(idx) {
    const it = cmdMatches[idx];
    if (!it) return;
    const dlg = $("#cmdPalette");
    if (dlg && dlg.open) dlg.close();

    if (it.kind === "YASA") {
      lawState.q = it.key; lawState.domain = ""; lawState.regime = ""; lawState.page = 1; lawState.highlightId = it.key;
      $("#lawSearch").value = it.key; $("#lawDomain").value = ""; $("#lawRegime").value = "";
      switchSection("laws");
    } else if (it.kind === "SEMBOL") {
      ontState.q = it.key; ontState.dim = ""; ontState.page = 1;
      $("#ontSearch").value = it.key; $("#ontDimFilter").value = "";
      renderOntology(); switchSection("ontology");
    } else if (it.kind === "CODATA") {
      nistState.q = it.key; nistState.page = 1;
      $("#nistSearch").value = it.key;
      renderNist(); switchSection("codata");
    } else if (it.kind === "MALZEME") {
      selectedMatId = it.key;
      $("#matSearch").value = ""; $("#matCategory").value = "";
      renderMaterials(); switchSection("materials");
    } else {
      switchSection(it.sec);
    }
  }

  // =========================================================================
  // Navigation (View Transitions API), Theme & Event Bindings
  // =========================================================================
  const VALID_SECTIONS = new Set(["overview", "sandbox", "laws", "ontology", "codata", "materials", "mcp", "benchmarks"]);
  let currentSection = "overview";
  const prefersReducedMotion = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  function applySectionSwap(secName, pushHash) {
    currentSection = secName;
    $$(".nav-pill").forEach((b) => b.classList.toggle("active", b.dataset.sec === secName));
    $$(".section").forEach((s) => s.classList.toggle("active", s.id === `sec-${secName}`));

    const activeNav = document.querySelector(`.nav-pill[data-sec="${secName}"]`);
    if (activeNav && activeNav.scrollIntoView) {
      activeNav.scrollIntoView({ behavior: "auto", inline: "center", block: "nearest" });
    }

    if (secName === "laws") renderLaws();
    if (secName === "sandbox") runSandbox();
    if (secName === "materials") renderMaterials();

    if (pushHash && window.history && window.history.replaceState) {
      try { window.history.replaceState(null, "", `#${secName}`); } catch (_) {}
    }
    window.scrollTo({ top: 0, behavior: "auto" });
  }

  function switchSection(secName, pushHash = true) {
    if (!VALID_SECTIONS.has(secName)) secName = "overview";
    if (document.startViewTransition && !prefersReducedMotion && secName !== currentSection) {
      document.startViewTransition(() => applySectionSwap(secName, pushHash));
    } else {
      applySectionSwap(secName, pushHash);
    }
  }

  let currentTheme = readStorage("citadel_theme_v2", "dark");
  function applyTheme(theme) {
    currentTheme = theme === "light" ? "light" : "dark";
    writeStorage("citadel_theme_v2", currentTheme);
    document.documentElement.setAttribute("data-theme", currentTheme);
    const btn = $("#themeToggle");
    if (btn) btn.textContent = currentTheme === "dark" ? "☀" : "☾";
  }

  function bindEvents() {
    $$(".nav-pill").forEach((btn) => btn.addEventListener("click", () => switchSection(btn.dataset.sec)));
    $("#themeToggle").addEventListener("click", () => applyTheme(currentTheme === "dark" ? "light" : "dark"));
    $("#cmdOpenBtn").addEventListener("click", openCmdPalette);
    $("#heroCmdBtn").addEventListener("click", openCmdPalette);

    // Hero live console
    $("#heroLhs").addEventListener("input", runHeroConsole);
    $("#heroRhs").addEventListener("input", runHeroConsole);
    $$("#heroPresets .preset-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        $("#heroLhs").value = btn.dataset.hlhs;
        $("#heroRhs").value = btn.dataset.hrhs;
        runHeroConsole();
      });
    });
    $("#heroOpenSandbox").addEventListener("click", () => {
      $("#lhsInput").value = $("#heroLhs").value;
      $("#rhsInput").value = $("#heroRhs").value;
      switchSection("sandbox");
    });

    // Hero quickrun toggle
    $$("[data-quickrun]").forEach((btn) => {
      btn.addEventListener("click", () => {
        $$("[data-quickrun]").forEach((b) => b.classList.toggle("active", b === btn));
        $("#heroQuickCmd").textContent = QUICKRUN_COMMANDS[btn.dataset.quickrun];
      });
    });
    $("#heroCopyBtn").addEventListener("click", () => copyText($("#heroQuickCmd").textContent, "Komut kopyalandı"));

    // Ashby Y toggle & hover/click
    $$("[data-ashby-y]").forEach((btn) => {
      btn.addEventListener("click", () => {
        $$("[data-ashby-y]").forEach((b) => b.classList.toggle("active", b === btn));
        ashbyYMode = btn.dataset.ashbyY;
        renderMaterials();
      });
    });

    $("#ashbySvg").addEventListener("mouseover", (e) => {
      const dot = e.target.closest("[data-ashby-id]");
      if (!dot) return;
      const mat = materialsEnriched.find((m) => m.id === dot.dataset.ashbyId);
      if (mat) renderAshbyHud(mat);
    });

    $("#ashbySvg").addEventListener("click", (e) => {
      const dot = e.target.closest("[data-ashby-id]");
      if (!dot) return;
      selectedMatId = dot.dataset.ashbyId;
      renderMaterials();
      const row = document.querySelector(`tr[data-mat-row="${CSS.escape(selectedMatId)}"]`);
      if (row) row.scrollIntoView({ behavior: prefersReducedMotion ? "auto" : "smooth", block: "center" });
    });

    // Global delegated clicks
    document.addEventListener("click", (e) => {
      const tabBtn = e.target.closest(".tab-btn[data-tab]");
      if (tabBtn) {
        const bar = tabBtn.closest(".tab-bar");
        if (bar) {
          bar.querySelectorAll(".tab-btn").forEach((b) => {
            const active = b === tabBtn;
            b.classList.toggle("active", active);
            b.setAttribute("aria-selected", String(active));
            const panel = document.getElementById(`tab-${b.dataset.tab}`);
            if (panel) panel.classList.toggle("active", active);
          });
        }
        return;
      }

      const stepBtn = e.target.closest("[data-scenario-step]");
      if (stepBtn && !stepBtn.disabled) { renderScenario(Number(stepBtn.dataset.scenarioStep)); return; }

      const copyTarget = e.target.closest("[data-copy-target]");
      if (copyTarget) {
        const node = document.getElementById(copyTarget.dataset.copyTarget);
        if (node) copyText(node.textContent, "Yapılandırma kopyalandı");
        return;
      }

      const copyBtn = e.target.closest("[data-copy]");
      if (copyBtn) { copyText(copyBtn.dataset.copy, "Kopyalandı"); return; }

      const navBtn = e.target.closest("[data-nav]");
      if (navBtn) { e.preventDefault(); switchSection(navBtn.dataset.nav); return; }

      const testBtn = e.target.closest("[data-test-lhs]");
      if (testBtn) {
        $("#lhsInput").value = testBtn.dataset.testLhs;
        $("#rhsInput").value = testBtn.dataset.testRhs;
        switchSection("sandbox");
        return;
      }

      const lawRef = e.target.closest("[data-jump-law]");
      if (lawRef) {
        const id = lawRef.dataset.jumpLaw;
        lawState.q = id; lawState.domain = ""; lawState.regime = ""; lawState.page = 1; lawState.highlightId = id;
        $("#lawSearch").value = id; $("#lawDomain").value = ""; $("#lawRegime").value = "";
        switchSection("laws");
        return;
      }

      const cmdItem = e.target.closest("[data-cmd-idx]");
      if (cmdItem) { executeCmdItem(Number(cmdItem.dataset.cmdIdx)); return; }
    });

    // Command palette events
    const dlg = $("#cmdPalette");
    dlg.addEventListener("click", (e) => { if (e.target === dlg) dlg.close(); });
    $("#cmdInput").addEventListener("input", (e) => renderCmdResults(e.target.value));
    $("#cmdInput").addEventListener("keydown", (e) => {
      if (!cmdMatches.length) return;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        cmdSelIdx = (cmdSelIdx + 1) % cmdMatches.length;
        $$("#cmdResults .cmd-item").forEach((el, i) => el.classList.toggle("selected", i === cmdSelIdx));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        cmdSelIdx = (cmdSelIdx - 1 + cmdMatches.length) % cmdMatches.length;
        $$("#cmdResults .cmd-item").forEach((el, i) => el.classList.toggle("selected", i === cmdSelIdx));
      } else if (e.key === "Enter") {
        e.preventDefault();
        executeCmdItem(cmdSelIdx);
      }
    });

    // Sandbox
    $("#runSandboxBtn").addEventListener("click", runSandbox);
    $("#lhsInput").addEventListener("keydown", (e) => { if (e.key === "Enter") runSandbox(); });
    $("#rhsInput").addEventListener("keydown", (e) => { if (e.key === "Enter") runSandbox(); });
    $$("#sandboxPresets .preset-btn").forEach((btn) => {
      btn.addEventListener("click", () => {
        $("#lhsInput").value = btn.dataset.lhs;
        $("#rhsInput").value = btn.dataset.rhs;
        runSandbox();
      });
    });

    // Filters
    $("#lawSearch").addEventListener("input", (e) => { lawState.q = e.target.value; lawState.page = 1; lawState.highlightId = ""; renderLaws(); });
    $("#lawDomain").addEventListener("change", (e) => { lawState.domain = e.target.value; lawState.page = 1; renderLaws(); });
    $("#lawRegime").addEventListener("change", (e) => { lawState.regime = e.target.value; lawState.page = 1; renderLaws(); });
    $("#lawReset").addEventListener("click", () => {
      lawState.q = ""; lawState.domain = ""; lawState.regime = ""; lawState.page = 1; lawState.highlightId = "";
      $("#lawSearch").value = ""; $("#lawDomain").value = ""; $("#lawRegime").value = "";
      renderLaws();
    });

    $("#ontSearch").addEventListener("input", (e) => { ontState.q = e.target.value; ontState.page = 1; renderOntology(); });
    $("#ontDimFilter").addEventListener("change", (e) => { ontState.dim = e.target.value; ontState.page = 1; renderOntology(); });
    $("#ontReset").addEventListener("click", () => { ontState.q = ""; ontState.dim = ""; ontState.page = 1; $("#ontSearch").value = ""; $("#ontDimFilter").value = ""; renderOntology(); });

    $("#nistSearch").addEventListener("input", (e) => { nistState.q = e.target.value; nistState.page = 1; renderNist(); });
    $("#nistReset").addEventListener("click", () => { nistState.q = ""; nistState.page = 1; $("#nistSearch").value = ""; renderNist(); });

    $("#matSearch").addEventListener("input", renderMaterials);
    $("#matCategory").addEventListener("change", renderMaterials);
    $("#matSort").addEventListener("change", renderMaterials);
    $("#matReset").addEventListener("click", () => { $("#matSearch").value = ""; $("#matCategory").value = ""; $("#matSort").value = "breaking_desc"; renderMaterials(); });

    // Global keyboard shortcuts (Ctrl/Cmd+K & /)
    window.addEventListener("hashchange", () => {
      const h = (window.location && window.location.hash || "").replace(/^#/, "");
      if (VALID_SECTIONS.has(h)) switchSection(h, false);
    });

    document.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        openCmdPalette();
        return;
      }
      const tag = document.activeElement ? document.activeElement.tagName : "";
      if (e.key === "/" && !["INPUT", "TEXTAREA", "SELECT"].includes(tag) && !dlg.open) {
        e.preventDefault();
        switchSection("laws");
        $("#lawSearch").focus();
      }
    });
  }

  // Init
  applyTheme(currentTheme);
  syncCounts();
  initLawFilters();
  initOntologyFilter();
  initMaterialsFilter();
  runHeroConsole();
  renderScenario(0);
  renderLatencyChart();
  renderLaws();
  renderOntology();
  renderNist();
  renderMaterials();
  renderMcpTools();
  runSandbox();
  bindEvents();

  const initialHash = (window.location && window.location.hash || "").replace(/^#/, "");
  if (VALID_SECTIONS.has(initialHash) && initialHash !== "overview") applySectionSwap(initialHash, false);

  window.addEventListener("load", () => {
    $$("#lawsList .katex-slot").forEach((slot) => renderLatexNode(slot, slot.dataset.tex, slot.dataset.eq));
  });
})();
