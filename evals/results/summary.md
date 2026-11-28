# Optional evaluation status

Last updated: 27 November 2026.

## Current conclusion

No controlled effectiveness claim is made. The paired evaluation harness is optional research tooling; real-model trials are not required for normal use, CI, or release readiness.

The repository now contains ten task specifications. For maintainers who choose to run a controlled comparison, the initial schedule is three repetitions for each task in both conditions:

- 10 tasks
- 2 conditions (skill available / baseline)
- 3 repetitions
- 60 real model runs

No results from those 60 runs are included in this repository. The CI mock agent tests harness mechanics only.

## Historical exploratory runs

The following runs predate the current isolated, randomized harness and must not be combined into a skill-effectiveness score:

| Task | Condition | Outcome | Interpretation |
|---|---|---|---|
| 10-no-trigger | Baseline | Python task passed | One historical negative-trigger smoke run |
| 10-no-trigger | Skill available | Python task passed without invoking Ratatui | One historical negative-trigger smoke run |
| 01-new-app | Skill available | Timed out at 180 seconds after generating a partial Rust project | Incomplete exploratory run; no paired baseline |

The two Python smoke runs do not prove skill quality. The partial Rust run cannot be counted as a successful implementation or compared with a baseline. Its raw output did not establish whether the model copied the dirview template or independently chose a similar structure.

## What CI verifies

CI verifies repository behavior and test infrastructure, not model capability:

- The harness tests exercise isolated trial setup, fixture staging, result validation, acceptance status, skill activation telemetry, baseline leakage detection, and incomplete-review handling using a deterministic mock agent.
- The dirview tests cover preview sorting, counts, symlinks, help rendering, and scrolling.
- A 50,000-entry directory regression test checks that directory preview work is dispatched to a background worker and that the exact total and bounded sample are returned.
- A Betamax tape exercises the interactive dirview terminal flow. Its CI status must be checked independently from the Rust test suite.

## Optional future research

1. If measuring skill effectiveness is useful, run the paired suite with one fixed provider/model version and identical settings in both conditions.
2. Preserve all trial directories, JSON event streams, compiler/test logs, model metadata, and randomized order schedule.
3. Complete task-specific acceptance checks, then obtain independent human review using human-review-template.json.
4. Report failures, timeouts, unverified skill activations, and excluded trials. Do not omit poor runs from the denominator.
5. Smoke-test the documented installation and invocation paths in each agent client before claiming runtime compatibility.

## Reproducing the evaluation

See evals/README.md for the exact OpenCode command, isolation model, paired schedule, metadata, and human-review workflow.

A complete run requires OpenCode and an operator-configured provider credential. Repository CI intentionally does not run model trials, and no contributor is expected to configure credentials or run paid evaluations.