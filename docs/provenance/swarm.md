# Provenance: tpt-algo-swarm

| Item | Source | Notes |
|------|--------|-------|
| Velocity obstacle (truncated cone) | Fiorini & Shiller, IJRR 1998 | Exact quadratic first-contact test |
| Reciprocal VO (sampled) | van den Berg, Lin, Manocha, ICRA 2008 | Apex at mean velocity |
| ORCA | van den Berg, Guy, Lin, Manocha, ISRR 2011 | Cited for context; not implemented |
| A* grid search | Hart, Nilsson, Raphael, IEEE TSSC 1968 | Integer 10/14 octile costs |
| Hysteresis, side bias, stop-as-fallback | Original engineering | Added after proptest found deadlocks |

Method: implemented from the author's memory of the published descriptions.
No third-party implementation source (GPL, LGPL, proprietary, or the Apache-2.0
RVO2 / ORCA reference) was read or copied. Dependencies: libm (MIT/Apache),
thiserror, optional serde. Test data: seeded/synthetic scenarios only.

Status: clean-room claims unreviewed by a human.
