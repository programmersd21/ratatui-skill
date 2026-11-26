# Eval 01: Create a new Ratatui application

## Task

Build a Rust TUI that displays a scrollable list of log entries. Each entry has
a timestamp, level (info/warn/error), and message. The user can filter by level
with number keys 1-3, scroll with j/k, and quit with q.

## Acceptance criteria

- Compiles with `cargo build`
- Uses ratatui 0.30 APIs (no deprecated `Spans`, `Frame::size`, etc.)
- List scrolls without panicking at 1000+ entries
- Filter by level works and updates the visible list
- Empty state shows a message
- Renders correctly at 40x8, 80x24, 200x60 (no panic, no clipping of labels)
- All keyboard bindings documented in footer or help
- `cargo clippy -- -D warnings` passes
- At least 3 unit tests for state transitions

## Should trigger skill: yes
