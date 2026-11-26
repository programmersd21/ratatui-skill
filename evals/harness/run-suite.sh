#!/usr/bin/env bash
set -Eeuo pipefail

TRIALS="${1:-1}"
TIMEOUT="${2:-300}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
RESULTS="${3:-${TMPDIR:-/tmp}/ratatui-skill-evals-$(date +%Y%m%d-%H%M%S)}"
[[ "$TRIALS" =~ ^[1-9][0-9]*$ ]] || { echo "TRIALS must be a positive integer" >&2; exit 2; }
[[ "$TIMEOUT" =~ ^[1-9][0-9]*$ ]] || { echo "TIMEOUT must be a positive integer" >&2; exit 2; }
mkdir -p "$RESULTS"
RESULTS="$(cd "$RESULTS" && pwd -P)"
case "$RESULTS/" in "$REPO_ROOT/"*) echo "results must be outside the repository" >&2; exit 2 ;; esac

echo "Running $TRIALS trial(s) per task per condition, timeout=${TIMEOUT}s"

TRIAL_COUNT=0
COMPLETED_COUNT=0
ACCEPTED_COUNT=0
UNGRADED_COUNT=0
FAILED_COUNT=0

for task in "$REPO_ROOT"/evals/tasks/*.md; do
  name="$(basename "$task" .md)"
  for skill in 0 1; do
    for trial in $(seq 1 "$TRIALS"); do
      dir="$RESULTS/$name-skill$skill-trial$trial"
      echo "=== $name skill=$skill trial=$trial ==="
      TRIAL_COUNT=$((TRIAL_COUNT + 1))
      if bash "$REPO_ROOT/evals/harness/run-trial.sh" "$task" "$skill" "$dir" "$TIMEOUT"; then
        COMPLETED_COUNT=$((COMPLETED_COUNT + 1))
        status="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["acceptance_pass"])' "$dir/result.json")"
        if [ "$status" = "true" ]; then
          ACCEPTED_COUNT=$((ACCEPTED_COUNT + 1))
        elif [ "$status" = "unverified" ]; then
          UNGRADED_COUNT=$((UNGRADED_COUNT + 1))
        else
          FAILED_COUNT=$((FAILED_COUNT + 1))
        fi
      else
        FAILED_COUNT=$((FAILED_COUNT + 1))
      fi
    done
  done
done

printf '\n=== Suite summary ===\n'
printf 'Trials: %s  Completed: %s  Automated acceptance: %s  Ungraded: %s  Failed: %s\n' \
  "$TRIAL_COUNT" "$COMPLETED_COUNT" "$ACCEPTED_COUNT" "$UNGRADED_COUNT" "$FAILED_COUNT"

if [ "$FAILED_COUNT" -gt 0 ]; then
  echo "SUITE FAILED: one or more agent or automated checks failed"
  exit 1
fi
if [ "$UNGRADED_COUNT" -gt 0 ]; then
  echo "SUITE INCOMPLETE: human review or automated acceptance is still required"
  exit 2
fi
echo "SUITE PASSED: all trials completed and received automated acceptance"
