# Evaluation suite

This suite is intended to measure whether the Ratatui skill improves agent results. It is not proof of effectiveness until comparable trials have been completed and graded.

## Current status

- Ten task specifications exist in `tasks/`.
- Existing-project tasks use explicit fixtures staged into isolated trial workspaces.
- Each trial records the skill content hash, task, condition, process exit status, elapsed time, language checks, and acceptance status.
- Only tasks with an acceptance script receive automated acceptance.
- Tasks without an automated acceptance script remain **ungraded** until a reviewer evaluates their stated criteria.
- Committed results are preliminary. They do not establish that the skill improves Rust task outcomes.

See `results/summary.md` for actual evidence and limitations.

## Run

The harness expects OpenCode and its configured model provider:

```bash
bash evals/harness/run-suite.sh 1 600
bash evals/harness/run-suite.sh 1 600 /tmp/ratatui-skill-evals
```

The optional third argument selects an output directory outside the repository. Set model metadata explicitly so results remain identifiable:

```bash
EVAL_MODEL="your-model-id" EVAL_PROVIDER="your-provider" EVAL_MODEL_SETTINGS="temperature=0;reasoning=high" \
  bash evals/harness/run-suite.sh 1 600
```

These are operator-supplied, non-secret labels; do not put API keys or credentials in them. The harness does not infer or verify provider-side settings. They are recorded in each trial's `result.json`, along with the repository commit, task-spec SHA-256, skill content hash, and expected trigger metadata.

Run one paired task manually:

```bash
bash evals/harness/run-trial.sh evals/tasks/02-add-widget.md 1 /tmp/widget-skill 600
bash evals/harness/run-trial.sh evals/tasks/02-add-widget.md 0 /tmp/widget-base 600
```

A trial directory contains the prompt, generated project, stdout, stderr, and `result.json`. Preserve these artifacts. Trial and results directories must be new or empty; the harness refuses to reuse old output to avoid contaminating later trials.

## Human review

Automated acceptance is not a substitute for code and behavior review. Copy the template into each trial directory and complete it:

```bash
cp evals/human-review-template.json /path/to/trial/human-review.json
python3 evals/harness/summarize-results.py /path/to/results
```

The review record must include a reviewer, ISO date, a `passed` or `failed` judgment, all ten 0-2 rubric scores, critical failures, and notes. A passing judgment with critical failures is rejected. The rubric deliberately has no universal numeric pass threshold; the reviewer must explain the judgment.

The summary script can be rerun after reviews are added without rerunning the agent trials. It exits 2 while acceptance or review is incomplete, and 1 when a trial fails.


## Method

1. Keep model, model version, provider, settings, task text, permissions, environment, and fixture commit consistent across conditions.
2. Isolate workspaces. Do not expose the skill or repository instructions to baseline runs.
3. Randomize execution order where practical.
4. Run multiple trials because model output is nondeterministic. Three paired trials per task is an initial floor, not proof of statistical power.
5. Use deterministic checks for objective behavior where practical.
6. Have a human assess architecture, maintainability, usability, and visual quality using the rubric.
7. Inspect trajectories as well as final artifacts: repository inspection, API research, unnecessary edits, recovery, verification, and skill activation.
8. Report failed, timed-out, incomplete, and ungraded trials. Explain any exclusions from denominators.

## Metrics

Report by task and condition: completion and timeout rate; build, Clippy, and test success; automated acceptance; human scores; API/version correctness; terminal interaction and resize behavior; unnecessary complexity; token usage, latency, and cost when reliable provider measurements are available; and activation precision including negative cases.

Keep objective checks and human judgments separate. Do not collapse all dimensions into one score without showing the underlying results.

## Structure

- `tasks/`: task specifications.
- `fixtures/`: starting projects for existing-project and compatibility tasks.
- `acceptance/`: deterministic checks.
- `harness/`: isolated trial execution, deterministic tests, and result summarization.
- `scoring-rubric.md`: human review criteria.
- `human-review-template.json`: structured review record for each trial.
- `results/`: committed preliminary artifacts and honest summaries. New trial artifacts are written outside the repository.

## Limitations

The harness currently supports OpenCode. Other agent integrations need separate implementation and validation. A successful process exit is not equivalent to a successful task. Trials without automated acceptance are ungraded and require human review.
