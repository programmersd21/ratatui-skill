---
name: ratatui
description: Build, modify, debug, or review Rust terminal user interfaces built with Ratatui. Do not use for non-Rust tasks or command-line applications without a terminal UI.
---

# Ratatui

Build terminal interfaces that are correct, usable, and appropriate to the project. This skill is a workflow and a set of technical references, not a universal architecture or visual style guide.

## Choose the workflow

Inspect the repository and identify the task first.

- **New application:** understand requirements, choose the smallest suitable structure, implement, and verify.
- **Existing application:** inspect relevant modules, conventions, dependencies, and tests. Preserve established design and make the smallest change that solves the task.
- **Focused widget or layout change:** follow existing state and rendering patterns. Do not introduce application-wide architecture for a local change.
- **Debugging or review:** reproduce the behavior, inspect the relevant code and dependency versions, then verify the proposed fix.

Do not copy the starter template into an existing application.

## 1. Inspect and research

1. Read the relevant `Cargo.toml`, `Cargo.lock`, workspace configuration, and nearby source.
2. Identify the Ratatui and terminal-backend versions actually used.
3. Preserve dependency constraints unless an upgrade is requested or required.
4. Consult documentation and source matching those versions. The latest release is not automatically compatible.
5. Check release notes when changing versions or relying on recently changed APIs.
6. Compile a minimal example when an API remains uncertain.

The bundled helper compares registry releases with versions resolved by a project:

```bash
$SKILL_DIR/scripts/crate-versions.sh ratatui crossterm unicode-width
$SKILL_DIR/scripts/crate-versions.sh --manifest path/to/Cargo.toml ratatui crossterm
```

Treat its output as a starting point, not a substitute for compatibility checks.

## 2. Choose an architecture that fits

Prefer the simplest design that makes behavior understandable and testable.

Often useful, but not mandatory:

- Keep application state and state transitions independent of terminal rendering.
- Keep rendering focused on drawing from state.
- Separate event translation from application actions when that reduces coupling.
- Centralize key bindings when input, help, and footer hints share them.
- Keep I/O and long-running work out of the draw path.
- Use modules that reflect real responsibilities.

Do not impose these patterns on a tiny program when they add more complexity than value. Do not restructure an existing project to match the template.

### Starter template

`templates/dirview/` is a working example, not a required foundation. Use it when its navigation, state, event, and rendering structure fits. Otherwise borrow only relevant ideas or start smaller. Read its code before copying anything.

## 3. Build for the actual interface

Derive layout, interaction, and visual choices from the requirements and data being displayed.

Consider where relevant:

- Clear hierarchy and useful information density.
- Discoverable keyboard behavior.
- Terminal resizing and small dimensions.
- Empty, loading, success, and error states.
- Long text, Unicode width, and untrusted terminal text.
- Focus and selection visibility.
- Color and monochrome terminals, including `NO_COLOR`.
- Cancellation and recovery from failures.
- Large datasets and bounded or incremental work.

These are design considerations, not a mandate to include every state or feature. Avoid decoration that does not help users understand or operate the interface. Do not present fake data as real functionality.

## 4. Keep the UI responsive

Rendering should not perform avoidable blocking work. For expensive operations, choose an approach appropriate to the workload: precompute, cache, paginate, bound, move work off the draw path, or show progress. Do not add asynchronous machinery to trivial operations without a demonstrated need.

Measure before making performance claims.

## 5. Test behavior at the right level

1. **State and logic:** unit-test transitions, filtering, sorting, selection, and errors without a terminal.
2. **Rendering:** use Ratatui's `TestBackend` for layout, clipping, styles, and important visible states. Test representative sizes where responsiveness matters.
3. **Terminal integration:** use a PTY, VHS, Betamax, or equivalent when lifecycle, keyboard input, terminal restoration, or resizing is part of the behavior.
4. **Manual review:** inspect hierarchy, readability, and awkward edge cases.

Do not snapshot every frame by default. Prefer assertions about meaningful behavior and visible state. A process-start smoke test alone does not establish that an interactive application works.

## 6. Verify the change

Run checks appropriate to the workspace and affected packages:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

Use `--manifest-path` or package selection for workspaces and scoped changes. Do not claim a check passed if it was not run.

The bundled PTY helper is available when a real terminal smoke test is useful:

```bash
$SKILL_DIR/scripts/pty-smoke.py PATH/TO/BINARY
```

Use its app-specific options as needed. The style scanner is heuristic, not a correctness analyzer:

```bash
$SKILL_DIR/scripts/lint-slop.sh PROJECT_DIR
```

Compiler-visible lints belong in Clippy. Human review is still needed for architecture, error handling, and visual quality.

## 7. Report honestly

Summarize what changed, important dependency/API decisions, checks actually run, relevant behavior verified, and anything not tested or uncertain.

Never report unrun checks as passing.

## References

Read only what the task needs:

- `references/research-checklist.md`: version and API research.
- `references/ux-principles.md`: interface design and terminal usability.
- `references/slop-tells.md`: visual and textual patterns worth reviewing.
