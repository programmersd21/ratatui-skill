---
name: ratatui
description: Build or modify terminal UIs in Rust with ratatui. Use for any TUI, terminal dashboard, file browser, log viewer, or "make this CLI interactive" task. Do not use for non-Rust or non-terminal tasks.
---

# ratatui

Write terminal UIs that work, look deliberate, and contain no filler.

Two failures this skill prevents:

1. **Stale or invented APIs.** ratatui changes between releases. Never write ratatui code from memory.
2. **Slop.** Default styling nobody chose, decorative panels, fake data, dead keys, narrating comments. If it is on screen it is real.

All paths below are relative to this skill's installed directory. Locate this
SKILL.md first, then derive the skill directory:
`SKILL_DIR=$(dirname "$(realpath <path-to-this-SKILL.md>)")`. Run every bundled
script through that absolute path. Never assume the working directory is the
skill or repository root.

## Which workflow

**New application** (no existing codebase): follow steps 1-5 in full.

**Existing application** (bug fix, small feature, layout tweak): skip the design
brief unless the change alters the layout archetype. Inspect the existing code
first. Make the smallest change that fixes the issue. Preserve existing
dependencies, patterns, and key bindings. Run the existing tests, not new ones
unless the bug lacks coverage. Do not restructure.

## 1. Research (before any code)

Run `$SKILL_DIR/scripts/crate-versions.sh ratatui color-eyre unicode-width` plus
every other crate you plan to use. Quote what it returns. Set `rust-version` in
Cargo.toml to the highest MSRV among your dependencies.

- Check `cargo --version` against each dependency's MSRV. Fix before continuing.
- Read the ratatui CHANGELOG and BREAKING-CHANGES for your version.
- Confirm widget crates support your ratatui version (`cargo tree -d`).
- If the app handles terminal events directly, declare `crossterm` as a direct
  dependency at the version compatible with Ratatui. Import from
  `crossterm::event` and verify `cargo tree -d` has no duplicate versions.
- Uncertain API: write a ten-line program and compile it.

See `references/research-checklist.md` for pitfalls found in practice.

## 2. Design brief (new apps only)

Write this before any code. Every line justified from the app's actual data.

- **Purpose:** one sentence, user, keyboard habits.
- **Data:** what is shown, how big (10 rows or 100,000?), how it changes.
- **Layout archetype:** pick one and say why (list+preview, multi-panel,
  drill-down, dashboard, header+list, popup/overlay).
- **Accent color:** exactly one ANSI color, what it marks. Secondary text is
  dim, not a second color.
- **Density:** compact or roomy, and why.
- **Modes and keys:** vim-style plus arrows. Never rebind Ctrl-C, Esc, Enter.
- **States:** empty, loading, failed, too small.
- **Not doing:** features left out on purpose.

## 3. Build

**New app:** start from `templates/dirview/`. Copy the structure, then
replace the domain. Rename the crate reference in `main.rs`.

**Existing app:** do not copy the template. Read the existing code first.
Make the smallest change. Preserve existing dependencies and patterns.

| File | Job |
|---|---|
| `action.rs` | Enum of everything the user can do |
| `keymap.rs` | One key table drives input, footer hints, and `?` help |
| `state.rs` | All state and `State::update(Action)`. No terminal types |
| `fs.rs` | I/O. No terminal types |
| `ui/` | Rendering only. Reads state, never changes it |
| `theme.rs` | Colors by role. The only file that names a color |
| `event.rs`, `app.rs`, `main.rs` | Input, loop, startup |

Rules:

- Everything visible works. No fake data, dead keys, placeholder rows.
- Measure text with `unicode-width`, never `.len()`. Sanitize untrusted text.
- Build only rows on screen. Cost per frame must not grow with data.
- 16 ANSI colors by default. Respect `NO_COLOR`. No forced background.
- One border style, plain Unicode, no emoji.
- Handle every state: empty, error, loading, too small, resize.
- Destructive actions confirm. Slow work never blocks the draw loop.
- `color-eyre` in `main`, typed errors in the library, no swallowed results.
- Comments say why, not what. No TODOs, no commented-out code.

## 4. Verify

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Then:

1. **Dump screens.** Render through `TestBackend` at 40x8, 80x24, 200x60 plus
   empty, error, help states. Read the text. Fix misalignment or clipping.
2. **PTY smoke.** `$SKILL_DIR/scripts/pty-smoke.py PATH/TO/BINARY`. Universal
   checks run by default (startup, alternate screen, tiny terminal, clean exit
   via Ctrl-C). Add `--quit-keys`, `--keys`, `--tiny-text`, `--check-ctrl-c`,
   `--check-color`, `--check-resize` for app-specific checks.
3. **Slop scan.** `$SKILL_DIR/scripts/lint-slop.sh PROJECT_DIR` must print
   `clean`.

If a check cannot run, say which and why. Do not report success for unrun checks.

## 5. Report

Say what was built, versions used, which checks passed, and anything unverified.
Keep it short. Do not recap the steps.

## References

- `references/research-checklist.md`: version and API research, pitfalls
- `references/ux-principles.md`: layouts, keys, color, responsiveness
- `references/slop-tells.md`: what reads as generated, with before/after
