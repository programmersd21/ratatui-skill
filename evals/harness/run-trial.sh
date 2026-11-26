#!/usr/bin/env bash
# Run one evaluation trial.
#
# usage: run-trial.sh TASK_FILE WITH_SKILL(0|1) TRIAL_DIR
set -euo pipefail

TASK_FILE="${1:?task file required}"
WITH_SKILL="${2:?0 or 1}"
TRIAL_DIR="${3:?trial dir required}"

mkdir -p "$TRIAL_DIR"
TASK=$(cat "$TASK_FILE")

PROJECT="$TRIAL_DIR/project"
mkdir -p "$PROJECT"
cd "$PROJECT"

# Initialize a minimal Cargo project.
cat > Cargo.toml <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
TOML

# Build prompt with or without skill reference.
if [ "$WITH_SKILL" = "1" ]; then
  PROMPT="Use the ratatui skill. $TASK"
else
  PROMPT="$TASK"
fi
printf '%s\n' "$PROMPT" > "$TRIAL_DIR/prompt.txt"

# Run opencode, capture output.
timeout 180 opencode run "$(cat "$TRIAL_DIR/prompt.txt")" \
  > "$TRIAL_DIR/opencode-output.txt" 2>&1 || true

# Collect results.
cd "$TRIAL_DIR"
{
  echo "{"
  echo "  \"task\": \"$(basename "$TASK_FILE")\","
  echo "  \"with_skill\": $WITH_SKILL,"
  echo "  \"compile_ok\": $(if cargo build --manifest-path \"$PROJECT/Cargo.toml\" >/dev/null 2>&1; then echo true; else echo false; fi),"
  echo "  \"clippy_ok\": $(if cargo clippy --manifest-path \"$PROJECT/Cargo.toml\" -- -D warnings >/dev/null 2>&1; then echo true; else echo false; fi),"
  echo "  \"test_ok\": $(if cargo test --manifest-path \"$PROJECT/Cargo.toml\" >/dev/null 2>&1; then echo true; else echo false; fi),"
  echo "  \"has_ratatui\": $(if grep -q ratatui \"$PROJECT/Cargo.toml\" 2>/dev/null; then echo true; else echo false; fi),"
  echo "  \"files_generated\": $(find "$PROJECT" -name '*.rs' | wc -l)"
  echo "}"
} > "$TRIAL_DIR/result.json"

cat "$TRIAL_DIR/result.json"
