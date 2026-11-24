"""Runs the shared parity fixtures through the Python bindings.

The same file (internal/tpt-algo-bindings-core/fixtures/parity.json) is run by
the Rust core tests and the WebAssembly tests. Numbers must agree within 1e-9.
"""

import json
import math
import os
import unittest

import tpt_algo

FIXTURE = os.path.join(
    os.path.dirname(os.path.abspath(__file__)),
    "..", "..", "..", "internal", "tpt-algo-bindings-core", "fixtures", "parity.json",
)
TOLERANCE = 1e-9


def approx_eq(actual, expected):
    if isinstance(expected, bool) or expected is None or isinstance(expected, str):
        return actual == expected
    if isinstance(expected, (int, float)):
        return isinstance(actual, (int, float)) and math.isclose(
            actual, expected, rel_tol=TOLERANCE, abs_tol=TOLERANCE
        )
    if isinstance(expected, list):
        return (
            isinstance(actual, list)
            and len(actual) == len(expected)
            and all(approx_eq(a, e) for a, e in zip(actual, expected))
        )
    if isinstance(expected, dict):
        return (
            isinstance(actual, dict)
            and set(actual) == set(expected)
            and all(approx_eq(actual[k], expected[k]) for k in expected)
        )
    return actual == expected


def run_forest(inp):
    kwargs = {k: inp[k] for k in ("n_trees", "subsample_size", "seed") if k in inp}
    forest = tpt_algo.IsolationForest.fit(inp["data"], **kwargs)
    queries = inp["queries"]
    return {
        "dimension": forest.dimension,
        "n_trees": forest.n_trees,
        "scores": [forest.score(q) for q in queries],
        "mean_path_length": [forest.mean_path_length(q) for q in queries],
    }


def run_ekf(inp):
    cell = inp["cell"]
    kwargs = {k: cell[k] for k in ("process_soc", "process_v1", "measurement") if k in cell}
    if "v1_variance" in inp:
        kwargs["v1_variance"] = inp["v1_variance"]
    ekf = tpt_algo.Ekf(
        [tuple(p) for p in inp["ocv"]],
        capacity_ah=cell["capacity_ah"],
        r0=cell["r0"],
        r1=cell["r1"],
        c1=cell["c1"],
        soc0=inp["soc0"],
        soc_variance=inp["soc_variance"],
        **kwargs,
    )
    states = []
    for step in inp["steps"]:
        if step["op"] == "predict":
            ekf.predict(step["current"], step["dt"])
        else:
            ekf.update(step["voltage"], step["current"])
        soc, v1, var = ekf.estimate()
        states.append({"soc": soc, "v1": v1, "soc_variance": var})
    t = inp["helpers"]["terminal"]
    return {
        "states": states,
        "covariance": ekf.covariance(),
        "ocv_voltage": ekf.ocv_voltage(inp["helpers"]["ocv_voltage_soc"]),
        "terminal_voltage": ekf.terminal_voltage(t[0], t[1], t[2]),
        "cell": ekf.model(),
    }


def run_error(inp):
    op = inp["op"]
    try:
        if op == "ekf_table":
            points = [(i / 8.0, 3.0) for i in range(inp["points"])]
            tpt_algo.Ekf(points, 2.0, 0.05, 0.02, 1000.0, 0.5, 0.05)
        elif op == "ekf_capacity":
            tpt_algo.Ekf([(0.0, 3.0), (1.0, 4.2)], inp["capacity_ah"], 0.05, 0.02, 1000.0, 0.5, 0.05)
        elif op == "forest_trees":
            tpt_algo.IsolationForest.fit([[1.0], [2.0]], n_trees=0, subsample_size=2, seed=0)
        elif op == "forest_empty":
            tpt_algo.IsolationForest.fit([], n_trees=10, subsample_size=2, seed=0)
        elif op == "forest_dim":
            forest = tpt_algo.IsolationForest.fit([[1.0, 2.0], [2.0, 3.0]], n_trees=10, subsample_size=2)
            forest.score([1.0])
        elif op == "routing_infeasible":
            tpt_algo.solve_routes(
                {"depot": [0, 0], "capacity": 10, "customers": [{"x": 1, "y": 0, "demand": 20}]}
            )
        elif op == "swarm_unreachable":
            tpt_algo.find_path(
                {"width": 3, "height": 1, "blocked": [[1, 0]], "start": [0, 0], "goal": [2, 0], "connectivity": 4}
            )
        elif op == "orderbook_unknown":
            tpt_algo.OrderBook().apply([{"type": "cancel", "id": 42}])
        else:
            raise AssertionError(f"unknown error op {op}")
    except tpt_algo.TptAlgoError as e:
        return {"kind": e.kind}
    raise AssertionError(f"{op} did not raise")


RUNNERS = {
    "forest": run_forest,
    "average_path_length": lambda inp: {"values": [tpt_algo.average_path_length(n) for n in inp["n"]]},
    "ekf": run_ekf,
    "routing": tpt_algo.solve_routes,
    "orderbook": lambda inp: tpt_algo.OrderBook().apply(inp["events"]),
    "swarm_avoid": lambda inp: tpt_algo.avoid(inp["agent"], inp.get("neighbors"), inp.get("config")),
    "swarm_find_path": tpt_algo.find_path,
    "consensus": tpt_algo.simulate_consensus,
    "error": run_error,
}


class ParityTests(unittest.TestCase):
    def test_shared_fixtures_match_expected(self):
        with open(FIXTURE, encoding="utf-8") as fh:
            doc = json.load(fh)
        failures = []
        for case in doc["cases"]:
            actual = RUNNERS[case["kind"]](case["input"])
            if not approx_eq(actual, case["expected"]):
                failures.append(f"{case['name']}: got {actual!r}, expected {case['expected']!r}")
        self.assertEqual(failures, [])


if __name__ == "__main__":
    unittest.main()
