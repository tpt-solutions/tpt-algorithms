# Provenance: `tpt-algo-fraud`

- **Algorithm:** Isolation Forest anomaly scoring
- **Crate:** `tpt-algo-fraud`
- **Papers / specifications (full citations):**
  1. F. T. Liu, K. M. Ting, Z.-H. Zhou, "Isolation Forest", IEEE ICDM 2008.
  2. F. T. Liu, K. M. Ting, Z.-H. Zhou, "Isolation-based Anomaly Detection", ACM TKDD 6(1), 2012.
  3. G. L. Steele, D. Lea, C. H. Flood, "Fast splittable pseudorandom number generators", OOPSLA 2014 (SplitMix64).
- **Reference implementations consulted:** none.
- **Datasets used for validation:** none fetched. Optional ignored test reads a local
  ULB/Worldline credit-card fraud CSV via `TPT_DATASET_DIR`; terms must be checked before use,
  nothing is redistributed. Synthetic data is generated in-repo.
- **Implemented by:** Claude (agent), for the repository owner
- **Date started / date finished (YYYY-MM-DD):** 2026-11-23 / 2026-11-23
- **Reviewed by (name, date):** pending
- **Verification status:** worked from memory of the published descriptions; original papers
  not yet re-read.
- **Known deviations from the paper:** constant features in a node are skipped by trying the
  next feature cyclically; c(2) = 1 exactly; single-row training yields score 0.5.
