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
if [[ -n "$(find "$RESULTS" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
  printf 'results directory must be empty: %s\n' "$RESULTS" >&2
  exit 2
fi

echo "Running $TRIALS trial(s) per task per condition, timeout=${TIMEOUT}s"

for task in "$REPO_ROOT"/evals/tasks/*.md; do
  name="$(basename "$task" .md)"
  for skill in 0 1; do
    for trial in $(seq 1 "$TRIALS"); do
      dir="$RESULTS/$name-skill$skill-trial$trial"
      echo "=== $name skill=$skill trial=$trial ==="
      if ! bash "$REPO_ROOT/evals/harness/run-trial.sh" "$task" "$skill" "$dir" "$TIMEOUT"; then
        echo "Trial did not pass automated execution checks: $dir" >&2
      fi
    done
  done
done

python3 "$REPO_ROOT/evals/harness/summarize-results.py" "$RESULTS"
