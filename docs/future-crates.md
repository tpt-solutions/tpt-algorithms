# Future crates: triage (todo Phase 18)

Written for: maintainers choosing what to build after v0.1.

Each candidate must pass the same gates as the v0.1 crates: a permissive, clean-room
implementation (see [clean-room.md](clean-room.md)), a reproducible validation path on a public or
synthetic dataset, and the crate template (see [crate-template.md](crate-template.md)).

| Priority | Candidate | Domain | Validation idea | Notes |
|---|---|---|---|---|
| 1 | tpt-algo-kalman-general | Generic Kalman / UKF filters | Known-answer against textbook examples | Generalises the EKF in `tpt-algo-soc`. |
| 2 | tpt-algo-lp-core | Linear programming (simplex) | Known-answer and duality checks | Shared dependency for scheduling work. |
| 3 | tpt-algo-graph | Shortest paths, min-cost flow | Known-answer and cross-check against brute force on small graphs | Reused by routing and swarm. |
| 4 | tpt-algo-timeseries | Anomaly detection on streams | Synthetic streams from `tpt-algo-synth-data` | Overlaps with `tpt-algo-fraud`. |

Phase 18 is a triage, not a commitment. Items are not started until they are approved.
