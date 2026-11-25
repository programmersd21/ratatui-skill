# todo-tui

A keyboard-driven todo list for the terminal. Tasks survive restarts: every
change is written immediately to a plain text file.

## Run

```bash
cargo run --bin todo
cargo run --bin todo -- /path/to/tasks.txt
TODO_FILE=/path/to/tasks.txt cargo run --bin todo
```

With no file argument the list is loaded from `~/.todo-tui.txt`, created on the
first change. `-h` prints usage.

## Color

The default uses 16 ANSI colors, so it follows your terminal palette. For the
Catppuccin Mocha truecolor theme:

```bash
TODO_THEME=catppuccin cargo run --bin todo
```

`NO_COLOR` overrides both and renders monochrome. No app-wide background is
painted; only the selected row is highlighted.

## Keys

| Key | Action |
|---|---|
| `j` / `Down`, `k` / `Up` | Move between tasks |
| `g` / `Home`, `G` / `End` | Jump to top / bottom |
| `Ctrl-u` / `PgUp`, `Ctrl-d` / `PgDn` | Page up / down |
| `a` | Add a task |
| `Space` | Toggle done / pending |
| `d` | Delete the selected task (asks first) |
| `?` | Show all keys |
| `q`, `Ctrl-c` | Quit |

While adding: type text, `Backspace` deletes a character, `Enter` saves, `Esc`
cancels. While confirming a delete: `y` deletes, `n` or `Esc` keeps the task.

## File format

One task per line. `+ ` means done, `- ` means pending, followed by the text:

```text
- buy milk
+ call mom
```

Control characters in the text are replaced with `·` when saving and loading,
so a hand-edited file cannot inject terminal escape sequences. Writes go to a
temporary file and are renamed over the target, so an interrupted write cannot
truncate the list.

## Building

Requires Rust 1.88 or newer. Dependencies: `ratatui` 0.30.2, `color-eyre` 0.6.5
and `unicode-width` 0.2.3.

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

This project was written with AI assistance; the layout, key handling, storage
and tests were all compiled and exercised locally.
