#!/usr/bin/env python3
import datetime
import json
import pathlib
import sys

SCORE_KEYS = {
    "functional_acceptance",
    "api_version_correctness",
    "terminal_lifecycle",
    "keyboard_interaction",
    "resize_layout",
    "state_error_handling",
    "architecture",
    "tests",
    "scope_discipline",
    "verification_quality",
}


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None


def review_status(trial_dir):
    review_path = trial_dir / "human-review.json"
    if not review_path.exists():
        return "pending"
    review = read_json(review_path)
    if not isinstance(review, dict):
        return "invalid"

    if not isinstance(review.get("reviewer"), str) or not review["reviewer"].strip():
        return "invalid"
    try:
        datetime.date.fromisoformat(review.get("reviewed_at", ""))
    except (TypeError, ValueError):
        return "invalid"
    if not isinstance(review.get("status"), str) or review["status"] not in {"passed", "failed"}:
        return "invalid"
    if not isinstance(review.get("notes"), str) or not review["notes"].strip():
        return "invalid"
    scores = review.get("scores")
    if not isinstance(scores, dict) or set(scores) != SCORE_KEYS:
        return "invalid"
    if any(
        type(score) is not int or score not in {0, 1, 2}
        for score in scores.values()
    ):
        return "invalid"
    failures = review.get("critical_failures")
    if not isinstance(failures, list) or any(not isinstance(item, str) for item in failures):
        return "invalid"
    if review["status"] == "passed" and failures:
        return "invalid"
    return review["status"]


def main():
    if len(sys.argv) != 2:
        print("usage: summarize-results.py RESULTS_DIR", file=sys.stderr)
        return 2

    root = pathlib.Path(sys.argv[1]).resolve()
    if not root.is_dir():
        print(f"results directory not found: {root}", file=sys.stderr)
        return 2

    result_paths = sorted(root.rglob("result.json"))
    if not result_paths:
        print("no trial result.json files found", file=sys.stderr)
        return 2

    completed = 0
    auto_accepted = 0
    acceptance_unverified = 0
    failed = 0
    human_passed = 0
    human_failed = 0
    human_pending = 0
    human_invalid = 0

    for result_path in result_paths:
        trial_dir = result_path.parent
        result = read_json(result_path)
        if not isinstance(result, dict):
            failed += 1
            print(f"FAIL  {trial_dir.name}: invalid result.json")
            continue

        trial_failed = False
        if result.get("exit_status") != 0 or result.get("timed_out") is True:
            trial_failed = True
        if result.get("acceptance_pass") == "false":
            trial_failed = True
        if result.get("task_language") == "rust":
            if not all(result.get(key) is True for key in ("compile_ok", "clippy_ok", "test_ok")):
                trial_failed = True

        if result.get("exit_status") == 0 and result.get("timed_out") is False:
            completed += 1
        if result.get("acceptance_pass") == "true":
            auto_accepted += 1
        elif result.get("acceptance_pass") == "unverified":
            acceptance_unverified += 1

        review = review_status(trial_dir)
        if review == "passed":
            human_passed += 1
        elif review == "failed":
            human_failed += 1
            trial_failed = True
        elif review == "invalid":
            human_invalid += 1
        else:
            human_pending += 1

        if trial_failed:
            failed += 1
            print(f"FAIL  {trial_dir.name}")
        elif review == "passed" and result.get("acceptance_pass") in {"true", "unverified"}:
            print(f"PASS  {trial_dir.name}")
        else:
            print(f"WAIT  {trial_dir.name}: acceptance or human review is incomplete")

    total = len(result_paths)
    print()
    print("=== Evaluation summary ===")
    print(f"Trials: {total}")
    print(f"Completed agent runs: {completed}")
    print(f"Automated acceptance passed: {auto_accepted}")
    print(f"Automated acceptance unverified: {acceptance_unverified}")
    print(f"Human reviews passed: {human_passed}")
    print(f"Human reviews failed: {human_failed}")
    print(f"Human reviews pending: {human_pending}")
    print(f"Human reviews invalid: {human_invalid}")
    print(f"Failed trials: {failed}")

    if failed > 0:
        print("SUITE FAILED: at least one trial failed an automated or human check")
        return 1
    if acceptance_unverified > human_passed or human_pending > 0 or human_invalid > 0:
        print("SUITE INCOMPLETE: trials still need automated acceptance or valid human review")
        return 2
    print("SUITE COMPLETE: automated checks and human reviews are recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
