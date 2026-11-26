#!/usr/bin/env bash
# Acceptance checks for Eval 10: Python task that should NOT trigger skill
set -uo pipefail
PROJECT="${1:?project dir required}"
FAIL=0
check() { if eval "$2"; then echo "PASS: $1"; else echo "FAIL: $1"; FAIL=$((FAIL+1)); fi; }

check "no ratatui dependency" "! grep -qi ratatui '$PROJECT/Cargo.toml' 2>/dev/null"
check "no rust files with ratatui" "! grep -rl ratatui '$PROJECT' --include='*.rs' 2>/dev/null | grep -q ."
check "python files exist" "find '$PROJECT' -name '*.py' | grep -q ."

exit "$FAIL"
