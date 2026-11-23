# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
# Matematik Çekirdeği — Boyut Analizi Kalkanı (Dimensional Analysis Shield)

"""
7 Temel SI Boyutu (Z^7 Serbest Abelyen Grubu) ve Sembolik AST Boyut Çıkarımı.
İcat motorunda türetilen veya veritabanına eklenen hiçbir formül,
tam sembolik boyutsal tutarlılık denetiminden (dim(LHS) == dim(RHS))
geçmeden kabul edilmiş fizik tabanına eklenemez.
"""

import ast
import re
from typing import Tuple, Dict, Any, Optional
import sympy as sp

_SAFE_IDENT_RE = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")
_BLOCKED_NAMES = frozenset({
    "eval", "exec", "open", "compile", "getattr", "setattr", "delattr",
    "globals", "locals", "vars", "dir", "type", "object", "input",
    "breakpoint", "help", "exit", "quit", "lambda", "import", "super",
})
_SAFE_MATH_FUNCS = {
    "sin": sp.sin,
    "cos": sp.cos,
    "tan": sp.tan,
    "asin": sp.asin,
    "acos": sp.acos,
    "atan": sp.atan,
    "sinh": sp.sinh,
    "cosh": sp.cosh,
    "tanh": sp.tanh,
    "exp": sp.exp,
    "log": sp.log,
    "ln": sp.log,
    "sqrt": sp.sqrt,
    "abs": sp.Abs,
    "Abs": sp.Abs,
}


def safe_parse_math_expr(
    expr_str: str,
    allowed_symbols: Optional[Dict[str, sp.Symbol]] = None,
) -> sp.Expr:
    """
    Sıfır-eval (eval/sympify çağrısız) güvenli matematik ifadesi ayrıştırıcısı.
    Python'un ast.parse(..., mode='eval') ayrıştırıcısını yalnızca beyaz-listeli
    aritmetik işleçler ve temel matematik fonksiyonlarıyla SymPy AST'sine çevirir;
    öznitelik erişimi (.), dunder (__), lambda ve rastgele çağrıları kesin reddeder.
    """
    if not isinstance(expr_str, str):
        raise ValueError("İfade metin (str) olmalıdır.")
    cleaned = expr_str.strip()
    if not cleaned:
        raise ValueError("Boş matematik ifadesi ayrıştırılamaz.")
    if len(cleaned) > 2048:
        raise ValueError("İfade uzunluğu güvenlik sınırını (2048 karakter) aşıyor.")
    if "__" in cleaned or ";" in cleaned or "\n" in cleaned or "\r" in cleaned:
        raise ValueError("Güvenlik ihlali: ifadede yasaklı karakter veya dunder dizisi tespit edildi.")

    normalized = cleaned.replace("^", "**")
    try:
        tree = ast.parse(normalized, mode="eval")
    except SyntaxError as exc:
        raise ValueError(f"Sözdizimi hatası: {exc}") from exc

    def _convert(node: ast.AST) -> sp.Expr:
        if isinstance(node, ast.Expression):
            return _convert(node.body)
        if isinstance(node, ast.Constant):
            val = node.value
            if isinstance(val, bool) or not isinstance(val, (int, float)):
                raise ValueError(f"Desteklenmeyen sabit türü: {type(val).__name__}")
            return sp.Integer(val) if isinstance(val, int) else sp.Float(val)
        if isinstance(node, ast.Name):
            ident = node.id
            if ident.startswith("_") or ident in _BLOCKED_NAMES or not _SAFE_IDENT_RE.match(ident):
                raise ValueError(f"Güvenlik ihlali: yasaklı veya geçersiz sembol adı '{ident}'.")
            if allowed_symbols is not None and ident in allowed_symbols:
                return allowed_symbols[ident]
            if ident == "pi":
                return sp.pi
            return sp.Symbol(ident)
        if isinstance(node, ast.UnaryOp):
            if isinstance(node.op, ast.UAdd):
                return _convert(node.operand)
            if isinstance(node.op, ast.USub):
                if isinstance(node.operand, ast.Constant) and not isinstance(node.operand.value, bool):
                    val = node.operand.value
                    if isinstance(val, int):
                        return sp.Integer(-val)
                    if isinstance(val, float):
                        return sp.Float(-val)
                return sp.Mul(sp.Integer(-1), _convert(node.operand), evaluate=False)
            raise ValueError(f"Desteklenmeyen tekli işleç: {type(node.op).__name__}")
        if isinstance(node, ast.BinOp):
            left = _convert(node.left)
            right = _convert(node.right)
            if isinstance(node.op, ast.Add):
                return sp.Add(left, right, evaluate=False)
            if isinstance(node.op, ast.Sub):
                return sp.Add(left, sp.Mul(sp.Integer(-1), right, evaluate=False), evaluate=False)
            if isinstance(node.op, ast.Mult):
                return sp.Mul(left, right, evaluate=False)
            if isinstance(node.op, ast.Div):
                if isinstance(left, sp.Integer) and isinstance(right, sp.Integer) and int(right) != 0:
                    return sp.Rational(int(left), int(right))
                return sp.Mul(left, sp.Pow(right, sp.Integer(-1), evaluate=False), evaluate=False)
            if isinstance(node.op, ast.Pow):
                return sp.Pow(left, right, evaluate=False)
            raise ValueError(f"Desteklenmeyen ikili işleç: {type(node.op).__name__}")
        if isinstance(node, ast.Call):
            if not isinstance(node.func, ast.Name) or node.keywords:
                raise ValueError("Güvenlik ihlali: öznitelik veya anahtar-kelime çağrıları yasaktır.")
            fname = node.func.id
            if fname not in _SAFE_MATH_FUNCS:
                raise ValueError(f"Güvenlik ihlali: izin listesinde olmayan fonksiyon '{fname}'.")
            args = [_convert(a) for a in node.args]
            if fname == "sqrt":
                if len(args) != 1:
                    raise ValueError("sqrt() tam olarak 1 argüman alır.")
                return sp.Pow(args[0], sp.Rational(1, 2), evaluate=False)
            if fname in ("log", "ln"):
                if len(args) not in (1, 2):
                    raise ValueError(f"{fname}() 1 veya 2 argüman alır.")
                return sp.log(*args)
            if len(args) != 1:
                raise ValueError(f"{fname}() tam olarak 1 argüman alır.")
            return _SAFE_MATH_FUNCS[fname](args[0])
        raise ValueError(f"Güvenlik/sözdizimi ihlali: desteklenmeyen ifade düğümü '{type(node).__name__}'.")

    return _convert(tree)


class Dimension:
    """
    7-Temel SI Boyut Vektörü:
    [M] Kütle (kg)
    [L] Uzunluk (m)
    [T] Zaman (s)
    [I] Elektrik Akımı (A)
    [Theta] Sıcaklık (K)
    [N] Madde Miktarı (mol)
    [J] Işık Şiddeti (cd)
    """
    __slots__ = ("m", "l", "t", "i", "theta", "n", "j")

    def __init__(self, m=0, l=0, t=0, i=0, theta=0, n=0, j=0):
        self.m = int(m)
        self.l = int(l)
        self.t = int(t)
        self.i = int(i)
        self.theta = int(theta)
        self.n = int(n)
        self.j = int(j)

    def as_tuple(self) -> Tuple[int, int, int, int, int, int, int]:
        return (self.m, self.l, self.t, self.i, self.theta, self.n, self.j)

    def is_dimensionless(self) -> bool:
        return self.as_tuple() == (0, 0, 0, 0, 0, 0, 0)

    def __eq__(self, other: Any) -> bool:
        if not isinstance(other, Dimension):
            return False
        return self.as_tuple() == other.as_tuple()

    def __hash__(self) -> int:
        return hash(self.as_tuple())

    def __mul__(self, other: "Dimension") -> "Dimension":
        return Dimension(
            self.m + other.m,
            self.l + other.l,
            self.t + other.t,
            self.i + other.i,
            self.theta + other.theta,
            self.n + other.n,
            self.j + other.j,
        )

    def __truediv__(self, other: "Dimension") -> "Dimension":
        return Dimension(
            self.m - other.m,
            self.l - other.l,
            self.t - other.t,
            self.i - other.i,
            self.theta - other.theta,
            self.n - other.n,
            self.j - other.j,
        )

    def __pow__(self, power: int) -> "Dimension":
        return Dimension(
            self.m * power,
            self.l * power,
            self.t * power,
            self.i * power,
            self.theta * power,
            self.n * power,
            self.j * power,
        )

    def rational_pow(self, p: int, q: int) -> "Dimension":
        """
        Rasyonel üs (p/q) altında Z^7 kafes bölünebilirliğini denetler.
        Örn: sqrt(L^2 * T^-2) -> L^1 * T^-1; fakat sqrt(L^1) -> ValueError!
        """
        if q == 0:
            raise ValueError("Rasyonel üs paydası 0 olamaz.")
        comps = [self.m * p, self.l * p, self.t * p, self.i * p, self.theta * p, self.n * p, self.j * p]
        for c in comps:
            if c % q != 0:
                raise ValueError(f"Kesirli boyut hatası: [{self}] ifadesinin ({p}/{q}) üssü Z^7 kafesinde tam bölünmüyor!")
        return Dimension(*(c // q for c in comps))

    def __repr__(self) -> str:
        parts = []
        labels = [("M", self.m), ("L", self.l), ("T", self.t),
                  ("I", self.i), ("Theta", self.theta), ("N", self.n), ("J", self.j)]
        for sym, exp in labels:
            if exp == 1:
                parts.append(sym)
            elif exp != 0:
                parts.append(f"{sym}^{exp}")
        return " ".join(parts) if parts else "1 (Boyutsuz)"


# Standart Kanonik Fiziksel Boyut Kütüphanesi
DIMENSIONS: Dict[str, Dimension] = {
    # Temel
    "dimensionless": Dimension(),
    "mass": Dimension(m=1),
    "length": Dimension(l=1),
    "time": Dimension(t=1),
    "current": Dimension(i=1),
    "temperature": Dimension(theta=1),
    "amount": Dimension(n=1),
    "luminous": Dimension(j=1),

    # Mekanik & Dinamik
    "area": Dimension(l=2),
    "volume": Dimension(l=3),
    "velocity": Dimension(l=1, t=-1),
    "acceleration": Dimension(l=1, t=-2),
    "force": Dimension(m=1, l=1, t=-2),
    "momentum": Dimension(m=1, l=1, t=-1),
    "energy": Dimension(m=1, l=2, t=-2),
    "power": Dimension(m=1, l=2, t=-3),
    "pressure": Dimension(m=1, l=-1, t=-2),
    "stress": Dimension(m=1, l=-1, t=-2),
    "density": Dimension(m=1, l=-3),
    "mass_flow_rate": Dimension(m=1, t=-1),
    "angular_velocity": Dimension(t=-1),
    "moment_of_inertia": Dimension(m=1, l=2),

    # Uzay & Tahrik
    "specific_impulse_s": Dimension(t=1),            # saniye cinsinden Isp
    "specific_impulse_v": Dimension(l=1, t=-1),      # efektif çıkış hızı (ve = g0 * Isp)
    "specific_energy": Dimension(l=2, t=-2),         # J/kg (Delta-v karesi veya yörünge enerjisi)
    "specific_strength": Dimension(l=2, t=-2),       # Pa / (kg/m3) = m2/s2 (Tether / Karbon Nanotüp)
    "gravitational_parameter": Dimension(l=3, t=-2), # mu = GM (m3/s2)

    # Termodinamik & Işınım
    "heat_capacity": Dimension(m=1, l=2, t=-2, theta=-1),
    "specific_heat": Dimension(l=2, t=-2, theta=-1),
    "gas_constant": Dimension(l=2, t=-2, theta=-1),
    "radiation_flux": Dimension(m=1, t=-3),          # W/m2 (Poynting / Güneş sabiti)
    "stefan_boltzmann_const": Dimension(m=1, t=-3, theta=-4),  # W / (m^2 * K^4)

    # Elektromanyetizma
    "electric_charge": Dimension(t=1, i=1),          # Coulomb
    "voltage": Dimension(m=1, l=2, t=-3, i=-1),      # Volt
    "electric_field": Dimension(m=1, l=1, t=-3, i=-1),
    "magnetic_field": Dimension(m=1, t=-2, i=-1),    # Tesla
    "permeability": Dimension(m=1, l=1, t=-2, i=-2), # H/m or N/A^2 (mu_0)
    "permittivity": Dimension(m=-1, l=-3, t=4, i=2), # F/m (eps_0)
    "inductance_gradient": Dimension(m=1, l=1, t=-2, i=-2),  # L' = dL/dx (H/m)
    "magnetic_pressure": Dimension(m=1, l=-1, t=-2), # B^2 / (2*mu0) -> Pascal
    "magnetic_flux": Dimension(m=1, l=2, t=-2, i=-1),# Weber (V*s)
    "resistance": Dimension(m=1, l=2, t=-3, i=-2),   # Ohm
    "electrical_resistivity": Dimension(m=1, l=3, t=-3, i=-2),     # Ohm*m
    "electrical_conductivity": Dimension(m=-1, l=-3, t=3, i=2),    # S/m
    "current_density": Dimension(l=-2, i=1),         # A/m^2
    "charge_density": Dimension(l=-3, t=1, i=1),     # C/m^3
    "inductance": Dimension(m=1, l=2, t=-2, i=-2),   # Henry
    "capacitance": Dimension(m=-1, l=-2, t=4, i=2),  # Farad

    # Akışkanlar, Isı Aktarımı, Kırılma Mekaniği, Kuantum & Gerçek-Zamanlı Sistemler
    "frequency": Dimension(t=-1),                    # Hz (1/s)
    "rate": Dimension(t=-1),                         # Kuyruk/olay hızı (1/s)
    "thermal_conductivity": Dimension(m=1, l=1, t=-3, theta=-1),   # W/(m*K)
    "heat_transfer_coeff": Dimension(m=1, t=-3, theta=-1),         # W/(m^2*K)
    "thermal_expansion_coeff": Dimension(theta=-1),                # 1/K
    "dynamic_viscosity": Dimension(m=1, l=-1, t=-1),               # Pa*s
    "kinematic_viscosity": Dimension(l=2, t=-1),                   # m^2/s
    "thermal_diffusivity": Dimension(l=2, t=-1),                   # m^2/s
    "mass_diffusivity": Dimension(l=2, t=-1),                      # m^2/s
    "surface_energy": Dimension(m=1, t=-2),                        # J/m^2 (Griffith/Irwin G_Ic)
    "surface_tension": Dimension(m=1, t=-2),                       # N/m
    "spring_stiffness": Dimension(m=1, t=-2),                      # N/m
    "molar_mass": Dimension(m=1, n=-1),                            # kg/mol
    "molar_gas_constant": Dimension(m=1, l=2, t=-2, theta=-1, n=-1), # J/(mol*K)
    "entropy": Dimension(m=1, l=2, t=-2, theta=-1),                # J/K
    "boltzmann_const": Dimension(m=1, l=2, t=-2, theta=-1),        # J/K (k_B)
    "specific_entropy": Dimension(l=2, t=-2, theta=-1),            # J/(kg*K)
    "angular_momentum": Dimension(m=1, l=2, t=-1),                 # kg*m^2/s
    "specific_angular_momentum": Dimension(l=2, t=-1),             # m^2/s
    "torque": Dimension(m=1, l=2, t=-2),                           # N*m
    "impulse": Dimension(m=1, l=1, t=-1),                          # N*s
    "wavenumber": Dimension(l=-1),                                 # 1/m
    "number_density": Dimension(l=-3),                             # 1/m^3
    "action": Dimension(m=1, l=2, t=-1),                           # J*s (Planck h, hbar)
    "volumetric_flow_rate": Dimension(l=3, t=-1),                  # m^3/s
    "area_moment_of_inertia": Dimension(l=4),                      # m^4 (Kesit atalet momenti I, J)
    "gravitational_constant": Dimension(m=-1, l=3, t=-2),          # m^3/(kg*s^2) (Newton G)
    "areal_density": Dimension(m=1, l=-2),                         # kg/m^2 (Yüzeysel kütle / balistik katsayı)
    "linear_density": Dimension(m=1, l=-1),                        # kg/m (Birim boy kütlesi)
    "compressibility": Dimension(m=-1, l=1, t=2),                  # 1/Pa
}


def check_dimensional_consistency(lhs_dim: Dimension, rhs_dim: Dimension) -> Tuple[bool, str]:
    """
    Sol ve sağ taraf boyutlarının matematiksel eşitliğini denetler.
    Eşit değilse açık hata raporu üretir.
    """
    if lhs_dim == rhs_dim:
        return True, "Boyutsal denklik doğrulandı."
    return False, f"Boyut Hatası: Sol taraf [{lhs_dim}] != Sağ taraf [{rhs_dim}]"


def infer_sympy_dimension(expr: sp.Expr, var_dims: Dict[str, Dimension]) -> Dimension:
    """
    Bir SymPy sembolik ifade ağacının (AST) fiziksel boyut vektörünü Z^7 üzerinde
    özyinelemeli (recursive) olarak hesaplar.
    Kurallar:
      1. Sayısal sabitler boyutsuzdur: dim(c) = 0.
      2. Toplama (Add): Tüm terimlerin boyutları birbirine eşit olmalıdır: dim(A + B) => dim(A) == dim(B).
      3. Çarpma (Mul): Terimlerin boyut vektörleri toplanır: dim(A * B) = dim(A) + dim(B).
      4. Üs (Pow): Üs boyutsuz olmalıdır; taban boyutu rasyonel üsle ölçeklenir.
      5. Aşkın fonksiyonlar (log, exp, sin, cos, tan): Argüman boyutsuz (1) olmak zorundadır ve sonuç boyutsuzdur.
    """
    if expr.is_Number or expr in (sp.pi, sp.E):
        return Dimension()

    if expr.is_Symbol:
        name = str(expr)
        if name not in var_dims:
            raise ValueError(f"Sembol '{name}' için boyut tanımı bulunamadı!")
        return var_dims[name]

    if expr.is_Add:
        arg_dims = [infer_sympy_dimension(arg, var_dims) for arg in expr.args]
        first = arg_dims[0]
        for idx, d in enumerate(arg_dims[1:], 1):
            if d != first:
                raise ValueError(
                    f"Toplamsal boyut homojenliği ihlali ({expr}): "
                    f"Terim 0 [{first}] != Terim {idx} [{d}]"
                )
        return first

    if expr.is_Mul:
        total = Dimension()
        for arg in expr.args:
            total = total * infer_sympy_dimension(arg, var_dims)
        return total

    if expr.is_Pow:
        base, exp = expr.as_base_exp()
        base_dim = infer_sympy_dimension(base, var_dims)
        exp_dim = infer_sympy_dimension(exp, var_dims)
        if not exp_dim.is_dimensionless():
            raise ValueError(f"Üs ifadesi ({exp}) boyutsuz olmalıdır, bulunan: [{exp_dim}]")
        if base_dim.is_dimensionless():
            return Dimension()
        rat = sp.nsimplify(exp, rational=True)
        if not getattr(rat, "is_Rational", False):
            try:
                rat = sp.nsimplify(float(exp.evalf()), rational=True)
            except Exception:
                pass
        if getattr(rat, "is_Rational", False):
            return base_dim.rational_pow(int(rat.p), int(rat.q))
        raise ValueError(f"Boyutlu tabanın ({base}) üssü ({exp}) rasyonel olmalıdır.")

    # Aşkın fonksiyonlar ve mutlak değer
    func_name = expr.func.__name__.lower()
    if func_name == "abs" and len(expr.args) == 1:
        return infer_sympy_dimension(expr.args[0], var_dims)
    if func_name in ("log", "ln", "exp", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh"):
        for arg in expr.args:
            arg_dim = infer_sympy_dimension(arg, var_dims)
            if not arg_dim.is_dimensionless():
                raise ValueError(
                    f"Aşkın fonksiyon '{func_name}({arg})' argümanı boyutsuz olmalıdır, bulunan: [{arg_dim}]"
                )
        return Dimension()

    raise ValueError(f"Desteklenmeyen AST düğüm türü: {expr} ({type(expr)})")


def verify_law_dimensional_homogeneity(law: Dict[str, Any]) -> Tuple[bool, str]:
    """
    Bir kanonik yasanın sol taraf (target) boyutu ile sağ taraf (RHS AST) boyutunu
    cebirsel olarak karşılaştırır: dim(LHS) == dim(RHS).
    """
    dims_map = law.get("dimensions", {})
    var_dims: Dict[str, Dimension] = {}
    for var_name, dim_key in dims_map.items():
        if dim_key not in DIMENSIONS:
            return False, f"Tanımsız boyut anahtarı: {var_name} -> {dim_key}"
        var_dims[var_name] = DIMENSIONS[dim_key]

    target = law.get("target", "")
    if target not in var_dims:
        return False, f"Hedef değişken '{target}' için boyut tanımlanmamış."

    eq_str = law.get("equation", "")
    if "=" not in eq_str:
        return False, "Denklem '=' içermiyor."

    _, rhs_str = eq_str.split("=", 1)
    try:
        local_symbols = {name: sp.Symbol(name) for name in var_dims}
        rhs_expr = safe_parse_math_expr(rhs_str.strip(), allowed_symbols=local_symbols)
        rhs_dim = infer_sympy_dimension(rhs_expr, var_dims)
        lhs_dim = var_dims[target]
        if lhs_dim != rhs_dim:
            return False, f"{law.get('id')}: Boyut Uyuşmazlığı! LHS({target})=[{lhs_dim}] != RHS({rhs_str.strip()})=[{rhs_dim}]"
        return True, f"LHS=[{lhs_dim}] == RHS=[{rhs_dim}]"
    except Exception as e:
        return False, f"{law.get('id')}: {e}"
