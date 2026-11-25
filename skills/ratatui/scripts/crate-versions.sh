#!/usr/bin/env bash
# Print the newest non-yanked, non-prerelease version of each crate, with its
# declared minimum Rust version, straight from the crates.io sparse index.
#
# usage: crate-versions.sh CRATE [CRATE...]
#
# Run this before writing a Cargo.toml. Never copy a version from memory,
# from an old tutorial, or from this skill's own examples.

set -eu
[ "$#" -gt 0 ] || { echo "usage: crate-versions.sh CRATE [CRATE...]" >&2; exit 2; }

index_path() {
  local n="$1"
  case "${#n}" in
    1) echo "1/$n" ;;
    2) echo "2/$n" ;;
    3) echo "3/${n:0:1}/$n" ;;
    *) echo "${n:0:2}/${n:2:2}/$n" ;;
  esac
}

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

for crate in "$@"; do
  name="$(printf '%s' "$crate" | tr 'A-Z' 'a-z')"
  curl -fsS --max-time 20 -o "$tmp" "https://index.crates.io/$(index_path "$name")" 2>/dev/null || {
    printf '%-24s not found or index unreachable\n' "$crate"
    continue
  }
  python3 - "$name" "$tmp" <<'PY'
import json, sys

name, path = sys.argv[1], sys.argv[2]
with open(path, encoding="utf-8") as f:
    rows = [json.loads(line) for line in f if line.strip()]
stable = [r for r in rows if not r["yanked"] and "-" not in r["vers"]]
if not stable:
    print(f"{name:24s} no stable release")
else:
    r = stable[-1]
    print(f"{name:24s} {r['vers']:12s} rust-version {r.get('rust_version') or 'unset'}")
PY
done
