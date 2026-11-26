#!/usr/bin/env bash
# Run the full evaluation suite: N trials per task per condition.
#
# usage: run-suite.sh [TRIALS_PER_CONDITION] [TIMEOUT]
set -uo pipefail

TRIALS="${1:-1}"
TIMEOUT="${2:-300}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RESULTS="$REPO_ROOT/evals/results"
mkdir -p "$RESULTS"

echo "Running $TRIALS trial(s) per task per condition, timeout=${TIMEOUT}s"

for task in "$REPO_ROOT"/evals/tasks/*.md; do
  name=$(basename "$task" .md)
  for skill in 0 1; do
    for trial in $(seq 1 "$TRIALS"); do
      dir="$RESULTS/$name-skill$skill-trial$trial"
      echo "=== $name skill=$skill trial=$trial ==="
      bash "$REPO_ROOT/evals/harness/run-trial.sh" "$task" "$skill" "$dir" "$TIMEOUT" || true
    done
  done
done

echo "Suite complete. Results in $RESULTS/"
