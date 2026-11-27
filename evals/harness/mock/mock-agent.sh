#!/usr/bin/env bash
# Mock agent for deterministic harness testing.
# Simulates opencode behavior based on MOCK_BEHAVIOR env var.
#
# MOCK_BEHAVIOR: success (default), fail, timeout, python, no-ratatui
set -uo pipefail

BEHAVIOR="${MOCK_BEHAVIOR:-success}"
PROMPT="$*"

case "$BEHAVIOR" in
  success)
    # Simulate a successful Rust app generation.
    mkdir -p src ui
    cat > Cargo.toml <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.30"
TOML
    cat > src/main.rs <<'RS'
fn main() { println!("hello"); }
RS
    cat > src/lib.rs <<'RS'
pub mod app;
RS
    echo "Generated successfully"
    ;;
  fail)
    echo "Error: something went wrong" >&2
    exit 1
    ;;
  timeout)
    sleep 300
    ;;
  python)
    cat > solution.py <<'PY'
import csv
import json
import math
import statistics
import sys

with open(sys.argv[1], newline="", encoding="utf-8") as stream:
    rows = list(csv.DictReader(stream))
result = {}
for key in rows[0] if rows else []:
    values = []
    valid = True
    for row in rows:
        value = row[key].strip()
        if not value:
            continue
        try:
            number = float(value)
        except ValueError:
            valid = False
            break
        if not math.isfinite(number):
            valid = False
            break
        values.append(number)
    if valid and values:
        result[key] = {
            "count": len(values),
            "mean": statistics.mean(values),
            "median": statistics.median(values),
            "std": statistics.pstdev(values),
        }
print(json.dumps(result))
PY
    ;;
  no-ratatui)
    mkdir -p src
    cat > Cargo.toml <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
TOML
    cat > src/main.rs <<'RS'
fn main() { println!("hello"); }
RS
    echo "Generated without ratatui"
    ;;
esac


if [[ "${MOCK_SKILL_CALL:-0}" == 1 ]]; then
  printf '%s\n' '{"type":"tool_use","part":{"tool":"skill","state":{"status":"completed","input":{"name":"ratatui"}}}}'
fi
if [[ "${MOCK_SUBAGENT_CALL:-0}" == 1 ]]; then
  printf '%s\n' '{"type":"tool_use","part":{"tool":"task","state":{"status":"completed","input":{"description":"delegate implementation"}}}}'
fi
printf '%s\n' '{"type":"text","part":{"type":"text","text":"Mock agent completed"}}'
