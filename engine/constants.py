# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
# Matematik Çekirdeği — Temel Fiziksel ve Astronomik Sabitler

"""
NIST CODATA ve Uluslararası Astronomi Birliği (IAU) Kanonik Sabitleri.
Tüm hesaplamalarda tekillik kaynağı (SSOT) olarak kullanılır.
"""

from typing import Dict, Any, Optional

class PhysicalConstants:
    """Temel Fiziksel ve Astronomik Sabitler Deposu."""

    # Evrensel Temel Sabitler (NIST CODATA 2022 & 2019 SI Tam Sabitler)
    c: float = 299792458.0                    # Işık hızı (m/s) [2019 SI Tam]
    G: float = 6.67430e-11                    # Evrensel Çekim Sabiti (m^3 kg^-1 s^-2) [CODATA 2022]
    h: float = 6.62607015e-34                 # Planck sabiti (J s) [2019 SI Tam]
    hbar: float = 1.0545718176461565e-34      # İndirgenmiş Planck sabiti h/(2*pi) (J s) [Tam]
    k_B: float = 1.380649e-23                 # Boltzmann sabiti (J/K) [2019 SI Tam]
    e: float = 1.602176634e-19                # Temel yük (C) [2019 SI Tam]
    alpha: float = 7.2973525643e-3            # İnce yapı sabiti [CODATA 2022]
    mu_0: float = 1.25663706127e-6            # Vakum manyetik geçirgenliği (N/A^2) [CODATA 2022]
    eps_0: float = 8.8541878188e-12           # Vakum elektriksel geçirgenliği (F/m) [CODATA 2022]
    sigma_SB: float = 5.6703744191844294e-8   # Stefan-Boltzmann sabiti (W m^-2 K^-4) [2019 SI Tam]
    N_A: float = 6.02214076e23                # Avogadro sayısı (mol^-1) [2019 SI Tam]
    R_gas: float = 8.31446261815324           # Evrensel gaz sabiti N_A*k_B (J mol^-1 K^-1) [Tam]
    g0: float = 9.80665                       # Standart yerçekimi ivmesi (m/s^2) [CGPM Tam]

    # Dünya ve Yörünge Parametreleri
    M_earth: float = 5.97219e24               # Dünya Kütlesi (kg)
    R_earth: float = 6371000.0                # Dünya Ortalama Yarıçapı (m)
    mu_earth: float = 3.986004418e14          # Dünya Standart Gravitasyonel Parametresi GM (m^3/s^2)
    v_orbit_leo: float = 7800.0               # Yaklaşık LEO (Alçak Dünya Yörüngesi) Hızı (m/s)
    v_escape_earth: float = 11186.0           # Dünya Yüzeyinden Kaçış Hızı (m/s)

    # Ek Dinamik Sabitler Sözlüğü
    _custom: Dict[str, float] = {}

    @classmethod
    def get(cls, name: str, default: Optional[float] = None) -> float:
        if hasattr(cls, name):
            return getattr(cls, name)
        return cls._custom.get(name, default)

    @classmethod
    def set_custom(cls, name: str, value: float):
        cls._custom[name] = float(value)

    @classmethod
    def all_constants(cls) -> Dict[str, float]:
        res = {
            "c": cls.c,
            "G": cls.G,
            "h": cls.h,
            "hbar": cls.hbar,
            "k_B": cls.k_B,
            "e": cls.e,
            "alpha": cls.alpha,
            "mu_0": cls.mu_0,
            "eps_0": cls.eps_0,
            "sigma_SB": cls.sigma_SB,
            "N_A": cls.N_A,
            "R_gas": cls.R_gas,
            "g0": cls.g0,
            "M_earth": cls.M_earth,
            "R_earth": cls.R_earth,
            "mu_earth": cls.mu_earth,
            "v_orbit_leo": cls.v_orbit_leo,
            "v_escape_earth": cls.v_escape_earth,
        }
        res.update(cls._custom)
        return res
