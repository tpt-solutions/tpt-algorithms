# Clean-room notes: tpt-algo-swarm

Written from memory of the published papers' descriptions. No source code of
any implementation (including RVO2 / ORCA reference code) was consulted or
copied. A human reviewer should check these notes against the papers.

## Sources
- Fiorini & Shiller, "Motion planning in dynamic environments using velocity
  obstacles", IJRR 17(7), 1998.
- van den Berg, Lin & Manocha, "Reciprocal velocity obstacles for real-time
  multi-agent navigation", ICRA 2008.
- van den Berg, Guy, Lin & Manocha, "Reciprocal n-body collision avoidance",
  ISRR 2011 (ORCA).
- Hart, Nilsson & Raphael, "A formal basis for the heuristic determination of
  minimum cost paths", IEEE Trans. SSC 4(2), 1968 (A*).

## Velocity obstacle (Fiorini & Shiller)
For disc agents A, B with radii rA, rB, the velocity obstacle VO(A|B) is the
set of velocities of A that lead to a collision with B at some future time,
assuming B keeps its velocity. Equivalently, in relative-velocity space, the
set of v = vA - vB such that the ray from pA with direction v hits the disc of
radius rA + rB around pB. Truncating at horizon tau keeps only collisions with
time <= tau (a cone cut by an arc).

Implementation choice: exact test by solving |p - v t|^2 = R^2 for the first
contact time (a quadratic) and comparing with tau. No sampling.

## Reciprocal velocity obstacle (van den Berg et al. 2008)
RVO(A|B) translates the VO apex from vB to (vA + vB)/2, so each agent takes
half the avoidance effort and oscillations of the plain VO are reduced.
Candidate velocity v' of A is inside RVO iff 2v' - vA - vB lies in the
(origin-centred) VO.

## ORCA (van den Berg et al. 2011)
ORCA replaces each RVO by a half-plane of permitted velocities and solves a
low-dimensional linear program. NOT implemented here. This crate uses the
sampled-candidate variant of RVO only; this is documented as such in the API.

## Selection rule used
Sampling of a deterministic polar grid over the max-speed disc plus the
preferred velocity. Free candidates (outside all obstacles) are ranked by
distance to the preferred velocity; if none is free, the latest earliest
collision time wins. Three heuristics are the crate author's own engineering,
not taken from the papers: a hysteresis term towards the current velocity,
a small right-hand-rule bias, and treating the zero velocity as a fallback
only. They exist to break symmetric deadlocks found by property testing.

## A*
Standard best-first search with f = g + h, closed set, consistent heuristic
(Manhattan for 4-connected, octile for 8-connected; costs scaled by 10 with
diagonal = 14 so arithmetic is exact in integers). Ties: larger g, then lower
cell index. Diagonal moves may not cut blocked corners.
