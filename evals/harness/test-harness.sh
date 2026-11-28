#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
MOCK="$ROOT/evals/harness/mock/mock-agent.sh"
export EVAL_MODEL="mock-model"

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
assert data["skill_activated"] == "false"
assert data["skill_leaked"] == "not_applicable"
assert len(data["skill_hash"]) == 64
assert data["expected_skill_trigger"] == "no"
assert "Use the ratatui skill" not in prompt
PY


MOCK_SKILL_CALL=1 MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$ROOT/evals/tasks/10-no-trigger.md" 1 "$TMP/skill-activation" 10 >/dev/null
python3 - "$TMP/skill-activation/result.json" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert data["skill_activated"] == "true"
assert data["skill_tool_calls"] == 1
PY

MOCK_SKILL_CALL=1 MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$ROOT/evals/tasks/10-no-trigger.md" 0 "$TMP/skill-leak" 10 >/dev/null
python3 - "$TMP/skill-leak/result.json" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert data["skill_leaked"] == "true"
PY

MOCK_SUBAGENT_CALL=1 MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$ROOT/evals/tasks/10-no-trigger.md" 1 "$TMP/subagent" 10 >/dev/null
python3 - "$TMP/subagent/result.json" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert data["skill_activated"] == "unverified"
assert data["skill_leaked"] == "not_applicable"
assert data["subagent_tool_calls"] == 1
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
grep -q '^name = "dirview"
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
if MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$TASK" 0 "$TMP/success" 10 >/dev/null 2>&1; then
  echo "non-empty trial directory was incorrectly reused" >&2
  exit 1
fi

mkdir -p "$TMP/summary/python-skill0-trial1"
cat > "$TMP/summary/python-skill0-trial1/result.json" <<'RESULT'
{
  "exit_status": 0,
  "timed_out": false,
  "task_language": "python",
  "with_skill": false,
  "expected_skill_trigger": "no",
  "skill_activated": "false",
  "skill_leaked": "false",
  "acceptance_pass": "true"
}
RESULT
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary" >/dev/null; then
  echo "summary accepted a trial without human review" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 2 ] || exit "$status"
fi
cp "$ROOT/evals/human-review-template.json" "$TMP/summary/python-skill0-trial1/human-review.json"
python3 - "$TMP/summary/python-skill0-trial1/human-review.json" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
data = json.loads(path.read_text())
data["reviewer"] = "ci-test"
data["reviewed_at"] = "2026-11-26"
data["status"] = "passed"
data["notes"] = "Deterministic summarizer fixture."
data["scores"] = {key: 2 for key in data["scores"]}
path.write_text(json.dumps(data, indent=2) + "\n")
PY
python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary" >/dev/null

mkdir -p "$TMP/summary-invalid/python-skill0-trial1"
cat > "$TMP/summary-invalid/python-skill0-trial1/result.json" <<'RESULT'
{
  "exit_status": 0,
  "timed_out": false,
  "task_language": "python",
  "acceptance_pass": "maybe"
}
RESULT
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary-invalid" >/dev/null; then
  echo "summary accepted an invalid acceptance status" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 1 ] || exit "$status"
fi

mkdir -p "$TMP/summary-missing/python-skill0-trial1"
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary-missing" 2 >/dev/null; then
  echo "summary ignored a missing trial result" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 1 ] || exit "$status"
fi

mkdir -p "$TMP/non-empty-results"
touch "$TMP/non-empty-results/old-run"
if bash "$ROOT/evals/harness/run-suite.sh" 1 10 "$TMP/non-empty-results" >/dev/null 2>&1; then
  echo "run-suite reused a non-empty results directory" >&2
  exit 1
fi

bash -n "$ROOT/evals/harness/"*.sh "$ROOT/evals/acceptance/"*.sh
if grep -nE '(^|[;&|[:space:]])eval[[:space:]]' "$ROOT/evals/acceptance/"*.sh; then
  echo "unsafe eval remains in acceptance scripts" >&2
  exit 1
fi

mkdir -p "$TMP/summary-suite/python-skill0-trial1"
cp "$TMP/summary/python-skill0-trial1/result.json" "$TMP/summary-suite/python-skill0-trial1/result.json"
cp "$TMP/summary/python-skill0-trial1/human-review.json" "$TMP/summary-suite/python-skill0-trial1/human-review.json"
printf '{"expected_trials":2}\\n' > "$TMP/summary-suite/suite.json"
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary-suite" >/dev/null 2>&1; then
  echo "summary ignored the recorded suite trial count" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 1 ] || exit "$status"
fi

mkdir -p "$TMP/summary-telemetry/skill-skill1-trial1"
cat > "$TMP/summary-telemetry/skill-skill1-trial1/result.json" <<'RESULT'
{
  "exit_status": 0,
  "timed_out": false,
  "task_language": "python",
  "with_skill": true,
  "expected_skill_trigger": null,
  "skill_activated": "unverified",
  "skill_leaked": "not_applicable",
  "acceptance_pass": "true"
}
RESULT
cp "$TMP/summary/python-skill0-trial1/human-review.json" "$TMP/summary-telemetry/skill-skill1-trial1/human-review.json"
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary-telemetry" >/dev/null 2>&1; then
  echo "summary accepted unknown skill activation telemetry" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 2 ] || exit "$status"
fi
echo "Harness tests passed."
 "$TMP/fixture/project/Cargo.toml"
test -f "$TMP/fixture/project/src/keymap.rs"
FILTER_FIXTURE_TASK="$TMP/filter-fixture-task.md"
cat > "$FILTER_FIXTURE_TASK" <<'TASK'
# filter regression fixture

language: rust
fixture: dirview-filter-reset

Fix the seeded selection reset regression.
TASK
if PATH="$TMP/bin:$PATH" MOCK_BEHAVIOR=fail OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$FILTER_FIXTURE_TASK" 0 "$TMP/filter-fixture" 10 >/dev/null 2>&1; then
  echo "expected fixture mock failure was not reported" >&2
  exit 1
fi
grep -A3 'Action::FilterInput(c)' "$TMP/filter-fixture/project/src/state.rs" | grep -q 'self.rebuild(None)'
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
if MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$TASK" 0 "$TMP/success" 10 >/dev/null 2>&1; then
  echo "non-empty trial directory was incorrectly reused" >&2
  exit 1
fi

mkdir -p "$TMP/summary/python-skill0-trial1"
cat > "$TMP/summary/python-skill0-trial1/result.json" <<'RESULT'
{
  "exit_status": 0,
  "timed_out": false,
  "task_language": "python",
  "with_skill": false,
  "expected_skill_trigger": "no",
  "skill_activated": "false",
  "skill_leaked": "false",
  "acceptance_pass": "true"
}
RESULT
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary" >/dev/null; then
  echo "summary accepted a trial without human review" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 2 ] || exit "$status"
fi
cp "$ROOT/evals/human-review-template.json" "$TMP/summary/python-skill0-trial1/human-review.json"
python3 - "$TMP/summary/python-skill0-trial1/human-review.json" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
data = json.loads(path.read_text())
data["reviewer"] = "ci-test"
data["reviewed_at"] = "2026-11-26"
data["status"] = "passed"
data["notes"] = "Deterministic summarizer fixture."
data["scores"] = {key: 2 for key in data["scores"]}
path.write_text(json.dumps(data, indent=2) + "\n")
PY
python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary" >/dev/null

mkdir -p "$TMP/summary-invalid/python-skill0-trial1"
cat > "$TMP/summary-invalid/python-skill0-trial1/result.json" <<'RESULT'
{
  "exit_status": 0,
  "timed_out": false,
  "task_language": "python",
  "acceptance_pass": "maybe"
}
RESULT
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary-invalid" >/dev/null; then
  echo "summary accepted an invalid acceptance status" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 1 ] || exit "$status"
fi

mkdir -p "$TMP/summary-missing/python-skill0-trial1"
if python3 "$ROOT/evals/harness/summarize-results.py" "$TMP/summary-missing" 2 >/dev/null; then
  echo "summary ignored a missing trial result" >&2
  exit 1
else
  status=$?
  [ "$status" -eq 1 ] || exit "$status"
fi

mkdir -p "$TMP/non-empty-results"
touch "$TMP/non-empty-results/old-run"
if bash "$ROOT/evals/harness/run-suite.sh" 1 10 "$TMP/non-empty-results" >/dev/null 2>&1; then
  echo "run-suite reused a non-empty results directory" >&2
  exit 1
fi

bash -n "$ROOT/evals/harness/"*.sh "$ROOT/evals/acceptance/"*.sh
if grep -nE '(^|[;&|[:space:]])eval[[:space:]]' "$ROOT/evals/acceptance/"*.sh; then
  echo "unsafe eval remains in acceptance scripts" >&2
  exit 1
fi
echo "Harness tests passed."
