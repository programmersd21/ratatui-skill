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
    cat > main.py <<'PY'
import csv, statistics
print("done")
PY
    echo "Generated Python script"
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
