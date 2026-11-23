# Maintenance runbook

Written for: the maintainer on duty.

## Routine

| Cadence | Task | How |
|---|---|---|
| Weekly (automatic) | Dependency update PRs | Dependabot (`.github/dependabot.yml`). Review and merge. |
| Weekly (automatic) | Security audit | `audit-schedule.yml` runs `cargo audit` and `cargo deny`. Check the workflow result. |
| Each release | Release PR | release-plz opens it; merge to publish. See [release.md](release.md). |

## Dependency updates

1. Read the Dependabot PR and its changelog.
2. Run `cargo test --workspace` and `cargo deny check`.
3. If a new license appears, follow [dependencies.md](dependencies.md) before merging.

## Security advisories

1. A RustSec advisory appears in the scheduled audit or in a Dependabot alert.
2. Check whether the affected crate is reachable from a published `tpt-algo-*` crate.
3. Update the dependency. If no fix exists, record the risk in the crate README and open an issue.
4. For a vulnerability in one of our own crates, release a patch version as soon as the fix is
   tested.

## Release flow

See [release.md](release.md).

## Validation refresh

Dataset validation tests are `#[ignore]` and run manually with `TPT_DATASET_DIR` set. Re-run them
before each minor release and record the result in the crate's README "Validation status".

## Contacts

Maintainer contact is not yet set (decision D6 in [decisions.md](decisions.md)).
