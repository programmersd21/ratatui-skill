# Evaluation suite

This is an optional evaluation toolkit for maintainers who want to compare agent results. It is not part of normal skill installation, use, or CI, and running it is not required to use or merge routine skill improvements. The harness is a reproducible runner, not evidence of effectiveness by itself.

## Current status

- Ten task specifications cover new projects, existing-project changes, compatibility, debugging, and a negative-trigger case.
- Existing-project tasks use explicit fixtures staged into isolated trial workspaces.
- Each trial records repository commit, task-spec SHA-256, skill content hash, condition, model/provider labels, timing, process exit status, tool-call telemetry, language checks, and acceptance status.
- Task-specific acceptance scripts provide objective checks where practical. Trials without one remain ungraded until human review.
- The committed report remains preliminary. It does not establish that the skill improves Rust task outcomes.

See results/summary.md for the actual result state and limitations.

## Optional: run a paired evaluation

Only run this when you intentionally want a controlled model comparison. It requires OpenCode, a configured provider credential, and an explicit provider/model ID. EVAL_MODEL is passed to OpenCode as --model, not merely recorded as a label.

~~~bash
EVAL_MODEL="provider/model-id" \
EVAL_PROVIDER="provider-name" \
EVAL_MODEL_SETTINGS="temperature=0;reasoning=high" \
bash evals/harness/run-suite.sh 3 600
~~~

Three repetitions across ten tasks create 60 model runs. This is an optional research exercise, not a release gate. A smaller smoke run checks mechanics only and cannot establish effectiveness.

The optional third argument selects an output directory outside the repository:

~~~bash
EVAL_MODEL="provider/model-id" bash evals/harness/run-suite.sh 3 600 /tmp/ratatui-skill-evals
~~~

EVAL_PROVIDER and EVAL_MODEL_SETTINGS are operator-supplied descriptive labels. They are not provider-side enforcement or automatic inspection. Keep the real settings identical between both conditions and document them. Never put credentials in task files, model labels, or result artifacts.

The suite randomizes baseline/skill order within each task repetition. It writes schedule.tsv and suite.json with the seed, schedule hash, repository commit, model metadata, and task hashes. Set EVAL_ORDER_SEED to reproduce the schedule order.

## Optional: one trial

~~~bash
EVAL_MODEL="provider/model-id" bash evals/harness/run-trial.sh evals/tasks/02-add-widget.md 1 /tmp/widget-skill 600
EVAL_MODEL="provider/model-id" bash evals/harness/run-trial.sh evals/tasks/02-add-widget.md 0 /tmp/widget-base 600
~~~

Use identical model/provider/settings and environment values for both commands. Each trial directory contains the prompt, generated project, OpenCode event stream, text output, standard error, per-check logs, acceptance output, and result.json.

## Isolation and telemetry

Every trial gets a temporary HOME, XDG config/data/cache/state directories, OpenCode config directory, and OpenCode data store. The runner copies only the OpenCode authentication database or legacy auth file. It does not copy user skill directories or user OpenCode configuration, which could contaminate baseline runs. Provider API-key environment variables are inherited.

The runner fails when it finds a machine-wide Ratatui skill in the known OpenCode system skill locations or a project-local skill in a baseline fixture. Trial directories and results directories must be new or empty.

The agent's newline-delimited JSON events are retained. A direct root-session skill tool call naming ratatui is counted as activation. Baseline calls are counted as leakage. OpenCode does not expose all child-session tool events in its root event stream, so delegated activation is recorded as unverified, not guessed. If telemetry is unverified, the suite remains incomplete.

## Optional human review

Automated acceptance does not replace code and behavior review. Copy the template into each trial directory and complete it:

~~~bash
cp evals/human-review-template.json /path/to/trial/human-review.json
python3 evals/harness/summarize-results.py /path/to/results
~~~

A review record requires a reviewer, ISO date, passed or failed status, all ten 0-2 rubric scores, critical failures, and explanatory notes. A passing status with critical failures is invalid. The rubric deliberately has no universal numeric pass threshold.

The summarizer can be rerun after reviews are added. It exits 2 while acceptance or review is incomplete, and 1 when a trial fails.

## Method

1. Keep model, model version, provider, actual settings, task text, permissions, environment, and fixture commit consistent across conditions.
2. Run baseline and skill conditions in separate workspaces. The baseline must not see the skill or project instructions.
3. Randomize execution order within each pair and retain the schedule.
4. Run multiple repetitions because model output is nondeterministic. Three paired trials per task is an initial floor, not proof of statistical power.
5. Use deterministic checks for objective behavior where practical.
6. Have a human review architecture, maintainability, usability, and visual quality using the rubric.
7. Inspect trajectories as well as final artifacts: repository inspection, API research, unnecessary edits, recovery, verification, and skill activation.
8. Report failed, timed-out, incomplete, ungraded, and telemetry-unknown trials. Explain any exclusions from denominators.

## Metrics

Report per task and condition: completion and timeout rate; format, build, Clippy, and test success; automated acceptance; human score distributions; API/version correctness; terminal interaction and resize behavior; unnecessary complexity; token usage, latency, and cost where provider measurements are reliable; and activation precision including negative cases.

Keep objective checks and human judgments separate. Do not collapse every dimension into one unqualified score.

## Limitations

The optional harness currently supports OpenCode only. Other agent integrations require separate runners and runtime validation. A successful process exit is not proof of a successful task. Tasks without deterministic acceptance are ungraded until reviewed. Skill activation through child agents may remain unverified if the runtime does not expose those events.