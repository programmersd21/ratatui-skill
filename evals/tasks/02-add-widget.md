# Eval 02: Add a widget to an existing application

## Context

language: rust
fixture: dirview

Given the dirview template, add a status bar at the bottom showing the current
directory's total size and file count.

## Acceptance criteria

- Smallest possible change (no restructuring)
- Existing tests still pass
- New status bar renders at 40x8 without clipping the existing footer
- File count and size are computed from real data, not hardcoded
- No new dependencies added

## Should trigger skill: yes
