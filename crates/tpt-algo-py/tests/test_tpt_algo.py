"""Smoke and behaviour tests for the tpt_algo Python bindings.

Standard library only. Run after installing the wheel:

    python -m unittest discover -s tests -v
"""

import math
import unittest

import tpt_algo


def cluster_with_outlier():
    rows = [[(i % 10) * 0.1, (i % 7) * 0.1] for i in range(200)]
    rows.append([50.0, 50.0])
    return rows


class IsolationForestTests(unittest.TestCase):
    def test_outlier_scores_higher_than_inlier(self):
        data = cluster_with_outlier()
        forest = tpt_algo.IsolationForest.fit(data, seed=7)
        scores = forest.score_samples(data)
        self.assertEqual(len(scores), len(data))
        self.assertGreater(scores[200], scores[0])
        self.assertTrue(all(0.0 <= s <= 1.0 for s in scores))

    def test_dimension_and_single_score(self):
        data = cluster_with_outlier()
        forest = tpt_algo.IsolationForest.fit(data, n_trees=50, seed=1)
        self.assertEqual(forest.dimension, 2)
        self.assertTrue(0.0 <= forest.score([0.3, 0.3]) <= 1.0)

    def test_same_seed_gives_same_scores(self):
        data = cluster_with_outlier()
        a = tpt_algo.IsolationForest.fit(data, seed=3).score_samples(data)
        b = tpt_algo.IsolationForest.fit(data, seed=3).score_samples(data)
        self.assertEqual(a, b)

    def test_empty_data_raises_value_error(self):
        with self.assertRaises(ValueError):
            tpt_algo.IsolationForest.fit([])

    def test_wrong_row_length_raises_value_error(self):
        forest = tpt_algo.IsolationForest.fit(cluster_with_outlier(), seed=1)
        with self.assertRaises(ValueError):
            forest.score([1.0])


class EkfTests(unittest.TestCase):
    OCV = [(0.0, 3.0), (0.5, 3.7), (1.0, 4.2)]

    def make(self, soc0=0.8):
        return tpt_algo.Ekf(
            self.OCV,
            capacity_ah=2.0,
            r0=0.05,
            r1=0.02,
            c1=1000.0,
            soc0=soc0,
            soc_variance=0.05,
        )

    def test_discharge_lowers_soc(self):
        ekf = self.make()
        before = ekf.soc
        ekf.predict(2.0, 600.0)
        self.assertLess(ekf.soc, before)

    def test_update_keeps_soc_in_unit_interval(self):
        ekf = self.make()
        for _ in range(50):
            ekf.predict(2.0, 60.0)
            ekf.update(3.6, 2.0)
            self.assertTrue(0.0 <= ekf.soc <= 1.0)

    def test_estimate_tuple_matches_soc(self):
        ekf = self.make(soc0=0.6)
        soc, v1, var = ekf.estimate()
        self.assertEqual(soc, ekf.soc)
        self.assertAlmostEqual(soc, 0.6)
        self.assertTrue(math.isfinite(v1))
        self.assertAlmostEqual(var, 0.05)

    def test_unsupported_table_length_raises_value_error(self):
        with self.assertRaises(ValueError):
            tpt_algo.Ekf(
                [(0.0, 3.0)],
                capacity_ah=2.0,
                r0=0.05,
                r1=0.02,
                c1=1000.0,
                soc0=0.5,
                soc_variance=0.05,
            )

    def test_invalid_capacity_raises_value_error(self):
        with self.assertRaises(ValueError):
            tpt_algo.Ekf(
                self.OCV,
                capacity_ah=0.0,
                r0=0.05,
                r1=0.02,
                c1=1000.0,
                soc0=0.5,
                soc_variance=0.05,
            )


if __name__ == "__main__":
    unittest.main()
