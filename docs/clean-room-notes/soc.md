# Clean-room notes: tpt-algo-soc

Written from memory of published descriptions; citations must be verified by a human.

## Sources (published descriptions only)
- R. E. Kalman, "A New Approach to Linear Filtering and Prediction Problems", J. Basic Eng., 1960.
- Extended Kalman Filter: first-order linearisation of nonlinear f and h about the current estimate (standard textbook treatment, e.g. Simon, "Optimal State Estimation", 2006; Bar-Shalom et al., "Estimation with Applications to Tracking and Navigation", 2001).
- G. L. Plett, "Extended Kalman filtering for battery management systems of LiPB-based HEV battery packs", J. Power Sources 134, 2004 (Parts 1-3): SoC as EKF state, OCV-SoC relation, equivalent-circuit measurement equation.
- First-order Thevenin (R0 + RC) equivalent-circuit model: widely published (e.g. He, Xiong et al., Energies 2011 style model comparisons).
- Joseph-form covariance update: Bucy and Joseph, "Filtering for Stochastic Processes", 1968; textbook description.
- Dataset (validation only, not used in code): NASA Prognostics Center Li-ion Battery Dataset (B. Saha and K. Goebel, 2007).

## Model implemented
State x = [SoC, V1]; input I (discharge positive).
- SoC' = SoC - dt*I/(3600*Q)
- V1' = V1*exp(-dt/(R1 C1)) + R1*(1 - exp(-dt/(R1 C1)))*I (exact zero-order-hold discretisation)
- z = OCV(SoC) - V1 - R0*I, OCV piecewise linear; H = [dOCV/dSoC, -1].
- F = diag(1, exp(-dt/(R1 C1))).
- Update: S = H P H^T + R, K = P H^T / S, x += K y, P = (I-KH) P (I-KH)^T + K R K^T, then symmetrise; SoC clamped to [0,1].

## Deviations
- dt is a `predict` argument, not a model parameter (allows variable sampling).
- OCV table is a const-generic array to keep the crate allocation-free.
- No parameter identification, temperature or hysteresis modelling.

## Clean-room statement
No code from any GPL/LGPL/proprietary implementation was consulted or copied. Equations are standard and re-derived.
