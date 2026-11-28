# Skill guidance classification

Status: initial maintainer audit. This is a working decision record, not a claim that every recommendation has been empirically validated.

The purpose is to prevent preferences from silently becoming universal requirements. Update this record when repository evidence, real usage, or evaluations change a decision.

## Classification

- **Required:** necessary for correctness, compatibility, an explicit user requirement, or a verified project constraint.
- **Recommended:** a useful default with known exceptions.
- **Optional:** a choice driven by product requirements or taste.
- **Experimental:** a hypothesis that needs additional evidence.

## Initial rule inventory

| Guidance | Classification | Reason and boundary |
|---|---|---|
| Inspect the manifest, lockfile, and existing code before changing an application | Required | Prevents incompatible APIs, accidental upgrades, and unnecessary rewrites. |
| Verify APIs against the version resolved by the project | Required | Ratatui and backend APIs vary by release. |
| Preserve existing dependencies unless there is a reason to change them | Recommended | Minimizes churn; a task may require a dependency change. |
| Keep the draw path free of avoidable blocking work | Required when work can block interaction | Responsiveness is a behavioral requirement; implementations may be synchronous, asynchronous, cached, bounded, or incremental. |
| Handle terminal restoration and errors correctly | Required | A TUI must not leave the user's terminal unusable after normal errors or exit. |
| Separate state from rendering | Recommended | Improves testability in many applications; very small applications may not need additional layers. |
| Centralize key bindings | Recommended when bindings are repeated across input and UI | A single source prevents drift; a small one-screen tool may not need a separate keymap module. |
| Use TestBackend | Recommended for rendering behavior | Useful for layout and visible-state tests, but not every behavior is a rendering concern. |
| Use PTY or recording-based integration tests | Required when terminal lifecycle or real input is in scope; otherwise optional | Unit tests cannot prove terminal interaction, but PTY tests are unnecessary for every local state change. |
| Use the dirview template | Optional | It is an example. Its architecture may not fit every product. |
| Write a design brief | Optional, scaled to complexity | A short plan helps complex interfaces; a tiny focused change does not need ceremony. |
| Use exactly one accent color or a fixed ANSI palette | Optional | Visual direction depends on the application, terminal support, and user requirements. | 
| Keep all color constructors inside a theme module | Optional repository convention | Can help larger applications centralize semantic styles; should not be enforced as a universal Ratatui rule. |
| Require a fixed file/module layout | Optional | Modules should represent real boundaries, not satisfy a template. |
| Ban emoji or decorative styling universally | Optional style guidance | Taste is not correctness; apply only when it fits the product. |

## Evidence policy

For each substantial rule:

1. Identify the concrete failure or user problem it addresses.
2. Find primary evidence: Ratatui documentation, source, examples, accepted changes, maintainer review, or a reproducible bug.
3. Record meaningful counterexamples and trade-offs.
4. Choose the narrowest classification justified by evidence.
5. Add a regression task when the rule addresses a repeatable failure.
6. Keep empirical claims separate from design rationale.

Do not cite a project preference as a Ratatui requirement. An accepted implementation is not necessarily the only valid implementation.

## Optional evaluation tooling

The repository includes task specifications and a trial harness for maintainers who want to study skill effectiveness. These tools are optional and are not required for normal use, CI, or release readiness. The harness stages existing-project fixtures and distinguishes automated acceptance from ungraded trials. No measured effectiveness claim is made.
