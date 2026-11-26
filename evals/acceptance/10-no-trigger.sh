#!/usr/bin/env bash
# Acceptance checks for Eval 10: Python task that should NOT trigger skill
set -uo pipefail
PROJECT="${1:?project dir required}"
FAIL=0
check() { shift
  if "$@"; then echo "PASS: $1"; else echo "FAIL: $1"; FAIL=$((FAIL+1)); fi; }

if [[ ! -f "$PROJECT/Cargo.toml" ]] || ! grep -qi ratatui "$PROJECT/Cargo.toml"; then echo "PASS: no ratatui dependency"; else echo "FAIL: no ratatui dependency"; FAIL=$((FAIL+1)); fi
if ! grep -rl ratatui "$PROJECT" --include='*.rs' 2>/dev/null | grep -q .; then echo "PASS: no rust files with ratatui"; else echo "FAIL: no rust files with ratatui"; FAIL=$((FAIL+1)); fi
check "python files exist" find "$PROJECT" -name '*.py' -print -quit | grep -q .

exit "$FAIL"
