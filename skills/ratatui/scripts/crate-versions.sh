#!/usr/bin/env bash
# Print the newest published version of each crate, with its declared minimum
# Rust version, using `cargo info` (Cargo's own registry client).
#
# usage: crate-versions.sh CRATE [CRATE...]
#
# Run this before writing a Cargo.toml. Never copy a version from memory,
# from an old tutorial, or from this skill's own examples.
# Exit 0 only when every crate was resolved.

set -u
[ "$#" -gt 0 ] || { echo "usage: crate-versions.sh CRATE [CRATE...]" >&2; exit 2; }

failed=0
for crate in "$@"; do
  # cargo info prints "name #keywords" on the first line, then fields.
  out="$(cargo info "$crate" 2>&1)" || {
    echo "$crate: cargo info failed" >&2
    failed=$((failed + 1))
    continue
  }
  version="$(printf '%s\n' "$out" | sed -n 's/^version: *//p' | head -1)"
  msrv="$(printf '%s\n' "$out" | sed -n 's/^rust-version: *//p' | head -1)"
  [ -n "$version" ] || { echo "$crate: no version in cargo info output" >&2; failed=$((failed + 1)); continue; }
  printf '%-24s %-12s rust-version %s\n' "$crate" "$version" "${msrv:-not declared}"
done

if [ "$failed" -gt 0 ]; then
  echo "$failed crate(s) unresolved" >&2
  exit 1
fi
