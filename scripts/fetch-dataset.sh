#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Download a validation dataset on demand, verify its SHA-256, and cache it in
# .datasets/<name>. Refuses to keep a file whose digest does not match.
#
# Usage: scripts/fetch-dataset.sh <url> <expected-sha256> <name>
set -euo pipefail

if [ "$#" -ne 3 ]; then
    echo "usage: $0 <url> <expected-sha256> <name>" >&2
    exit 2
fi

url="$1"
expected="$2"
name="$3"

case "$name" in
    *[!A-Za-z0-9._-]*|"") echo "error: name must be [A-Za-z0-9._-]+" >&2; exit 2 ;;
esac

if ! [[ "$expected" =~ ^[0-9a-fA-F]{64}$ ]]; then
    echo "error: expected digest must be 64 hex characters" >&2
    exit 2
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
cache="$root/.datasets"
dest="$cache/$name"
mkdir -p "$cache"

if [ -f "$dest" ]; then
    actual="$(sha256sum "$dest" | cut -d' ' -f1)"
    if [ "$actual" = "$expected" ]; then
        echo "cached and verified: $dest"
        exit 0
    fi
    echo "cached file digest mismatch; re-downloading" >&2
    rm -f "$dest"
fi

tmp="$(mktemp "$cache/.download.XXXXXX")"
trap 'rm -f "$tmp"' EXIT

curl --fail --location --silent --show-error --output "$tmp" "$url"
actual="$(sha256sum "$tmp" | cut -d' ' -f1)"

if [ "$actual" != "$(echo "$expected" | tr 'A-F' 'a-f')" ]; then
    echo "error: SHA-256 mismatch for $url" >&2
    echo "  expected: $expected" >&2
    echo "  actual:   $actual" >&2
    exit 1
fi

mv "$tmp" "$dest"
trap - EXIT
echo "verified and cached: $dest"
