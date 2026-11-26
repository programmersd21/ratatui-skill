# Building ratatui apps

For any Rust terminal UI task, follow `skills/ratatui/SKILL.md`. It is the
full process; this file is the short version for agents that read `AGENTS.md`
instead of loading skills.

## Which workflow

- **New application:** follow steps 1-5 in full, including the design brief
  and the template.
- **Existing application:** skip the design brief unless the layout archetype
  changes. Inspect the existing code first. Make the smallest change. Preserve
  existing dependencies and patterns. Do not copy the template.

## Before writing code

1. **Research.** Run `$SKILL_DIR/scripts/crate-versions.sh <crates>` for the
   newest stable version and minimum Rust version of every dependency. Read the
   ratatui `CHANGELOG.md` and `BREAKING-CHANGES.md` for that version. Never
   write ratatui code, versions or feature flags from memory. If an API is
   uncertain, compile a ten-line program that calls it.
2. **Design brief (new apps only).** State the layout archetype, one ANSI
   accent color, density, modes and keys, and what each empty, error and
   too-small state shows, justified from the app's data.

## Building

- New apps: start from `skills/ratatui/templates/dirview/` and copy its
  structure: `action`, `keymap`, `state` (no terminal types), `ui` (no side
  effects), `theme` (the only file naming colors). Rename the crate reference
  in `main.rs` to match your package name.
- Existing apps: do not copy the template. Read the existing code first.
- One key table drives input, footer hints and the `?` help.
- Everything visible works. No fake data, decorative panels or dead keys.
- Measure text with `unicode-width`, sanitize untrusted text, draw only visible
  rows.
- Colors by role, 16 ANSI by default, respect `NO_COLOR`, no forced background,
  no emoji.
- If handling terminal events directly, declare a compatible direct `crossterm` dependency and import from `crossterm::event`. Verify `cargo tree -d` has no duplicate versions.
- No `.unwrap()` outside tests, no swallowed errors, no TODOs, comments say why.

## Done means

See `skills/ratatui/SKILL.md` section 4 (Verify) for the canonical checklist.
Do not duplicate it here.

Also read the rendered screens as text at several sizes
(`skills/ratatui/templates/dirview/examples/dump.rs`). Report what was built,
the versions used, what passed, and anything you could not check.

More detail: `skills/ratatui/references/` (`research-checklist.md`,
`ux-principles.md`, `slop-tells.md`).
