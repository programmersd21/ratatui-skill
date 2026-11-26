# Evaluation results summary

## Environment

- Model: beam-free (opencode, OpenRouter-backed)
- Date: 2026-11-26
- Harness: `evals/harness/run-trial.sh` (v2, honest exit status, timeout classification)
- Skill commit: 33b5d2b

## Trials run

| Task | Condition | Compile | Clippy | Tests | Ratatui | Files | Timed out | Notes |
|---|---|---|---|---|---|---|---|---|
| 10-no-trigger | no-skill | N/A | N/A | N/A | No | 0 | No | Correctly did not use Ratatui (Python task) |
| 10-no-trigger | with-skill | N/A | N/A | N/A | No | 0 | No | Correctly did not use Ratatui (Python task) |
| 01-new-app | with-skill | No | No | No | Yes | 6 | Yes (180s) | Generated partial code (action, state, keymap, event, lib, Cargo.toml). Timed out before main.rs/ui/theme. |

## Key findings

1. The skill correctly does not trigger on non-Rust tasks (Eval 10, both conditions).
2. The skill's "copy structure" instruction caused the agent to copy the dirview template including `use dirview::App;` without renaming. This has been documented in `references/research-checklist.md` and `SKILL.md` §3.
3. The 180s timeout is insufficient for a full app generation. The harness now supports configurable timeouts.

## Limitations

- Only 1 partial Rust trial completed (timed out).
- No no-skill comparison for Rust tasks completed.
- Results are preliminary and not statistically significant.
- Full evaluation requires ~10 min per trial.

## Harness improvements made

- Real skill injection (copy to opencode skills dir, verify presence)
- Honest exit status (no `|| true`)
- Timeout classification (exit 124 = timeout)
- Separate stdout/stderr logs
- Skill commit SHA recorded
- Elapsed time recorded
- Configurable timeout
