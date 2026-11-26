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

All 10 tasks are **proposed**. One partial trial has been run
(Eval 01, with skill). Results in `evals/results/`. The trial timed out but
produced useful findings (template crate-rename pitfall, now documented). Results will be
added to `evals/results/` once executed.
