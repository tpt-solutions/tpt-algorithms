# Clean-room notes: tpt-algo-fraud (Isolation Forest)

Written from memory of the published descriptions; no third-party source code
was consulted. A human should check against the papers.

## References
- F. T. Liu, K. M. Ting, Z.-H. Zhou, "Isolation Forest", IEEE ICDM 2008.
- F. T. Liu, K. M. Ting, Z.-H. Zhou, "Isolation-based Anomaly Detection",
  ACM TKDD 6(1), 2012.

## Algorithm as implemented
- iTree: on a node's points, pick a random feature q and a split value p
  uniform between min and max of q in the node; left gets x_q < p. Stop at
  depth limit ceil(log2 psi), one point, or when all points are identical.
- iForest: t trees, each on a subsample of size psi drawn without replacement
  (defaults t = 100, psi = 256).
- Path length h(x) = edges traversed + c(size) at the external node.
- c(n) = 2 H(n-1) - 2(n-1)/n, H(i) ~ ln i + 0.5772156649; c(1) = 0, c(2) = 1.
- Score s(x, n) = 2^(-E[h(x)] / c(n)); near 1 anomalous, below 0.5 normal.

## Deviations and choices
- If the chosen feature is constant in a node, the next features are tried
  cyclically (the papers just stop). Exact c(2) = 1 rather than the approximation.
- Single-row training gives c = 0; score defined as 0.5.
- RNG: SplitMix64, written independently.
