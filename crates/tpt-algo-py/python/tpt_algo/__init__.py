"""Python bindings for the tpt-algo crates.

Algorithms: Isolation Forest anomaly scores, a battery state-of-charge Extended
Kalman filter, capacitated vehicle routing, a limit order book, collision
avoidance and grid paths for swarms, and a seeded consensus simulator.

Errors raise ``TptAlgoError`` (a ``ValueError``) with a ``kind`` attribute.
"""

from __future__ import annotations

import json
from typing import Any, Mapping, Optional, Sequence, Union

from ._native import (
    Ekf,
    IsolationForest,
    TptAlgoError,
    average_path_length,
)
from ._native import OrderBookCore as _OrderBookCore
from ._native import (
    avoid_json as _avoid_json,
    find_path_json as _find_path_json,
    simulate_consensus_json as _simulate_consensus_json,
    solve_routes_json as _solve_routes_json,
)

__version__ = "0.0.0"

__all__ = [
    "Ekf",
    "IsolationForest",
    "OrderBook",
    "TptAlgoError",
    "avoid",
    "average_path_length",
    "find_path",
    "simulate_consensus",
    "solve_routes",
    "__version__",
]

JsonInput = Union[str, Mapping[str, Any]]


def _dumps(value: Any) -> str:
    """Accepts a JSON string as-is, or serialises a dict or list."""
    return value if isinstance(value, str) else json.dumps(value)


class OrderBook:
    """Limit order book. Apply events in batches; trades and stats come back.

    Each event is a dict such as ``{"type": "add", "id": 1, "side": "bid",
    "price": 99, "qty": 10}``. Types: ``add``, ``cancel``, ``modify``,
    ``execute`` and ``execute_order``. A batch is atomic: if any event is
    rejected, the book is unchanged.
    """

    def __init__(self) -> None:
        self._core = _OrderBookCore()

    def apply(self, events: Union[str, Sequence[Mapping[str, Any]]]) -> dict[str, Any]:
        """Applies a batch; returns ``{"trades": [...], "stats": {...}}``.

        ``trades`` holds only the trades made by this batch. ``stats.traded``
        is the traded quantity.
        """
        return json.loads(self._core.apply_json(_dumps(events)))

    def snapshot(self) -> dict[str, Any]:
        """Top of book, depth and sequence number."""
        return json.loads(self._core.snapshot_json())


def solve_routes(problem: JsonInput) -> dict[str, Any]:
    """Solves a capacitated routing problem. See ``docs/bindings.md``.

    Returns ``{"routes": [[...], ...], "cost": float, "vehicles": int}``.
    """
    return json.loads(_solve_routes_json(_dumps(problem)))


def avoid(
    agent: Mapping[str, Any],
    neighbors: Optional[Sequence[Mapping[str, Any]]] = None,
    config: Optional[Mapping[str, Any]] = None,
) -> dict[str, Any]:
    """Collision-free velocity for ``agent`` given its ``neighbors``.

    Returns ``{"velocity": [vx, vy]}``.
    """
    doc: dict[str, Any] = {"agent": dict(agent), "neighbors": list(neighbors or [])}
    if config is not None:
        doc["config"] = dict(config)
    return json.loads(_avoid_json(_dumps(doc)))


def find_path(grid: JsonInput) -> dict[str, Any]:
    """Shortest path on a grid. Returns ``{"cells": [[x, y], ...], "cost": float}``."""
    return json.loads(_find_path_json(_dumps(grid)))


def simulate_consensus(config: JsonInput) -> dict[str, Any]:
    """Runs the seeded consensus simulator. There are no live replicas.

    Returns the committed logs, counters and a ``prefix_consistent`` flag.
    """
    return json.loads(_simulate_consensus_json(_dumps(config)))
