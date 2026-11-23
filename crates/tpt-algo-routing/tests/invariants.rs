// SPDX-License-Identifier: MIT OR Apache-2.0
//! Property tests: coverage, capacity, time windows, monotone local search.
#![cfg(feature = "alloc")]
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use tpt_algo_routing::{clarke_wright_savings, local_search, solve, Config, Customer, Problem};

fn problem_strategy() -> impl Strategy<Value = Problem> {
    (
        15.0f64..40.0,
        any::<bool>(),
        proptest::collection::vec(
            (
                0.0f64..100.0,
                0.0f64..100.0,
                1.0f64..10.0,
                0.0f64..100.0,
                150.0f64..400.0,
                0.0f64..5.0,
            ),
            1..25,
        ),
    )
        .prop_map(|(cap, windows, cs)| {
            let customers = cs
                .into_iter()
                .map(|(x, y, d, e, w, s)| {
                    let c = Customer::new(x, y, d);
                    if windows {
                        c.with_window(e, e + w, s)
                    } else {
                        c
                    }
                })
                .collect();
            Problem::new((50.0, 50.0), cap, customers).unwrap()
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn savings_is_valid(p in problem_strategy()) {
        let s = clarke_wright_savings(&p).unwrap();
        prop_assert!(s.validate(&p).is_ok());
        let served: usize = s.routes.iter().map(Vec::len).sum();
        prop_assert_eq!(served, p.customers().len());
        for r in &s.routes {
            let load: f64 = r.iter().map(|&c| p.customers()[c].demand).sum();
            prop_assert!(load <= p.capacity() + 1e-9);
        }
    }

    #[test]
    fn local_search_never_worsens(p in problem_strategy()) {
        let mut s = clarke_wright_savings(&p).unwrap();
        let before = s.cost(&p);
        local_search(&p, &mut s, &Config::default()).unwrap();
        prop_assert!(s.validate(&p).is_ok());
        prop_assert!(s.cost(&p) <= before + 1e-9);
    }

    #[test]
    fn solve_is_deterministic(p in problem_strategy()) {
        let a = solve(&p, &Config::default()).unwrap();
        let b = solve(&p, &Config::default()).unwrap();
        prop_assert_eq!(a, b);
    }
}
