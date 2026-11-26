#!/usr/bin/env bash
# Run one evaluation trial.
#
# usage: run-trial.sh TASK_FILE WITH_SKILL(0|1) TRIAL_DIR [TIMEOUT]
#
# Sets up a temp Rust project, runs opencode with the task prompt, captures
# all output, and writes a structured result.json with honest exit status.
set -uo pipefail

TASK_FILE="${1:?task file required}"
WITH_SKILL="${2:?0 or 1}"
TRIAL_DIR="${3:?trial dir required}"
TIMEOUT="${4:-300}"

# Resolve to absolute path under repo root (opencode sandbox requirement).
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$TRIAL_DIR"
TRIAL_DIR="$(cd "$TRIAL_DIR" && pwd)"
if [[ "$TRIAL_DIR" != "$REPO_ROOT"* ]]; then
  echo "TRIAL_DIR must be under repo root for opencode sandbox" >&2
  exit 2
fi

TASK=$(cat "$TASK_FILE")
PROJECT="$TRIAL_DIR/project"
mkdir -p "$PROJECT"

# Skill injection: copy skill to opencode skills dir and verify.
SKILL_SHA=""
if [ "$WITH_SKILL" = "1" ]; then
  SKILL_SRC="$REPO_ROOT/skills/ratatui"
  SKILL_DST="$HOME/.config/opencode/skills/ratatui"
  rm -rf "$SKILL_DST"
  cp -r "$SKILL_SRC" "$SKILL_DST"
  SKILL_SHA=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "unknown")
  # Verify skill is discoverable.
  if [ ! -f "$SKILL_DST/SKILL.md" ]; then
    echo "skill injection failed" >&2
    exit 2
  fi
  PROMPT="Use the ratatui skill. $TASK"
else
  # Remove skill to ensure no leakage.
  rm -rf "$HOME/.config/opencode/skills/ratatui"
  PROMPT="$TASK"
fi

# Write prompt.
printf '%s\n' "$PROMPT" > "$TRIAL_DIR/prompt.txt"

# Initialize Cargo project.
cat > "$PROJECT/Cargo.toml" <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
TOML

# Run opencode with timeout. Capture exit status honestly.
START_TIME=$(date +%s)
timeout "$TIMEOUT" opencode run "$(cat "$TRIAL_DIR/prompt.txt")" \
  > "$TRIAL_DIR/stdout.txt" 2> "$TRIAL_DIR/stderr.txt"
EXIT_STATUS=$?
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

# Classify exit.
if [ "$EXIT_STATUS" = "124" ]; then
  TIMED_OUT=true
elif [ "$EXIT_STATUS" = "0" ]; then
  TIMED_OUT=false
else
  TIMED_OUT=false
fi

# Collect results (do not suppress errors).
COMPILE_OK=false
CLIPPY_OK=false
TEST_OK=false
HAS_RATATUI=false
FILES_GENERATED=0

if cargo build --manifest-path "$PROJECT/Cargo.toml" >/dev/null 2>&1; then
  COMPILE_OK=true
fi
if cargo clippy --manifest-path "$PROJECT/Cargo.toml" -- -D warnings >/dev/null 2>&1; then
  CLIPPY_OK=true
fi
if cargo test --manifest-path "$PROJECT/Cargo.toml" >/dev/null 2>&1; then
  TEST_OK=true
fi
if grep -q ratatui "$PROJECT/Cargo.toml" 2>/dev/null; then
  HAS_RATATUI=true
fi
FILES_GENERATED=$(find "$PROJECT" -name '*.rs' -not -path '*/target/*' | wc -l)

# Write structured result.
cat > "$TRIAL_DIR/result.json" <<JSON
{
  "task": "$(basename "$TASK_FILE")",
  "with_skill": $WITH_SKILL,
  "exit_status": $EXIT_STATUS,
  "timed_out": $TIMED_OUT,
  "elapsed_seconds": $ELAPSED,
  "skill_sha": "$SKILL_SHA",
  "compile_ok": $COMPILE_OK,
  "clippy_ok": $CLIPPY_OK,
  "test_ok": $TEST_OK,
  "has_ratatui": $HAS_RATATUI,
  "files_generated": $FILES_GENERATED
}
JSON

cat "$TRIAL_DIR/result.json"
