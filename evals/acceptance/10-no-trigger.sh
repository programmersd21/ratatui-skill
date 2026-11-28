#!/usr/bin/env bash
set -Eeuo pipefail
PROJECT="${1:?project dir required}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

test -f "$PROJECT/solution.py"
python3 -m py_compile "$PROJECT/solution.py"

cat > "$TMP/input.csv" <<'CSV'
A,B,C,Name,Empty
1,2,,alpha,
3,4,5,beta,
,6,7,gamma,
CSV

python3 "$PROJECT/solution.py" "$TMP/input.csv" > "$TMP/output.json"
python3 - "$TMP/output.json" <<'PY'
import json
import math
import pathlib
import sys

data = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert set(data) == {"A", "B", "C"}
assert data["A"]["count"] == 2
assert data["A"]["mean"] == 2
assert data["A"]["median"] == 2
assert math.isclose(data["A"]["std"], 1)
assert data["B"]["count"] == 3
assert data["B"]["mean"] == 4
assert data["B"]["median"] == 4
assert math.isclose(data["B"]["std"], math.sqrt(8 / 3))
assert data["C"]["count"] == 2
assert data["C"]["mean"] == 6
assert data["C"]["median"] == 6
assert math.isclose(data["C"]["std"], 1)
PY

if python3 "$PROJECT/solution.py" "$TMP/missing.csv" > "$TMP/missing.out" 2> "$TMP/missing.err"; then
  echo "missing input unexpectedly succeeded" >&2
  exit 1
fi
test -s "$TMP/missing.err"
if find "$PROJECT" -type f \( -name Cargo.toml -o -name '*.rs' \) -print -quit | grep -q .; then
  echo "non-TUI Python task created a Rust project" >&2
  exit 1
fi
echo "Python CSV acceptance passed."
