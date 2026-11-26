# Evaluation suite for ratatui-skill

Measures whether the skill improves agent output. Each task is a self-contained
specification with acceptance criteria that can be checked by compilation, tests,
and PTY interaction.

## Structure

- `tasks/` - one Markdown file per evaluation task
- `harness/` - scripts to run trials and collect results

## Method

1. For each task, run N trials with the skill enabled and N without.
2. Same task spec, same environment, same model.
3. Capture: generated code, compile result, test result, PTY output, token count.
4. Score each trial against the task's acceptance criteria.
5. Report: pass rate, failure modes, token overhead, workflow adherence.

## Status

All 10 tasks are **proposed**. Harness v2 is implemented with:
- Real skill injection (copy to isolated opencode config, verify presence)
- Honest exit status (no `|| true` suppression)
- Timeout classification (exit 124 = timeout)
- Separate stdout/stderr logs
- Skill content hash (not git HEAD)
- Language-aware (Rust vs Python)
- Configurable timeout

Two partial trials have been run. Results in `evals/results/`.
Full trials require ~10 min each with a real model. Results will be
added to `evals/results/` once executed.
