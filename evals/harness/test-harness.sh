#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
MOCK="$ROOT/evals/harness/mock/mock-agent.sh"

TASK="$TMP/python-task.md"
cat > "$TASK" <<'TASK'
# deterministic Python task

language: python

Create a Python CSV summary utility.
TASK

EVAL_MODEL="mock-model" EVAL_PROVIDER="mock-provider" EVAL_MODEL_SETTINGS="deterministic-test" MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$TASK" 0 "$TMP/success" 10 >/dev/null
python3 - "$TMP/success/result.json" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert data["exit_status"] == 0
assert data["task_language"] == "python"
assert data["source_files_after_run"] == 1
assert data["acceptance_pass"] == "unverified"
assert data["human_review"] == "required"
assert data["model"] == "mock-model"
assert data["provider"] == "mock-provider"
assert data["model_settings"] == "deterministic-test"
PY

MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$ROOT/evals/tasks/10-no-trigger.md" 1 "$TMP/no-trigger-skill" 10 >/dev/null
python3 - "$TMP/no-trigger-skill/result.json" "$TMP/no-trigger-skill/prompt.txt" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
prompt = pathlib.Path(sys.argv[2]).read_text()
assert data["skill_injected"] is True
assert data["acceptance_pass"] == "true"
assert data["skill_activated"] == "unverified"
assert len(data["skill_hash"]) == 64
assert data["expected_skill_trigger"] == "no"
assert "Use the ratatui skill" not in prompt
PY

mkdir -p "$TMP/bin"
cat > "$TMP/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
exit 0
CARGO
chmod +x "$TMP/bin/cargo"
FIXTURE_TASK="$TMP/fixture-task.md"
cat > "$FIXTURE_TASK" <<'TASK'
# fixture staging test

language: rust
fixture: dirview

Preserve the existing project.
TASK
if PATH="$TMP/bin:$PATH" MOCK_BEHAVIOR=fail OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$FIXTURE_TASK" 0 "$TMP/fixture" 10 >/dev/null 2>&1; then
  echo "expected mock failure was not reported" >&2
  exit 1
fi
grep -q '^name = "dirview"$' "$TMP/fixture/project/Cargo.toml"
test -f "$TMP/fixture/project/src/keymap.rs"
python3 - "$TMP/fixture/result.json" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert data["task_fixture"] == "dirview"
assert data["expected_skill_trigger"] is None
assert data["repository_commit"]
assert data["task_spec_sha256"]
PY

if MOCK_BEHAVIOR=fail OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$TASK" 0 "$TMP/failure" 10 >/dev/null 2>&1; then
  echo "agent failure was incorrectly accepted" >&2
  exit 1
fi
if MOCK_BEHAVIOR=success OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$TASK" 0 "$ROOT/inside-repo" 10 >/dev/null 2>&1; then
  echo "repository-local trial directory was incorrectly accepted" >&2
  exit 1
fi
bash -n "$ROOT/evals/harness/"*.sh "$ROOT/evals/acceptance/"*.sh
if grep -nE '(^|[;&|[:space:]])eval[[:space:]]' "$ROOT/evals/acceptance/"*.sh; then
  echo "unsafe eval remains in acceptance scripts" >&2
  exit 1
fi
echo "Harness tests passed."
