# Evaluation results summary

## Environment

- Model: beam-free (opencode, OpenRouter-backed)
- Date: 2026-11-26
- Harness: `evals/harness/run-trial.sh`

## Trials run

| Task | With skill | Compile | Clippy | Tests | Ratatui | Notes |
|---|---|---|---|---|---|---|
| 10-no-trigger | no | N/A (Python) | N/A | N/A | No | Correctly did not trigger skill |
| 10-no-trigger | yes | N/A (Python) | N/A | N/A | No | Correctly did not trigger skill |
| 01-new-app | yes | FAIL | FAIL | FAIL | Yes | Template crate-rename pitfall; agent followed skill structure but forgot to rename `dirview::App` in main.rs |

## Limitations

- Only 1 Rust trial completed (timed out at 180s).
- No no-skill comparison for Rust tasks completed.
- The opencode tool sandbox restricts file access to pre-existing directories, so trials must run from the repository root. The harness has been updated to handle this.
- Full evaluation requires ~10 minutes per trial; results are preliminary.

## Key finding

The skill's "copy structure" instruction caused the agent to copy the dirview template including `use dirview::App;` without renaming. This has been documented in `references/research-checklist.md` and `SKILL.md` §3. This is a real, verifiable defect found by the evaluation process.
