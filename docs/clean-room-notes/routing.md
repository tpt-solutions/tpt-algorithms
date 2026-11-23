# Clean-room notes: tpt-algo-routing

Written from memory of published descriptions; no third-party source code was
consulted or copied. No GPL/LGPL VRP solver source was used.

## Savings (Clarke & Wright 1964)
Start with one route per customer (depot-i-depot). Savings
s(i,j) = d(0,i) + d(0,j) - d(i,j). Process pairs by decreasing savings; merge
the two routes if i and j are endpoints of different routes and the merged
route is feasible (capacity; here also time windows). Parallel version.
Tie-break by index for determinism.

## Local search (Croes 1958; Lin 1965; Toth & Vigo 2014)
- 2-opt: reverse a segment of one route; delta from the two replaced edges.
- Relocate: move one customer to another position, within or between routes.
- Swap: exchange two customers between two routes.
First-improvement, strict improvement threshold, bounded move count.

## Time windows (Solomon 1987)
Arrival before the window opens means waiting; arrival after it closes is
infeasible. Service time added after start of service. Depot return deadline
optional. Travel time equals Euclidean distance.

## References
- Clarke & Wright, Operations Research 12(4), 1964.
- Croes, Operations Research 6(6), 1958.
- Lin, Bell System Technical Journal 44, 1965.
- Solomon, Operations Research 35(2), 1987.
- Toth & Vigo (eds.), Vehicle Routing: Problems, Methods, and Applications, SIAM, 2014.
