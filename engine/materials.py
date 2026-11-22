# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# Matematik Çekirdeği — Malzeme Bilimi ve Yapısal Dayanım Kalkanı (Materials Engine)

"""
Havacılık, Uzay ve Aşırı Ortam Malzeme Bilimi Motoru.
İnce cidarlı kabuk mekaniği (Thin-Walled Cylinder Biaxial Von Mises) ve
Sutton-Graves konvektif-radyatif denge (Stefan-Boltzmann) hipersonik ısınma denetimi.

AGENTS.md Değişmez 4 (Sayısal ve Malzeme Gerçekliği Zorunluluğu) uyarınca inşa edilmiştir.
"""

import os
import json
import math
from typing import Dict, List, Any, Optional
from .constants import PhysicalConstants


class Material:
    """Tekil bir malzemenin fiziksel ve mekanik sınırları."""

    def __init__(self, data: Dict[str, Any]):
        self.id = data["id"]
        self.name = data["name"]
        self.category = data.get("category", "general")
        self.density = float(data["density_kg_m3"])                      # kg/m^3
        self.tensile_strength = float(data["tensile_strength_mpa"]) * 1e6 # Pa
        self.yield_strength = float(data["yield_strength_mpa"]) * 1e6     # Pa
        self.youngs_modulus = float(data["youngs_modulus_gpa"]) * 1e9     # Pa
        self.max_temperature = float(data.get("max_temperature_k", 500.0)) # K
        self.thermal_conductivity = float(data.get("thermal_conductivity_w_mk", 10.0)) # W/(m*K)
        self.description = data.get("description", "")

    @property
    def specific_strength(self) -> float:
        """Özgül Mukavemet = Akma Dayanımı / Yoğunluk (Pa / (kg/m^3) = m^2/s^2)."""
        return self.yield_strength / self.density if self.density > 0 else 0.0

    @property
    def breaking_length_km(self) -> float:
        """Kendi ağırlığı altında kopmadan asılabileceği azami uzunluk (km)."""
        g0 = PhysicalConstants.g0
        return (self.yield_strength / (self.density * g0)) / 1000.0 if self.density > 0 else 0.0

    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "category": self.category,
            "density_kg_m3": self.density,
            "yield_strength_MPa": self.yield_strength / 1e6,
            "tensile_strength_MPa": self.tensile_strength / 1e6,
            "youngs_modulus_GPa": self.youngs_modulus / 1e9,
            "max_temperature_K": self.max_temperature,
            "max_temperature_C": self.max_temperature - 273.15,
            "specific_strength_kN_m_kg": self.specific_strength / 1000.0,
            "breaking_length_km": round(self.breaking_length_km, 1),
            "description": self.description,
        }


class MaterialsDatabase:
    """Kapsamlı Malzeme Kütüphanesi ve Arama Motoru."""

    def __init__(self, json_path: Optional[str] = None):
        if json_path is None:
            json_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), "database", "materials.json")
        self.json_path = json_path
        self.materials: Dict[str, Material] = {}
        self.load()

    def load(self):
        if not os.path.exists(self.json_path):
            raise FileNotFoundError(f"Malzeme veritabanı bulunamadı: {self.json_path}")
        with open(self.json_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        for item in data:
            mat = Material(item)
            self.materials[mat.id] = mat

    def get(self, mat_id: str) -> Optional[Material]:
        return self.materials.get(mat_id)

    def rank_by_breaking_length(self) -> List[Material]:
        """Malzemeleri uzay asansörü / kablo kopma uzunluğuna göre sıralar."""
        return sorted(self.materials.values(), key=lambda m: m.breaking_length_km, reverse=True)

    def rank_by_temperature(self) -> List[Material]:
        """Malzemeleri azami sıcaklık direncine göre sıralar."""
        return sorted(self.materials.values(), key=lambda m: m.max_temperature, reverse=True)


class StructuralStressChecker:
    """Yapısal Dayanım, Kabuk Mekaniği ve Sutton-Graves Termal Kalkan Denetim Motoru."""

    SAFETY_FACTOR = 1.35  # Havacılık ve Uzay Asgari Emniyet Katsayısı (AIAA Standardı)
    SUTTON_GRAVES_K = 1.7415e-4  # Dünya atmosferi (N2-O2) Sutton-Graves sabiti [kg^0.5 / m]

    @classmethod
    def check_rocket_hull(
        cls,
        material: Material,
        total_mass_kg: float,
        g_force: float,
        max_q_pa: float,
        hull_radius_m: float = 1.5,
        wall_thickness_mm: float = 4.0,
        tank_pressure_bar: float = 3.0,
        drag_coeff: float = 0.30,
    ) -> Dict[str, Any]:
        """
        İnce cidarlı silindirik roket gövdesinde (Thin-Walled Cylinder):
          1. Eksenel basma kuvveti: F_axial = m * g_load * g0 + q_max * C_d * (pi * r^2)
          2. Eksenel gerilme (basma, negatif): sigma_z = - F_axial / (2 * pi * r * t)
          3. Çevresel (Hoop) gerilme (çekme, pozitif): sigma_theta = (P_int * r) / t
          4. Düzlem gerilme Von Mises eşdeğeri:
             sigma_VM = sqrt(sigma_z^2 - sigma_z * sigma_theta + sigma_theta^2)
        """
        t_m = wall_thickness_mm / 1000.0
        r_m = hull_radius_m

        a_shell = 2.0 * math.pi * r_m * t_m
        a_frontal = math.pi * r_m * r_m

        inertial_force_n = total_mass_kg * g_force * PhysicalConstants.g0
        aero_drag_force_n = max_q_pa * drag_coeff * a_frontal
        total_axial_compression_n = inertial_force_n + aero_drag_force_n

        # Basma gerilmesi negatif, çevresel tank çekme gerilmesi pozitiftir:
        sigma_z = -(total_axial_compression_n / a_shell) if a_shell > 0 else 0.0
        p_internal_pa = tank_pressure_bar * 100000.0
        sigma_theta = (p_internal_pa * r_m) / t_m if t_m > 0 else 0.0

        # İki eksenli (biaxial) Von Mises: sigma_z < 0 ve sigma_theta > 0 olduğunda
        # (-sigma_z * sigma_theta) terimi pozitif katkı verir (en kritik kayma durumu).
        sigma_vm = math.sqrt(sigma_z ** 2 - sigma_z * sigma_theta + sigma_theta ** 2)

        allowable_stress = material.yield_strength / cls.SAFETY_FACTOR
        margin_of_safety = (allowable_stress / sigma_vm) - 1.0 if sigma_vm > 0 else 0.0
        survives = margin_of_safety >= 0.0

        return {
            "material_name": material.name,
            "calculated_von_mises_MPa": round(sigma_vm / 1e6, 2),
            "material_yield_limit_MPa": round(material.yield_strength / 1e6, 2),
            "allowable_with_safety_MPa": round(allowable_stress / 1e6, 2),
            "safety_factor_achieved": round((material.yield_strength / sigma_vm) if sigma_vm > 0 else 999.0, 2),
            "margin_of_safety": round(margin_of_safety, 2),
            "survives_flight": survives,
            "verdict": (
                f"[✓ GÜVENLİ] Malzeme uçuş yüklerine başarıyla dayanır (Emniyet: {cls.SAFETY_FACTOR}x)."
                if survives else
                "[✗ YIKICI HASAR] Malzeme akma sınırını aştı! Gövde ezilir veya yırtılır."
            ),
        }

    @classmethod
    def check_aerodynamic_heating(
        cls,
        material: Material,
        max_speed_ms: float,
        ambient_temp_k: Optional[float] = None,
        heating_altitude_m: float = 45000.0,
        nose_radius_m: float = 0.5,
        emissivity: float = 0.85,
    ) -> Dict[str, Any]:
        """
        USSA-1976 Katmanlı Hidrostatik Atmosfer + Sutton-Graves Konvektif Isı Akısı ve Stefan-Boltzmann Radyatif Denge:
          - q_conv = k_SG * sqrt(rho / R_n) * v_eff^3   [W/m^2]
          - Radyatif denge: q_conv = epsilon * sigma_SB * (T_wall^4 - T_amb^4)
          - T_wall = min(T_stag, (q_conv / (epsilon * sigma_SB) + T_amb^4)^(1/4))
        """
        from .simulator import AtmosphereModel

        if max_speed_ms > 4000.0:
            v_eff = max_speed_ms * 0.42
            ref_alt_m = heating_altitude_m
        else:
            v_eff = max_speed_ms
            ref_alt_m = 15000.0

        rho_eff, _p_pa, ussa_temp_k, ussa_sound_ms = AtmosphereModel.state_at(ref_alt_m)
        eff_temp_k = float(ambient_temp_k) if ambient_temp_k is not None else ussa_temp_k
        speed_of_sound = (
            math.sqrt(AtmosphereModel.GAMMA_AIR * AtmosphereModel.R_AIR * eff_temp_k)
            if ambient_temp_k is not None
            else ussa_sound_ms
        )

        mach = v_eff / speed_of_sound
        t_stag = eff_temp_k * (1.0 + 0.2 * (mach ** 2))

        # Sutton-Graves konvektif ısı akısı (W/m^2)
        q_conv = cls.SUTTON_GRAVES_K * math.sqrt(rho_eff / max(0.05, nose_radius_m)) * (v_eff ** 3)
        sigma_sb = PhysicalConstants.sigma_SB
        t_rad_eq = ((q_conv / (emissivity * sigma_sb)) + (eff_temp_k ** 4)) ** 0.25

        # Duvar sıcaklığı adyabatik durgunluk sıcaklığını (t_stag) fiziksel olarak aşamaz
        t_wall_estimate = min(t_stag, t_rad_eq)
        survives = t_wall_estimate < material.max_temperature

        return {
            "material_name": material.name,
            "flight_mach": round(mach, 2),
            "convective_heat_flux_kW_m2": round(q_conv / 1000.0, 2),
            "stagnation_temp_K": round(t_stag, 1),
            "stagnation_temp_C": round(t_stag - 273.15, 1),
            "wall_temp_estimate_C": round(t_wall_estimate - 273.15, 1),
            "material_max_temp_C": round(material.max_temperature - 273.15, 1),
            "survives_heating": survives,
            "verdict": (
                "[✓ TERMAL ONAY] Malzeme hipersonik sürtünme sıcaklığına dayanır."
                if survives else
                "[✗ TERMAL ERİME] Sıcaklık malzemenin termal sınırını aştı! Burun erir veya alev alır."
            ),
        }
