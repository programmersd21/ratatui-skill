# Eval 08: Work with an older Ratatui version

## Context

language: rust
fixture: ratatui-028

Given a small project pinned to ratatui 0.28.0, add a new widget without
upgrading Ratatui or changing the existing dependency constraint.

## Acceptance criteria

- Does not upgrade ratatui or change its version constraint
- Uses APIs compatible with the fixture's Ratatui version
- Compiles against the pinned version
- Does not introduce a duplicate Ratatui version
- Existing tests pass

## Should trigger skill: yes
