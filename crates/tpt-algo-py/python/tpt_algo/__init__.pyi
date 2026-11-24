"""Type stubs for tpt_algo. The package is typed (PEP 561, py.typed)."""

from collections.abc import Mapping, Sequence
from typing import Any, Union, final

__version__: str

JsonInput = Union[str, Mapping[str, Any]]

class TptAlgoError(ValueError):
    """Raised for every binding error.

    ``kind`` is one of ``"invalid_input"``, ``"invalid_config"``,
    ``"unsupported"``, ``"infeasible"`` or ``"unreachable"``.
    """

    kind: str

@final
class IsolationForest:
    """Seeded Isolation Forest. Scores lie in [0, 1]; values near 1 are anomalous."""

    @staticmethod
    def fit(
        data: Sequence[Sequence[float]],
        n_trees: int = 100,
        subsample_size: int = 256,
        seed: int = 0,
    ) -> IsolationForest: ...
    @property
    def dimension(self) -> int: ...
    @property
    def n_trees(self) -> int: ...
    def score(self, row: Sequence[float]) -> float: ...
    def score_samples(self, data: Sequence[Sequence[float]]) -> list[float]: ...
    def mean_path_length(self, row: Sequence[float]) -> float: ...

def average_path_length(n: int) -> float:
    """Average path length c(n) of an unsuccessful search in a binary tree."""
    ...

@final
class Ekf:
    """Extended Kalman filter for battery state of charge.

    ``ocv`` is a list of (soc, volts) pairs, 2 to 8 entries, strictly increasing soc.
    """

    def __init__(
        self,
        ocv: Sequence[tuple[float, float]],
        capacity_ah: float,
        r0: float,
        r1: float,
        c1: float,
        soc0: float,
        soc_variance: float,
        v1_variance: float = 1e-4,
        process_soc: float = 1e-9,
        process_v1: float = 1e-6,
        measurement: float = 1e-4,
    ) -> None: ...
    def predict(self, current: float, dt: float) -> None: ...
    def update(self, voltage: float, current: float) -> None: ...
    @property
    def soc(self) -> float: ...
    def estimate(self) -> tuple[float, float, float]:
        """(soc, v1, soc_variance)."""
        ...
    def covariance(self) -> list[list[float]]:
        """2x2 error covariance of (soc, v1)."""
        ...
    def model(self) -> dict[str, float]:
        """Cell parameters: capacity_ah, r0, r1, c1."""
        ...
    def ocv_voltage(self, soc: float) -> float: ...
    def terminal_voltage(self, soc: float, v1: float, current: float) -> float: ...

@final
class OrderBook:
    """Limit order book. Events are dicts; see the docs for the event shapes."""

    def __init__(self) -> None: ...
    def apply(self, events: Union[str, Sequence[Mapping[str, Any]]]) -> dict[str, Any]:
        """Applies a batch atomically. Returns {"trades": [...], "stats": {...}}."""
        ...
    def snapshot(self) -> dict[str, Any]: ...

def solve_routes(problem: JsonInput) -> dict[str, Any]:
    """Returns {"routes": [[int, ...], ...], "cost": float, "vehicles": int}."""
    ...

def avoid(
    agent: Mapping[str, Any],
    neighbors: Sequence[Mapping[str, Any]] | None = None,
    config: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    """Returns {"velocity": [vx, vy]}."""
    ...

def find_path(grid: JsonInput) -> dict[str, Any]:
    """Returns {"cells": [[x, y], ...], "cost": float}."""
    ...

def simulate_consensus(config: JsonInput) -> dict[str, Any]:
    """Seeded consensus simulation. Returns logs, stats and prefix_consistent."""
    ...
