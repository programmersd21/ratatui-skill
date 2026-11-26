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

failed=0
for crate in "$@"; do
  if ! info="$(cargo info "$crate" 2>&1)"; then
    printf '%s: cargo info failed\n' "$crate" >&2
    failed=$((failed + 1))
    continue
  fi
  latest="$(printf '%s\n' "$info" | sed -n 's/^version: *//p' | head -1)"
  msrv="$(printf '%s\n' "$info" | sed -n 's/^rust-version: *//p' | head -1)"
  if [[ -z "$latest" ]]; then
    printf '%s: no version in cargo info output\n' "$crate" >&2
    failed=$((failed + 1))
    continue
  fi

  project="not checked"
  if [[ -n "$manifest" ]]; then
    if tree="$(cargo tree --manifest-path "$manifest" -i "$crate" --depth 0 2>/dev/null)"; then
      project="$(printf '%s\n' "$tree" | sed -nE 's/.* v([0-9][^ ]*).*/\1/p' | sort -u | paste -sd ',' -)"
      [[ -n "$project" ]] || project="not resolved"
    else
      project="not a direct or resolved dependency"
    fi
  fi

  printf '%-20s latest=%-12s project=%-24s rust-version=%s\n' "$crate" "$latest" "$project" "${msrv:-not declared}"
done

if [[ "$failed" -gt 0 ]]; then
  printf '%s crate(s) unresolved\n' "$failed" >&2
  exit 1
fi
