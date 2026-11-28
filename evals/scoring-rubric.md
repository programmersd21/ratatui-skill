# Evaluation scoring rubric

Use this rubric for human review. Record objective checks separately; do not infer them from a reviewer's impression.

Score each dimension from 0 to 2:

| Dimension | 0 | 1 | 2 |
|---|---|---|---|
| Functional acceptance | Required behavior missing | Partially implemented | All stated behavior verified |
| API and version correctness | Incorrect or incompatible | Minor issues | Correct for fixture dependencies |
| Terminal lifecycle | Broken or unsafe | Partially handled | Correct setup, exit, and restoration |
| Keyboard interaction | Broken or undiscoverable | Partially correct | Documented behavior works |
| Resize and layout | Panics or unusable | Degrades awkwardly | Works at specified sizes |
| State and error handling | Important states missing | Partial handling | Relevant states handled clearly |
| Architecture | Unnecessarily coupled or complex | Adequate | Clear, proportionate, maintainable |
| Tests | Missing or ineffective | Partial coverage | Meaningful behavior is tested |
| Scope discipline | Unrelated rewrites or dependencies | Some avoidable churn | Minimal changes appropriate to task |
| Verification quality | Claims without evidence | Partial evidence | Reproducible checks and honest report |

Maximum: 20 points.

## No universal pass threshold

A task can have a critical acceptance criterion that must pass regardless of total score. A high subjective score must not compensate for a build failure or missing required behavior.

For each trial, record the task and fixture commit, skill condition and hash, model/provider/settings, reviewer and date, objective check results, per-dimension scores, critical failures, and rationale for non-obvious scores.

## Comparison

Report skill and baseline outcomes per task, not only pooled averages. Include completion and timeout rates, build/test/acceptance rates, human score distributions, condition-specific failures, token/latency/cost overhead when available, and negative-trigger behavior.

Do not claim statistical significance from a small sample. Do not label ungraded trials as passes.
