# Building ratatui apps

For any Rust terminal UI task, follow `skills/ratatui/SKILL.md`. It is the full process; this file is the short version for agents that read `AGENTS.md` instead of loading skills.

## Before writing code

1. **Research.** Run `skills/ratatui/scripts/crate-versions.sh <crates>` for the newest stable version and minimum Rust version of every dependency. Read the ratatui `CHANGELOG.md` and `BREAKING-CHANGES.md` for that version. Never write ratatui code, versions or feature flags from memory. If an API is uncertain, compile a ten-line program that calls it.
2. **Design brief.** State the layout archetype, one ANSI accent color, density, modes and keys, and what each empty, error and too-small state shows, justified from the app's data.

## Building

- Start from `skills/ratatui/templates/dirview/` and copy its structure: `action`, `keymap`, `state` (no terminal types), `ui` (no side effects), `theme` (the only file naming colors).
- One key table drives input, footer hints and the `?` help.
- Everything visible works. No fake data, decorative panels or dead keys.
- Measure text with `unicode-width`, sanitize untrusted text, draw only visible rows.
- Colors by role, 16 ANSI by default, respect `NO_COLOR`, no forced background, no emoji.
- Import crossterm through `ratatui::crossterm`; keep dependencies few.
- No `.unwrap()` outside tests, no swallowed errors, no TODOs, comments say why.

## Done means

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python3 skills/ratatui/scripts/pty-smoke.py target/debug/<binary>
skills/ratatui/scripts/lint-slop.sh .
```

Also read the rendered screens as text at several sizes (`templates/dirview/examples/dump.rs`). Report what was built, the versions used, what passed, and anything you could not check.

More detail: `skills/ratatui/references/` (`research-checklist.md`, `ux-principles.md`, `slop-tells.md`).
