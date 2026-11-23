# Provenance: tpt-algo-routing

- Source of algorithms: published papers and book listed in
  `docs/clean-room-notes/routing.md` (Clarke & Wright 1964; Croes 1958;
  Lin 1965; Solomon 1987; Toth & Vigo 2014).
- Implementation: original code written from algorithm descriptions, from
  memory; no code copied from any VRP solver (GPL, LGPL or proprietary).
- Third-party dependencies: `libm`, `thiserror`, optional `serde`
  (workspace-pinned, MIT/Apache-2.0).
- Datasets: none bundled. Validation instances (CVRPLIB, Solomon) are
  user-supplied via `TPT_DATASET_DIR` and not redistributed.
- Human review needed: confirm clean-room claim and citations.
