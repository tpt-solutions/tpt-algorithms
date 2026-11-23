// SPDX-License-Identifier: MIT OR Apache-2.0
//! Dataset-backed validation (NASA Battery Dataset). Not run in CI.
#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn dataset_dir() -> Option<PathBuf> {
    match std::env::var_os("TPT_DATASET_DIR") {
        Some(d) => Some(PathBuf::from(d)),
        None => {
            eprintln!("TPT_DATASET_DIR is not set; skipping NASA Battery Dataset validation");
            None
        }
    }
}

#[test]
#[ignore = "requires dataset; set TPT_DATASET_DIR"]
fn nasa_battery_dataset_soc_rmse() {
    let Some(dir) = dataset_dir() else {
        return;
    };
    // Structure only: a human must (1) fit the OCV table and R0/R1/C1 for the
    // chosen cell from the dataset's reference discharge, (2) replay a
    // discharge cycle's current/voltage through the EKF, (3) compare against
    // Coulomb-counted reference SoC and assert the RMSE against the bound
    // claimed in the cited paper. Not implemented until the dataset is
    // available locally.
    assert!(dir.exists(), "TPT_DATASET_DIR does not exist: {dir:?}");
    eprintln!("dataset dir found at {dir:?}; replay not yet implemented");
}
