# Validation datasets

Written for: maintainers running dataset validation.

Datasets are never committed. They are downloaded on demand by
`scripts/fetch-dataset.sh`, checked against a SHA-256 digest, and
cached in `.datasets/` (gitignored). Before any dataset is used in a published workflow, its terms
must be checked and recorded here.

| Dataset | Used by | Source | License / terms | SHA-256 | Status |
|---|---|---|---|---|---|
| NASA Battery Data Set | tpt-algo-soc | NASA Prognostics Center of Excellence | Check the source page before publishing a workflow. | TBD | Not fetched |
| LOBSTER sample data | tpt-algo-orderbook | lobsterdata.com | Terms: check before publishing a workflow. Some files are free samples; full data is licensed. | TBD | Not fetched |
| Solomon VRPTW instances | tpt-algo-routing | Solomon (1987) benchmark set | Academic use; check the distributing site's terms. | TBD | Not fetched |
| CVRPLIB instances | tpt-algo-routing | CVRPLIB (Uchoa et al.) | Check the site's terms. | TBD | Not fetched |
| Credit-card / KDD fraud data | tpt-algo-fraud | To be chosen | To be chosen; must be a redistributable or freely fetchable dataset. | TBD | Not chosen |

Digests stay `TBD` until a dataset is actually fetched and verified by a maintainer. Do not fill
in a digest from memory.

## Using the fetch script

```sh
scripts/fetch-dataset.sh <url> <expected-sha256> <name>
```

The script refuses to cache a file whose digest does not match. Re-run with the correct digest
once the file has been checked.
