# Provenance: tpt-algo-soc

| Item | Origin | Licence / status |
|---|---|---|
| Extended Kalman Filter equations | Published textbook/paper descriptions (Kalman 1960; Plett 2004; Simon 2006) | Mathematical method; not copyrightable. Implemented independently |
| First-order RC equivalent circuit | Published literature (see clean-room notes) | As above |
| Joseph-form covariance update | Bucy and Joseph 1968; textbooks | As above |
| Source code | Written from scratch in this repository | MIT OR Apache-2.0 |
| Test data | Seeded synthetic generator (in-test SplitMix64) | Generated, no third-party data |
| NASA Battery Dataset | Referenced only in an ignored test; never downloaded or redistributed | Public dataset; check terms before use |

Dependencies: libm, thiserror, optional serde (all MIT/Apache-2.0).

Caveat: clean-room claims were written from memory of the published descriptions. A human must verify citations and that no external code was consulted.
