#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
TASK="$TMP/python-task.md"
cat > "$TASK" <<'TASK'
# deterministic Python task

language: python

Create a Python CSV summary utility.
TASK
MOCK="$ROOT/evals/harness/mock/mock-agent.sh"
MOCK_BEHAVIOR=python OPENCODE_BIN="$MOCK" bash "$ROOT/evals/harness/run-trial.sh" "$TASK" 0 "$TMP/success" 10 >/dev/null
python3 - "$TMP/success/result.json" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert data["exit_status"] == 0
assert data["task_language"] == "python"
assert data["files_generated"] == 1
assert data["acceptance_pass"] == "unverified"
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
