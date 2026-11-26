#!/usr/bin/env bash
# Run one evaluation trial with honest skill injection and isolation.
#
# usage: run-trial.sh TASK_FILE WITH_SKILL(0|1) TRIAL_DIR [TIMEOUT]
set -uo pipefail

TASK_FILE="${1:?task file required}"
WITH_SKILL="${2:?0 or 1}"
TRIAL_DIR="${3:?trial dir required}"
TIMEOUT="${4:-300}"

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

# Isolated opencode config per trial.
CONFIG_DIR="$TRIAL_DIR/opencode-config"
mkdir -p "$CONFIG_DIR"
# Copy auth from real config (read-only reference, not modified).
if [ -f "$HOME/.config/opencode/auth.json" ]; then
  cp "$HOME/.config/opencode/auth.json" "$CONFIG_DIR/auth.json"
fi
if [ -f "$HOME/.config/opencode/opencode.json" ]; then
  cp "$HOME/.config/opencode/opencode.json" "$CONFIG_DIR/opencode.json"
fi

# Skill injection: copy skill only for skill-enabled trials.
SKILL_SHA="none"
SKILL_INJECTED=false
if [ "$WITH_SKILL" = "1" ]; then
  SKILL_SRC="$REPO_ROOT/skills/ratatui"
  SKILL_DST="$CONFIG_DIR/skills/ratatui"
  mkdir -p "$CONFIG_DIR/skills"
  cp -r "$SKILL_SRC" "$SKILL_DST"
  SKILL_SHA=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "unknown")
  SKILL_INJECTED=true
  # Verify skill file present.
  if [ ! -f "$SKILL_DST/SKILL.md" ]; then
    echo "skill injection failed: SKILL.md not found" >&2
    exit 2
  fi
fi

# Build prompt.
if [ "$WITH_SKILL" = "1" ]; then
  PROMPT="Use the ratatui skill. $TASK"
else
  PROMPT="$TASK"
fi
printf '%s\n' "$PROMPT" > "$TRIAL_DIR/prompt.txt"

# Initialize Cargo project.
cat > "$PROJECT/Cargo.toml" <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
TOML

# Run opencode with isolated config, capture stdout/stderr separately.
START_TIME=$(date +%s)
OPENCODE_CONFIG_DIR="$CONFIG_DIR" timeout "$TIMEOUT" opencode run "$(cat "$TRIAL_DIR/prompt.txt")" \
  > "$TRIAL_DIR/stdout.txt" 2> "$TRIAL_DIR/stderr.txt"
EXIT_STATUS=$?
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

# Classify.
TIMED_OUT=false
if [ "$EXIT_STATUS" = "124" ]; then
  TIMED_OUT=true
fi

# Check skill activation (skill-enabled only).
SKILL_ACTIVATED=false
if [ "$WITH_SKILL" = "1" ]; then
  if grep -qi 'ratatui skill\|SKILL.md\|crate-versions' "$TRIAL_DIR/stdout.txt" 2>/dev/null; then
    SKILL_ACTIVATED=true
  fi
fi

# Verify no skill leakage in baseline.
SKILL_LEAKED=false
if [ "$WITH_SKILL" = "0" ]; then
  if grep -qi 'ratatui skill\|SKILL.md' "$TRIAL_DIR/stdout.txt" 2>/dev/null; then
    SKILL_LEAKED=true
  fi
fi

# Collect results.
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

# Write result using Python for valid JSON.
python3 - "$TRIAL_DIR" "$TASK_FILE" "$WITH_SKILL" "$EXIT_STATUS" "$TIMED_OUT" "$ELAPSED" "$SKILL_SHA" "$SKILL_INJECTED" "$SKILL_ACTIVATED" "$SKILL_LEAKED" "$COMPILE_OK" "$CLIPPY_OK" "$TEST_OK" "$HAS_RATATUI" "$FILES_GENERATED" <<'PY'
import json, sys, os
trial_dir, task, with_skill, exit_status, timed_out, elapsed, skill_sha, injected, activated, leaked, compile_ok, clippy_ok, test_ok, has_ratatui, files = sys.argv[1:]
result = {
    "task": os.path.basename(task),
    "with_skill": with_skill == "1",
    "exit_status": int(exit_status),
    "timed_out": timed_out == "true",
    "elapsed_seconds": int(elapsed),
    "skill_sha": skill_sha,
    "skill_injected": injected == "true",
    "skill_activated": activated == "true",
    "skill_leaked": leaked == "true",
    "compile_ok": compile_ok == "true",
    "clippy_ok": clippy_ok == "true",
    "test_ok": test_ok == "true",
    "has_ratatui": has_ratatui == "true",
    "files_generated": int(files),
}
with open(os.path.join(trial_dir, "result.json"), "w") as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
PY
