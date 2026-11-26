#!/usr/bin/env bash
# Acceptance checks for Eval 01: new Ratatui application (log viewer)
set -uo pipefail
PROJECT="${1:?project dir required}"
FAIL=0
check() { if eval "$2"; then echo "PASS: $1"; else echo "FAIL: $1"; FAIL=$((FAIL+1)); fi; }

check "compiles" "cargo build --manifest-path '$PROJECT/Cargo.toml' >/dev/null 2>&1"
check "clippy clean" "cargo clippy --manifest-path '$PROJECT/Cargo.toml' -- -D warnings >/dev/null 2>&1"
check "has tests" "cargo test --manifest-path '$PROJECT/Cargo.toml' >/dev/null 2>&1"
check "uses ratatui" "grep -q ratatui '$PROJECT/Cargo.toml'"
check "has main.rs" "test -f '$PROJECT/src/main.rs'"
check "has state module" "test -f '$PROJECT/src/state.rs'"
check "has ui module" "test -d '$PROJECT/src/ui' -o -f '$PROJECT/src/ui.rs'"
check "no unwrap in non-test" "! grep -rn '\\.unwrap()' '$PROJECT/src' --include='*.rs' | grep -v '#\\[cfg(test)\\]' | grep -v 'tests/' | grep -q ."

exit "$FAIL"
