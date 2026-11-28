# dirview

A small directory browser and reference implementation. Borrow the parts that fit your application; its architecture is not a requirement for other Ratatui projects.

    cargo run -- [DIR]

Press `?` inside for the full key list.

| Key | Action |
|---|---|
| `j` `k`, arrows | Move |
| `l`, Enter | Open directory |
| `h`, Backspace | Parent directory |
| `g` `G` | Top, bottom |
| Ctrl-u, Ctrl-d | Page up, down |
| `/` | Filter by name (Enter keeps it, Esc clears it) |
| `.` | Show or hide dotfiles |
| `s` | Cycle sort: name, size, modified |
| `q`, Ctrl-c | Quit |

The preview pane appears at 80 columns or wider. Below 40x8 it shows a size message instead of the interface. Setting `NO_COLOR` switches to a monochrome theme. Directory previews run on a background worker, cancel when the selection changes, and do not block key handling. Listing the current directory on navigation is still synchronous.

## Layout

    src/action.rs   everything the user can do
    src/keymap.rs   the one key table; input, footer hints and help read it
    src/state.rs    all state and `State::update`; no terminal types
    src/fs.rs       reading directories and previews
    src/ui/         rendering only
    src/theme.rs    colors by role; the only file that names a color

## Checks

    cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
    cargo run --example dump -- . 100 24 j      # a screen as plain text
    python3 ../../scripts/pty-smoke.py target/debug/dirview .
    ../../scripts/lint-slop.sh .

Requires Rust 1.88 or newer (ratatui 0.30's minimum).

Written with AI assistance and checked by the tests and scripts above.
