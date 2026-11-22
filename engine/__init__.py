# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# Matematik Çekirdeği — Evrensel Bilgi ve İcat Motoru (Universal Invention Engine)

"""
Citadel-Math / Matematik Çekirdeği İcat ve Keşif Motoru.
Boyut analizi, kanonik fizik grafı ve sembolik türetim sistemi.
"""

from .dimensions import Dimension, DIMENSIONS, check_dimensional_consistency
from .constants import PhysicalConstants
from .knowledge_graph import KnowledgeGraph
from .inventor import InventionEngine
from .fetcher import OnlineKnowledgeFetcher
from .harvester import LatexEquationHarvester
from .simulator import AtmosphereModel, RocketVehicle, RK4TrajectorySimulator
from .materials import Material, MaterialsDatabase, StructuralStressChecker

__all__ = [
    "Dimension",
    "DIMENSIONS",
    "check_dimensional_consistency",
    "PhysicalConstants",
    "KnowledgeGraph",
    "InventionEngine",
    "OnlineKnowledgeFetcher",
    "LatexEquationHarvester",
    "AtmosphereModel",
    "RocketVehicle",
    "RK4TrajectorySimulator",
    "Material",
    "MaterialsDatabase",
    "StructuralStressChecker",
]
