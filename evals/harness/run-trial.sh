#!/usr/bin/env bash
# Run one evaluation trial with honest skill injection and isolation.
#
# usage: run-trial.sh TASK_FILE WITH_SKILL(0|1) TRIAL_DIR [TIMEOUT]
#
# Sets up an isolated workspace, runs the agent, captures all output,
# and writes a structured result.json with honest exit status.
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

# Isolated opencode config per trial (never touch user's real config).
CONFIG_DIR="$TRIAL_DIR/opencode-config"
mkdir -p "$CONFIG_DIR"
if [ -f "$HOME/.config/opencode/auth.json" ]; then
  cp "$HOME/.config/opencode/auth.json" "$CONFIG_DIR/auth.json"
fi
if [ -f "$HOME/.config/opencode/opencode.json" ]; then
  cp "$HOME/.config/opencode/opencode.json" "$CONFIG_DIR/opencode.json"
fi

# Skill injection: copy skill only for skill-enabled trials.
SKILL_HASH="none"
SKILL_INJECTED=false
SKILL_ACTIVATED="unverified"
SKILL_LEAKED="unverified"

if [ "$WITH_SKILL" = "1" ]; then
  SKILL_SRC="$REPO_ROOT/skills/ratatui"
  SKILL_DST="$CONFIG_DIR/skills/ratatui"
  mkdir -p "$CONFIG_DIR/skills"
  rm -rf "$SKILL_DST"
  cp -r "$SKILL_SRC" "$SKILL_DST"
  # Hash actual skill contents (not git HEAD).
  SKILL_HASH=$(python3 -c "import hashlib, pathlib, sys; h=hashlib.sha256(); [h.update(p.read_bytes()) for p in sorted(pathlib.Path(sys.argv[1]).rglob('*')) if p.is_file()]; print(h.hexdigest())" "$SKILL_SRC")
  SKILL_INJECTED=true
  if [ ! -f "$SKILL_DST/SKILL.md" ]; then
    echo "skill injection failed: SKILL.md not found" >&2
    exit 2
  fi
  PROMPT="Use the ratatui skill. $TASK"
else
  # Ensure no skill leaks into baseline.
  rm -rf "$CONFIG_DIR/skills"
  PROMPT="$TASK"
fi

printf '%s\n' "$PROMPT" > "$TRIAL_DIR/prompt.txt"

# Determine task language from explicit metadata in the task file.
TASK_LANG=$(grep -m1 '^language:' "$TASK_FILE" 2>/dev/null | sed 's/^language: *//' | tr -d ' ' || true)
[ -n "$TASK_LANG" ] || TASK_LANG="rust"

# Set up project based on language.
if [ "$TASK_LANG" = "rust" ]; then
  cat > "$PROJECT/Cargo.toml" <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
TOML
else
  # Python task: no Cargo project.
  :
fi

# Run opencode FROM the project directory (critical fix).
START_TIME=$(date +%s)
(
  cd "$PROJECT"
  OPENCODE_CONFIG_DIR="$CONFIG_DIR" timeout "$TIMEOUT" "${OPENCODE_BIN:-opencode}" run "$(cat "$TRIAL_DIR/prompt.txt")" \
    > "$TRIAL_DIR/stdout.txt" 2> "$TRIAL_DIR/stderr.txt"
)
EXIT_STATUS=$?
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

TIMED_OUT=false
if [ "$EXIT_STATUS" = "124" ]; then
  TIMED_OUT=true
fi

# Skill activation cannot be verified from output text alone.
# Report as unverified; a future harness version can check agent tool calls.
SKILL_ACTIVATED="unverified"
# Skill leakage cannot be verified from output text alone.
SKILL_LEAKED="unverified"

# Collect results based on language.
COMPILE_OK=false
CLIPPY_OK=false
TEST_OK=false
HAS_RATATUI=false
FILES_GENERATED=0

if [ "$TASK_LANG" = "rust" ]; then
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
fi
FILES_GENERATED=$(find "$PROJECT" -name '*.rs' -o -name '*.py' | grep -v target | wc -l)

# Run task-specific acceptance checks if available.
ACCEPTANCE_SCRIPT="$REPO_ROOT/evals/acceptance/$(basename "$TASK_FILE" .md).sh"
ACCEPTANCE_PASS="unverified"
ACCEPTANCE_DETAILS=""
if [ -x "$ACCEPTANCE_SCRIPT" ]; then
  ACCEPTANCE_OUTPUT=$("$ACCEPTANCE_SCRIPT" "$PROJECT" 2>&1) || true
  if echo "$ACCEPTANCE_OUTPUT" | grep -q "FAIL:"; then
    ACCEPTANCE_PASS="false"
  else
    ACCEPTANCE_PASS="true"
  fi
  ACCEPTANCE_DETAILS=$(echo "$ACCEPTANCE_OUTPUT" | tr '\n' ' ')
fi

# Write result using Python for valid JSON.
python3 - "$TRIAL_DIR" "$TASK_FILE" "$WITH_SKILL" "$EXIT_STATUS" "$TIMED_OUT" "$ELAPSED" "$SKILL_HASH" "$SKILL_INJECTED" "$SKILL_ACTIVATED" "$SKILL_LEAKED" "$COMPILE_OK" "$CLIPPY_OK" "$TEST_OK" "$HAS_RATATUI" "$FILES_GENERATED" "$TASK_LANG" "$ACCEPTANCE_PASS" "$ACCEPTANCE_DETAILS" <<'PY'
import json, sys, os
(trial_dir, task, with_skill, exit_status, timed_out, elapsed, skill_hash, injected,
 activated, leaked, compile_ok, clippy_ok, test_ok, has_ratatui, files, lang, acceptance, details) = sys.argv[1:]
result = {
    "task": os.path.basename(task),
    "with_skill": with_skill == "1",
    "exit_status": int(exit_status),
    "timed_out": timed_out == "true",
    "elapsed_seconds": int(elapsed),
    "skill_hash": skill_hash,
    "skill_injected": injected == "true",
    "skill_activated": activated,
    "skill_leaked": leaked,
    "task_language": lang,
    "compile_ok": compile_ok == "true",
    "clippy_ok": clippy_ok == "true",
    "test_ok": test_ok == "true",
    "has_ratatui": has_ratatui == "true",
    "files_generated": int(files),
    "acceptance_pass": acceptance,
    "acceptance_details": details,
}
with open(os.path.join(trial_dir, "result.json"), "w") as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
# Exit nonzero if the trial failed its acceptance checks.
if acceptance == "false":
    sys.exit(1)
PY
