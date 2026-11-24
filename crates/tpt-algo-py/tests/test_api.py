"""Tests for the wider Python API: fraud helpers, EKF helpers, routing, order
book, swarm, consensus, error kinds, and the package metadata."""

import math
import os
import re
import unittest

import tpt_algo

from test_tpt_algo import cluster_with_outlier

OCV = [(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)]


class ForestHelperTests(unittest.TestCase):
    def test_n_trees_and_mean_path_length(self):
        forest = tpt_algo.IsolationForest.fit(cluster_with_outlier(), n_trees=25, seed=1)
        self.assertEqual(forest.n_trees, 25)
        outlier = forest.mean_path_length([50.0, 50.0])
        inlier = forest.mean_path_length([0.3, 0.3])
        self.assertLess(outlier, inlier)

    def test_average_path_length_values(self):
        self.assertEqual(tpt_algo.average_path_length(1), 0.0)
        self.assertAlmostEqual(tpt_algo.average_path_length(2), 1.0)
        self.assertGreater(tpt_algo.average_path_length(256), 10.0)

    def test_defaults_match_rust_defaults(self):
        data = cluster_with_outlier()
        a = tpt_algo.IsolationForest.fit(data)
        self.assertEqual(a.n_trees, 100)
        b = tpt_algo.IsolationForest.fit(data, n_trees=100, subsample_size=256, seed=0)
        self.assertEqual(a.score_samples(data), b.score_samples(data))


class EkfHelperTests(unittest.TestCase):
    def make(self):
        return tpt_algo.Ekf(OCV, capacity_ah=2.0, r0=0.05, r1=0.02, c1=1000.0,
                            soc0=0.8, soc_variance=0.05)

    def test_covariance_is_symmetric_2x2(self):
        p = self.make().covariance()
        self.assertEqual(len(p), 2)
        self.assertEqual(p[0][1], p[1][0])

    def test_model_reports_cell_parameters(self):
        m = self.make().model()
        self.assertEqual(m, {"capacity_ah": 2.0, "r0": 0.05, "r1": 0.02, "c1": 1000.0})

    def test_ocv_and_terminal_voltage(self):
        ekf = self.make()
        self.assertAlmostEqual(ekf.ocv_voltage(0.5), 3.7)
        self.assertAlmostEqual(ekf.terminal_voltage(0.5, 0.1, 1.0), 3.55)


class RoutingTests(unittest.TestCase):
    def test_solves_and_reports_cost(self):
        out = tpt_algo.solve_routes({
            "depot": [0.0, 0.0],
            "capacity": 10.0,
            "customers": [{"x": 1.0, "y": 0.0, "demand": 5.0},
                          {"x": -1.0, "y": 0.0, "demand": 5.0}],
        })
        served = sorted(c for route in out["routes"] for c in route)
        self.assertEqual(served, [0, 1])
        self.assertGreater(out["cost"], 0.0)
        self.assertEqual(out["vehicles"], len(out["routes"]))

    def test_accepts_json_string(self):
        out = tpt_algo.solve_routes(
            '{"depot":[0,0],"capacity":10,"customers":[{"x":1,"y":0,"demand":5}]}'
        )
        self.assertEqual(out["routes"], [[0]])

    def test_infeasible_kind(self):
        with self.assertRaises(tpt_algo.TptAlgoError) as ctx:
            tpt_algo.solve_routes(
                {"depot": [0, 0], "capacity": 10, "customers": [{"x": 1, "y": 0, "demand": 20}]}
            )
        self.assertEqual(ctx.exception.kind, "infeasible")


class OrderBookTests(unittest.TestCase):
    def test_batch_returns_trades_and_stats(self):
        book = tpt_algo.OrderBook()
        out = book.apply([
            {"type": "add", "id": 1, "side": "bid", "price": 99, "qty": 10},
            {"type": "add", "id": 2, "side": "ask", "price": 101, "qty": 5},
            {"type": "add", "id": 3, "side": "bid", "price": 101, "qty": 5},
        ])
        self.assertEqual(len(out["trades"]), 1)
        self.assertEqual(out["trades"][0]["price"], 101)
        self.assertEqual(out["stats"]["traded"], 5)
        self.assertEqual(book.snapshot()["best_bid"], 99)

    def test_failed_batch_is_atomic(self):
        book = tpt_algo.OrderBook()
        book.apply([{"type": "add", "id": 1, "side": "bid", "price": 99, "qty": 10}])
        before = book.snapshot()
        with self.assertRaises(tpt_algo.TptAlgoError) as ctx:
            book.apply([{"type": "cancel", "id": 1}, {"type": "cancel", "id": 42}])
        self.assertEqual(ctx.exception.kind, "invalid_input")
        self.assertEqual(book.snapshot(), before)


class SwarmTests(unittest.TestCase):
    def test_avoid_returns_velocity(self):
        agent = {"position": [0.0, 0.0], "velocity": [1.0, 0.0], "radius": 0.5,
                 "preferred_velocity": [1.0, 0.0], "max_speed": 1.0}
        other = dict(agent, position=[4.0, 0.0], velocity=[0.0, 0.0], preferred_velocity=[0.0, 0.0])
        v = tpt_algo.avoid(agent, [other])["velocity"]
        self.assertEqual(len(v), 2)
        self.assertTrue(all(math.isfinite(x) for x in v))

    def test_find_path_unreachable_kind(self):
        with self.assertRaises(tpt_algo.TptAlgoError) as ctx:
            tpt_algo.find_path({"width": 3, "height": 1, "blocked": [[1, 0]],
                                "start": [0, 0], "goal": [2, 0]})
        self.assertEqual(ctx.exception.kind, "unreachable")

    def test_find_path_bad_connectivity_is_config_error(self):
        with self.assertRaises(tpt_algo.TptAlgoError) as ctx:
            tpt_algo.find_path({"width": 2, "height": 2, "start": [0, 0], "goal": [1, 1],
                                "connectivity": 6})
        self.assertEqual(ctx.exception.kind, "invalid_config")


class ConsensusTests(unittest.TestCase):
    def test_honest_run_is_prefix_consistent(self):
        out = tpt_algo.simulate_consensus({"n": 4, "seed": 3, "ticks": 80})
        self.assertTrue(out["prefix_consistent"])
        self.assertEqual(len(out["honest_logs"]), 4)
        self.assertEqual(out["stats"]["safety_violations"], 0)

    def test_same_seed_same_output(self):
        cfg = {"n": 4, "seed": 9, "ticks": 60}
        self.assertEqual(tpt_algo.simulate_consensus(cfg), tpt_algo.simulate_consensus(cfg))


class ErrorTests(unittest.TestCase):
    def test_errors_are_value_errors_with_kind(self):
        with self.assertRaises(ValueError) as ctx:
            tpt_algo.Ekf([(0.0, 3.0)], 2.0, 0.05, 0.02, 1000.0, 0.5, 0.05)
        self.assertIsInstance(ctx.exception, tpt_algo.TptAlgoError)
        self.assertEqual(ctx.exception.kind, "unsupported")

    def test_invalid_config_kind(self):
        with self.assertRaises(tpt_algo.TptAlgoError) as ctx:
            tpt_algo.IsolationForest.fit(cluster_with_outlier(), n_trees=0)
        self.assertEqual(ctx.exception.kind, "invalid_config")


class PackageTests(unittest.TestCase):
    def test_version_is_string(self):
        self.assertTrue(re.fullmatch(r"\d+\.\d+\.\d+", tpt_algo.__version__))

    def test_py_typed_marker_and_stub_present(self):
        pkg = os.path.dirname(tpt_algo.__file__)
        self.assertTrue(os.path.isfile(os.path.join(pkg, "py.typed")))
        self.assertTrue(os.path.isfile(os.path.join(pkg, "__init__.pyi")))


if __name__ == "__main__":
    unittest.main()
