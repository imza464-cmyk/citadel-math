# Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
# Matematik Çekirdeği — Sayısal Simülasyon ve Yörünge Entegratörü (True RK4 ODE Simulator)

"""
Gerçek 4. Derece Runge-Kutta (RK4) Diferansiyel Denklem Çözücüsü ve Atmosferik Uçuş Simülatörü.
Durum vektörü y = [x, h, vx, vh, m]^T üzerinde 4 aşamalı eğim değerlendirmesi
(k1, k2, k3, k4) ve O(dt^4) global kesme hatası ile entegrasyon yürütür:
  y_{n+1} = y_n + (dt / 6) * (k1 + 2*k2 + 2*k3 + k4)

AGENTS.md Değişmez 4 (Sayısal Gerçeklik Zorunluluğu) uyarınca inşa edilmiştir.
"""

import math
from typing import Dict, List, Tuple, Any, Optional
from .constants import PhysicalConstants


class AtmosphereModel:
    """7-Katmanlı Hidrostatik US Standard Atmosphere 1976 (USSA-1976) ve Termosfer Geçiş Modeli."""

    R0_GEOPOTENTIAL = 6356766.0
    R_STAR = 8.31432
    M0_AIR = 0.0289644
    R_AIR = R_STAR / M0_AIR                  # 287.052874247 J/(kg*K)
    GAMMA_AIR = 1.4
    GMR = (PhysicalConstants.g0 * M0_AIR) / R_STAR  # 0.0341631947363 K/m
    R_EARTH = PhysicalConstants.R_earth
    RHO_0 = 1.225
    P_0 = 101325.0
    T_0 = 288.15

    # (Z_b [m], T_b [K], L_b [K/m], P_b [Pa])
    USSA76_LAYERS: Tuple[Tuple[float, float, float, float], ...] = (
        (0.0, 288.15, -0.0065, 101325.0),
        (11000.0, 216.65, 0.0, 22632.06397346292),
        (20000.0, 216.65, 0.0010, 5474.8886696777745),
        (32000.0, 228.65, 0.0028, 868.0186847552279),
        (47000.0, 270.65, 0.0, 110.90630555496608),
        (51000.0, 270.65, -0.0028, 66.9388731186873),
        (71000.0, 214.65, -0.0020, 3.956420428040732),
    )

    @classmethod
    def state_at(cls, altitude_m: float) -> Tuple[float, float, float, float]:
        """Geometrik irtifadan (h [m]) USSA-1976 tam atmosfer durumunu döndürür:
        (yoğunluk_kg_m3, basınç_pa, sıcaklık_k, ses_hızı_ms)
        """
        h = max(0.0, float(altitude_m))
        if h > 150000.0:
            t_exo = 474.946
            a_exo = math.sqrt(cls.GAMMA_AIR * cls.R_AIR * t_exo)
            return (0.0, 0.0, t_exo, a_exo)

        z = (cls.R0_GEOPOTENTIAL * h) / (cls.R0_GEOPOTENTIAL + h)
        if z <= 84852.0:
            if z < 11000.0:
                idx = 0
            elif z < 20000.0:
                idx = 1
            elif z < 32000.0:
                idx = 2
            elif z < 47000.0:
                idx = 3
            elif z < 51000.0:
                idx = 4
            elif z < 71000.0:
                idx = 5
            else:
                idx = 6

            zb, tb, lb, pb = cls.USSA76_LAYERS[idx]
            dz = z - zb
            if abs(lb) < 1e-12:
                temp_k = tb
                press_pa = pb * math.exp((-cls.GMR * dz) / tb)
            else:
                temp_k = tb + lb * dz
                press_pa = pb * ((tb / temp_k) ** (cls.GMR / lb))
            rho = press_pa / (cls.R_AIR * temp_k)
            a_sound = math.sqrt(cls.GAMMA_AIR * cls.R_AIR * temp_k)
            return (rho, press_pa, temp_k, a_sound)

        z_top = 84852.0
        zb, tb, lb, pb = cls.USSA76_LAYERS[6]
        t_86 = tb + lb * (z_top - zb)
        p_86 = pb * ((tb / t_86) ** (cls.GMR / lb))
        dh = max(0.0, h - 86000.0)
        temp_k = t_86 + 0.0045 * dh
        press_pa = p_86 * math.exp(-dh / 6500.0)
        rho = press_pa / (cls.R_AIR * temp_k)
        a_sound = math.sqrt(cls.GAMMA_AIR * cls.R_AIR * temp_k)
        return (rho, press_pa, temp_k, a_sound)

    @classmethod
    def density(cls, altitude_m: float) -> float:
        """İrtifaya bağlı USSA-1976 hava yoğunluğu (kg/m^3)."""
        return cls.state_at(altitude_m)[0]

    @classmethod
    def pressure(cls, altitude_m: float) -> float:
        """İrtifaya bağlı USSA-1976 statik basınç (Pa)."""
        return cls.state_at(altitude_m)[1]

    @classmethod
    def temperature(cls, altitude_m: float) -> float:
        """İrtifaya bağlı USSA-1976 ortam sıcaklığı (K)."""
        return cls.state_at(altitude_m)[2]

    @classmethod
    def speed_of_sound(cls, altitude_m: float) -> float:
        """İrtifaya bağlı yerel ses hızı (m/s)."""
        return cls.state_at(altitude_m)[3]

    @classmethod
    def gravity(cls, altitude_m: float) -> float:
        """İrtifaya bağlı yerçekimi ivmesi (m/s^2). Newton Çekim Kanunu."""
        r = cls.R_EARTH + max(0.0, altitude_m)
        return PhysicalConstants.mu_earth / (r * r)


class FlightState:
    """Belirli bir t anındaki uçuş durum vektörü."""
    __slots__ = ("t", "x", "h", "vx", "vh", "m", "thrust", "drag", "q_dyn", "g_load")

    def __init__(self, t=0.0, x=0.0, h=0.0, vx=0.0, vh=0.0, m=1.0, thrust=0.0, drag=0.0, q_dyn=0.0, g_load=1.0):
        self.t = t
        self.x = x
        self.h = h
        self.vx = vx
        self.vh = vh
        self.m = m
        self.thrust = thrust
        self.drag = drag
        self.q_dyn = q_dyn
        self.g_load = g_load

    @property
    def speed(self) -> float:
        return math.sqrt(self.vx * self.vx + self.vh * self.vh)

    def to_dict(self) -> Dict[str, float]:
        return {
            "time_s": round(self.t, 2),
            "altitude_km": round(self.h / 1000.0, 2),
            "speed_ms": round(self.speed, 2),
            "speed_kmh": round(self.speed * 3.6, 1),
            "downrange_km": round(self.x / 1000.0, 2),
            "mass_kg": round(self.m, 2),
            "thrust_kN": round(self.thrust / 1000.0, 2),
            "max_q_kPa": round(self.q_dyn / 1000.0, 2),
            "g_force": round(self.g_load, 2),
        }


class RocketVehicle:
    """Fiziksel Roket veya Fırlatma Aracı Tanımı."""

    def __init__(
        self,
        name: str,
        dry_mass_kg: float,
        propellant_mass_kg: float,
        thrust_vacuum_n: float,
        isp_vacuum_s: float,
        cross_section_m2: float,
        drag_coeff: float = 0.30,
        burn_time_s: Optional[float] = None,
        initial_velocity_ms: float = 0.0,
        initial_altitude_m: float = 0.0,
        stage_2_dry_mass_kg: float = 0.0,
        stage_2_propellant_kg: float = 0.0,
        stage_2_thrust_n: float = 0.0,
        stage_2_isp_s: float = 0.0,
    ):
        self.name = name
        self.dry_mass = float(dry_mass_kg)
        self.propellant_mass = float(propellant_mass_kg)
        self.total_mass = self.dry_mass + self.propellant_mass
        self.thrust_vac = float(thrust_vacuum_n)
        self.isp = float(isp_vacuum_s)
        self.area = float(cross_section_m2)
        self.cd = float(drag_coeff)

        self.v_e = self.isp * PhysicalConstants.g0
        if burn_time_s and burn_time_s > 0:
            self.burn_time = float(burn_time_s)
            self.m_dot = self.propellant_mass / self.burn_time
        else:
            self.m_dot = self.thrust_vac / self.v_e if self.v_e > 0 else 0.0
            self.burn_time = self.propellant_mass / self.m_dot if self.m_dot > 0 else 0.0

        self.initial_velocity = initial_velocity_ms
        self.initial_altitude = initial_altitude_m

        self.has_stage_2 = stage_2_propellant_kg > 0
        self.s2_dry_mass = float(stage_2_dry_mass_kg)
        self.s2_propellant = float(stage_2_propellant_kg)
        self.s2_thrust = float(stage_2_thrust_n)
        self.s2_isp = float(stage_2_isp_s)
        self.s2_ve = self.s2_isp * PhysicalConstants.g0
        self.s2_mdot = self.s2_thrust / self.s2_ve if self.s2_ve > 0 else 0.0
        self.s2_burn_time = self.s2_propellant / self.s2_mdot if self.s2_mdot > 0 else 0.0


class RK4TrajectorySimulator:
    """
    Gerçek 4. Derece Runge-Kutta (RK4) Sayısal Yörünge Entegratörü.
    Durum vektörü y = (x, h, vx, vh, m) için:
      k1 = f(t, y)
      k2 = f(t + dt/2, y + dt/2 * k1)
      k3 = f(t + dt/2, y + dt/2 * k2)
      k4 = f(t + dt,   y + dt   * k3)
      y_{n+1} = y_n + (dt / 6) * (k1 + 2*k2 + 2*k3 + k4)
    """

    def __init__(self, dt_seconds: float = 0.1):
        self.dt = dt_seconds

    @staticmethod
    def _pitch_angle(alt: float) -> float:
        if alt < 1200.0:
            return math.radians(90.0)
        if alt > 160000.0:
            return 0.0
        frac = (alt - 1200.0) / (160000.0 - 1200.0)
        deg = 90.0 * (1.0 - math.sqrt(frac))
        return math.radians(max(0.0, deg))

    def _derivatives(
        self,
        t: float,
        state: Tuple[float, float, float, float, float],
        vehicle: RocketVehicle,
        stage: int,
        s2_start_t: float,
    ) -> Tuple[float, float, float, float, float, float, float, float, float]:
        """
        Verilen (t, state) noktasında (dx/dt, dh/dt, dvx/dt, dvh/dt, dm/dt) türevlerini
        ve anlık (thrust, drag, q, g_load) telemetrisini döndürür.
        """
        _x, h, vx, vh, m = state
        eff_m = max(1.0, m)

        if stage == 1:
            burning = (t < vehicle.burn_time) and (eff_m > vehicle.dry_mass)
            thrust = vehicle.thrust_vac if burning else 0.0
            mdot = vehicle.m_dot if burning else 0.0
        else:
            s2_elapsed = t - s2_start_t
            burning = (s2_elapsed < vehicle.s2_burn_time) and (eff_m > vehicle.s2_dry_mass)
            thrust = vehicle.s2_thrust if burning else 0.0
            mdot = vehicle.s2_mdot if burning else 0.0

        rho = AtmosphereModel.density(h)
        g = AtmosphereModel.gravity(h)
        v_total = math.sqrt(vx * vx + vh * vh)

        q = 0.5 * rho * v_total * v_total
        drag = q * vehicle.cd * vehicle.area if v_total > 0.0 else 0.0

        r_curr = PhysicalConstants.R_earth + max(0.0, h)
        centrifugal_lift = (vx * vx) / r_curr
        coriolis_tangential = (vx * vh) / r_curr

        pitch = self._pitch_angle(h)
        fx_thrust = thrust * math.cos(pitch)
        fh_thrust = thrust * math.sin(pitch)

        if v_total > 0.0:
            fx_drag = drag * (vx / v_total)
            fh_drag = drag * (vh / v_total)
        else:
            fx_drag = 0.0
            fh_drag = 0.0

        a_proper_x = (fx_thrust - fx_drag) / eff_m
        a_proper_h = (fh_thrust - fh_drag) / eff_m

        ax = a_proper_x - coriolis_tangential
        ah = a_proper_h - g + centrifugal_lift

        g_load = math.hypot(a_proper_x, a_proper_h) / PhysicalConstants.g0

        return (vx, vh, ax, ah, -mdot, thrust, drag, q, g_load)

    def simulate(self, vehicle: RocketVehicle, target_orbit_km: float = 200.0, max_time_s: float = 1200.0) -> Dict[str, Any]:
        t = 0.0
        x = 0.0
        h = vehicle.initial_altitude
        v0 = vehicle.initial_velocity
        vh = v0 * 0.8
        vx = v0 * 0.6
        m = vehicle.total_mass

        telemetry: List[FlightState] = []
        max_q_observed = 0.0
        max_g_observed = 0.0
        max_alt_reached = h
        orbit_achieved = False

        current_stage = 1
        staged_already = False
        s2_start_time = 0.0
        dt = self.dt

        while t < max_time_s and (h >= 0.0 or t < 5.0):
            # Aşama geçişi (Staging) kontrolü
            if current_stage == 1:
                burning_s1 = (t < vehicle.burn_time) and (m > vehicle.dry_mass)
                if not burning_s1 and vehicle.has_stage_2 and not staged_already:
                    current_stage = 2
                    staged_already = True
                    m = vehicle.s2_dry_mass + vehicle.s2_propellant
                    s2_start_time = t

            y0 = (x, h, vx, vh, m)
            k1 = self._derivatives(t, y0, vehicle, current_stage, s2_start_time)

            y1 = tuple(y0[i] + 0.5 * dt * k1[i] for i in range(5))
            k2 = self._derivatives(t + 0.5 * dt, y1, vehicle, current_stage, s2_start_time)

            y2 = tuple(y0[i] + 0.5 * dt * k2[i] for i in range(5))
            k3 = self._derivatives(t + 0.5 * dt, y2, vehicle, current_stage, s2_start_time)

            y3 = tuple(y0[i] + dt * k3[i] for i in range(5))
            k4 = self._derivatives(t + dt, y3, vehicle, current_stage, s2_start_time)

            # 4. Derece Runge-Kutta Ağırlıklı Ortalaması: (k1 + 2*k2 + 2*k3 + k4) / 6
            x = x + (dt / 6.0) * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0])
            h = h + (dt / 6.0) * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1])
            vx = vx + (dt / 6.0) * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2])
            vh = vh + (dt / 6.0) * (k1[3] + 2.0 * k2[3] + 2.0 * k3[3] + k4[3])
            min_mass = vehicle.dry_mass if current_stage == 1 else vehicle.s2_dry_mass
            m = max(min_mass, m + (dt / 6.0) * (k1[4] + 2.0 * k2[4] + 2.0 * k3[4] + k4[4]))
            t += dt

            thrust, drag, q, g_load = k1[5], k1[6], k1[7], k1[8]
            if q > max_q_observed:
                max_q_observed = q
            if g_load > max_g_observed:
                max_g_observed = g_load
            if h > max_alt_reached:
                max_alt_reached = h

            if int(round(t * 10)) % 10 == 0:
                telemetry.append(FlightState(t, x, h, vx, vh, m, thrust, drag, q, g_load))

            if h >= 180000.0 and vx >= 7750.0:
                orbit_achieved = True
                break

        final_speed = math.sqrt(vx * vx + vh * vh)
        target_orbital_speed = math.sqrt(
            PhysicalConstants.mu_earth / (PhysicalConstants.R_earth + target_orbit_km * 1000.0)
        )
        remaining_fuel = (m - vehicle.dry_mass) if current_stage == 1 else (m - vehicle.s2_dry_mass)

        return {
            "vehicle_name": vehicle.name,
            "integrator": "RK4 (4-Stage Classic Runge-Kutta, O(dt^4))",
            "flight_duration_s": round(t, 2),
            "final_altitude_km": round(h / 1000.0, 2),
            "max_altitude_km": round(max_alt_reached / 1000.0, 2),
            "final_speed_ms": round(final_speed, 2),
            "target_orbital_speed_ms": round(target_orbital_speed, 2),
            "orbit_achieved": orbit_achieved,
            "remaining_fuel_kg": round(max(0.0, remaining_fuel), 2),
            "max_q_kPa": round(max_q_observed / 1000.0, 2),
            "max_g_force": round(max_g_observed, 2),
            "total_telemetry_points": len(telemetry),
            "summary": (
                "YÖRÜNGE BAŞARIYLA YAKALANDI (LEO Orbit Achieved)"
                if orbit_achieved else
                "YÖRÜNGEYE ULAŞILAMADI (Suborbital / Velocity Deficit)"
            ),
        }
