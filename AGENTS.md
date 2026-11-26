# Building Ratatui applications

Use `skills/ratatui/SKILL.md` when creating, modifying, debugging, or reviewing a Rust terminal UI built with Ratatui.

## Workflow

- Inspect the repository, manifest, lockfile, source, and tests first.
- For a new application, choose an architecture that fits its complexity.
- For an existing application, preserve established patterns and make a targeted change.
- Use `skills/ratatui/templates/dirview/` as an example when its structure fits. It is not a mandatory starter.
- Do not apply this guidance to non-Ratatui or non-TUI tasks.

## Engineering

- Verify APIs against the Ratatui version resolved by the project.
- Avoid unnecessary dependencies and version changes.
- Keep rendering responsive; move genuinely expensive work out of the draw path.
- Handle terminal resizing, input, and relevant empty or error states.
- Prefer state and behavior tests; use `TestBackend` for relevant rendering checks.
- Use PTY, VHS, Betamax, or equivalent integration tests when real terminal behavior is in scope.
- Treat architecture and visual conventions as context-dependent unless correctness requires them.

## Done means

Run checks appropriate to the workspace and changed packages. Report exactly what ran, what passed, and what remains unverified. See `skills/ratatui/SKILL.md` for the detailed workflow and references.
