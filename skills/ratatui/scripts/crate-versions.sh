#!/usr/bin/env bash
set -euo pipefail

manifest=""
if [[ "${1:-}" == "--manifest" ]]; then
  [[ $# -ge 3 ]] || { echo "usage: crate-versions.sh [--manifest Cargo.toml] CRATE..." >&2; exit 2; }
  manifest="$2"
  shift 2
fi
[[ $# -gt 0 ]] || { echo "usage: crate-versions.sh [--manifest Cargo.toml] CRATE..." >&2; exit 2; }
if [[ -n "$manifest" && ! -f "$manifest" ]]; then
  printf 'manifest not found: %s\n' "$manifest" >&2
  exit 2
fi

metadata=""
if [[ -n "$manifest" ]]; then
  if ! metadata="$(cargo metadata --format-version 1 --manifest-path "$manifest" 2>&1)"; then
    printf 'cargo metadata failed for %s:\n%s\n' "$manifest" "$metadata" >&2
    exit 1
  fi
fi

failed=0
for crate in "$@"; do
  if ! info="$(cargo info "$crate" 2>&1)"; then
    printf '%s: cargo info failed:\n%s\n' "$crate" "$info" >&2
    failed=$((failed + 1))
    continue
  fi

  latest="$(printf '%s\n' "$info" | sed -n 's/^version: *//p' | head -1)"
  msrv="$(printf '%s\n' "$info" | sed -n 's/^rust-version: *//p' | head -1)"
  if [[ -z "$latest" ]]; then
    printf '%s: no version found in cargo info output\n' "$crate" >&2
    failed=$((failed + 1))
    continue
  fi

  project="not checked"
  if [[ -n "$manifest" ]]; then
    if ! project="$(printf '%s\n' "$metadata" | python3 -c '
import json
import sys

crate = sys.argv[1]
data = json.load(sys.stdin)
resolved = {node["id"] for node in (data.get("resolve") or {}).get("nodes", [])}
versions = sorted({
    package["version"]
    for package in data["packages"]
    if package["id"] in resolved and package["name"] == crate
})
print(",".join(versions) if versions else "not in resolved dependency graph")
' "$crate")"; then
      printf '%s: failed to inspect resolved Cargo metadata\n' "$crate" >&2
      failed=$((failed + 1))
      continue
    fi
  fi

  printf '%-20s latest=%-12s project=%-32s rust-version=%s\n' "$crate" "$latest" "$project" "${msrv:-not declared}"
done

if [[ "$failed" -gt 0 ]]; then
  printf '%s crate(s) could not be inspected\n' "$failed" >&2
  exit 1
fi
