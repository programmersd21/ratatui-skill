---
name: ratatui
description: Build or improve terminal UIs in Rust with ratatui. Use for any TUI, terminal dashboard, file browser, log viewer, or "make this CLI interactive" task in Rust. Forces fresh research of crate versions and APIs, a written design brief, a working modular codebase, and verification (build, clippy, tests, screen dumps, a real PTY run, slop scan) before anything is called done.
---

# ratatui

Write terminal UIs that work, look deliberate, and contain no filler.

The two failures this skill exists to prevent:

1. **Stale or invented APIs.** ratatui changes between releases and models remember old versions. Never write ratatui code from memory.
2. **Slop.** A UI that looks generated: default styling nobody chose, decorative panels, fake data, buttons that do nothing, comments that narrate the code. Slop is the gap between what the program claims and what it does.

Work through the five steps in order. Do not skip a step because the task looks small.

## 1. Research (before any code)

Run these. Quote what they return when you pick versions; do not paraphrase from memory.

```bash
scripts/crate-versions.sh ratatui color-eyre unicode-width   # plus every other crate you plan to use

All paths below are relative to this skill's installed directory.

Resolving the installed directory: this skill is invoked from an agent that may
be working in a project directory unrelated to the skill. Compute the skill
directory from the skill file location rather than from the current working
directory, e.g. `SKILL_DIR=$(dirname "$(realpath SKILL.md)")` or, in Python,
`pathlib.Path(__file__).resolve().parent`. Run every bundled script through
that path, e.g. `$SKILL_DIR/scripts/crate-versions.sh`.
```

- That prints the newest stable version and the minimum Rust version of each crate. Use those, and set `rust-version` in `Cargo.toml` to the highest one you depend on.
- Check the installed toolchain with `cargo --version`. If it is older than a dependency's `rust-version`, say so and fix it before continuing.
- Set `edition` to the newest the installed toolchain supports (`cargo init` picks it).
- Read the release notes for the ratatui version you are using: its `CHANGELOG.md` and `BREAKING-CHANGES.md` on GitHub, and its docs.rs page. Search the web for them; do not assume their contents.
- For every third-party widget crate (text input, tree, scroll view, image, effects), run the version script, then open its docs.rs page and confirm it supports the ratatui version you chose. A crate pinned to an older ratatui will pull in a second copy of ratatui and fail to compile with confusing type errors. Prefer fewer crates; most apps need only `ratatui`, `color-eyre` and `unicode-width`.
- Add dependencies with `cargo add` and run `cargo tree -d` to confirm there are no duplicate `ratatui` or `crossterm` entries.
- Import crossterm through `ratatui::crossterm`. Do not add `crossterm` to `Cargo.toml` unless you have a reason you can state.

Anything you could not verify, say so. Never invent a method, feature flag or crate. If an API is uncertain, write the smallest program that calls it and compile it.

See `references/research-checklist.md` for the API pitfalls found while building the template.

## 2. Design brief (before any code)

Write this in your reply. Every line must be justified from the app's actual data, not from taste.

- **Purpose:** one sentence, and who uses it, with what keyboard habits.
- **Data:** what is shown, how big it gets (10 rows or 100,000?), how it changes (static, polled, streamed).
- **Layout archetype:** pick one and say why. Options: list + preview, multi-panel, drill-down (Miller columns), dashboard, header + list, popup/overlay. See `references/ux-principles.md`.
- **Accent color:** exactly one, an ANSI color, and what it marks (selection? focus? directories?). Secondary text is dim, not a second color.
- **Density:** compact (one row per item, no padding) or roomy. State the reason.
- **Modes and keys:** the modes, and the keys for each. Prefer vim-style plus arrows. Never rebind Ctrl-C, Esc, or Enter to something surprising.
- **States:** what the screen shows when empty, loading, failed, and too small.
- **Not doing:** features you are leaving out on purpose.

Choosing a look because it is the first one that comes to mind is the failure. If the brief reads like it could describe any app, rewrite it.

## 3. Build

Start from `templates/dirview/`. It is a complete, tested ratatui 0.30 program (a directory browser) with the structure to copy:

| File | Job |
|---|---|
| `action.rs` | Enum of everything the user can do |
| `keymap.rs` | The one table of keys. Input, footer hints and `?` help all read it |
| `state.rs` | All state and `State::update(Action)`. No terminal types, fully unit-tested |
| `fs.rs` (your data source) | I/O. No terminal types |
| `ui/` | Rendering only. Reads state, never changes it |
| `theme.rs` | Colors by role. The only file that names a color |
| `event.rs`, `app.rs`, `main.rs` | Input, the loop, startup |

Copy the structure, then replace the domain. Do not ship the directory-browser domain code in an unrelated app.

Rules, each with a reason:

- **Everything visible works.** No decorative panels, fake numbers, placeholder rows, or keys that do nothing. If it is on screen it is real.
- **One keymap table.** A documented key always works and a working key is always documented, because both come from the same data (`keymap.rs` has the tests).
- **State has no terminal types; `ui` has no side effects.** This is what makes the logic testable and the render predictable.
- **Measure text with `unicode-width`, never `.len()`.** Truncate with an ellipsis. Sanitize untrusted text (file names, log lines) so control characters cannot move the cursor.
- **Build only the rows on screen.** Cost per frame must not grow with the data.
- **Colors by role, in `theme.rs`.** Default to the 16 ANSI colors so the user's own palette applies. Respect `NO_COLOR`. Do not force a background. Make bright 24-bit themes opt-in.
- **One border style, plain Unicode, no emoji.** Borders group things; whitespace separates them.
- **Footer hints show the keys that matter now**, and change with the mode. `?` opens the full list.
- **Handle every state:** empty (say why and what to do), error (show the reason), loading (show progress, never freeze), too small (a message, not a panic), resize.
- **Destructive actions confirm.**
- **Slow work never blocks the draw loop.** Use a thread or task and send results back as actions.
- **Errors:** `color-eyre` in `main`, typed errors in the library, no swallowed results.
- **Comments say why, not what.** No chat artifacts, no TODOs, no commented-out code.
- **README is concrete:** real commands, real keys, one honest line on AI use. No emoji headers, no benefit lists.

## 4. Verify (the app is not done until all of this passes)

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Then look at it:

1. **Dump screens.** Render the real screens through `ratatui::backend::TestBackend` and read the text at several sizes (see `templates/dirview/examples/dump.rs`): at least 40x8, 80x24 and 200x60, plus the empty, error and help states. Fix anything misaligned, clipped or confusing.
2. **Run it in a real terminal.** `scripts/pty-smoke.py PATH/TO/BINARY` runs the binary in a pseudo-terminal, sends keys, and checks that it enters and restores the alternate screen, quits on `q` and Ctrl-C, survives a tiny terminal, and drops color under `NO_COLOR`. A TUI cannot be run through a plain pipe; use the PTY.
3. **Scan for slop.** `scripts/lint-slop.sh PROJECT_DIR` must print `clean`.

If a check cannot run in this environment, say which one and why. Do not report success for a check you did not run.

## 5. Report

Say what was built, the versions used (from step 1), which checks passed, and anything unverified. Keep it short. Do not recap the steps.

## References

- `references/research-checklist.md`: how to research versions and APIs, and the pitfalls found in practice
- `references/ux-principles.md`: layout archetypes, keys, color, responsiveness
- `references/slop-tells.md`: what reads as generated, with before and after
