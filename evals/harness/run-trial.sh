#!/usr/bin/env bash
# Run one evaluation trial with honest skill injection and isolation.
#
# usage: run-trial.sh TASK_FILE WITH_SKILL(0|1) TRIAL_DIR [TIMEOUT]
#
# Sets up an isolated workspace, runs the agent, captures all output,
# and writes a structured result.json with honest exit status.
set -Eeuo pipefail
umask 077

TASK_FILE="${1:?task file required}"
WITH_SKILL="${2:?0 or 1}"
TRIAL_DIR="${3:?trial dir required}"
TIMEOUT="${4:-600}"
CHECK_TIMEOUT="${EVAL_CHECK_TIMEOUT:-300}"
EVAL_MODEL="${EVAL_MODEL:-}"

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ "$WITH_SKILL" == 0 || "$WITH_SKILL" == 1 ]] || { echo "WITH_SKILL must be 0 or 1" >&2; exit 2; }
[[ "$TIMEOUT" =~ ^[1-9][0-9]*$ ]] || { echo "TIMEOUT must be a positive integer" >&2; exit 2; }
[[ "$CHECK_TIMEOUT" =~ ^[1-9][0-9]*$ ]] || { echo "EVAL_CHECK_TIMEOUT must be a positive integer" >&2; exit 2; }
[[ -n "$EVAL_MODEL" ]] || { echo "Set EVAL_MODEL to an explicit provider/model ID." >&2; exit 2; }
TASK_FILE="$(cd "$(dirname "$TASK_FILE")" && pwd -P)/$(basename "$TASK_FILE")"
[[ -f "$TASK_FILE" ]] || { echo "task file not found" >&2; exit 2; }
mkdir -p "$TRIAL_DIR"
TRIAL_DIR="$(cd "$TRIAL_DIR" && pwd -P)"
case "$TRIAL_DIR/" in "$REPO_ROOT/"*) echo "TRIAL_DIR must be outside the repository to prevent AGENTS.md leakage" >&2; exit 2 ;; esac
if [[ -n "$(find "$TRIAL_DIR" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
  printf 'trial directory is not empty: %s\n' "$TRIAL_DIR" >&2
  exit 2
fi

TASK=$(cat "$TASK_FILE")
PROJECT="$TRIAL_DIR/project"
mkdir -p "$PROJECT"

ORIGINAL_HOME="$HOME"
ORIGINAL_XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$ORIGINAL_HOME/.config}"
ORIGINAL_XDG_DATA_HOME="${XDG_DATA_HOME:-$ORIGINAL_HOME/.local/share}"
ORIGINAL_DB="${OPENCODE_DB:-$ORIGINAL_XDG_DATA_HOME/opencode/opencode.db}"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/ratatui-skill-trial.XXXXXX")"
CONFIG_DIR="$SCRATCH/opencode-config"
ISOLATED_HOME="$SCRATCH/home"
ISOLATED_XDG_CONFIG="$SCRATCH/xdg-config"
ISOLATED_XDG_DATA="$SCRATCH/xdg-data"
ISOLATED_XDG_CACHE="$SCRATCH/xdg-cache"
ISOLATED_XDG_STATE="$SCRATCH/xdg-state"
mkdir -p "$CONFIG_DIR" "$ISOLATED_HOME" "$ISOLATED_XDG_CONFIG" "$ISOLATED_XDG_DATA/opencode" "$ISOLATED_XDG_CACHE" "$ISOLATED_XDG_STATE"
chmod 700 "$SCRATCH" "$CONFIG_DIR" "$ISOLATED_HOME"
cleanup() { rm -rf "$SCRATCH"; }
trap cleanup EXIT INT TERM

for system_skill in "/etc/opencode/skills/ratatui/SKILL.md" "/Library/Application Support/opencode/skills/ratatui/SKILL.md"; do
  if [ -f "$system_skill" ]; then
    echo "Machine-wide Ratatui skill found at $system_skill; trial isolation is not guaranteed." >&2
    exit 2
  fi
done

for auth_file in "$ORIGINAL_XDG_DATA_HOME/opencode/auth.json" "$ORIGINAL_XDG_CONFIG_HOME/opencode/auth.json"; do
  if [ -f "$auth_file" ]; then
    install -m 600 "$auth_file" "$ISOLATED_XDG_DATA/opencode/auth.json"
  fi
done
if [ -f "$ORIGINAL_DB" ]; then
  python3 - "$ORIGINAL_DB" "$ISOLATED_XDG_DATA/opencode/opencode.db" <<'PYDB'
import pathlib
import sqlite3
import sys

source = sqlite3.connect(pathlib.Path(sys.argv[1]).resolve().as_uri() + "?mode=ro", uri=True)
target = sqlite3.connect(sys.argv[2])
try:
    source.backup(target)
finally:
    target.close()
    source.close()
PYDB
fi

export HOME="$ISOLATED_HOME"
export XDG_CONFIG_HOME="$ISOLATED_XDG_CONFIG"
export XDG_DATA_HOME="$ISOLATED_XDG_DATA"
export XDG_CACHE_HOME="$ISOLATED_XDG_CACHE"
export XDG_STATE_HOME="$ISOLATED_XDG_STATE"
export OPENCODE_CONFIG_DIR="$CONFIG_DIR"
export OPENCODE_DB="$ISOLATED_XDG_DATA/opencode/opencode.db"
unset OPENCODE_CONFIG OPENCODE_CONFIG_CONTENT

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
  SKILL_HASH=$(python3 - "$SKILL_SRC" <<'PYHASH'
import hashlib
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
digest = hashlib.sha256()
for path in sorted(item for item in root.rglob("*") if item.is_file()):
    relative = path.relative_to(root).as_posix().encode("utf-8")
    content = path.read_bytes()
    digest.update(len(relative).to_bytes(8, "big"))
    digest.update(relative)
    digest.update(len(content).to_bytes(8, "big"))
    digest.update(content)
print(digest.hexdigest())
PYHASH
)
  SKILL_INJECTED=true
  if [ ! -f "$SKILL_DST/SKILL.md" ]; then
    echo "skill injection failed: SKILL.md not found" >&2
    exit 2
  fi
  PROMPT="$TASK"
else
  # Ensure no skill leaks into baseline.
  rm -rf "$CONFIG_DIR/skills"
  PROMPT="$TASK"
fi

printf '%s\n' "$PROMPT" > "$TRIAL_DIR/prompt.txt"

# Determine task language from explicit metadata in the task file.
TASK_LANG=$(grep -m1 '^language:' "$TASK_FILE" 2>/dev/null | sed 's/^language: *//' | tr -d ' ' || true)
[ -n "$TASK_LANG" ] || TASK_LANG="rust"

TASK_TRIGGER=$(sed -nE 's/^[#[:space:]]*Should trigger skill:[[:space:]]*(yes|no).*$/\1/ip' "$TASK_FILE" | head -1 | tr '[:upper:]' '[:lower:]')
TASK_FIXTURE=$(grep -m1 '^fixture:' "$TASK_FILE" 2>/dev/null | sed 's/^fixture: *//' | tr -d ' ' || true)

case "$TASK_FIXTURE" in
  "")
    if [ "$TASK_LANG" = "rust" ]; then
      cat > "$PROJECT/Cargo.toml" <<'TOML'
[package]
name = "eval-app"
version = "0.1.0"
edition = "2024"

[dependencies]
TOML
    fi
    ;;
  dirview)
    cp -a "$REPO_ROOT/skills/ratatui/templates/dirview/." "$PROJECT/"
    ;;
  todo)
    cp -a "$REPO_ROOT/todo/." "$PROJECT/"
    ;;
  ratatui-028)
    cp -a "$REPO_ROOT/evals/fixtures/ratatui-028/." "$PROJECT/"
    ;;
  *)
    printf 'unknown fixture: %s\n' "$TASK_FIXTURE" >&2
    exit 2
    ;;
esac

# Run opencode FROM the project directory (critical fix).
START_TIME=$(date +%s)
set +e
(
  cd "$PROJECT"
  timeout "$TIMEOUT" "${OPENCODE_BIN:-opencode}" run --standalone --format json --model "$EVAL_MODEL" "$(cat "$TRIAL_DIR/prompt.txt")" \
    > "$TRIAL_DIR/events.ndjson" 2> "$TRIAL_DIR/stderr.txt"
)
EXIT_STATUS=$?
set -e
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

TELEMETRY="$(python3 - "$TRIAL_DIR/events.ndjson" "$TRIAL_DIR/stdout.txt" "$WITH_SKILL" <<'PYEVENTS'
import json
import pathlib
import sys

event_path = pathlib.Path(sys.argv[1])
text_path = pathlib.Path(sys.argv[2])
with_skill = sys.argv[3] == "1"
valid_events = 0
skill_calls = 0
subagent_calls = 0
tool_names = set()
text_parts = []

for line in event_path.read_text(encoding="utf-8", errors="replace").splitlines():
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        if line.strip():
            text_parts.append(line)
        continue
    if not isinstance(event, dict):
        continue
    valid_events += 1
    part = event.get("part")
    if not isinstance(part, dict):
        part = {}
    if event.get("type") == "text" and isinstance(part.get("text"), str):
        text_parts.append(part["text"])
    if event.get("type") != "tool_use":
        continue
    tool = part.get("tool")
    if not isinstance(tool, str):
        continue
    tool_names.add(tool)
    state = part.get("state")
    if not isinstance(state, dict):
        state = {}
    inputs = state.get("input")
    if not isinstance(inputs, dict):
        inputs = {}
    name = inputs.get("name", inputs.get("skill", inputs.get("skill_name")))
    if tool == "skill" and name == "ratatui":
        skill_calls += 1
    if tool in {"task", "subagent"}:
        subagent_calls += 1

text_path.write_text("\n".join(text_parts) + ("\n" if text_parts else ""), encoding="utf-8")
if skill_calls:
    activated = "true"
elif subagent_calls or valid_events == 0:
    activated = "unverified"
else:
    activated = "false"
if with_skill:
    leaked = "not_applicable"
elif skill_calls:
    leaked = "true"
elif subagent_calls or valid_events == 0:
    leaked = "unverified"
else:
    leaked = "false"
print(json.dumps({
    "skill_activated": activated,
    "skill_leaked": leaked,
    "skill_tool_calls": skill_calls,
    "subagent_tool_calls": subagent_calls,
    "tool_names": sorted(tool_names),
    "json_event_count": valid_events
}, separators=(",", ":")))
PYEVENTS
)"
SKILL_ACTIVATED="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["skill_activated"])' "$TELEMETRY")"
SKILL_LEAKED="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["skill_leaked"])' "$TELEMETRY")"

TIMED_OUT=false
if [ "$EXIT_STATUS" = "124" ]; then
  TIMED_OUT=true
fi


# Collect results based on language.
COMPILE_OK=false
FMT_OK=false
CLIPPY_OK=false
TEST_OK=false
HAS_RATATUI=false
FILES_GENERATED=0
mkdir -p "$TRIAL_DIR/checks"

run_check() {
  local name="$1"
  shift
  if timeout "$CHECK_TIMEOUT" "$@" > "$TRIAL_DIR/checks/$name.log" 2>&1; then
    printf 'true'
  else
    printf 'false'
  fi
}

if [ "$TASK_LANG" = "rust" ]; then
  FMT_OK="$(run_check fmt cargo fmt --manifest-path "$PROJECT/Cargo.toml" --check)"
  COMPILE_OK="$(run_check build cargo build --manifest-path "$PROJECT/Cargo.toml")"
  CLIPPY_OK="$(run_check clippy cargo clippy --manifest-path "$PROJECT/Cargo.toml" --all-targets -- -D warnings)"
  TEST_OK="$(run_check test cargo test --manifest-path "$PROJECT/Cargo.toml")"
  if grep -q ratatui "$PROJECT/Cargo.toml" 2>/dev/null; then
    HAS_RATATUI=true
  fi
fi
FILES_GENERATED=$(find "$PROJECT" -type f \( -name '*.rs' -o -name '*.py' \) ! -path '*/target/*' | wc -l | tr -d ' ')
REPO_COMMIT=$(git -C "$REPO_ROOT" rev-parse HEAD)
TASK_HASH=$(python3 -c "import hashlib, pathlib, sys; print(hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest())" "$TASK_FILE")

# Run task-specific acceptance checks if available.
ACCEPTANCE_SCRIPT="$REPO_ROOT/evals/acceptance/$(basename "$TASK_FILE" .md).sh"
ACCEPTANCE_PASS="unverified"
ACCEPTANCE_DETAILS=""
if [ -x "$ACCEPTANCE_SCRIPT" ]; then
  set +e
  ACCEPTANCE_OUTPUT=$("$ACCEPTANCE_SCRIPT" "$PROJECT" 2>&1)
  ACCEPTANCE_STATUS=$?
  set -e
  if [ "$ACCEPTANCE_STATUS" -eq 0 ]; then ACCEPTANCE_PASS="true"; else ACCEPTANCE_PASS="false"; fi
  ACCEPTANCE_DETAILS=$(echo "$ACCEPTANCE_OUTPUT" | tr '\n' ' ')
fi

# Write result using Python for valid JSON.
python3 - "$TRIAL_DIR" "$TASK_FILE" "$WITH_SKILL" "$REPO_COMMIT" "$TASK_HASH" "$EXIT_STATUS" "$TIMED_OUT" "$ELAPSED" "$SKILL_HASH" "$SKILL_INJECTED" "$SKILL_ACTIVATED" "$SKILL_LEAKED" "$FMT_OK" "$COMPILE_OK" "$CLIPPY_OK" "$TEST_OK" "$HAS_RATATUI" "$FILES_GENERATED" "$TASK_LANG" "$TASK_FIXTURE" "$TASK_TRIGGER" "$ACCEPTANCE_PASS" "$ACCEPTANCE_DETAILS" "$TELEMETRY" <<'PY'
import json, sys, os
(trial_dir, task, with_skill, repository_commit, task_hash, exit_status, timed_out, elapsed, skill_hash, injected,
 activated, leaked, fmt_ok, compile_ok, clippy_ok, test_ok, has_ratatui, files, lang, fixture, expected_trigger, acceptance, details, telemetry_json) = sys.argv[1:]
telemetry = json.loads(telemetry_json)
result = {
    "task": os.path.basename(task),
    "repository_commit": repository_commit,
    "task_spec_sha256": task_hash,
    "with_skill": with_skill == "1",
    "exit_status": int(exit_status),
    "timed_out": timed_out == "true",
    "elapsed_seconds": int(elapsed),
    "skill_hash": skill_hash,
    "skill_injected": injected == "true",
    "skill_activated": activated,
    "skill_leaked": leaked,
    "skill_tool_calls": telemetry["skill_tool_calls"],
    "subagent_tool_calls": telemetry["subagent_tool_calls"],
    "tool_names": telemetry["tool_names"],
    "agent_json_event_count": telemetry["json_event_count"],
    "task_language": lang,
    "model": os.environ.get("EVAL_MODEL", "not recorded"),
    "provider": os.environ.get("EVAL_PROVIDER", "not recorded"),
    "model_settings": os.environ.get("EVAL_MODEL_SETTINGS", "not recorded"),
    "task_fixture": fixture or None,
    "expected_skill_trigger": expected_trigger or None,
    "fmt_ok": fmt_ok == "true",
    "compile_ok": compile_ok == "true",
    "clippy_ok": clippy_ok == "true",
    "test_ok": test_ok == "true",
    "has_ratatui": has_ratatui == "true",
    "source_files_after_run": int(files),
    "acceptance_pass": acceptance,
    "acceptance_details": details,
    "human_review": "required",
}
with open(os.path.join(trial_dir, "result.json"), "w") as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
# A trial is successful only when the agent, required language checks, and acceptance checks pass.
failed = int(exit_status) != 0 or timed_out == "true" or acceptance == "false"
if lang == "rust" and (fmt_ok != "true" or compile_ok != "true" or clippy_ok != "true" or test_ok != "true"):
    failed = True
sys.exit(1 if failed else 0)
PY
