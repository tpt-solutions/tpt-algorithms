# Provenance: tpt-algo-consensus

| Item | Origin | Licence / status |
| --- | --- | --- |
| Chained HotStuff vote, lock and commit rules | Yin et al., *HotStuff*, PODC 2019 (paper text, implemented from description) | Algorithm description; no code copied |
| `n = 3f + 1`, quorum intersection, view-change concept | Castro and Liskov, *PBFT*, OSDI 1999 (paper text) | Algorithm description; no code copied |
| Consecutive-view three-chain formulation | Equivalent restatement of the paper's direct-parent rule without dummy blocks | Original wording in this crate |
| `(f + 1)` view-synchronisation jump, last vote in `NewView`, any-block vote quorum readiness | Design choices made for this crate (see clean-room notes) | Original |
| SplitMix64 mixing constants and finalizer (`mix64`) | Steele, Lea, Flood, OOPSLA 2014 (published algorithm and constants) | Public algorithm; reimplemented |
| Simulator, type-state `Round`, tests, scenarios | Written for this crate | MIT OR Apache-2.0 |

- Runtime dependencies: `thiserror` (MIT OR Apache-2.0), optional `serde`
  (MIT OR Apache-2.0). `libm` is declared in the manifest but not used: the
  crate has no floating-point code.
- Dev dependencies: `proptest` (MIT OR Apache-2.0). `tpt-algo-synth-data` is
  declared but not used.
- No datasets were downloaded. Validation uses scripted scenarios in
  `tests/scenarios.rs`. The ignored `tests/validation.rs` reads optional
  local files only.
- Clean-room statement: implemented from the papers as recalled by the
  author. No third-party HotStuff, LibraBFT/DiemBFT or PBFT source code was
  viewed or copied. A human reviewer should confirm the paper citations and
  rule descriptions against the published papers.
