# Eval 09: Fix a bug without restructuring

## Context

language: rust

The dirview template has a bug: when filtering, the selected entry jumps to
index 0 instead of staying on the current entry. Fix it.

## Acceptance criteria

- Smallest possible diff (no restructuring, no new abstractions)
- Existing tests still pass
- New test specifically for the bug
- No changes to unrelated code
- Key bindings unchanged

## Should trigger skill: yes
