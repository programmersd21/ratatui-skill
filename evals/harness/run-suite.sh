#!/usr/bin/env bash
# Run the full evaluation suite: N trials per task per condition.
#
# usage: run-suite.sh [TRIALS_PER_CONDITION] [TIMEOUT]
#
# Exit status: 0 if all required checks pass, 1 otherwise.
set -uo pipefail

TRIALS="${1:-1}"
TIMEOUT="${2:-300}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RESULTS="$REPO_ROOT/evals/results"
mkdir -p "$RESULTS"

echo "Running $TRIALS trial(s) per task per condition, timeout=${TIMEOUT}s"

SUITE_FAILED=0
TRIAL_COUNT=0
PASS_COUNT=0
FAIL_COUNT=0

for task in "$REPO_ROOT"/evals/tasks/*.md; do
  name=$(basename "$task" .md)
  for skill in 0 1; do
    for trial in $(seq 1 "$TRIALS"); do
      dir="$RESULTS/$name-skill$skill-trial$trial"
      echo "=== $name skill=$skill trial=$trial ==="
      TRIAL_COUNT=$((TRIAL_COUNT + 1))
      if bash "$REPO_ROOT/evals/harness/run-trial.sh" "$task" "$skill" "$dir" "$TIMEOUT"; then
        PASS_COUNT=$((PASS_COUNT + 1))
      else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        SUITE_FAILED=1
      fi
    done
  done
done

echo ""
echo "=== Suite summary ==="
echo "Trials: $TRIAL_COUNT  Passed: $PASS_COUNT  Failed: $FAIL_COUNT"
if [ "$SUITE_FAILED" -ne 0 ]; then
  echo "SUITE FAILED: $FAIL_COUNT trial(s) did not pass"
  exit 1
fi
echo "SUITE PASSED"
exit 0
