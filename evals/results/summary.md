# Evaluation results summary

## Environment

- Model: beam-free (opencode, OpenRouter-backed)
- Date: 2026-11-26
- Harness: `evals/harness/run-trial.sh` (v3, honest exit status, language-aware, isolated config)
- Skill commit: 33b5d2b (first trial), 2da8f8e (harness fixes)

## Trials run

| Task | Condition | Compile | Clippy | Tests | Ratatui | Files | Timed out | Acceptance | Notes |
|---|---|---|---|---|---|---|---|---|---|
| 10-no-trigger | no-skill | N/A (Python) | N/A | N/A | No | 0 | No | PASS | Correctly did not use Ratatui |
| 10-no-trigger | with-skill | N/A (Python) | N/A | N/A | No | 0 | No | PASS | Correctly did not trigger skill |
| 01-new-app | with-skill | No | No | No | Yes | 6 | Yes (180s) | N/A (timed out) | Generated partial code (action, state, keymap, event, lib, Cargo.toml). Timed out before main.rs/ui/theme. |

## Key findings

1. The skill correctly does not trigger on non-Rust tasks (Eval 10, both conditions).
2. The skill's "copy structure" instruction caused the agent to follow the skill's module structure (action/keymap/state/event split) but the trial timed out before completing. Whether the agent copied the dirview template verbatim or not cannot be determined from the partial output.
3. The 180s timeout is insufficient for a full app generation. The harness now supports configurable timeouts.

## Contradiction resolution

An earlier version of this summary claimed the agent copied the dirview template and forgot to rename `use dirview::App;`. This claim was based on a different trial's partial output. The raw logs for the only completed Rust trial (Eval 01, trial 1) show 6 source files generated (action, state, keymap, event, lib, Cargo.toml) with no main.rs or ui/ directory. Whether the agent copied the template or wrote from scratch cannot be determined from this partial output. The claim has been retracted. Raw artifacts are the source of truth.

## Limitations

- Only 1 partial Rust trial completed (timed out at 180s).
- No no-skill comparison for Rust tasks completed.
- Results are preliminary and not statistically significant.
- Full evaluation requires ~10 min per trial with a real model.
- Skill activation is reported as "unverified" (cannot be proven from output text).

## Harness improvements made (v3)

- Real skill injection (copy to isolated opencode config, verify presence)
- Honest exit status (no `|| true` suppression)
- Timeout classification (exit 124 = timeout)
- Separate stdout/stderr logs
- Skill content hash (SHA256 of all skill files, not git HEAD)
- Language-aware (Rust vs Python via task metadata)
- Configurable timeout
- Task-specific acceptance scripts
- No skill leakage into baseline (config isolated)
