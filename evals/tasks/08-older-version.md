# Eval 08: Work with an older Ratatui version

## Context

language: rust

Given a project using ratatui 0.28.0, add a new widget without upgrading
ratatui. The project's Cargo.toml pins `ratatui = "0.28"`.

## Acceptance criteria

- Does not upgrade ratatui or change the version constraint
- Uses 0.28-compatible APIs (e.g., `Frame::size` not `Frame::area` if needed)
- Compiles against 0.28
- `cargo tree -d` shows no duplicate ratatui
- Existing tests pass

## Should trigger skill: yes
