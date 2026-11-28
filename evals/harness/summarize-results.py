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
    if len(sys.argv) not in {2, 3}:
        print("usage: summarize-results.py RESULTS_DIR [EXPECTED_TRIALS]", file=sys.stderr)
        return 2

    expected_trials = None
    if len(sys.argv) == 3:
        try:
            expected_trials = int(sys.argv[2])
        except ValueError:
            print("EXPECTED_TRIALS must be a non-negative integer", file=sys.stderr)
            return 2
        if expected_trials < 0:
            print("EXPECTED_TRIALS must be a non-negative integer", file=sys.stderr)
            return 2

    root = pathlib.Path(sys.argv[1]).resolve()
    if expected_trials is None:
        metadata = read_json(root / "suite.json")
        if isinstance(metadata, dict) and "expected_trials" in metadata:
            recorded = metadata["expected_trials"]
            if type(recorded) is not int or recorded < 0:
                print("suite.json has an invalid expected_trials value", file=sys.stderr)
                return 2
            expected_trials = recorded
    if not root.is_dir():
        print(f"results directory not found: {root}", file=sys.stderr)
        return 2

    trial_dirs = sorted(path for path in root.iterdir() if path.is_dir())
    if expected_trials is not None and len(trial_dirs) != expected_trials:
        print(
            f"expected {expected_trials} trial directories, found {len(trial_dirs)}",
            file=sys.stderr,
        )
        return 1
    if not trial_dirs:
        print("no trial directories found", file=sys.stderr)
        return 2

    completed = 0
    auto_accepted = 0
    acceptance_unverified = 0
    failed = 0
    human_passed = 0
    human_failed = 0
    human_pending = 0
    human_invalid = 0
    activation_unverified = 0
    activation_mismatch = 0

    for trial_dir in trial_dirs:
        result_path = trial_dir / "result.json"
        if not result_path.is_file():
            failed += 1
            print(f"FAIL  {trial_dir.name}: result.json missing")
            continue

        result = read_json(result_path)
        if not isinstance(result, dict):
            failed += 1
            print(f"FAIL  {trial_dir.name}: invalid result.json")
            continue

        trial_failed = False
        acceptance = result.get("acceptance_pass")
        if not isinstance(acceptance, str) or acceptance not in {"true", "false", "unverified"}:
            trial_failed = True
            acceptance = "invalid"
            print(f"FAIL  {trial_dir.name}: invalid acceptance_pass value")
        if type(result.get("exit_status")) is not int or type(result.get("timed_out")) is not bool:
            trial_failed = True
            print(f"FAIL  {trial_dir.name}: invalid execution status fields")
        if result.get("exit_status") != 0 or result.get("timed_out") is True:
            trial_failed = True

        activated = result.get("skill_activated")
        leaked = result.get("skill_leaked")
        expected_trigger = result.get("expected_skill_trigger")
        with_skill = result.get("with_skill")
        activation_is_unverified = False
        if activated not in {"true", "false", "unverified"} or type(with_skill) is not bool:
            trial_failed = True
            activation_is_unverified = True
        elif with_skill:
            if activated == "unverified":
                activation_is_unverified = True
            elif expected_trigger in {"yes", "no"} and ((expected_trigger == "yes" and activated == "false") or (expected_trigger == "no" and activated == "true")):
                activation_mismatch += 1
                trial_failed = True
        if with_skill is False:
            if leaked == "unverified":
                activation_is_unverified = True
            elif leaked == "true":
                activation_mismatch += 1
                trial_failed = True
            elif leaked != "false":
                trial_failed = True
        if activation_is_unverified:
            activation_unverified += 1
        if acceptance == "false":
            trial_failed = True
        if result.get("task_language") == "rust":
            if not all(result.get(key) is True for key in ("fmt_ok", "compile_ok", "clippy_ok", "test_ok")):
                trial_failed = True

        if result.get("exit_status") == 0 and result.get("timed_out") is False:
            completed += 1
        if acceptance == "true":
            auto_accepted += 1
        elif acceptance == "unverified":
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
        elif review == "passed" and acceptance in {"true", "unverified"}:
            print(f"PASS  {trial_dir.name}")
        else:
            print(f"WAIT  {trial_dir.name}: acceptance or human review is incomplete")

    total = len(trial_dirs)
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
    print(f"Activation telemetry unverified: {activation_unverified}")
    print(f"Activation mismatches or baseline leaks: {activation_mismatch}")
    print(f"Failed trials: {failed}")

    if failed > 0:
        print("SUITE FAILED: at least one trial failed an automated or human check")
        return 1
    if acceptance_unverified > human_passed or human_pending > 0 or human_invalid > 0 or activation_unverified > 0:
        print("SUITE INCOMPLETE: trials still need automated acceptance or valid human review")
        return 2
    print("SUITE COMPLETE: automated checks and human reviews are recorded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
