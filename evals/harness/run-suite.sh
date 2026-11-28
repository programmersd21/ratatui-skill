#!/usr/bin/env bash
set -Eeuo pipefail

TRIALS="${1:-1}"
TIMEOUT="${2:-600}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
RESULTS="${3:-${TMPDIR:-/tmp}/ratatui-skill-evals-$(date +%Y%m%d-%H%M%S)}"
[[ "$TRIALS" =~ ^[1-9][0-9]*$ ]] || { echo "TRIALS must be a positive integer" >&2; exit 2; }
[[ "$TIMEOUT" =~ ^[1-9][0-9]*$ ]] || { echo "TIMEOUT must be a positive integer" >&2; exit 2; }
[[ -n "${EVAL_MODEL:-}" ]] || { echo "Set EVAL_MODEL to an explicit provider/model ID." >&2; exit 2; }
mkdir -p "$RESULTS"
RESULTS="$(cd "$RESULTS" && pwd -P)"
case "$RESULTS/" in "$REPO_ROOT/"*) echo "results must be outside the repository" >&2; exit 2 ;; esac
if [[ -n "$(find "$RESULTS" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
  printf 'results directory must be empty: %s\n' "$RESULTS" >&2
  exit 2
fi

ORDER_SEED="${EVAL_ORDER_SEED:-$(date +%s)}"
SCHEDULE="$RESULTS/schedule.tsv"
python3 - "$REPO_ROOT/evals/tasks" "$SCHEDULE" "$TRIALS" "$ORDER_SEED" <<'PYSCHEDULE'
import pathlib
import random
import sys

task_root = pathlib.Path(sys.argv[1])
schedule = pathlib.Path(sys.argv[2])
rng = random.Random(int(sys.argv[4]))
rows = []
for task in sorted(task_root.glob("*.md")):
    for trial in range(1, int(sys.argv[3]) + 1):
        conditions = [0, 1]
        rng.shuffle(conditions)
        for condition in conditions:
            rows.append(f"{task.stem}\t{condition}\t{trial}")
schedule.write_text("\n".join(rows) + "\n", encoding="utf-8")
PYSCHEDULE

EXPECTED_TRIALS="$(wc -l < "$SCHEDULE" | tr -d ' ')"
python3 - "$REPO_ROOT" "$RESULTS" "$SCHEDULE" "$ORDER_SEED" "$EXPECTED_TRIALS" <<'PYSUITE'
import hashlib
import json
import os
import pathlib
import subprocess
import sys

repo, results, schedule = map(pathlib.Path, sys.argv[1:4])
metadata = {
    "repository_commit": subprocess.check_output(["git", "-C", str(repo), "rev-parse", "HEAD"], text=True).strip(),
    "model": os.environ.get("EVAL_MODEL", "not recorded"),
    "provider": os.environ.get("EVAL_PROVIDER", "not recorded"),
    "model_settings": os.environ.get("EVAL_MODEL_SETTINGS", "not recorded"),
    "order_seed": sys.argv[4],
    "schedule_sha256": hashlib.sha256(schedule.read_bytes()).hexdigest(),
    "expected_trials": int(sys.argv[5]),
    "task_hashes": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((repo / "evals/tasks").glob("*.md"))},
}
(results / "suite.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
PYSUITE

echo "Running $EXPECTED_TRIALS trials, $TRIALS repetitions per condition, timeout=${TIMEOUT}s, order_seed=$ORDER_SEED"
RUNNER_FAILURES=0
while IFS=$'\t' read -r name skill trial; do
  dir="$RESULTS/$name-skill$skill-trial$trial"
  echo "=== $name skill=$skill trial=$trial ==="
  mkdir "$dir"
  if ! bash "$REPO_ROOT/evals/harness/run-trial.sh" "$REPO_ROOT/evals/tasks/$name.md" "$skill" "$dir" "$TIMEOUT"; then
    RUNNER_FAILURES=$((RUNNER_FAILURES + 1))
    echo "Trial did not pass automated execution checks: $dir" >&2
  fi
done < "$SCHEDULE"

SUMMARY_STATUS=0
python3 "$REPO_ROOT/evals/harness/summarize-results.py" "$RESULTS" "$EXPECTED_TRIALS" || SUMMARY_STATUS=$?
if [[ "$RUNNER_FAILURES" -gt 0 ]]; then
  printf 'SUITE FAILED: %s trial runner invocation(s) failed\n' "$RUNNER_FAILURES" >&2
  exit 1
fi
exit "$SUMMARY_STATUS"