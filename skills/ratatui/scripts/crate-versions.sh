#!/usr/bin/env bash
# Print the newest non-yanked, non-prerelease version of each crate, with its
# declared minimum Rust version, straight from the crates.io sparse index.
#
# usage: crate-versions.sh CRATE [CRATE...]
#
# Run this before writing a Cargo.toml. Never copy a version from memory,
# from an old tutorial, or from this skill's own examples.
# Exit 0 only when every crate was resolved.

set -u
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
failed=0

for crate in "$@"; do
  name="$(printf '%s' "$crate" | tr 'A-Z' 'a-z')"
  case "$name" in
    ''|*[!a-z0-9_-]*)
      printf '%-24s invalid crate name\n' "$crate" >&2
      failed=$((failed + 1))
      continue
      ;;
  esac

  if ! curl -fsS --max-time 20 -o "$tmp" "https://index.crates.io/$(index_path "$name")" 2>/dev/null; then
    # Distinguish a missing crate (404) from a network problem.
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 20 \
      "https://index.crates.io/$(index_path "$name")" 2>/dev/null || echo 000)"
    if [ "$code" = 404 ]; then
      printf '%-24s not found in crates.io index\n' "$crate" >&2
    else
      printf '%-24s index unreachable (http %s)\n' "$crate" "$code" >&2
    fi
    failed=$((failed + 1))
    continue
  fi

  if ! python3 - "$name" "$tmp" <<'PY'
import json, sys

name, path = sys.argv[1], sys.argv[2]
with open(path, encoding="utf-8") as f:
    rows = [json.loads(line) for line in f if line.strip()]
stable = [r for r in rows if not r.get("yanked") and "-" not in r["vers"]]
if not stable:
    print(f"{name:24s} no stable release")
    sys.exit(1)

def parts(v):
    return tuple(int(p) for p in v.split(".")[:3])

latest = max(stable, key=lambda r: parts(r["vers"]))
msrv = latest.get("rust_version") or "not declared"
print(f"{name:24s} {latest['vers']:12s} rust-version {msrv}")
PY
  then
    failed=$((failed + 1))
  fi
done

if [ "$failed" -gt 0 ]; then
  echo "$failed crate(s) unresolved" >&2
  exit 1
fi
