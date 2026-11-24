# tpt-algorithms developer tasks. Run `just --list` to see them.
#
# Requires `just` (cargo install just, or winget install Casey.Just). Recipes also need
# the tool they call: cargo-deny, maturin, wasm-pack, mdbook, and bash for fetch-data.
#
# Windows: `just` runs recipes with the shell below. Git for Windows provides `sh`; make sure
# its bin directory is on PATH. If `just` is not installed, run the commands in each recipe
# directly; CONTRIBUTING.md lists them.

set windows-shell := ["sh", "-cu"]

default:
    @just --list

# Format and lint check. Writes nothing.
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

# The Rust part of CI: check, tests with and without default features, and cargo-deny.
ci: check
    cargo test --workspace
    cargo test --workspace --no-default-features
    cargo deny check

# Download a validation dataset, verify its SHA-256 and cache it in .datasets/<name>.
# Example: just fetch-data lobster-sample https://example.org/file.zip <64-hex-sha256>
fetch-data name url sha256:
    scripts/fetch-dataset.sh "{{ url }}" "{{ sha256 }}" "{{ name }}"

# Criterion benchmarks. Extra arguments go to `cargo bench`.
# Example: just bench --bench routing
bench *args:
    cargo bench -p tpt-algo-benches {{ args }}

# Python (maturin, then unittest) and WASM (wasm-pack, Node) binding tests.
bindings-test:
    cd crates/tpt-algo-py && maturin build --release --out dist && python -m pip install --force-reinstall dist/*.whl && python -m unittest discover -s tests -v
    cd crates/tpt-algo-wasm && wasm-pack test --node

# Build the mdBook documentation into target/book.
book:
    mdbook build

# Build the WebAssembly package for the browser demo (docs/demo). Serve docs/demo over HTTP to run it.
demo:
    wasm-pack build crates/tpt-algo-wasm --target web --out-dir ../../docs/demo/pkg --release
