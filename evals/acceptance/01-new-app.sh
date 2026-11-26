#!/usr/bin/env bash
# Acceptance checks for Eval 01: new Ratatui application (log viewer)
set -uo pipefail
PROJECT="${1:?project dir required}"
FAIL=0
check() {
  local label="$1"
  shift
  if "$@"; then echo "PASS: $label"; else echo "FAIL: $label"; FAIL=$((FAIL+1)); fi
}

check "compiles" cargo build --manifest-path "$PROJECT/Cargo.toml"
check "clippy clean" cargo clippy --manifest-path "$PROJECT/Cargo.toml" -- -D warnings
check "has tests" cargo test --manifest-path "$PROJECT/Cargo.toml"
check "uses ratatui" grep -q ratatui "$PROJECT/Cargo.toml"
check "has main.rs" test -f "$PROJECT/src/main.rs"
check "has state module" test -f "$PROJECT/src/state.rs"
if [[ -d "$PROJECT/src/ui" || -f "$PROJECT/src/ui.rs" ]]; then echo "PASS: has ui module"; else echo "FAIL: has ui module"; FAIL=$((FAIL+1)); fi
if ! grep -rn \.unwrap "$PROJECT/src" --include='*.rs' --exclude-dir=tests | grep -qv '#\[cfg(test)\]'; then echo "PASS: no unwrap in non-test"; else echo "FAIL: no unwrap in non-test"; FAIL=$((FAIL+1)); fi

exit "$FAIL"
