# Research checklist

Use this in step 1. Facts below were checked by building and testing `templates/dirview` against ratatui 0.30.2 at its tested MSRV of 1.88 (November 2026). Versions will move on: rerun `skills/ratatui/scripts/crate-versions.sh` instead of trusting any number here.

## Finding out what is current

| Question | How to answer it |
|---|---|
| Latest stable version and minimum Rust | `skills/ratatui/scripts/crate-versions.sh NAME...` |
| Does my toolchain meet it? | `cargo --version`, compare with `rust-version` |
| What changed since I last used this? | The crate's `CHANGELOG.md` / `BREAKING-CHANGES.md` on GitHub; web search for the release notes |
| Exact signature of a method | docs.rs for the exact version, or `cargo doc --open`, or read it under `~/.cargo/registry/src/*/ratatui-*/` after a `cargo fetch` |
| Does a widget crate fit my ratatui? | Its docs.rs page and `Cargo.toml` dependencies; then `cargo tree -d` after adding it |
| Is it maintained? | Release dates and open issues on the repository |

When unsure whether a call exists, write a ten-line program that uses it and run `cargo build`. A compile is the only proof.

## Pitfalls confirmed in practice

- **Terminal setup is one call.** `ratatui::run(|terminal| ...)` enters the alternate screen and raw mode, restores them on exit, installs a panic hook that restores the terminal before panicking, and passes a `DefaultTerminal`. The ratatui docs say to call it *after* installing any other panic hook, so install `color-eyre` first (as `main.rs` does). Do not hand-roll `enable_raw_mode` and friends for a normal app.
- **Use a direct crossterm dependency for terminal events.** Declare the release compatible with Ratatui and import from `crossterm::event::{...}`. Check `cargo tree -d` and ensure only one crossterm version is present; do not rely on a transitive re-export as a stable public API.
- **Key events arrive as press, repeat and release on some platforms.** Ignore `KeyEventKind::Release` or every key fires twice on Windows.
- **Shifted letters** arrive as the uppercase character, sometimes also with the `SHIFT` modifier. Normalize before matching (see `keymap::resolve`).
- **`Layout::...areas()` destructures into arrays:** `let [a, b] = Layout::vertical([...]).areas(area);`. The number of names must match the number of constraints.
- **`Constraint::Fill(n)`** shares leftover space; use it instead of guessing percentages.
- **`impl ExactSizeIterator` is the wrong return type** for a `filter_map` chain; return `impl Iterator`. The compiler error is long but this is the cause.
- **Wide characters occupy two cells.** In a `TestBackend` buffer the second cell is a filler; skip it when converting to text (see `ui::buffer_text`), or CJK text looks broken in tests when it is fine on screen.
- **`Terminal::new(TestBackend::new(w, h))` and `terminal.draw` return `Infallible` errors,** not `io::Error`. In tests return `Box<dyn std::error::Error>`.
- **The terminal diff skips blank cells,** so words in a captured byte stream are positioned separately. When checking a real terminal's output, look for text written as one piece, or for the escape sequences, not for a whole sentence.
- **ANSI cyan is written as `38;5;6`** by the crossterm backend (possibly combined with other codes), not `36`.
- **`NO_COLOR`** means "any non-empty value". Check emptiness.
- **A hook that prints locations and backtrace hints** is noise for end users. `color_eyre::config::HookBuilder::default().display_location_section(false).display_env_section(false).install()` gives short messages. Decide deliberately which you want.
- **Canonicalize user-supplied paths** once at startup so `parent()` works for `.` and relative paths.

## Dependency policy

- Use the fewest crates that do the job. Each one is a version to track and a place for ratatui types to mismatch.
- Pin by letting `cargo add` write the current version; commit `Cargo.lock` for binaries.
- Set `rust-version` in `Cargo.toml` to the highest MSRV among your dependencies, and say so in the README.
- Put lint levels in `[lints]` in `Cargo.toml` so they travel with the project.

## Template copying pitfall

When copying `templates/dirview/`, rename the crate reference in `src/main.rs`
(`use dirview::App;`) to match the new crate name. The library crate name comes
from the `[package] name` in `Cargo.toml` (with hyphens converted to underscores).
Forgetting this produces `unresolved import` at compile time.
